.PHONY: test lint typecheck fmtcheck fmt package wasm clean

test: wasm
	npm run test

lint: wasm
	npm run lint

typecheck: wasm
	npm run typecheck

fmtcheck: clean
	npm run fmtcheck

fmt: clean
	npm run fmtwrite

package: wasm
	npm run package

CARGO_FLAGS ?= --dev

wasm: clean
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) OUT_DIR=$$(pwd)/out -C ../does-it-throw-wasm build

node_modules: package.json
	bun install
	@touch $@

clean: node_modules
	rm -rf out
	mkdir -p out
