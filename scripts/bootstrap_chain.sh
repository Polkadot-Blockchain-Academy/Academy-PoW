#!/bin/bash

# set -x
set -eo pipefail

# TODO : use docker image
export NODE_BINARY=/tmp/academy-pow/target/release/academy-pow

# Sr25519 key for block mining

OUTPUT=$($NODE_BINARY key generate --scheme Sr25519 --output-type json)
NODE_SEED=$(echo $OUTPUT | jq .secretPhrase)
NODE_ACCOUNT_ID=$(echo $OUTPUT | jq .ss58PublicKey)

echo $NODE_SEED
echo $NODE_ACCOUNT_ID

# TODO generate P2P key
OUTPUT=$($NODE_BINARY key generate --scheme Ed25519 --output-type json)
SECRET_SEED=$(echo $OUTPUT | jq .secretSeed)

echo $SECRET_SEED

# TODO generate chainspec
# $NODE_BINARY build-spec --disable-default-bootnode --chain 'local' --chain_name 'Smartnet' > /tmp/academy-pow/chainspec.academy.json

# head /tmp/chainspec.academy.json

# --chain-id a0smnet --token-symbol SZERO --chain-name 'Aleph Zero Smartnet'

# TODO generate seconds set of p2p2 & miner keys

# TODO run chain

# TODO dockerize & compose
