#!/bin/bash

# set -x
set -eo pipefail

# --- CONSTANTS

export NODE_IMAGE=academy-pow-node:latest
export BASE_PATH=/tmp/academy-pow
export ALICE=0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
export BOB=0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
export EVE=0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB

# --- FUNCTIONS

function bootstrap_node() {
  local __account_id_resultvar=$1
  local __peer_id_resultvar=$2

  local output=$(docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
                        "academy-pow key generate --scheme Sr25519 --output-type json")

  local account_id=$(echo $output | jq -r .ss58PublicKey)

  mkdir -p $BASE_PATH/$account_id
  echo $(echo $output | jq -r .secretPhrase) > $BASE_PATH/$account_id/account_secret_phrase.txt

  docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
         "academy-pow key generate-node-key --file /data/$account_id/p2p_secret.txt"

  local peer_id=$(docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
                         "academy-pow key inspect-node-key --file /data/$account_id/p2p_secret.txt")

  eval $__account_id_resultvar="'$account_id'"
  eval $__peer_id_resultvar="'$peer_id'"
}

# --- RUN

bootstrap_node NODE01_ACCOUNT_ID NODE01_PEER_ID
echo "Node01 with peer id $NODE01_PEER_ID has $NODE01_ACCOUNT_ID as public key"

bootstrap_node NODE02_ACCOUNT_ID NODE02_PEER_ID
echo "Node02 with peer id $NODE02_PEER_ID has $NODE02_ACCOUNT_ID as public key"

# generate chainspec
docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
       "academy-pow build-spec --disable-default-bootnode --chain-name 'Academy PoW Local' --chain-id 'academy_pow_local' --endowed-accounts '$ALICE,$BOB,$EVE' --initial-difficulty 4000000 > /data/chainspec.academy.json"

export NODE01_ACCOUNT_ID=$NODE01_ACCOUNT_ID
export NODE01_PEER_ID=$NODE01_PEER_ID

export NODE02_ACCOUNT_ID=$NODE02_ACCOUNT_ID
export NODE02_PEER_ID=$NODE02_PEER_ID

docker network create academy-network || true
docker-compose -f docker/academy-chain-compose.yml up --remove-orphans

exit $?
