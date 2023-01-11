SVD := ../hello-lunasoc/build/gensvd/lunasoc.svd

# TODO --const_generic

svd2rust:
	rm -rf build src/generated.rs src/generated/
	mkdir build

	cp $(SVD) svd/lunasoc.svd
	svd2rust -i svd/lunasoc.svd -o build/ --target riscv --const_generic --make_mod
	mv build/mod.rs src/generated.rs
	mv build/build.rs .
	mv build/device.x .

	form -i src/generated.rs -o src/generated/
	cd src/ && cargo fmt -- --emit files



svd2rust_old:
	rm -rf build src svd
	mkdir build src svd

	cp $(SVD) svd/lunasoc.svd

	svd2rust -i svd/lunasoc.svd -o build/ --target riscv --const_generic
	mv build/lib.rs src
	mv build/build.rs .
	mv build/device.x .

	form -i src/lib.rs -o src/
	cd src && cargo fmt -- --emit files


distclean:
	rm -rf build/ target/ src/generated.rs src/generated/ svd/lunasoc.svd
