from gpio                                    import GpioPeripheral
from vexsoc                                  import LunaSoC

from luna                                    import configure_default_logging, top_level_cli
from luna.gateware.usb.usb2.device           import USBDevice, USBDeviceController
from luna.gateware.usb.usb2.interfaces.eptri import SetupFIFOInterface, InFIFOInterface, OutFIFOInterface

from amaranth                                import Elaboratable, Module, Cat
from amaranth.hdl.rec                        import Record
from lambdasoc.periph                        import Peripheral

import logging
import os
import sys

CLOCK_FREQUENCIES_MHZ = {
    'sync': 60
}

# - LedPeripheral -------------------------------------------------------------

class LedPeripheral(Peripheral, Elaboratable):
    """ Example peripheral that controls the board's LEDs. """

    def __init__(self):
        super().__init__()

        # Create our LED register.
        # Note that there's a bunch of 'magic' that goes on behind the scenes, here:
        # a memory address will automatically be reserved for this register in the address
        # space it's attached to; and the SoC utilities will automatically generate header
        # entires and stub functions for it.
        bank            = self.csr_bank()
        self._output    = bank.csr(6, "w")

        # ... and convert our register into a Wishbone peripheral.
        self._bridge    = self.bridge(data_width=32, granularity=8, alignment=2)
        self.bus        = self._bridge.bus


    def elaborate(self, platform):
        m = Module()
        m.submodules.bridge = self._bridge

        # Grab our LEDS...
        leds = Cat(platform.request("led", i) for i in range(6))

        # ... and update them on each register write.
        with m.If(self._output.w_stb):
            m.d.sync += leds.eq(self._output.w_data)

        return m

# - CynthionSoC ---------------------------------------------------------------

class CynthionSoC(Elaboratable):
    def __init__(self, clock_frequency=int(60e6)):

        # Create a stand-in for our UART.
        self.uart_pins = Record([
            ('rx', [('i', 1)]),
            ('tx', [('o', 1)])
        ])

        # Create our SoC...
        self.soc = LunaSoC(clock_frequency, internal_sram_size=65536)

        # Add bios and core peripherals
        self.soc.add_bios_and_peripherals(uart_pins=self.uart_pins)

        # ... add some bulk RAM ...
        # TODO soc.add_ram(0x4000, name="bulkram")

        # ... add a GpioPeripheral for the PMOD connectors ...
        self.gpioa = GpioPeripheral(width=8)
        self.gpiob = GpioPeripheral(width=8)
        self.soc.add_peripheral(self.gpioa)
        self.soc.add_peripheral(self.gpiob)

        # ... add our LED peripheral, for simple output.
        self.leds = LedPeripheral()
        self.soc.add_peripheral(self.leds)

        # ... and the core USB controllers and eptri peripherals ...
        self.usb0 = USBDeviceController()
        self.usb0_ep_control = SetupFIFOInterface()
        self.usb0_ep_in = InFIFOInterface()
        self.usb0_ep_out = OutFIFOInterface()
        self.soc.add_peripheral(self.usb0, addr=0x80002000)
        self.soc.add_peripheral(self.usb0_ep_control, as_submodule=False)
        self.soc.add_peripheral(self.usb0_ep_in, as_submodule=False)
        self.soc.add_peripheral(self.usb0_ep_out, as_submodule=False)

        self.usb1 = USBDeviceController()
        self.usb1_ep_control = SetupFIFOInterface()
        self.usb1_ep_in = InFIFOInterface()
        self.usb1_ep_out = OutFIFOInterface()
        self.soc.add_peripheral(self.usb1, addr=0x80003000)
        self.soc.add_peripheral(self.usb1_ep_control, as_submodule=False)
        self.soc.add_peripheral(self.usb1_ep_in, as_submodule=False)
        self.soc.add_peripheral(self.usb1_ep_out, as_submodule=False)

        self.usb2 = USBDeviceController()
        self.usb2_ep_control = SetupFIFOInterface()
        self.usb2_ep_in = InFIFOInterface()
        self.usb2_ep_out = OutFIFOInterface()
        self.soc.add_peripheral(self.usb2, addr=0x80004000)
        self.soc.add_peripheral(self.usb2_ep_control, as_submodule=False)
        self.soc.add_peripheral(self.usb2_ep_in, as_submodule=False)
        self.soc.add_peripheral(self.usb2_ep_out, as_submodule=False)

    def elaborate(self, platform):
        m = Module()
        m.submodules.soc = self.soc

        # generate our domain clocks/resets
        m.submodules.car = platform.clock_domain_generator(clock_frequencies=CLOCK_FREQUENCIES_MHZ)

        # connect up our UART
        uart_io = platform.request("uart", 0)
        m.d.comb += [
            uart_io.tx.o.eq(self.uart_pins.tx),
            self.uart_pins.rx.eq(uart_io.rx)
        ]
        if hasattr(uart_io.tx, 'oe'):
            m.d.comb += uart_io.tx.oe.eq(~self.soc.uart._phy.tx.rdy),

        # connect the GpioPeripheral to the pmod ports
        pmoda_io = platform.request("user_pmod", 0)
        pmodb_io = platform.request("user_pmod", 1)
        m.d.comb += [
            self.gpioa.pins.connect(pmoda_io),
            self.gpiob.pins.connect(pmodb_io)
        ]

        # wire the cpu external reset signal up to a user port
        user1_io = platform.request("user_io", 1)
        m.d.comb += user1_io.oe.eq(1)
        m.d.comb += self.soc.cpu.ext_reset.eq(user1_io.i)

        # create our USB devices, connect device controllers and add eptri endpoint handlers
        ulpi0 = platform.request(platform.default_usb_connection) # target_phy
        usb0_device = USBDevice(bus=ulpi0)
        usb0_device.add_endpoint(self.usb0_ep_control)
        usb0_device.add_endpoint(self.usb0_ep_in)
        usb0_device.add_endpoint(self.usb0_ep_out)
        m.d.comb += self.usb0.attach(usb0_device)
        m.submodules.usb0_device = usb0_device

        ulpi1 = platform.request("host_phy")
        usb1_device = USBDevice(bus=ulpi1)
        usb1_device.add_endpoint(self.usb1_ep_control)
        usb1_device.add_endpoint(self.usb1_ep_in)
        usb1_device.add_endpoint(self.usb1_ep_out)
        m.d.comb += self.usb1.attach(usb1_device)
        m.submodules.usb1_device = usb1_device

        ulpi2 = platform.request("sideband_phy")
        usb2_device = USBDevice(bus=ulpi2)
        usb2_device.add_endpoint(self.usb2_ep_control)
        usb2_device.add_endpoint(self.usb2_ep_in)
        usb2_device.add_endpoint(self.usb2_ep_out)
        m.d.comb += self.usb2.attach(usb2_device)
        m.submodules.usb2_device = usb2_device

        return m


# - main ----------------------------------------------------------------------

import luna
from luna.gateware.platform.ulx3s     import ULX3S_85F_Platform
from luna.gateware.platform.luna_r0_4 import LUNAPlatformRev0D4


if __name__ == "__main__":
    from generate import Generate

    # Disable UnusedElaborable warnings
    from amaranth._unused import MustUse
    MustUse._MustUse__silence = True

    build_dir = os.path.join("build")

    # configure logging
    configure_default_logging()
    logging.getLogger().setLevel(logging.DEBUG)

    # select platform
    platform = luna.gateware.platform.get_appropriate_platform()
    #platform = LUNAPlatformRev0D4()
    #platform = ULX3S_85F_Platform()

    # create design
    if isinstance(platform, LUNAPlatformRev0D4):
        logging.info("Building for Luna r04")
        design = CynthionSoC(clock_frequency=int(60e6))
    elif isinstance(platform, ULX3S_85F_Platform):
        logging.info("Building for ULX3s")
        design = CynthionSoC(clock_frequency=int(48e6))
    else:
        logging.error("Unsupported platform: {}".format(platform))
        sys.exit()

    # TODO fix litex build
    thirdparty = os.path.join(build_dir, "lambdasoc.soc.cpu/bios/3rdparty/litex")
    if not os.path.exists(thirdparty):
        logging.info("Fixing build, creating output directory: {}".format(thirdparty))
        os.makedirs(thirdparty)

    # build litex bios
    logging.info("Building bios")
    design.soc.build(name="soc",
                     build_dir=build_dir,
                     do_init=True)

    # build soc
    logging.info("Building soc")
    overrides = {
        "debug_verilog": True,
        "verbose": False,
    }
    products = platform.build(design, do_program=False, build_dir=build_dir, **overrides)

    # log resources
    from lunasoc import Introspect
    Introspect(design.soc).log_resources()

    # generate artifacts
    generate = Generate(design.soc)

    # generate: c-header and ld-script
    path = os.path.join(build_dir, "genc")
    if not os.path.exists(path):
        os.makedirs(path)

    logging.info("Generating c-header and ld-script: {}".format(path))
    with open(os.path.join(path, "resources.h"), "w") as f:
        generate.c_header(platform_name=platform.name, file=f)
    with open(os.path.join(path, "soc.ld"), "w") as f:
        generate.ld_script(file=f)

    # generate: svd file
    path = os.path.join(build_dir, "gensvd")
    if not os.path.exists(path):
        os.makedirs(path)

    logging.info("Generating svd file: {}".format(path))
    with open(os.path.join(path, "lunasoc.svd"), "w") as f:
        generate.svd(file=f)

    # generate: rust memory.x file
    path = os.path.join(build_dir, "genrust")
    if not os.path.exists(path):
        os.makedirs(path)

    logging.info("Generating memory.x file: {}".format(path))
    with open(os.path.join(path, "memory.x"), "w") as f:
        generate.memory_x(file=f)

    print("Build completed. Use 'make load' to load bitsream to device.")

    # TODO
    #top_level_cli(design)
