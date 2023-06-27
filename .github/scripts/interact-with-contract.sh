#!/usr/bin/env bash

set -eo pipefail

mkdir --parents test-contract/
cd test-contract/

CC="docker run --network host --rm -v $(pwd):/sources paritytech/contracts-ci-linux:9a513893-20230620"

# Create a new contract-
$CC cargo contract new --target-dir /sources flipper

# Build the contract
$CC cargo contract build --release --manifest-path=/sources/flipper/Cargo.toml

# Upload and instantiate code
ADDRESS=$($CC cargo contract instantiate --suri //Alice --url ws://localhost:9933 --skip-confirm --args false -x --output-json /sources/flipper/target/ink/flipper.wasm | jq -jr '.contract')

# Call the contract
$CC cargo contract call --suri //Alice --url ws://localhost:9933 --contract $ADDRESS --message flip --manifest-path=/sources/flipper/Cargo.toml -x --skip-confirm
