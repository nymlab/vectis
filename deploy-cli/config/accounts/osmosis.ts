const common = {
    admin: {
        address: "osmo16g2rahf5846rxzp3fwlswy08fz8ccuwk3cxl02",
        mnemonic:
            "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
    },
    user: {
        address: "osmo1tcxyhajlzvdheqyackfzqcmmfcr760map0n83d",
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
    },
    guardian_1: {
        address: "osmo1qwwx8hsrhge9ptg4skrmux35zgna47pwd7jl6m",
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
    },
    guardian_2: {
        address: "osmo1wk2r0jrhuskqmhc0gk6dcpmnz094sc2a7h7ws5",
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
    },
    relayer_1: {
        address: "osmo1ucl9dulgww2trng0dmunj348vxneufu533hpmv",
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
    },
    relayer_2: {
        address: "osmo1yjammmgqu62lz4sxk5seu7ml4fzdu7gklv243p",
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
    },
};

export const osmosis_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.OSMOSIS_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.OSMOSIS_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.OSMOSIS_COMMITTEE2_ADDRESS,
    },
};
export const osmosis_testnet = {
    ...common,
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_OSMOSIS_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_OSMOSIS_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_OSMOSIS_COMMITTEE2_ADDRESS,
    },
};
