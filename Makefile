SERVER_BIN := target/debug/server

.PHONY: build-wasm build run start

build-wasm:
	wasm-pack build --target web client
	cp -r client/pkg static

build: build-wasm
	cargo build --manifest-path server/Cargo.toml

run: build
	$(SERVER_BIN)

start: build
	$(SERVER_BIN) start 