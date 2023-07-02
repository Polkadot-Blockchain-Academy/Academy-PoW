#!/bin/bash

# set -x
set -eo pipefail

# --- GLOBAL CONSTANTS

CONTRACTS_PATH=${PWD}/contracts
ADDRESSES_FILE=${PWD}/contracts/addresses.json
NODE=ws://127.0.0.1:9944
INK_DEV_IMAGE=public.ecr.aws/p6e8q1z1/ink-dev:1.5.0

AUTHORITY=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
AUTHORITY_SEED=//Alice

FILIP=5D2ZNVZ5xnMrs9SRJvVSe9ACnsbhTwmdC2PmeC1MXJVt8Drf
FILIP_SEED=//Filip

# ---  FUNCTIONS

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

function get_address {
  local contract_name=$1
  cat $ADDRESSES_FILE | jq --raw-output ".$contract_name"
}

function is_betting_over() {
  local roulette_address=$(get_address roulette)
  local  __resultvar=$1

  cd "$CONTRACTS_PATH"/roulette
  local result=$(cargo_contract call --url "$NODE" --contract "$roulette_address" --suri "$AUTHORITY_SEED" --message is_betting_period_over --output-json | jq  -r '.data.Tuple.values' | jq '.[].Bool')  
  eval $__resultvar="'$result'"
}

function spin() {
  local roulette_address=$(get_address roulette)
  cd "$CONTRACTS_PATH"/roulette
  cargo_contract call --url "$NODE" --contract "$roulette_address" --suri "$AUTHORITY_SEED" --message spin --execute --skip-confirm
}

function place_bet() {
  local roulette_address=$(get_address roulette)
  cd "$CONTRACTS_PATH"/roulette

  cargo_contract call --url "$NODE" --contract "$roulette_address" --suri "$AUTHORITY_SEED" --message place_bet --args "Even" --execute --skip-confirm --value 1000000000000
}

# --- RUN

run_ink_dev

# eval basic roulette interactions

is_betting_over IS_OVER

if [ $IS_OVER == "true" ]; then
  echo "Past betting period, resetting"
  spin
  echo "Placing a bet"
  place_bet
else
  echo "Placing a bet"
  place_bet
fi

# basic psp22 interaction
