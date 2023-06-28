#!/bin/bash

# set -x
set -eo pipefail

# --- GLOBAL CONSTANTS

BETTING_PERIOD_LENGTH=5
MAXIMAL_NUMBER_OF_BETS=5
MINIMAL_BET_AMOUNT=1000000000000

INK_DEV_IMAGE=public.ecr.aws/p6e8q1z1/ink-dev:1.5.0
NODE=ws://127.0.0.1:9944
AUTHORITY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
AUTHORITY_SEED=//Alice

CONTRACTS_PATH=$(pwd)/contracts

# --- FUNCTIONS

function run_ink_dev() {
  docker start ink_dev || docker run \
                                 --network host \
                                 -v "${CONTRACTS_PATH}:/code" \
                                 -v ~/.cargo/git:/usr/local/cargo/git \
                                 -v ~/.cargo/registry:/usr/local/cargo/registry \
                                 -u "$(id -u):$(id -g)" \
                                 --name ink_dev \
                                 --platform linux/amd64 \
                                 --detach \
                                 --rm public.ecr.aws/p6e8q1z1/ink-dev:1.5.0 sleep 1d
}

function cargo_contract() {
  contract_dir=$(basename "${PWD}")
  docker exec \
         -u "$(id -u):$(id -g)" \
         -w "/code/$contract_dir" \
         -e RUST_LOG=info \
         ink_dev cargo contract "$@"
}

# --- RUN

run_ink_dev

# compile contracts

cd "$CONTRACTS_PATH"/roulette
cargo_contract build --release
ROULETTE_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')

ROULETTE=$(cargo_contract instantiate --url "$NODE" --constructor new --args $BETTING_PERIOD_LENGTH $MAXIMAL_NUMBER_OF_BETS $MINIMAL_BET_AMOUNT --suri "$AUTHORITY_SEED" --skip-confirm --output-json --execute | jq -r '.contract')

# spit adresses to a JSON file
cd "$CONTRACTS_PATH"

jq -n \
   --arg roulette "$ROULETTE" \
   --arg roulette_code_hash "$ROULETTE_CODE_HASH" \
   '{
      roulette: $roulette,
      roulette_code_hash: $roulette_code_hash
    }' > addresses.json

cat addresses.json

exit $?
