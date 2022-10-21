export const juno_testnet = {
    chainId: "uni-5",
    chainName: "Juno Testnet",
    addressPrefix: "juno",
    rpcUrl: "https://rpc.uni.junonetwork.io/",
    httpUrl: "https://api.uni.junonetwork.io/",
    feeToken: "ujunox",
    stakingToken: "ujunox",
    estimatedBlockTime: 7000,
    estimatedIndexerTime: 250,
    gasPrice: 0.025,
};

export const juno_local = {
    chainId: "testing",
    chainName: "Juno Localnet",
    addressPrefix: "juno",
    rpcUrl: "http://localhost:26657",
    httpUrl: "http://localhost:1317",
    feeToken: "ujunox",
    stakingToken: "ujunox",
    estimatedBlockTime: 7000,
    estimatedIndexerTime: 250,
    gasPrice: 0.025,
};
