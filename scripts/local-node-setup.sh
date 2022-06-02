#!/bin/bash

set -e

echo "👀 Checking and setting up requirements on your machine..."

source ../.env

command -v docker >/dev/null 2>&1 || { echo >&2 "Docker is not installed on your machine, local Juno node can't be ran. Install it from here: https://www.docker.com/get-started"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo >&2 "Rust is not installed on your machine, Vectis contracts can't be compiled. Install it from here: https://www.rust-lang.org/tools/install"; exit 1; }
rustup target add wasm32-unknown-unknown

NODE_NETWORK=$NETWORK || "juno_local"

CW_CONTAINER=`docker ps --format="{{.ID}}\t{{.Ports}}" | grep 26656 | awk '{print $1}'`

if [[ "$CW_CONTAINER" != "" ]]; then
echo "Removing docker container ${CW_CONTAINER}"
docker rm -f ${CW_CONTAINER} > /dev/null;
fi

echo "⚙️  Running ${NODE_NETWORK} node on Docker..."

if [[ ${NODE_NETWORK} =~ "juno" ]]; then 
docker run -d \
  --name ${NODE_NETWORK}_node \
  -p 1317:1317 \
  -p 26656:26656 \
  -p 26657:26657 \
  -e STAKE_TOKEN=ujunox \
  -e UNSAFE_CORS=true \
  ghcr.io/cosmoscontracts/juno:main \
  ./setup_and_run.sh \
  juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q;
elif [[ ${NODE_NETWORK} =~ "wasmd" ]]; then
docker run -d \
  --name ${NODE_NETWORK}_node \
  -p 1317:1317 \
  -p 26656:26656 \
  -p 26657:26657 \
  -e STAKE_TOKEN=ucosm \
  cosmwasm/wasmd:v0.27.0 \
  ./setup_and_run.sh \
  wasm1jcdyqsjyvp86g6tuzwwryfkpvua89fau728ctm wasm1tcxyhajlzvdheqyackfzqcmmfcr760marg3zw5 wasm1qwwx8hsrhge9ptg4skrmux35zgna47pw0es69z wasm1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ausut0d wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4 wasm1yjammmgqu62lz4sxk5seu7ml4fzdu7gkatgswc;
else
echo "🚫  Invalid network name, please use juno or wasmd"
exit 1
fi

sleep 5

echo "🛠️ Building Vectis contracts"

sh ./build.sh

echo "📖️ Deploying Vectis contracts and running tests..."

cd ../cli

npm ci
npm test

echo "✅️ All done, have fun!"
