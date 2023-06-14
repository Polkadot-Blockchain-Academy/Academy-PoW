#!/bin/bash

# set -x
set -eo pipefail

# TODO : use docker image
export NODE_BINARY=/tmp/academy-pow/target/release/academy-pow

# Sr25519 key for block mining

OUTPUT=$($NODE_BINARY key generate --scheme Sr25519 --output-type json)
PUBLIC_KEY=$(echo $OUTPUT | jq -r .publicKey)
PUBLIC_KEY=$(echo $PUBLIC_KEY | sed s/"0x"//)

echo "Node public key $PUBLIC_KEY"

# generate P2P key
OUTPUT=$($NODE_BINARY key generate --scheme Ed25519 --output-type json)
P2P_KEY_SEED=$(echo $OUTPUT | jq -r .secretSeed)
P2P_KEY_SEED=$(echo $P2P_KEY_SEED | sed s/"0x"//)

echo "P2P key seed: $P2P_KEY_SEED"
# 12D3KooWC2Q5RggT5nFCdJRzoZiknQ599NKYAsDW6bY5WpRqw9R5

# TODO alter command to allow passing osme more custom args
# generate chainspec
$NODE_BINARY build-spec --disable-default-bootnode --chain 'local' > /tmp/academy-pow/chainspec.academy.json

head /tmp/academy-pow/chainspec.academy.json

# purge chain
rm -rf /tmp/node01/chains/

# run first node
$NODE_BINARY \
  --sr25519-public-key $PUBLIC_KEY \
  --base-path /tmp/node01 \
  --chain /tmp/academy-pow/chainspec.academy.json \
  --port 30333 \
  --ws-port 9945 \
  --rpc-port 9933 \
  --rpc-methods Unsafe \
  --name Node01 \
  --node-key $P2P_KEY_SEED \
  --no-prometheus \
  --no-telemetry

# TODO generate seconds set of p2p2 & miner keys

# TODO run second node

# TODO dockerize & compose
