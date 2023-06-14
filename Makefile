clippy:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/target/ cargo clippy --all-targets -- --no-deps -D warnings

fmt:
	cargo fmt --all

test:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/target/ cargo test --verbose

watch:
	cargo watch -s 'WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/target/ cargo check' -c
