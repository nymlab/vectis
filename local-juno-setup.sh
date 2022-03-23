#!/bin/bash

docker run -it \
  -p 26656:26656 \
  -p 26657:26657 \
  -e STAKE_TOKEN=ujunox \
  -e UNSAFE_CORS=true \
  ghcr.io/cosmoscontracts/juno:v2.3.0-beta.1 \
  ./setup_and_run.sh \
  # admin
  juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y \
  # user
  juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr \

