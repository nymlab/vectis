# Vectis - Smart Contract Wallet Infrastructure

[![Website](https://img.shields.io/badge/WEBSITE-https%3A%2F%2Fvectis.space-green?style=for-the-badge)](https://vectis.space)
[![Discord](https://img.shields.io/discord/989088257323188264?color=green&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/xp3vFSAMgS)

[![Cosmowasm 0.28.0](https://img.shields.io/badge/CosmWasm-0.28.0-green)](https://github.com/CosmWasm/wasmd/releases)
[![codecov_img](https://img.shields.io/codecov/c/github/nymlab/vectis)](https://img.shields.io/codecov/c/github/nymlab/vectis)

![Twitter](https://img.shields.io/twitter/follow/VectisDAO?style=social)

## Overview

Vectis is a smart contract wallet infrastructure project that allows user to interact with dApps on the blockchain with the same amount of autonomy of a classic non-custodial solution,
but supercharged with functionalities,
designed to provide the user with better experience and security.

### Features

_Details on Vectis user features can be found in our [introductory article]._

Vectis accounts come with base features:

- Guardianship - Social Recovery: in case of lost of private keys / device.
- Guardianship - Account Freezing: for temporary disabling account (such as travel).
- Extensible plugin features: supercharge and customise your Vectis Account with automated transactions and pre-transaction workflow / guards. See [plugin section].
- Gasless transactions: Sign transaction offline and have approved service to transact onchain.
- Cross chain transactions: Authorise transaction on IBC / other chains with our threshold signing infrastructure (next version).
- Integration with Self Sovereign Identity (SSI) protocols: Vectis mobile wallet combines blockchain wallet with identity wallet, an important integration to allow onchain transactions to embed ZK-proofs for identity, an important feature for mainstream adoption, especially for onchain financial applications.

[introductory article]: https://nymlab.notion.site/Introducing-Vectis-3578c478316b40d098dcc5832e3a267b
[plugin section]: #plugins

With the above features, Vectis provides functionality for both retail users and businesses to satisfy regulatory requirements:

- consumer protection: guardianship features can help recovery user funds and manage fraud risk
- Pre-Transaction check plugins: a multisig plugin tailored to business workflow provides separation of control and verifiability
- Transparency: Users can at all time check any status on their wallet, enhanced with Vectis clients push notifications

### Plugins

Vectis is designed to be extensible with a plugin system.
Much like the extensions to your browsers,
Vectis accounts can add plugins to perform authorised (automated) transactions and pre-execution workflow and guards.

Plugins to Vectis Accounts can be built by any developers,
Vectis provides test suite to allow developers to easily test plugins in the Vectis ecosystem.

Plugins can choose to apply to the Vectis Plugin Registry (VPR) to give users a certain level of assurance.
Vectis and partnership teams is responsible for the soundness of the plugins in the registry.
For more information on plugins, please see our [plugins repository].

## Progresive Decentralisation

VectisDAO aims to be the organisation that provides governance to the this infrastructure. This will be introduced in the next phase of Vectis.

At this phase of development, all of the Vectis codebase is open source and can be found in the [Nymlab github account].

[nymlab github account]: https://github.com/nymlab?q=vectis&type=all&language=&sort=
[wiki]: https://github.com/nymlab/vectis/wiki
[plugins repository]: https://github.com/nymlab/vectis-plugins/

---

## Contribute

We welcome PRs and issues 🤝

### Contract Testing

```sh
# For individual contract tests
cd contracts/contract-you-want-to-test
cargo test

# For all contract unit tests
cargo test-unit

# For all multi-test
cargo test-multi
```

### Integration Tests with Docker

This spins up two docker containers,
one per chain, to test IBC interactions.

##### 1. Set Environment

First we set up the two networks (one node each) locally.
Please ensure you have set up the `.env` file according to the `example.env`.

You should check supported chains in `cli/config/chains` directory.

```sh
make nodes-setup
```

##### 2. Compile Contracts

```sh
make build
```

##### 3. Upload and instantiate contracts

```sh
make deploy
```

The contracts are deployed as such:

- Remote network: all networks without DAO
- DAO network: where the VectisDAO will live has:
  - Vectis Plugin Registry
  - Vectis Factory
  - Instances of the Proxy contract
  - Where natively the VEC tokens are minted

After upload and deploy contracts it will check the contract have the right checksum and admin.

### Interacting with the blockchain

We are using [CosmJS](https://github.com/cosmos/cosmjs) to interact with the smart contracts and [Jest](https://jestjs.io/) as testing framework for client-side tests.
