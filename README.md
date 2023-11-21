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

- Seedless accounts: Accounts are by default controlled by an `Entity` using Passkey creation and tx signing
- Extensible plugin features: supercharge and customise your Vectis Account with automated transactions and pre-transaction workflow / guards. See [plugin section]
- (Soon) Account controller rotation: Accounts can rotate their own `Entity` in the case of updates, this can be extended in the future for guardianships
- ICA: the creation of ICA is baked into the Vectis Accounts

[introductory article]: https://nymlab.notion.site/Introducing-Vectis-3578c478316b40d098dcc5832e3a267b
[plugin section]: #plugins

With the above features, Vectis provides functionality for both retail users and businesses to satisfy regulatory requirements:

- consumer protection: guardianship features can help recovery user funds and manage fraud risk
- Pre-Transaction checks: Vectis is a compliant solution and satisfies crypto AML / travel checks
- Transparency: Users can at all time check any status on their wallet, enhanced with Vectis clients push notifications

### Plugins

Vectis is designed to be extensible with a plugin system.
Much like the extensions to your browsers,
Vectis accounts can add plugins to perform authorised (automated) transactions and pre-execution workflow and guards.

Plugins to Vectis Accounts can be built by any developers,
Vectis provides test suite to allow developers to easily test plugins in the Vectis ecosystem.

Plugins can choose to apply to the Vectis Plugin Registry (VPR) to give users a certain level of assurance.
Vectis and partnership teams is responsible for the soundness of the plugins in the registry.
For more information on plugins.


[nymlab github account]: https://github.com/nymlab?q=vectis&type=all&language=&sort=

---


### Contract building

```sh
# Builds wasm files
make build

# Builds schemas
make schemas && cd ts && npm run generate
```

### Contract Testing

```sh
# For all contracts using test-tube
# This requires wasm contracts from previous step
cargo test -- test-tube

# For all cw-multi-tests
cargo test -- unit_tests

# For all tests
cargo test
```
