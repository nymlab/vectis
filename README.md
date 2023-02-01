# Vectis - Smart Contract Wallet

[![Cosmowasm 0.20.0](https://img.shields.io/badge/CosmWasm-0.20.0-green)](https://github.com/CosmWasm/wasmd/releases)
[![codecov_img](https://img.shields.io/codecov/c/github/nymlab/vectis)](https://img.shields.io/codecov/c/github/nymlab/vectis)

[![Website](https://img.shields.io/badge/WEBSITE-https%3A%2F%2Fvectis.nymlab.it%2F-green?style=for-the-badge)](https://vectis.nymlab.it/)

[![Discord](https://discord.com/api/guilds/989088257323188264/widget.png?style=banner2)](https://discord.gg/xp3vFSAMgS)

## Overview

Smart Contract Wallet allows user to interact with DAPPs on the blockchain with the same amount of autonomy of a classic non-custodial solution,
with the additional functionalities designed to provide the user with better experience and security.

SCW also provide functions that allow businesses to satisfy regulatory requirements regarding support of users,
transparency,
separation of control duties and verifiability.

VectisDAO is the organisation that provides governance to the this infrastructure.
VectisDAO lives on [Juno Network] and leverages the [DAO DAO] stack.
Every SCW wallet has the right to purchase a set amount of Govec tokens at a set price,
the Govec will be minted and can be staked to vote.
Contributors can enter VectisDAO with investment (typically development and other useful efforts).
The amount of Govec from contributions will be determined by the DAO.

Through [IBC], Vectis wallets can also be deployed on other chains.
From the perspective of DAO participation,
there is no difference between a wallet on [Juno Network] or others.
Staking and voting will be done via IBC calls from the wallet.

Please see our [wiki] for details.

[dao dao]: https://daodao.zone
[juno network]: https://www.junonetwork.io/
[ibc]: https://github.com/cosmos/ibc
[wiki]: https://github.com/nymlab/vectis/wiki

---

## Hack

### Contract Tests

```sh
# For individual contract tests
cd contracts/contract-you-want-to-test
cargo test

# For all tests
cargo test
```

### Integration Tests

#### Docker Images

##### 1. Set Environment

First we set up the two networks (one node each) locally.
Please ensure you have set up the `.env` file according to the `example.env`. <br>
You should check supported chains in `cli/config/chains` directory.

```sh
make nodes-setup
```

##### 2. Compile Contracts

```sh
make build
```

##### 3. Upload and Instantiate Contracts

```sh
make deploy
```

The deployment of the DAO on the host chain has the following steps:

1. _Relayer_: Creates channels between Dao-tunnel and Remote-tunnel
1. Upload all required contracts (in ./upload.ts) to dao chain + remote chains
   1. Host: Factory, Govec, Proxy, Dao-tunnel, Dao contracts (core, proposal, vote, cw20-staking, pre-proposal)
   1. Remote: Remote-Factory, Proxy, Remote-tunnel, Remote ICA
1. _DAO Chain_: Instantiate Govec contract (with admin having initial balance for proposing for DAO to deploy Factory)
1. _DAO Chain_: Instantiate dao-core contract (which will instantiate one proposal module and one vote module contracts)
   - note: vote contracts also instantiates a new staking contract as we use staked-balance for voting,
     proposal module also instantitates the pre-proposal which is the `dao-pre-proposal-approval-single` contract.
1. _DAO Chain_: Admin propose and execute on DAO to deploy factory and Dao-tunnel contracts
1. _Remote Chain_: Remote Admin instantiate Remote-tunnel with Dao-tunnel as port and connection Id from step 1
1. _DAO Chain_: Admin propose and execute on DAO to allow ibc-connection to the Remote-tunnel
1. _DAO Chain_: Admin propose and execute on DAO to deploy remote-factory
1. _Remote Chain_: Execute instantiate remote-factory
1. _DAO Chain_: Admin updates Govec staking address to the staking contract in step 4.
1. _DAO Chain_: Admin updates Govec minter address to the factory contract addr and dao-tunnel addr
1. _DAO Chain_: Admin updates Govec contract DAO_ADDR as DAO
1. _DAO Chain_: Admin updates Govec contract admin as DAO (for future upgrades)
1. _DAO Chain_: Admin unstakes and burn its govec (exits system)

After upload and deploy contracts it will check the contract have the right checksum and the contracts have DAO as admin.

#### Native Local Node

##### Juno Option

###### Build locally by following [instructions](https://docs.junonetwork.io/smart-contracts-and-junod-development/installation)

> **Note:** this requires you to do a setup script to seed the accounts use for cli.

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
make build
```

### Interacting with the blockchain

We are using [CosmJS](https://github.com/cosmos/cosmjs) to interact with the smart contracts and [Jest](https://jestjs.io/) as testing framework for client-side tests.
