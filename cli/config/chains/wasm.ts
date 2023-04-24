export const wasm_testnet = {
    chainId: "malaga-420",
    chainName: "Wasmd Testnet",
    addressPrefix: "wasm",
    rpcUrl: "https://rpc.malaga-420.cosmwasm.com:443",
    httpUrl: "https://api.malaga-420.cosmwasm.com",
    feeToken: "umlg",
    stakingToken: "uand",
    estimatedBlockTime: 7000,
    estimatedIndexerTime: 250,
    gasPrice: 0.08,
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
