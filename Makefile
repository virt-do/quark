.PHONY: build

build:
	cargo build

pre-test:
	@echo "Seting tests environment..."
	@mkdir -p /tmp/quark/builds/testquardle/dossier1/
	@touch /tmp/quark/builds/testquardle/fichier1.txt
	@touch /tmp/quark/builds/testquardle/fichier2.txt
	@touch /tmp/quark/builds/testquardle/dossier1/fichier3.txt
	@echo "Done. Your quadle env is named: testquardle"

test: pre-test
	@echo "Running tests..."
	cargo run -- build --image sometest --quardle testquardle
	mkdir out/testquardle && tar -C out/testquardle -xvf out/testquardle.qrk
