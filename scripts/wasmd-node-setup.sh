#!/bin/bash

set -ex

# Check to see if genesis exists
if [ -d ".wasmd" ]; then
    rm -rf .wasmd
fi

# Check to see if keys exists 
if [ -d ".wasmd_keys" ]; then
    rm -rf .wasmd_keys
fi

# Configuration variables
APP_HOME="./.wasmd"
CHAIN_ID="localnet"
KEYRING=(--keyring-backend=test)
KEYDIR=(--keyring-dir=./.wasmd_keys)
ADMIN_ADDR="wasm1jcdyqsjyvp86g6tuzwwryfkpvua89fau728ctm"
USER_ADDR="wasm1tcxyhajlzvdheqyackfzqcmmfcr760marg3zw5"
GUARDIAN1_ADDR="wasm1qwwx8hsrhge9ptg4skrmux35zgna47pw0es69z"
GUARDIAN2_ADDR="wasm1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ausut0d"
RELAYER1_ADDR="wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4"
RELAYER2_ADDR="wasm1yjammmgqu62lz4sxk5seu7ml4fzdu7gkatgswc"

# initialize wasmd configuration files
wasmd init localnet --chain-id ${CHAIN_ID} --home ${APP_HOME} -o

# add minimum gas prices config to app configuration file
sed -i -r 's/minimum-gas-prices = ""/minimum-gas-prices = "0.01ucosm"/' ${APP_HOME}/config/app.toml

# Add your wallet addresses to genesis to fund them
# Please provide the corresponding address, user private key and mnemonic in .env.dev file at the root of the directory 
# Admin - stores contracts and instantiates factory (also the factory admin)
# User - user of the proxy wallet
# Guardians - guardians of the proxy wallet
# Relayers - can relay user signatures
wasmd add-genesis-account $ADMIN_ADDR 20000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $USER_ADDR  20000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $GUARDIAN1_ADDR 20000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $GUARDIAN2_ADDR 20000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $RELAYER1_ADDR 20000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $RELAYER2_ADDR 20000000000ucosm,10000000000stake --home ${APP_HOME}

# create validator address
wasmd keys add validator $KEYRING $KEYDIR
wasmd add-genesis-account $(wasmd keys show -a validator $KEYRING $KEYDIR) 10000000000ucosm,10000000000stake --home ${APP_HOME}
wasmd gentx validator 1000000000stake --home ${APP_HOME} --chain-id ${CHAIN_ID} $KEYRING $KEYDIR

# collect gentxs to genesis
wasmd collect-gentxs --home ${APP_HOME}

# validate the genesis file
wasmd validate-genesis --home ${APP_HOME}

# run the node
wasmd start --home ${APP_HOME} --wasm.query_gas_limit 9000000

