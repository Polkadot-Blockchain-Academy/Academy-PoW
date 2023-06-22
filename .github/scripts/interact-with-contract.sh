#!/usr/bin/env bash

mkdir --parents test-contract/
cd test-contract/

alias cc='docker run --network host -u 1000:1000 --rm -it -v $(pwd):/sources paritytech/contracts-ci-linux:9a513893-20230620'

# Create a new contract
cc cargo contract new --target-dir /sources flipper

# Build the contract
cc cargo contract build --release --manifest-path=/sources/flipper/Cargo.toml

# Upload and instantiate code
ADDRESS=$(cc cargo contract instantiate --suri //Alice --skip-confirm --args false -x --output-json /sources/flipper/target/ink/flipper.wasm | jq -jr '.contract')

# Call the contract
cc cargo contract call --suri //Alice --contract $ADDRESS --message flip --manifest-path=/sources/flipper/Cargo.toml -x --skip-confirm
