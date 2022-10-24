export const wasm_testnet = {
    chainId: "cliffnet-1",
    chainName: "Wasmd Testnet",
    addressPrefix: "wasm",
    rpcUrl: "https://rpc.cliffnet.cosmwasm.com/",
    httpUrl: "https://lcd.cliffnet.cosmwasm.com/",
    feeToken: "upebble",
    stakingToken: "urock",
    estimatedBlockTime: 7000,
    estimatedIndexerTime: 250,
    gasPrice: 0.025,
};

export const wasm_localnet = {
    chainId: "wasm-local",
    chainName: "Wasmd Localnet",
    addressPrefix: "wasm",
    rpcUrl: "http://localhost:26647",
    httpUrl: "http://localhost:1317",
    feeToken: "ucosm",
    stakingToken: "ustake",
    estimatedBlockTime: 7000,
    estimatedIndexerTime: 250,
    gasPrice: 0.025,
};
