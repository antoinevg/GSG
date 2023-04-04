## Dependencies

    brew install sbt



## pythondata_cpu_vexriscv.git

Repository:

    cd toolchain/
    git clone https://github.com/litex-hub/pythondata-cpu-vexriscv.git

    # submodules
    git submodule update --init --recursive

    # patch to add a IMAC capable cpu without dcache
    patch -p 1 < ~/GreatScott/gsg.git/luna/hello-vexriscv/pythondata_cpu_vexriscv.patch

Generate Verilog for lunasoc:

    cd pythondata_cpu_vexriscv/verilog/

    sbt clean reload
    make VexRiscv_IMACNoDcache.v


## VexRiscv.git:

Repository:

    cd toolchain/
    git clone https://github.com/SpinalHDL/VexRiscv.git VexRiscv.git

Build demo:

     sbt clean reload
     sbt "runMain vexriscv.demo.GenFull"



## Links

* [pythondata_cpu_vexriscv](https://github.com/litex-hub/pythondata-cpu-vexriscv/tree/master/pythondata_cpu_vexriscv/verilog)
