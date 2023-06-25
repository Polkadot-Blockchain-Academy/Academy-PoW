# Polkadot Scripts README

This repository contains two scripts to interact with the Polkadot network: `balance.js` and `transfer.js`.

## Prerequisites

These scripts require [Node.js](https://nodejs.org/) to run. They also depend on the `@polkadot/api` and `@polkadot/keyring` libraries. You can install these dependencies by running:

```bash
npm install @polkadot/api @polkadot/keyring
```


## balance.js

The `balance.js` script fetches and prints the balance of a specified address on the Polkadot network.

### Usage

Run the `balance.js` script with one argument: the address whose balance you want to fetch.

For example:

```bash
$ node balance.js 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
```

This will print the free, reserved, and total balance of the account with the address `0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac`.

## transfer.js

The `transfer.js` script sends a specified amount of units from one account to another on the Polkadot network.

### Usage

Run the `transfer.js` script with three arguments: the private key of the sender's account, the address of the recipient's account, and the amount of units to transfer. 

For example:

```bash
$ node transfer.js 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 0xA63133446f5ef88800640AD669FA8F4A44C5000a 12345
```


This will transfer 12345 units from the account with the private key `0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133` to the account with the address `0xA63133446f5ef88800640AD669FA8F4A44C5000a`. The transaction hash will be printed upon completion.

**Please note that these scripts are meant for educational purposes and should be used responsibly.**
