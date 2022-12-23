from lunasoc                           import LunaSoC

from luna                              import configure_default_logging, top_level_cli
from luna.gateware.platform.ulx3s      import ULX3S_85F_Platform
from luna.gateware.platform.luna_r0_4  import LUNAPlatformRev0D4

from amaranth                          import Elaboratable, Module, Cat
from amaranth.hdl.rec                  import Record
from lambdasoc.periph                  import Peripheral

import logging
import os
import sys


# - LEDPeripheral -------------------------------------------------------------

class LEDPeripheral(Peripheral, Elaboratable):
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


# - LunaSoCExample ------------------------------------------------------------

class LunaSoCExample(Elaboratable):
    def __init__(self, clock_frequency=int(50e6)):

        # Create a stand-in for our UART.
        self.uart_pins = Record([
            ('rx', [('i', 1)]),
            ('tx', [('o', 1)])
        ])

        # Create our SoC...
        self.soc = LunaSoC(clock_frequency)

        # Add bios and core peripherals
        self.soc.add_bios_and_peripherals(uart_pins=self.uart_pins)

        # ... add some bulk RAM ...
        # TODO soc.add_ram(0x4000, name="bulkram")

        # ... and add our LED peripheral.
        self.leds = LEDPeripheral()
        # TODO soc.add_peripheral(leds)
        self.soc._bus_decoder.add(self.leds.bus)


    def elaborate(self, platform):
        m = Module()
        m.submodules.soc = self.soc

        # Connect up our UART.
        uart_io = platform.request("uart", 0)
        m.d.comb += [
            uart_io.tx.o.eq(self.uart_pins.tx),
            self.uart_pins.rx.eq(uart_io.rx)
        ]
        if hasattr(uart_io.tx, 'oe'):
            m.d.comb += uart_io.tx.oe.eq(~self.soc.uart._phy.tx.rdy),

        # Connect up our LED peripheral
        m.submodules.leds = self.leds

        return m


# - main ----------------------------------------------------------------------

if __name__ == "__main__":
    from generate import Generate

    build_dir = os.path.join("build")

    # configure logging
    configure_default_logging()
    logging.getLogger().setLevel(logging.DEBUG)

    # select platform
    platform = LUNAPlatformRev0D4()
    #platform = ULX3S_85F_Platform()
    #platform = luna.gateware.platform.get_appropriate_platform()

    # create design
    # TODO ideally we should be able to get clk_freq from platform
    if isinstance(platform, LUNAPlatformRev0D4):
        logging.info("Building for Luna r04")
        design = LunaSoCExample(clock_frequency=int(60e6))
    elif isinstance(platform, ULX3S_85F_Platform):
        logging.info("Building for ULX3s")
        design = LunaSoCExample(clock_frequency=int(48e6))
    else:
        logging.error("Unsupported platform: {}".format(platform))
        sys.exit()

    # TODO fix build
    thirdparty = os.path.join(build_dir, "lambdasoc.soc.cpu/bios/3rdparty/litex")
    if not os.path.exists(thirdparty):
        logging.info("Fixing build, creating output directory: ", thirdparty)
        os.makedirs(thirdparty)

    # build bios
    logging.info("Building bios")
    design.soc.build(name="soc",
                     build_dir=build_dir,
                     do_init=True)

    # build soc
    logging.info("Building soc")
    products = platform.build(design, do_program=False, build_dir=build_dir)

    # generate c-header and ld-script
    logging.info("Generating c-header and ld-script")
    generate = Generate(design.soc)
    with open("resources.h", "w") as f:
        generate.c_header(platform_name=platform.name, file=f)
    with open("soc.ld", "w") as f:
        generate.ld_script(file=f)

    # TODO generate svd
    logging.info("Generating svd file")
    generate = Generate(design.soc)
    with open("lunasoc.svd", "w") as f:
        generate.svd(file=f)

    # Log resources
    from lunasoc import Introspect
    Introspect(design.soc).log_resources()

    print("Build completed. Use 'make load' to load bitsream to device.")

    # TODO
    #top_level_cli(design)
