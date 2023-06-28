fmt:
	cd roulette && cargo fmt --all

check:
	cd roulette && WASM_BUILD_WORKSPACE_HINT=${PWD}/roulette CARGO_TARGET_DIR=/tmp/academy-pow/contracts/roulette/target/ cargo contract check

test:
	cd roulette && WASM_BUILD_WORKSPACE_HINT=${PWD}/roulette CARGO_TARGET_DIR=/tmp/academy-pow/contracts/roulette/target/ cargo test --verbose

watch:
	cd roulette && cargo watch -s 'WASM_BUILD_WORKSPACE_HINT=${PWD}/roulette CARGO_TARGET_DIR=/tmp/academy-pow/contracts/roulette/target/ cargo contract check' -c

release:
	cd roulette && WASM_BUILD_WORKSPACE_HINT=${PWD}/roulette CARGO_TARGET_DIR=/tmp/academy-pow/contracts/roulette/target/ cargo build --release
