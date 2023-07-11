#!/bin/bash

set -x
set -eo pipefail

# --- GLOBAL CONSTANTS ---

TOML_FILE="Cargo.toml"

# --- FUNCTIONS ---

function parse_toolchain() {
  local toml_file=$1
  local  __resultvar=$2

  channel=$(cat $toml_file | grep channel)
  channel=${channel:10}
  # Remove leading and trailing whitespace, and quotes from the parsed value
  channel=$(echo "$channel" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//')
  channel=${channel}-x86_64-unknown-linux-gnu

  eval $__resultvar="'$channel'"
}

# --- RUN ---

parse_toolchain "contracts/rust-toolchain.toml" RUST_CONTRACTS_TOOLCHAIN

# Read the TOML file and extract the `exclude` entries
packages=$(awk -F ' *= *' '/^exclude *= *\[/ {found=1} found && /^\]$/ {found=0} found' "$TOML_FILE")

packages="$(echo ${packages} | sed 's/[][,]/ /g' | sed 's/\s\+/\n/g' | sed '/^$/d')"

# Remove leading and trailing whitespace, and quotes from the entries
packages=$(echo "$packages" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//')

packages="${packages//'%0A'/$'\n'}"

# Remove the key
packages=${packages:10}

for p in ${packages[@]}; do
  echo "Checking package $p ..."
  pushd "$p"

  if [[ $p =~ .*contracts.* ]]; then
    cargo +${RUST_CONTRACTS_TOOLCHAIN} contract build --release
    cargo test
  fi

  cargo fmt --all --check
  popd
done
