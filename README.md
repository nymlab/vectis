# Vectis - Smart Contract Wallet

[![Cosmowasm 0.20.0](https://img.shields.io/badge/CosmWasm-0.20.0-green)](https://github.com/CosmWasm/wasmd/releases)

## Overview

Smart Contract Wallet allows user to interact with DAPPs on the blockchain with the same amount of autonomy of a classic non-custodial solution, but with more flexibility by providing functionalities designed to serve the user.
SCW also provide functions that allow businesses to satisfy regulatory requirements regarding support of users, transparency, separation of control duties and verifiability.

### Design

SCW is designed to provide the user with confidence whilst interacting with the DApps by providing the most amount of control yet allowing recoverability.
It also enables companies to drive mass adoption of blockchain-based services by their customers, providing a better user experience, solving the problems of buying gas in advance, increasing the resilience, security and verifiability of their solutions.

At the core, the SCW builds on [cw-1 specifications](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw1/README.md) for proxy contracts, with the addition of roles and functionalities listed below.

We also provide a [factory](/contracts/factory/src/contract.rs) contract to instantiate the SCW,
this allows service providers to help users instantiate SCW and keep track of the wallets they potentially are guardians / relayers of.

[cw-1 specifications]: (https://crates.io/crates/cw1)

#### Roles

There are 3 roles in a SCW:

1. [_user:_](/contracts/README.md#User) the address that this wallet services, they have full control over the roles assignment and wallet operations
1. [_guardians:_](/contracts/README.md#Guardians) the addresses appointed by the user to protect the user (via key recovery and / or account freezing)
1. [_relayers_](/contracts/README.md#Relayers) the addresses appointed by the user to allow for user's off-chain transaction signatures be committed on-chain with gas.

## Hack

### Contracts Code Test

```sh
cd contracts
cargo test
```

### Local Node

#### 1. Set up `wasmd` locally, which has the Cosmwasm module and a CLI

```sh
git clone https://github.com/CosmWasm/wasmd.git
cd wasmd
# replace the v0.18.0 with the most stable version on https://github.com/CosmWasm/wasmd/releases
git checkout v0.20.0
make install

# verify the installation
wasmd version
```

> **INFO:** `make install` will copy wasmd to `$HOME/go/bin` or the default directory for binaries from Go,
> please make sure that is in your `PATH`.

#### 2. Start the node

```sh
./local-node-setup.sh
```

#### 3. Compile smart contracts

```sh
# compile the contracts with suitable RUSTFLAGS for compact wasm code size
RUSTFLAGS='-C link-arg=-s' cargo wasm-factory
RUSTFLAGS='-C link-arg=-s' cargo wasm-proxy
```

### Interact with the Local Node

We are using [cosmJS](https://github.com/cosmos/cosmjs) to test the smart contracts with the local node.
The testing framework used is [Jasmine](https://jasmine.github.io/)
The JS app is in the `js-app` directory.

Please ensure you have set up the `.env.dev` file according to the `example.env`.
More details of the roles are in `./local-node-setup.sh`.

> Note: _The tests include deploying the contracts_

```sh
cd js-app
npm i           # Install all dependencies
npm test        # Run tests
```

### Gitpod integration

The `/contracts` directory is generated from the [cosmwasm template](https://github.com/CosmWasm/cw-template) which provides config for gitpod.

[Gitpod](https://www.gitpod.io/) container-based development platform will be enabled on your project by default.

Workspace contains:

- **rust**: for builds
- [wasmd](https://github.com/CosmWasm/wasmd): for local node setup and client
- **jq**: shell JSON manipulation tool

Follow [Gitpod Getting Started](https://www.gitpod.io/docs/getting-started) and launch your workspace.
