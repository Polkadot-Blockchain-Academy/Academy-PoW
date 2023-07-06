#!/bin/bash

# set -x
set -eo pipefail

# --- GLOBAL CONSTANTS

BETTING_PERIOD_LENGTH=100
MAXIMAL_NUMBER_OF_BETS=5
MINIMAL_BET_AMOUNT=1000000000000

TOTAL_SUPPLY=100000000000000000000

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

# compile & deploy contracts

cd "$CONTRACTS_PATH"/roulette
cargo_contract build --release
ROULETTE_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')

ROULETTE=$(cargo_contract instantiate --url "$NODE" --constructor new --args $BETTING_PERIOD_LENGTH $MAXIMAL_NUMBER_OF_BETS $MINIMAL_BET_AMOUNT --suri "$AUTHORITY_SEED" --value 100000000000000 --skip-confirm --output-json --execute | jq -r '.contract')

cd "$CONTRACTS_PATH"/psp22
cargo_contract build --release
PSP22_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')

TOKEN_ONE=$(cargo_contract instantiate --url "$NODE" --constructor new --args $TOTAL_SUPPLY --suri "$AUTHORITY_SEED" --salt 0x0001 --skip-confirm --output-json --execute | jq -r '.contract')
TOKEN_TWO=$(cargo_contract instantiate --url "$NODE" --constructor new --args $TOTAL_SUPPLY --suri "$AUTHORITY_SEED" --salt 0x0002 --skip-confirm --output-json --execute | jq -r '.contract')

cd "$CONTRACTS_PATH"/simple-dex
cargo_contract build --release
DEX_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')

DEX=$(cargo_contract instantiate --url "$NODE" --constructor new --args 0 "[$TOKEN_ONE,$TOKEN_TWO]" --suri "$AUTHORITY_SEED" --salt 0x0001 --skip-confirm --output-json --execute | jq -r '.contract')

cd "$CONTRACTS_PATH"/old_a
cargo_contract build --release
OLD_A_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')
OLD_A=$(cargo_contract instantiate --url "$NODE" --constructor new --suri "$AUTHORITY_SEED" --skip-confirm --output-json --execute | jq -r '.contract')

cd "$CONTRACTS_PATH"/new_a
cargo_contract build --release
NEW_A_CODE_HASH=$(cargo_contract upload --url "$NODE" --suri "$AUTHORITY_SEED" --output-json --execute | jq -s . | jq -r '.[1].code_hash')

# spit adresses to a JSON file
cd "$CONTRACTS_PATH"

jq -n \
   --arg roulette "$ROULETTE" \
   --arg roulette_code_hash "$ROULETTE_CODE_HASH" \
   --arg token_one "$TOKEN_ONE" \
   --arg token_two "$TOKEN_TWO" \
   --arg psp22_code_hash "$PSP22_CODE_HASH" \
   --arg dex "$DEX" \
   --arg dex_code_hash "$DEX_CODE_HASH" \
   --arg old_a_code_hash "$OLD_A_CODE_HASH" \
   --arg old_a "$OLD_A" \
   --arg new_a_code_hash "$NEW_A_CODE_HASH" \
   '{
      roulette: $roulette,
      roulette_code_hash: $roulette_code_hash,
      token_one: $token_one,
      token_two: $token_two,
      psp22_code_hash: $psp22_code_hash,
      dex: $dex,
      dex_code_hash: $dex_code_hash,
      old_a_code_hash: $old_a_code_hash,
      old_a: $old_a,
      new_a_code_hash: $new_a_code_hash
    }' > addresses.json

cat addresses.json

exit $?
