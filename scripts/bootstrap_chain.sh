#!/bin/bash

# set -x
set -eo pipefail

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

# reserved_nodes

# run first node
$NODE_BINARY \
  --sr25519-public-key $PUBLIC_KEY \
  --base-path /tmp/node01 \
  --chain /tmp/academy-pow/chainspec.academy.json \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --rpc-methods Unsafe \
  --unsafe-ws-external \
  --rpc-cors all \
  --rpc-external \
  --name Node01 \
  --node-key-file /tmp/node01/p2p_secret.txt \
  --no-prometheus \
  --no-telemetry \
  --enable-log-reloading \
  --allow-private-ip \
  -lruntime::contracts=debug,sync=trace \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/$PEER_ID

# TODO generate seconds set of p2p2 & miner keys

# TODO run second node

# TODO dockerize & compose
