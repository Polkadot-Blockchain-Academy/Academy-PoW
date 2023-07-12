check:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/target/ cargo check

clippy:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/target/ cargo clippy --all-targets -- --no-deps -D warnings

fmt:
	cargo fmt --all

test:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/target/ cargo test --verbose

watch:
	cargo watch -s 'WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/target/ cargo clippy' -c

release:
	WASM_BUILD_WORKSPACE_HINT=${PWD} CARGO_TARGET_DIR=/tmp/academy-pow/target/ cargo build --release

image:
	mkdir --parents /tmp/academy-pow/docker/ && \
	cp docker/.dockerignore /tmp/academy-pow && \
	docker build --tag academy-pow-node:latest -f ${PWD}/docker/Dockerfile /tmp/academy-pow  && \
	docker image tag academy-pow-node:latest academy-pow-node:$(shell git rev-parse --short=10 HEAD)

chain:
	./scripts/bootstrap_chain.sh
