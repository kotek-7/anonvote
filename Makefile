SERVER_BIN := target/debug/server

.PHONY: build-wasm watch-wasm build-server watch-server build-web watch-web build run start

build-server:
	cargo build --manifest-path server/Cargo.toml

watch-server:
	cargo watch -w ./wasm -s 'cargo build --manifest-path server/Cargo.toml'

build-wasm:
	wasm-pack build --target web --out-dir ../web/wasm ./wasm

watch-wasm: 
	cargo watch -w ./wasm -s 'wasm-pack build --target web --dev --out-dir ../web/wasm ./wasm'

build-web:
	pnpm -C ./web build

watch-web:
	pnpm -C ./web dev

build: build-wasm build-server

run: build
	$(SERVER_BIN)

start: build
	$(SERVER_BIN) start 