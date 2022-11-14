#!/bin/bash

set -e

NODE_1=`docker ps -a --format="{{.Names}}" | grep juno_local | awk '{print $1}'`
NODE_2=`docker ps -a --format="{{.Names}}" | grep wasm_local | awk '{print $1}'`

if [[ "$NODE_1" != "" ]]; then
echo "Removing existing node container $NODE_1"
docker rm -f $NODE_1 > /dev/null;
fi

if [[ "$NODE_2" != "" ]]; then
echo "Removing existing node container $NODE_2"
docker rm -f $NODE_2 > /dev/null;
fi

echo "⚙️  Running juno_local on Docker..."

docker run -d \
--name juno_local \
-p 1317:1317 \
-p 26656:26656 \
-p 26657:26657 \
-e STAKE_TOKEN=ujunox \
-e UNSAFE_CORS=true \
-e CHAIN_ID=juno-local \
ghcr.io/cosmoscontracts/juno:10.1 \
./setup_and_run.sh \
juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q;

echo "⚙️  Running wasm_local node on Docker..."

docker run -d \
--platform linux/amd64 \
--name wasm_local \
-p 1327:1317 \
-p 26646:26656 \
-p 26647:26657 \
-e UNSAFE_CORS=true \
-e CHAIN_ID=wasm-local \
cosmwasm/wasmd:v0.29.0 \
./setup_and_run.sh \
wasm1jcdyqsjyvp86g6tuzwwryfkpvua89fau728ctm wasm1tcxyhajlzvdheqyackfzqcmmfcr760marg3zw5 wasm1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ausut0d wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4 wasm1yjammmgqu62lz4sxk5seu7ml4fzdu7gkatgswc;