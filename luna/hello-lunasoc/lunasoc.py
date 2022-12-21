# This file is part of LUNA.
#
# Copyright (c) 2020 Great Scott Gadgets <info@greatscottgadgets.com>
# SPDX-License-Identifier: BSD-3-Clause

"""Minerva SoC for LUNA firmware."""

from amaranth         import *
from amaranth.build   import *

import amaranth_soc
from amaranth_soc          import wishbone
from amaranth_soc.periph   import ConstantMap
from amaranth_stdio.serial import AsyncSerial
from amaranth_boards.ulx3s import ULX3S_85F_Platform

import lambdasoc
from lambdasoc.cores       import litedram, liteeth
from lambdasoc.cpu.minerva import MinervaCPU

from lambdasoc.periph.intc   import GenericInterruptController
from lambdasoc.periph.serial import AsyncSerialPeripheral
from lambdasoc.periph.sram   import SRAMPeripheral
from lambdasoc.periph.timer  import TimerPeripheral
from lambdasoc.periph.sdram  import SDRAMPeripheral
from lambdasoc.periph.eth    import EthernetMACPeripheral

from lambdasoc.soc.cpu import CPUSoC, BIOSBuilder

import logging


# - CoreSoC -------------------------------------------------------------------

class CoreSoC(CPUSoC, Elaboratable):
    def __init__(self, clock_frequency=int(60e6)):
        super().__init__()

        # create cpu - TODO maybe do this _after_ we've added our RAM
        self.internal_sram_size = 8192
        self.internal_sram_addr = 0x40000000
        cpu = MinervaCPU(
            with_debug    = False,
            with_icache   = True,
            icache_nlines = 16,
            icache_nwords = 4,
            icache_nways  = 1,
            icache_base   = self.internal_sram_addr,
            icache_limit  = self.internal_sram_addr + self.internal_sram_size,
            with_dcache   = True,
            dcache_nlines = 16,
            dcache_nwords = 4,
            dcache_nways  = 1,
            dcache_base   = self.internal_sram_addr,
            dcache_limit  = self.internal_sram_addr + self.internal_sram_size,
            with_muldiv   = True,
            reset_address = 0x00000000,
        )

        # create system bus
        bus_address_width = 30
        self._bus_decoder = wishbone.Decoder(addr_width=bus_address_width, data_width=32, granularity=8,
                                             features={"cti", "bte", "err"})
        self._bus_arbiter = wishbone.Arbiter(addr_width=bus_address_width, data_width=32, granularity=8,
                                             features={"cti", "bte", "err"})
        self._bus_arbiter.add(cpu.ibus)
        self._bus_arbiter.add(cpu.dbus)

        # create interrupt controller
        intc = GenericInterruptController(width=len(cpu.ip))

        # assign CPUSoC socproperties
        self.sync_clk_freq = clock_frequency
        self.cpu = cpu
        self.intc = intc

        # Things we don't have but lambdasoc's jinja2 templates expect
        # TODO should we just make them properties?
        self.sdram = None
        self.ethmac = None

        # random state it would be nice to get rid of
        self._interrupt_map = {}


    # - CPUSoC @property overrides --

    @property
    def constants(self):
        return super().constants.union(
            #SDRAM  = self.sdram .constant_map if self.sdram  is not None else None,
            #ETHMAC = self.ethmac.constant_map if self.ethmac is not None else None,
            SOC = ConstantMap(
                #WITH_SDRAM        = self.sdram  is not None,
                #WITH_ETHMAC       = self.ethmac is not None,
                MEMTEST_ADDR_SIZE = self.internal_sram_size,
                MEMTEST_DATA_SIZE = self.internal_sram_size,
            ),
        )

    @property
    def memory_map(self):
        return self._bus_decoder.bus.memory_map

    # - Elaboratable --

    def elaborate(self, platform):
        m = Module()

        if isinstance(platform, ULX3S_85F_Platform):
            m.submodules.car = platform.clock_domain_generator()

        m.submodules.cpu        = self.cpu
        m.submodules.arbiter    = self._bus_arbiter
        m.submodules.decoder    = self._bus_decoder
        m.submodules.intc       = self.intc

        m.d.comb += [
            self._bus_arbiter.bus.connect(self._bus_decoder.bus),
            self.cpu.ip.eq(self.intc.ip),
        ]

        return m


# - LunaSoC -------------------------------------------------------------------

class LunaSoC(CoreSoC):
    """ Class used for building simple, example system-on-a-chip architectures.

    Intended to facilitate demonstrations (and very simple USB devices) by providing
    a wrapper that can be updated as the Amaranth-based-SoC landscape changes. Hopefully,
    this will eventually be filled by e.g. Amaranth-compatible-LiteX. :)

    SimpleSoC devices integrate:
        - A simple riscv32i processor.
        - One or more read-only or read-write memories.
        - A number of amaranth-soc peripherals.

    The current implementation uses a single, 32-bit wide Wishbone bus
    as the system's backend; and uses lambdasoc as its backing technology.
    This is subject to change.
    """

    # - LunaSoC user api --

    def add_bios_and_peripherals(self, uart_pins, uart_baud_rate=115200, fixed_addresses=False):
        """ Adds a simple BIOS that allows loading firmware, and the requisite peripherals.

        Automatically adds the following peripherals:
            self.bootrom        -- A ROM memory used for the BIOS. (required by LambdaSoC)
            self.scratchpad     -- A RAM memory used for the BIOS. (required by LambdaSoC)
            self.uart           -- An AsyncSerialPeripheral used for serial I/O.
            self.timer          -- A TimerPeripheral used for BIOS timing.
            self._internal_sram -- Program RAM.

        Parameters:
            uart_pins       -- The UARTResource to be used for UART communications; or an equivalent record.
            uart_baud_rate  -- The baud rate to be used by the BIOS' uart.
        """

        # memory
        bootrom_addr       = 0x00000000
        bootrom_size       = 0x4000
        scratchpad_addr    = 0x00004000
        scratchpad_size    = 0x1000
        internal_sram_size = self.internal_sram_size
        internal_sram_addr = self.internal_sram_addr

        # uart
        uart_addr  = 0x80000000
        uart_irqno = 1
        uart_core  = AsyncSerial(
            data_bits = 8,
            divisor   = int(self.sync_clk_freq // uart_baud_rate),
            pins      = uart_pins,
        )

        # timer
        timer_addr  = 0x80001000
        timer_width = 32
        timer_irqno = 0

        # add bootrom, scratchpad, uart, timer, _internal_sram
        self.bootrom = SRAMPeripheral(size=bootrom_size, writable=False)
        self._bus_decoder.add(self.bootrom.bus, addr=bootrom_addr)

        self.scratchpad = SRAMPeripheral(size=scratchpad_size)
        self._bus_decoder.add(self.scratchpad.bus, addr=scratchpad_addr)

        self.uart = AsyncSerialPeripheral(core=uart_core)
        self._bus_decoder.add(self.uart.bus, addr=uart_addr)
        self.intc.add_irq(self.uart.irq, index=uart_irqno)
        self._interrupt_map[uart_irqno] = self.uart

        self.timer = TimerPeripheral(width=timer_width)
        self._bus_decoder.add(self.timer.bus, addr=timer_addr)
        self.intc.add_irq(self.timer.irq, index=timer_irqno)
        self._interrupt_map[timer_irqno] = self.timer

        self._internal_sram = SRAMPeripheral(size=internal_sram_size)
        self._bus_decoder.add(self._internal_sram.bus, addr=internal_sram_addr)


    def add_rom(self, data, size, addr=0, is_main_rom=True):
        """ Creates a simple ROM and adds it to the design.

        Parameters:
            data -- The data to fill the relevant ROM.
            size -- The size for the rom that should be created.
            addr -- The address at which the ROM should reside.
        """
        pass


    def add_ram(self, size: int, addr: int = None, is_main_mem: bool = True):
        """ Creates a simple RAM and adds it to our design.

        Parameters:
            size -- The size of the RAM, in bytes. Will be rounded up to the nearest power of two.
            addr -- The address at which to place the RAM.
        """
        pass


    def add_peripheral(self, p: lambdasoc.periph.Peripheral, *, as_submodule=True, **kwargs):
        """ Adds a peripheral to the SoC.

        For now, this is identical to adding a peripheral to the SoC's wishbone bus.
        For convenience, returns the peripheral provided.
        """
        pass


    # - LambdaSoC @property overrides --

    @property
    def mainram(self):
        # TODO what is @mainram to LambdaSoC / Luna?
        return self.sram

    @property
    def sram(self):
        # TODO what is @sram to LambdaSoC / Luna?
        # TODO this is currently being set by LunaSoC
        return self._internal_sram


    # - Elaboratable --

    def elaborate(self, platform):
        m = super().elaborate(platform)

        m.submodules.uart       = self.uart
        m.submodules.timer      = self.timer
        m.submodules.bootrom    = self.bootrom
        m.submodules.scratchpad = self.scratchpad

        # TODO
        #if self.sram is not None:
        #    m.submodules.sram = self.sram
        if self._internal_sram is not None:
            m.submodules.sram = self._internal_sram

        return m


    # - LambdaSoC build --

    def build(self, name=None, build_dir="build", do_build=True, do_init=True):
        """ Builds any internal artifacts necessary to create our CPU.

        This is usually used for e.g. building our BIOS.

        Parmeters:
            name      -- The name for the SoC design.
            build_dir -- The directory where our main Amaranth build is being performed.
                         We'll build in a subdirectory of it.
        """

        super().build(build_dir=build_dir, name=name, do_build=do_build, do_init=do_init)


    # - integration API -- TODO remove these entirely and call Introspect directly from generators

    def resources(self, omit_bios_mem=True):
        return Introspect(self).resources(omit_bios_mem)

    def range_for_peripheral(self, target_peripheral: lambdasoc.periph.Peripheral):
        return Introspect(self).range_for_peripheral(target_peripheral)

    def irq_for_peripheral_window(self, target_peripheral_window: amaranth_soc.memory.MemoryMap):
        return Introspect(self).irq_for_peripheral_window(target_peripheral_window)



# - Introspect ----------------------------------------------------------------

class Introspect:
    def __init__(self, soc: CPUSoC):
        self._soc = soc

    # - public API --

    # TODO s/resources/peripherals
    # TODO attach irq to peripheral if there is one so we don't have to maintain it separately
    # TODO add a "memories()" ?
    def resources(self, omit_bios_mem=True):
        """ Creates an iterator over each of the device's addressable resources.

        Yields (MemoryMap, ResourceInfo, address, size) for each resource.

        Parameters:
            omit_bios_mem -- If True, BIOS-related memories are skipped when generating our
                             resource listings. This hides BIOS resources from the application.
        """

        # Grab the memory map for this SoC...
        memory_map = self._soc._bus_decoder.bus.memory_map

        # ... find each addressable peripheral...
        window: amaranth_soc.memory.MemoryMap
        for window, (window_start, _end, _granularity) in memory_map.windows():

            # Grab the resources for this peripheral
            resources = window.all_resources()

            # ... find the peripheral's resources...
            resource_info: amaranth_soc.memory.ResourceInfo
            for resource_info in resources:
                resource = resource_info.resource
                register_offset = resource_info.start
                register_end_offset = resource_info.end
                _local_granularity = resource_info.width

                # TODO
                if True: #self._soc._build_bios and omit_bios_mem:
                    # If we're omitting bios resources, skip the BIOS ram/rom.
                    if (self._soc.sram._mem is resource) or (self._soc.bootrom._mem is resource):
                        continue

                # ... and extract the peripheral's range/vitals...
                size = register_end_offset - register_offset
                yield window, resource_info, window_start + register_offset, size


    def range_for_peripheral(self, target_peripheral: lambdasoc.periph.Peripheral):
        """ Returns size information for the given peripheral.
        Returns:
            addr, size -- if the given size is known; or
            None, None    if not
        """

        # Grab the memory map for this SoC...
        memory_map = self._soc._bus_decoder.bus.memory_map

        # Search our memory map for the target peripheral.
        resource_info: amaranth_soc.memory.ResourceInfo
        for resource_info in memory_map.all_resources():
            if resource_info.name[0] is target_peripheral.name:
                return resource_info.start, (resource_info.end - resource_info.start)

        return None, None


    def irq_for_peripheral_window(self, target_peripheral_window: amaranth_soc.memory.MemoryMap):
        """ Returns any interrupt associated with the given peripheral.
        Returns:
            irqno, peripheral -- if the given peripheral has an interrupt; or
            None, None    if not
        """
        for irqno, peripheral in self._soc._interrupt_map.items():
            if peripheral.name is target_peripheral_window.name:
                return irqno, peripheral

        return None, None


    def log_resources(self):
        """ Logs a summary of our resource utilization to our running logs. """

        # Grab the memory map for this SoC...
        memory_map = self._soc.bus_decoder.bus.memory_map

        # Resource addresses:
        logging.info("Physical address allocations:")
        #for peripheral, (start, end, _granularity) in memory_map.all_resources():
        #    logging.info(f"    {start:08x}-{end:08x}: {peripheral}")

        resource_info: amaranth_soc.memory.ResourceInfo
        for resource_info in memory_map.all_resources():
            start = resource_info.start
            end = resource_info.end
            peripheral = resource_info.resource
            logging.info(f"    {start:08x}-{end:08x}: {peripheral}")

        logging.info("")

        # IRQ numbers
        logging.info("IRQ allocations:")
        for irq, peripheral in self._soc._irqs.items():
            logging.info(f"    {irq}: {peripheral.name}")
        logging.info("")

        # Main memory.
        if self._build_bios:
            memory_location = self.main_ram_address()

            logging.info(f"Main memory at 0x{memory_location:08x}; upload using:")
            logging.info(f"    flterm --kernel <your_firmware> --kernel-addr 0x{memory_location:08x} --speed {self._uart_baud}")
            logging.info("or")
            logging.info(f"    lxterm --kernel <your_firmware> --kernel-adr 0x{memory_location:08x} --speed {self._uart_baud}")

        logging.info("")

    def main_ram_address(self):
        """ Returns the address of the main system RAM. """
        start, _  = self.range_for_peripheral(self._main_ram)
        return start
