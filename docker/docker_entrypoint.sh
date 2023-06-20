#!/usr/bin/env bash

set -euo pipefail

# load env variables from a file if ENV_FILE is set
if [[ -n "${ENV_FILE:-}" ]] && [[ -f "${ENV_FILE}" ]]; then
  set -o allexport
  source ${ENV_FILE}
  set +o allexport
fi

# env variables with defaults

PUBLIC_KEY=${PUBLIC_KEY:?'Public key should be specified'}
NODE_KEY_FILE=${NODE_KEY_FILE:?'Node key file should be specified'}
BASE_PATH=${BASE_PATH:?'Base path should be specified'}
CHAIN=${CHAIN:?'Chain should be specified'}
PORT=${PORT:-30333}
WS_PORT=${WS_PORT:-9944}
RPC_PORT=${RPC_PORT:-9933}
NAME=${NAME:?'Name should be specified'}

# booleans
VALIDATOR=${VALIDATOR:-true}
ALLOW_PRIVATE_IP=${ALLOW_PRIVATE_IP:-true}
DISCOVER_LOCAL=${DISCOVER_LOCAL:-false}
INSTANT_SEAL=${INSTANT_SEAL:-false}

ARGS=(
  --execution Wasm
  --sr25519-public-key "${PUBLIC_KEY}"
  --base-path "${BASE_PATH}"
  --chain "${CHAIN}"
  --port "${PORT}"
  --ws-port "${WS_PORT}"
  --rpc-port "${RPC_PORT}"
  --rpc-cors all
  --no-mdns
  --unsafe-ws-external
  --unsafe-rpc-external
  --name "${NAME}"
  --node-key-file "${NODE_KEY_FILE}"
  --no-prometheus
  --no-telemetry
  --enable-log-reloading
)

if [[ "true" == "$VALIDATOR" ]]; then
  ARGS+=(--validator)
fi

if [[ -n "${BOOT_NODES:-}" ]]; then
  ARGS+=(--bootnodes ${BOOT_NODES})
fi

if [[ -n "${RESERVED_NODES:-}" ]]; then
  ARGS+=(--reserved-nodes "${RESERVED_NODES}")
fi

if [[ -n "${RESERVED_ONLY:-}" ]]; then
  ARGS+=(--reserved-only)
fi

if [[ -n "${PUBLIC_ADDR:-}" ]]; then
  ARGS+=(--public-addr "${PUBLIC_ADDR}")
fi

if [[ "true" == "$ALLOW_PRIVATE_IP" ]]; then
  ARGS+=(--allow-private-ip)
fi

if [[ "true" == "$DISCOVER_LOCAL" ]]; then
  ARGS+=(--discover-local)
fi

if [[ "true" == "$INSTANT_SEAL" ]]; then
  ARGS+=(--instant-seal)
fi

echo "${CUSTOM_ARGS}" | xargs academy-pow-node "${ARGS[@]}"
