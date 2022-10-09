#!/bin/bash

set -e

RELAYER_HOME="../cli/core/config/relayer"
NODE_1=`docker ps -a --format="{{.Names}}" | grep juno-1 | awk '{print $1}'`
NODE_2=`docker ps -a --format="{{.Names}}" | grep juno-2 | awk '{print $1}'`
FOUNDED_ACCOUNTS="juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q juno1varrv85sq4j3pphcf3prjz3r22d90xztaaypfa"

echo "👀 Checking and setting up requirements on your machine..."

if [[ `command -v docker` == "" ]]; then
echo "Installing relayer from @confio/relayer"
npm install -g @confio/relayer
fi

if [[ "$NODE_1" != "" ]]; then
echo "Removing existing node container $NODE_1"
docker rm -f $NODE_1 > /dev/null;
fi

if [[ "$NODE_2" != "" ]]; then
echo "Removing existing node container $NODE_2"
docker rm -f $NODE_2 > /dev/null;
fi

echo "⚙️  Running ${NODE_1} node on Docker..."

docker run -d \
--name juno-1 \
-p 1327:1317 \
-p 26646:26656 \
-p 26647:26657 \
-e STAKE_TOKEN=ujunox \
-e UNSAFE_CORS=true \
-e CHAIN_ID=juno-1 \
ghcr.io/cosmoscontracts/juno:10.0 \
./setup_and_run.sh \
$FOUNDED_ACCOUNTS

echo "⚙️  Running ${NODE_2} node on Docker..."

docker run -d \
--name juno-2 \
-p 1317:1317 \
-p 26656:26656 \
-p 26657:26657 \
-e STAKE_TOKEN=ujunox \
-e UNSAFE_CORS=true \
-e CHAIN_ID=juno-2 \
ghcr.io/cosmoscontracts/juno:10.0 \
./setup_and_run.sh \
$FOUNDED_ACCOUNTS

sleep 10

echo "Init relayer connection" 

ibc-setup connect --home $RELAYER_HOME

echo "🛠️ Building Vectis contracts"

sh ./build.sh

echo "📖️ Deploying Vectis contracts and running tests..."

#docker cp artifacts/cw_ibc_example.wasm juno-1:/ibc.wasm
#docker exec -i juno-1 junod tx wasm store "/ibc.wasm" --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -y -b block --chain-id juno-1 --from validator --o json
#docker exec -i juno-1 junod tx wasm instantiate 1 "{}" --label ibc-test --no-admin --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -y -b block --chain-id juno-1 --from validator --o json 

echo "Creating a channel"

#ibc-setup channel --src-connection=connection-0 --dest-connection=connection-0 --src-port=wasm.juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8 --dest-port=wasm.juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8 --version=vectis-tunnel
