watch:
	cargo watch -s 'WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/contracts/psp22/target/ cargo contract check' -c

release:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/contracts/psp22/target/ cargo contract build --release
