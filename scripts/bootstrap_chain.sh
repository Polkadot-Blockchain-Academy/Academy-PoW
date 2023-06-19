#!/bin/bash

# set -x
set -eo pipefail

<<<<<<< HEAD
# TODO : use docker image
export NODE_BINARY=/tmp/academy-pow/target/release/academy-pow

# Sr25519 key for block mining
OUTPUT=$($NODE_BINARY key generate --scheme Sr25519 --output-type json)
PUBLIC_KEY=$(echo $OUTPUT | jq -r .publicKey)
PUBLIC_KEY=$(echo $PUBLIC_KEY | sed s/"0x"//)
# ACCOUNT_ID=$(echo $OUTPUT | jq -r .ss58PublicKey)

echo "Node public key: $PUBLIC_KEY"
# echo "Node AccountId $ACCOUNT_ID"

# P2P key for networking
$NODE_BINARY key generate-node-key --file /tmp/node01/p2p_secret.txt
PEER_ID=$($NODE_BINARY key inspect-node-key --file /tmp/node01/p2p_secret.txt)

echo "Node p2p peer id: $PEER_ID"

# generate chainspec
# TODO alter the default command to allow passing some custom args like chain-id
$NODE_BINARY build-spec --disable-default-bootnode --chain 'local' > /tmp/academy-pow/chainspec.academy.json

# head /tmp/academy-pow/chainspec.academy.json

# purge chain
rm -rf /tmp/node01/chains/

# run first node
$NODE_BINARY \
  --sr25519-public-key $PUBLIC_KEY \
  --validator \
  --base-path /tmp/node01 \
  --chain /tmp/academy-pow/chainspec.academy.json \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --rpc-methods Unsafe \
  --rpc-external \
  --rpc-cors all \
  --name Node01 \
  --node-key-file /tmp/node01/p2p_secret.txt \
  --no-prometheus \
  --no-telemetry \
  --enable-log-reloading \
  --allow-private-ip \
  --log runtime::contracts=debug,sync=trace,pow=trace,manual-seal=trace \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/$PEER_ID \
  --reserved-nodes /ip4/127.0.0.1/tcp/30333/p2p/$PEER_ID

# TODO generate seconds set of p2p2 & miner keys

# TODO run second node

# TODO dockerize & compose
=======
# --- CONSTANTS

export NODE_IMAGE=academy-pow-node:latest

export BASE_PATH=/tmp/academy-pow

# --- FUNCTIONS

function bootstrap_node() {
  local __public_key_resultvar=$1
  local __account_id_resultvar=$2
  local __peer_id_resultvar=$3

  local output=$(docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
                        "academy-pow-node key generate --scheme Sr25519 --output-type json")

  local public_key=$(echo $output | jq -r .publicKey | sed s/"0x"//)
  local account_id=$(echo $output | jq -r .ss58PublicKey)

  mkdir -p $BASE_PATH/$account_id
  echo $(echo $output | jq -r .secretPhrase) > $BASE_PATH/$account_id/account_secret_phrase.txt

  docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
         "academy-pow-node key generate-node-key --file /data/$account_id/p2p_secret.txt"

  local peer_id=$(docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
                         "academy-pow-node key inspect-node-key --file /data/$account_id/p2p_secret.txt")

  eval $__public_key_resultvar="'$public_key'"
  eval $__account_id_resultvar="'$account_id'"
  eval $__peer_id_resultvar="'$peer_id'"
}

# --- RUN

bootstrap_node NODE01_PUBLIC_KEY NODE01_ACCOUNT_ID NODE01_PEER_ID
echo "Node01 with peer id $NODE01_PEER_ID has $NODE01_ACCOUNT_ID($NODE01_PUBLIC_KEY) as pk"

bootstrap_node NODE02_PUBLIC_KEY NODE02_ACCOUNT_ID NODE02_PEER_ID
echo "Node02 with peer id $NODE02_PEER_ID has $NODE02_ACCOUNT_ID($NODE02_PUBLIC_KEY) as pk"

# generate chainspec
docker run --rm -v $BASE_PATH:/data --entrypoint "/bin/sh" -e RUST_LOG=debug "${NODE_IMAGE}" -c \
       "academy-pow-node build-spec --disable-default-bootnode --chain-name 'Academy PoW Local' --chain-id 'academy_pow_local' --endowed-accounts '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty,5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw,$NODE01_ACCOUNT_ID,$NODE02_ACCOUNT_ID' --initial-difficulty 1 > /data/chainspec.academy.json"

export NODE01_ACCOUNT_ID=$NODE01_ACCOUNT_ID
export NODE01_PUBLIC_KEY=$NODE01_PUBLIC_KEY
export NODE01_PEER_ID=$NODE01_PEER_ID

export NODE02_ACCOUNT_ID=$NODE02_ACCOUNT_ID
export NODE02_PUBLIC_KEY=$NODE02_PUBLIC_KEY
export NODE02_PEER_ID=$NODE02_PEER_ID

docker network create academy-network || true
docker-compose -f docker/academy-chain-compose.yml up --remove-orphans

exit $?
>>>>>>> dockerize-node
