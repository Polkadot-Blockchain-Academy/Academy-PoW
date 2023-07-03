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

ALICE=$AUTHORITY

BOB=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
BOB_SEED=//Bob

EVE=5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw
EVE_SEED=//Eve

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

function transfer() {
  local address=$(get_address $1)
  local to=$2
  local amount=$3
  local suri="${4:-$AUTHORITY_SEED}"

  cd "$CONTRACTS_PATH"/psp22
  cargo_contract call --url "$NODE" --contract "$address" --suri "$suri" --message PSP22::transfer --args $to $amount --execute --skip-confirm
}

function approve() {
  local address=$(get_address $1)
  local spender=$2
  local amount=$3
  local suri="${4:-$AUTHORITY_SEED}"

  cd "$CONTRACTS_PATH"/psp22
  cargo_contract call --url "$NODE" --contract "$address" --suri "$suri" --message PSP22::approve --args $spender $amount --execute --skip-confirm
}

function transfer_from() {
  local address=$(get_address $1)
  local from=$2
  local to=$3
  local amount=$4
  local suri="${5:-$AUTHORITY_SEED}"

  cd "$CONTRACTS_PATH"/psp22
  cargo_contract call --url "$NODE" --contract "$address" --suri "$suri" --message PSP22::transfer_from --args $from $to $amount --execute --skip-confirm
}

function balance_of() {
  local  __resultvar=$1
  local address=$(get_address $2)
  local account=$3

  cd "$CONTRACTS_PATH"/psp22
  local result=$(cargo_contract call --url "$NODE" --contract "$address" --suri "$AUTHORITY_SEED" --message PSP22::balance_of --args $account --output-json | jq  -r '.data.Tuple.values' | jq '.[].UInt')
  eval $__resultvar="'$result'"
}

function deposit() {
  local issued_shares=$1
  local token_one=$(get_address $2)
  local token_two=$(get_address $3)
  local suri="${4:-$AUTHORITY_SEED}"
  local dex=$(get_address dex)

  # give DEX allowances to spend PSP22's when depositing
  cd "$CONTRACTS_PATH"/simple-dex
  local deposit=$(cargo_contract call --url "$NODE" --contract "$dex" --suri $suri --message deposit_given_shares --args $token_one $issued_shares --output-json | jq  -r '.data.Tuple.values' | jq '.[].Tuple.values' | jq '.[].UInt')
  approve $2 $dex $deposit

  cd "$CONTRACTS_PATH"/simple-dex
  local deposit=$(cargo_contract call --url "$NODE" --contract "$dex" --suri $suri --message deposit_given_shares --args $token_two $issued_shares --output-json | jq  -r '.data.Tuple.values' | jq '.[].Tuple.values' | jq '.[].UInt')
  approve $3 $dex $deposit

  # make the deposit
  cd "$CONTRACTS_PATH"/simple-dex
  cargo_contract call --url "$NODE" --contract $dex --message deposit --args $issued_shares --suri $suri --execute --skip-confirm
}

function out_given_in() {
  local token_in=$(get_address $1)
  local token_out=$(get_address $2)
  local amount_token_in=$3
  local dex=$(get_address dex)

  cd "$CONTRACTS_PATH"/simple-dex
  cargo_contract call --url "$NODE" --contract "$dex" --message out_given_in --args $token_in $token_out $amount_token_in  --suri "$AUTHORITY_SEED" --output-json
}

# TODO
function swap() {
  local token_in=$(get_address $1)
  local token_out=$(get_address $2)
  local amount_token_in=$3
  local suri="${4:-$AUTHORITY_SEED}"
  local dex=$(get_address dex)

  # give DEX allowance
  approve $1 $dex $amount_token_in

  # eval min

  let min_amount_token_out=$(out_given_in $1 $2 $amount_token_in | jq  -r '.data.Tuple.values' | jq '.[].Tuple.values' | jq '.[].UInt')

  echo "setting min_amount_out: $min_amount_token_out"

  cd "$CONTRACTS_PATH"/simple-dex
  cargo_contract call --url "$NODE" --contract "$dex" --message swap --args $token_in $token_out $amount_token_in $min_amount_token_out --suri $suri --execute --skip-confirm
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

# basic psp22 interactions

# Alice sends Bob 10 units of psp22 token
AMOUNT=10000000000000
echo "Transferring $AMOUNT of $(get_address token_one) from $ALICE to $BOB"
transfer token_one $BOB $AMOUNT

# Bob approves Eve to spend 1 unit of psp22 token on his behalf
AMOUNT=1000000000000
echo "Approving $EVE to spend up to $AMOUNT of token $(get_address token_one)"
approve token_one $EVE $AMOUNT $BOB_SEED

# Eve can now send Alice up to 1 unit of the token on behalf of Bob
AMOUNT=1000000000000
echo "$EVE sends $AMOUNT of token $(get_address token_one) to $ALICE from $BOB"
transfer_from token_one $BOB $ALICE $AMOUNT $EVE_SEED

balance_of BALANCE_OF token_one $BOB
echo "${BOB}'s balance of $(get_address token_one): $BALANCE_OF"

# basic DEX interactions

# seed DEX with some liquidity
AMOUNT=1000000000000
transfer token_one $(get_address dex) $AMOUNT
transfer token_two $(get_address dex) $AMOUNT

# make a deposit and get some LP shares
deposit 1000 token_one token_two

# swap 1 token_one for token_two
AMOUNT=1000000000000
swap token_one token_two $AMOUNT
