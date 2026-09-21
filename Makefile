SERVER_BIN := target/debug/server
.DEFAULT_GOAL := build

.PHONY: dev build-wasm build-wasm-dev watch-wasm build-server watch-server build-web watch-web build run start check

build-server:
	cargo build --manifest-path server/Cargo.toml

watch-server:
	cargo watch -w ./server -w Cargo.toml -w Cargo.lock -x 'run -p server -- start'

build-wasm:
	wasm-pack build --target web --dev --out-dir ../web/wasm ./wasm

watch-wasm:
	cargo watch -w ./wasm/src -w ./wasm/Cargo.toml -w Cargo.toml -w Cargo.lock -s 'wasm-pack build --target web --dev --out-dir ../web/wasm ./wasm'

build-web:
	pnpm -C ./web build

watch-web:
	pnpm -C ./web dev

build: build-server build-wasm build-web

start: build
	$(SERVER_BIN) start
