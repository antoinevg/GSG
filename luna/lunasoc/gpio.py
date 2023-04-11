from amaranth         import Elaboratable, Module, Signal, Record
from amaranth.hdl.rec import DIR_FANIN, DIR_FANOUT
from lambdasoc.periph import Peripheral

class GpioPeripheral(Peripheral, Elaboratable):
    """ GPIO peripheral. """

    def __init__(self, width=8):
        super().__init__()

        self.pins = Record([
            ("oe", width, DIR_FANOUT),
            ("o",  width, DIR_FANOUT),
            ("i",  width, DIR_FANIN),
        ])
        self.width = width

        # peripheral control registers
        self._mode  = self.csr_bank().csr(width, "rw")
        self._odr   = self.csr_bank().csr(width, "w")
        self._idr   = self.csr_bank().csr(width, "r")

        # peripheral bus
        self._bridge = self.bridge(data_width=32, granularity=8, alignment=2)
        self.bus     = self._bridge.bus

    def elaborate(self, platform):
        m = Module()

        # set pin output enable states: 0=output, 1=input
        reg_mode = Signal(self.width)
        with m.If(self._odr.w_stb):
            m.d.sync += reg_mode.eq(self._mode.w_data)
        m.d.comb += self.pins.oe.eq(reg_mode)

        # set pin output states
        reg_out = Signal(self.width)
        with m.If(self._odr.w_stb):
            m.d.sync += reg_out.eq(self._odr.w_data)
        m.d.comb += self.pins.o.eq(reg_out)

        # get pin input states
        reg_in = Signal(self.width)
        m.d.comb += reg_in.eq(self.pins.i)
        with m.If(self._idr.r_stb):
            m.d.sync += self._idr.r_data.eq(reg_in)

        # submodules
        m.submodules.bridge = self._bridge

        return m
