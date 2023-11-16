.PHONY: build

build: wasm
	npm run compile

wasm:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm build

clean:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm clean

package: build
	vsce package

clippy:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm clippy

test:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm test
	npm run test
