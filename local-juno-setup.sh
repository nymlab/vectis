#!/bin/bash

cd ./js-app

echo "⚙️  Running Juno local node on Docker..."

docker rm -f juno-local-node > /dev/null

docker run -d \
  --name juno-local-node \
  -p 1317:1317 \
  -p 26656:26656 \
  -p 26657:26657 \
  -e STAKE_TOKEN=ujunox \
  -e UNSAFE_CORS=true \
  ghcr.io/cosmoscontracts/juno:v2.3.1 \
  ./setup_and_run.sh \
  juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q0

sleep 5

echo "📖️ Deploying Vectis contracts and running tests..."

export NODE_ENV=juno-local && npm test

echo "✅️ All done, have fun!"
