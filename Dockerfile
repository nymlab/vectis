FROM ghcr.io/cosmoscontracts/juno:main

ARG ACCOUNTS="juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4 juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6 juno1ucl9dulgww2trng0dmunj348vxneufu50c822z juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q"
ENV NETWORK=juno_local
ENV STAKE_TOKEN=ujunox
ENV UNSAFE_CORS=true
ENV VECTIS_CW_PATH=/app/artifacts
ENV DOWNLOADED_CW_PATH=./wasm

RUN ./setup_junod.sh ${ACCOUNTS}

WORKDIR /app

COPY ./artifacts /app/artifacts
COPY ./js-app /app/js-app

RUN apk add --update nodejs npm

WORKDIR /app/js-app

RUN junod start --rpc.laddr tcp://0.0.0.0:26657 & sleep 5 && npm ci && npm test && killall -9 junod

EXPOSE 1317
EXPOSE 26656
EXPOSE 26657

CMD ["junod", "start", "--rpc.laddr", "tcp://0.0.0.0:26657"]