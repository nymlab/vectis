OPTIMIZER_VER=0.12.6

if [[ `uname -m` =~ "arm" ]]; then 
OPTIMIZER_IMG=cosmwasm/workspace-optimizer-arm64; else 
OPTIMIZER_IMG=cosmwasm/workspace-optimizer;
fi

cd ..

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  ${OPTIMIZER_IMG}:${OPTIMIZER_VER}

cd scripts