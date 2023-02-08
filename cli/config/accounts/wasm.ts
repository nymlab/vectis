const common = {
    admin: {
        mnemonic:
            "day tide foil title grief build consider front tell peanut must captain photo pistol purity similar gentle clay marble total lens veteran shrug visa",
        address: "wasm1jcdyqsjyvp86g6tuzwwryfkpvua89fau728ctm",
    },
    user: {
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
        address: "wasm1tcxyhajlzvdheqyackfzqcmmfcr760marg3zw5",
    },
    guardian_1: {
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
        address: "wasm1qwwx8hsrhge9ptg4skrmux35zgna47pw0es69z",
    },
    guardian_2: {
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
        address: "wasm1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ausut0d",
    },
    relayer_1: {
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
        address: "wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4",
    },
    relayer_2: {
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
        address: "wasm1yjammmgqu62lz4sxk5seu7ml4fzdu7gkatgswc",
    },
    committee1: {
        mnemonic:
            "cave topple history exercise carpet crash answer correct one benefit fury tiger medal emerge canoe acquire pig chuckle mystery confirm alley security exit mixture",
        address: "wasm1dfd5vtxy2ty5gqqv0cs2z23pfucnpym92kjfzm",
    },
    committee2: {
        mnemonic:
            "divorce park goat subject cake arrive liar reward favorite shed market spot harsh garden wet general enlist limb chair message current grant curtain that",
        address: "wasm1ndxfpxzxg267ujpc6wwhw9fs2rvgfh0675s7yr",
    },
};
export const wasm_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.WASM_ADMIN_MNEMONIC,
        address: process.env.WASM_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.WASM_COMMITTEE1_MNEMONIC,
        address: process.env.WASM_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.WASM_COMMITTEE2_MNEMONIC,
        address: process.env.WASM_COMMITTEE2_ADDRESS,
    },
};
export const wasm_testnet = { ...common };
export const wasm_localnet = { ...common };
