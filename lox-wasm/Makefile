.PHONY: build 

OUT_DIR ?= '../lox-vsce/out/wasm'

build: 
	wasm-pack build --dev --target nodejs --out-dir $(OUT_DIR)
	rm -f $(OUT_DIR)/.gitignore $(OUT_DIR)/package.json

clean:
	cargo clean
	$(MAKE) -C ../lox clean

clippy:
	cargo clippy
	$(MAKE) -C ../lox clippy

test:
	cargo test
	$(MAKE) -C ../lox test
