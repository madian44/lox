.PHONY: build

build: wasm
	mkdir -p out/syntaxes
	cp syntaxes/* out/syntaxes
	npm run compile

wasm:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm build

clean:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm clean
	npm run clean
	rm -f lox-vsce-*.vsix

package: build
	npm run package

clippy:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm clippy

test: build
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../lox-wasm test
	npm run test

init:
	npm install
	npm outdated
	#npm update
