# Academy PoW

![Unit Tests](https://github.com/Polkadot-Blockchain-Academy/Academy-PoW/actions/workflows/unit-tests-suite.yml/badge.svg)
![E2E Tests](https://github.com/Polkadot-Blockchain-Academy/Academy-PoW/actions/workflows/e2e-tests-suite.yml/badge.svg)

A Proof of Work blockchain node for use in the Polkadot Blockchain Academy.
It happens to be based on Substrate, but no Substrate knowledge is required.
Students will use this node to start their own network, perform hard and soft forks, and execute smart contracts.

Instructors, planning to host such an activity should see the docs on [setting up a bootnode](./SettingUpTheBootnode.md).

## Connecting the UI

You can connect a UI to a public node by going to https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Facademy.bootnodes.net%2Falice#/explorer

## Building the Node

To help decentralize the network, you can compile the node by

1. Cloning this repo
2. Running `cargo build --release`

## Connect UI to Your Node

Once you have your own node running, you can connect your UI to your own node instead of the public bootnode.

In the left panel of the UI, switch to the `Local Node` endpoint.

## More Help

This code will be used primarily as an in-class activity that is instructor led, so just wait for details in class.
