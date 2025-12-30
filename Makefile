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

run: 
	RUST_LOG=debug ./src-tauri/target/release/bundle/macos/SSLBoard.app/Contents/MacOS/SSLBoard-desktop

clean-data:
	rm ~/Library/Application Support/com.sslboard.desktop/*.sqlite
	security delete-generic-password -s "sslboard-desktop" -a "master_key"