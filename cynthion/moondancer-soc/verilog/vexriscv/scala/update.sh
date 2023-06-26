#rm VexRiscv_IMACNoDcache.* VexRiscv_IMACDcache.* ; make VexRiscv_IMACNoDcache.v VexRiscv_IMACDcache.v ; cp VexRiscv_IMACNoDcache.* VexRiscv_IMACDcache.* ~/GreatScott/gsg.git/luna/lunasoc/verilog/

#vexriscv_cynthion: $(SRC)
#    sbt compile "runMain vexriscv.GenCoreCynthion"

# sbt clean reload
rm -f vexriscv_cynthion.*
sbt compile "runMain vexriscv.GenCoreCynthion"
cp vexriscv_cynthion.* ~/GreatScott/gsg.git/luna/lunasoc/verilog/
