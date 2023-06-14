clippy:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/target/ cargo clippy

fmt:
	cargo fmt

watch:
	cargo watch -s 'WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/target/ cargo check' -c
