.PHONY: build

wasm:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm build

build: wasm
	npm run compile

package: build
	vsce package
