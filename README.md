# Academy PoW

A Proof of Work blockchain node for use in the Polkadot Blockchain Academy.
It happens to be based on Substrate, but no Substrate knowledge is required.
Students will use this node to start their own network, and perform hard, soft, and contentious forks.

Instructors, planning to host such an activity should see the docs on [setting up a bootnode](./SettingUpTheBootnode.md). TODO Expand that doc into a full guide to running the workshop.

## Building the Node

First you will need a [Substrate developer environment](https://docs.substrate.io/install/).

1. Cloning this repo
2. Running `cargo build --release`

## Run a Single Development Node

You can use a native binary if you built it in the previous section. Otherwise you can use Docker.

```sh
./target/release/academy-pow --dev
```

### Docker Single Node

If you use docker, you need to map the RPC port and specify `--rpc-external` so that the node listens on `0.0.0.0`.

Networking can be challenging for new Docker users, like me. If you are on linux, you may prefer to use `--network host` which allows the containerized node to operate directly on the local network. Consult the Docker docs for more details.

```sh
docker run -p 9944:9944 ghcr.io/polkadot-blockchain-academy/academy-pow:main --dev --rpc-external
```

## MultiNode Testnet

When using the local networking, you can use the `--discover-local` flag to discover peers on your local network.

```sh
# Start the first node.
# Same as a single node network above.
./target/release/academy-pow --dev --mining-algo keccak

# Start the second node
# Choose a non-default rpc port because the default port is taken by the first node.
./target/release/academy-pow --dev --mining-algo sha3 --rpc-port 9955 --discover-local
```

### Docker Multi Node

With Docker, we can't rely on local peer discovery. Instead, copy the bootnode address from the logs of the first node.
Look for a line like this

```
🏷  Local node identity is: 12D3KooWR2y4tUSpqrPgvSoqfcx9bT8aV2LwHR3BJkWJFTBjZMbZ 
```

```sh
# Start the first node.
# Same as a single node network above.
docker run -p 9944:9944 ghcr.io/polkadot-blockchain-academy/academy-pow:main \
    --dev \
    --mining-algo keccak \
    --rpc-external

# Start the second node.
# Publish all exposed ports to RANDOM ports on the host.
# Specify the bootnode address you copied from the first node
docker run --publish-all ghcr.io/polkadot-blockchain-academy/academy-pow:main \
    --dev \
    --mining-algo sha3 \
    --rpc-external \
    --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWR2y4tUSpqrPgvSoqfcx9bT8aV2LwHR3BJkWJFTBjZMbZ

```

Actually, even the above command doesn't get the nodes to peer properly. The problem is that I don't know the proper ip address to use for the bootnode. It isn't `127.0.0.1` because that is in the container still. See https://stackoverflow.com/questions/24319662/from-inside-of-a-docker-container-how-do-i-connect-to-the-localhost-of-the-mach?noredirect=1&lq=1 for possible solutions using docker compose.

## More Help

```sh
academy-pow --help
```