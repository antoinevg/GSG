all: cynthion imac imac+dcache

cynthion:
	cd vexriscv/scala && sbt compile "runMain lunasoc.GenCoreCynthion"
	mv vexriscv/scala/vexriscv_cynthion.* vexriscv/

imac:
	cd vexriscv/scala && sbt compile "runMain lunasoc.GenCoreImac"
	mv vexriscv/scala/vexriscv_imac.* vexriscv/

imac+dcache:
	cd vexriscv/scala && sbt compile "runMain lunasoc.GenCoreImacDcache"
	mv vexriscv/scala/vexriscv_imac+dcache.* vexriscv/

clean:
	cd vexriscv/scala && sbt clean reload
	rm -rf vexriscv/target/ vexriscv/scala/target/ vexriscv/scala/project/target/
