#!/usr/bin/env bash

# seed associated with ethereum-style derivation of Alith dev account. I found it in
# https://github.com/polkadot-js/common/blob/0ec894b7324ac048d9f521a889e5349d59ad5696/packages/keyring/src/testingPairs.spec.ts#L43C41-L43C107
ALITH_SEED="0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"

set -eo pipefail

mkdir --parents test-contract/
cd test-contract/

CC="docker run --network host --rm -v $(pwd):/sources paritytech/contracts-ci-linux:9a513893-20230620"

# Create a new contract-
$CC cargo contract new --target-dir /sources flipper

# Build the contract
$CC cargo contract build --release --manifest-path=/sources/flipper/Cargo.toml

# Upload and instantiate code
# Cargo contract does not currently support AccountId20, so we use Polkadot js for now.
# https://github.com/paritytech/cargo-contract/issues/1182

# ADDRESS=$($CC cargo contract instantiate --suri //Alice --skip-confirm --args false -x --output-json /sources/flipper/target/ink/flipper.wasm | jq -jr '.contract')
polkadot-js-api --seed $ALITH_SEED --sign ethereum tx.contracts.instantiateWithCode 0 '{"refTime": 1000000000, "proofSize": 1000000000}' 1000000000000 @flipper/target/ink/flipper.wasm 0xed4b9d1b 0x
# TODO parse the contract address from the output
# "event": {
#     "index": "0x0003",
#     "data": [
#     "0x25379eeFE1fdd60c602FB0234F38F373c7F5F164"
#     ]
# },

# Call the contract
#$CC cargo contract call --suri //Alice --contract $ADDRESS --message flip --manifest-path=/sources/flipper/Cargo.toml -x --skip-confirm
polkadot-js-api --seed $ALITH_SEED --sign ethereum tx.contracts.call 0x25379eeFE1fdd60c602FB0234F38F373c7F5F164 0 '{"refTime": 1000000000, "proofSize": 1000000000}' 1000000000000 0x633aa551

