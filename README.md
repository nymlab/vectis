# Vectis - Smart Contract Wallet

[![Cosmowasm 0.20.0](https://img.shields.io/badge/CosmWasm-0.20.0-green)](https://github.com/CosmWasm/wasmd/releases)
[![codecov_img](https://img.shields.io/codecov/c/github/nymlab/vectis)](https://img.shields.io/codecov/c/github/nymlab/vectis)

- [Website](https://vectis.nymlab.it/)
- [Discord](https://discord.gg/xp3vFSAMgS)

## Overview

Smart Contract Wallet allows user to interact with DAPPs on the blockchain with the same amount of autonomy of a classic non-custodial solution, but with more flexibility by providing functionalities designed to serve the user.
SCW also provide functions that allow businesses to satisfy regulatory requirements regarding support of users, transparency, separation of control duties and verifiability.

VectisDAO is the organisation that provides governance to the this infrastructure.
Every wallet has a Govec token that is minted during wallet creation and can be staked to vote.
VectisDAO lives on [Juno Network] and leverages the [DAO DAO] stack.

Thanks to [IBC], Vectis wallets can also be deployed on other chains.
From the perspective of DAO participation,
there is no difference between a wallet on [Juno Network] or others.
Staking and voting will be done via IBC calls from the wallet.
Please see [IBC Architecture] for details.

[Juno Network]:
[DAO DAO]:
[IBC Architecture]:

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

#### IBC Architecture

##### Juno Network Components

- DAODAO contracts: [cw-core], [cw-proposal-single], [cw20-staked-balance-voting], [cw20-stake]
- [Factory]
- [Proxy]
- [Govec]: This allows balance of Govec token for proxy addresses of different chains
- [IBC Host]: This is inspire by the `simple-ica-host` provided in the [Confio demo],
  it stores, in its states, of the acceptable [`connection-id`] and [`port-id`] of a Vectis [`IBC Remote`] contract as voted in by the VectisDAO,
  it will effectively communitcate to the Govec contract for new voting tokens and their votes etc

##### Remote Network Components

- [Factory Remote]: Same as [Factory] with the addition of messages for the [IBC Remote] contract
- [Proxy Remote]: Same as [Proxy] with the addition of [Govec] operations,
  this is to minimise each proxy having to create their own channel
- [IBC Remote]: This is inspired by the `simple-ica-remote` provided in the [Confio demo],
  its main job is to route IBC messages to the [IBC Host] for minting new Govec tokens and other Govec operations

[factory]: https://github.com/nymlab/vectis/tree/main/contracts/factory
[proxy]: https://github.com/nymlab/vectis/tree/main/contracts/proxy
[govec]: https://github.com/nymlab/vectis/tree/main/contracts/govec
[cw-core]: https://github.com/DA0-DA0/dao-contracts/tree/v1.0.0/contracts/cw-core
[cw-proposal-single]: https://github.com/DA0-DA0/dao-contracts/tree/v1.0.0/contracts/cw-proposal-single
[cw20-staked-balance-voting]: https://github.com/DA0-DA0/dao-contracts/tree/v1.0.0/contracts/cw-staked-balance-voting
[cw20-stake]: https://github.com/DA0-DA0/dao-contracts/tree/v1.0.0/contracts/cw20-stake
[confio demo]: https://github.com/confio/cw-ibc-demo

### Note on Relayer gas fees

When a message is relayed, the smart contract wallet has to verify the relayed transaction has been signed by the user and that it was not replayed.
This operation adds some amount of gas to a transaction that was just directly sent by the user to the SCW.

For reference (tested on a local node):

- A user directly sends a bank message to the SCW: 138700
- A relayer sends a signed bank message for execution: 147285

## Hack

### Contracts Code Test

```sh
cd contracts
cargo test
```

### Runner Options

#### Docker Vectis Image

```bash
docker run -d --name vectis_node -p 1317:1317 -p 26656:26656 -p 26657:26657 ghcr.io/nymlab/vectis:main
```

#### Docker Local Node

##### 1. Set env variable

You must create an .env file from example.env template and define NETWORK env variable.

Supported networks are: `juno_local` `juno_testnet` `wasmd_local` `wasmd_testnet`

##### 2. Run Script

In scripts folder there is a script called `./local-node-setup.sh` which will spin up a wasmd or juno container and run the setup script to seed some accounts, then compile Rust contracts and run the E2E spec to deploy contracts and test functionalities. When everything is done, you can use the local node on your machine.

The script will check if you have both Docker and Rust installed. In case you haven't, a link will be shown to install said dependencies.

#### Native Local Node

##### Juno Option

###### Build locally by following [instructions](https://docs.junonetwork.io/smart-contracts-and-junod-development/installation)

> **Note:** this requires you to do a setup script to seed the accounts use for cli test/utils folder specified in the `.env`

##### Wasmd Option

```sh
git clone https://github.com/CosmWasm/wasmd.git
cd wasmd
# replace the v0.18.0 with the most stable version on https://github.com/CosmWasm/wasmd/releases
git checkout v0.24.0
make install

# verify the installation
wasmd version
```

> **INFO:** `make install` will copy wasmd to `$HOME/go/bin` or the default directory for binaries from Go,
> please make sure that is in your `PATH`.

##### 1. Start the node

```sh
./scripts/wasmd-node-setup.sh
```

##### 2. Compile smart contracts

```sh
./scripts/build.sh
```

### Interacting with the blockchain

We are using [CosmJS](https://github.com/cosmos/cosmjs) to test the smart contracts.
The testing framework used is [Jest](https://jestjs.io/)
The CLI is in the `cli` directory.

Please ensure you have set up the `.env` file according to the `example.env`.

> Note: _The tests include storing and instantiating the contracts_

```sh
cd cli
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
