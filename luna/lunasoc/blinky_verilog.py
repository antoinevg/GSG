#!/usr/bin/env python3
#
# This file is part of LUNA.
#
# Copyright (c) 2020 Great Scott Gadgets <info@greatscottgadgets.com>
# SPDX-License-Identifier: BSD-3-Clause

import os, sys

from amaranth import Signal, Module, Elaboratable, ClockDomain, ClockSignal, Cat, Instance

import luna
from luna import top_level_cli
from luna.gateware.platform import NullPin

class Blinky(Elaboratable):
    def elaborate(self, platform):
        m = Module()

        # Grab our I/O connectors.
        leds    = [platform.request_optional("led", i, default=NullPin()).o for i in range(0, 8)]
        user_io = [platform.request_optional("user_io", i, default=NullPin()).o for i in range(0, 8)]

        # Clock divider / counter.
        counter = Signal(28)
        m.d.sync += counter.eq(counter + 1)

        # Attach the LEDs and User I/O to the MSBs of our counter.
        m.d.comb += Cat(leds).eq(counter[-7:-1])
        m.d.comb += Cat(user_io).eq(counter[7:21])

        # Return our elaborated module.
        return m


class BlinkyVerilog(Elaboratable):
    def elaborate(self, platform):
        m = Module()

        # add module verilog file to the build
        with open("blinky.v", "r") as f:
            verilog = f.read()
            platform.add_file("blinky.v", verilog)

        # module connectors
        input_clk = ClockSignal("sync")
        output_1 = Signal(6)
        output_2 = Signal(2)

        # instantiate module - also see hello-rusthdl example for reference
        blinky_verilog = Instance(
            # module name in verilog file
            "blinky_top",
            # inputs
            i_clk_60mhz = input_clk,
            # outputs
            o_led     = output_1,
            o_user_io = output_2,
        )
        m.submodules.blinky_verilog = blinky_verilog

        # Grab our I/O connectors.
        leds    = [platform.request_optional("led", i, default=NullPin()).o for i in range(0, 8)]
        user_io = [platform.request_optional("user_io", i, default=NullPin()).o for i in range(0, 8)]

        # Attach the LEDs and User I/O to the module
        m.d.comb += Cat(leds).eq(output_1)
        m.d.comb += Cat(user_io).eq(output_2)

        return m


if __name__ == "__main__":
    #top_level_cli(Blinky)

    # instantiate design
    #top = Blinky()
    top = BlinkyVerilog()

    # build
    build_dir = os.path.join("build")
    platform = luna.gateware.platform.get_appropriate_platform()
    platform.build(top, do_program=True, build_dir=build_dir)
