.PHONY: tests check dev build

-include .env.local
export

tests:
	cargo test --manifest-path src-tauri/Cargo.toml --features integration-tests --test integration

check:
	cargo check --manifest-path src-tauri/Cargo.toml

dev:
	npm run tauri dev

build:
	npm run build
