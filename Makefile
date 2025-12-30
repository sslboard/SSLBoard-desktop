.PHONY: tests check dev build

-include .env.local
export

tests:
	cargo test --manifest-path src-tauri/Cargo.toml --features integration-tests --test integration

check:
	cargo check --manifest-path src-tauri/Cargo.toml

dev:
	RUST_LOG=debug npm run tauri dev

build:
	npm run tauri build
