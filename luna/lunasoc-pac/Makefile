SVD := ../hello-lunasoc/build/gensvd/lunasoc.svd


svd2rust:
	rm -rf build src svd
	mkdir build src svd

	cp $(SVD) svd/lunasoc.svd

	svd2rust -i svd/lunasoc.svd -o build/ --target riscv
	mv build/lib.rs src
	mv build/build.rs .
	mv build/device.x .

	form -i src/lib.rs -o src/
	cd src && cargo fmt -- --emit files


clean:
	rm -rf build src svd
