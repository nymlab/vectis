#!/bin/bash

###
# This script generates TypeScript interfaces based on contract schemas.
# Resulting types are stored in the `types` directory.
# Do not modify types manually! Always run this script to generate up-to-date interfaces.
###

# Factory
./node_modules/.bin/cosmwasm-typescript-gen generate \
    --schema ../contracts/factory/schema \
    --out ./types \
    --name Factory

# Govec
./node_modules/.bin/cosmwasm-typescript-gen generate \
    --schema ../contracts/govec/schema \
    --out ./types \
    --name Govec

# Proxy
./node_modules/.bin/cosmwasm-typescript-gen generate \
    --schema ../contracts/proxy/schema \
    --out ./types \
    --name Proxy
