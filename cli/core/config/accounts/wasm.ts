const common = {
    admin: {
        mnemonic:
            "day tide foil title grief build consider front tell peanut must captain photo pistol purity similar gentle clay marble total lens veteran shrug visa",
        address: "wasm1jcdyqsjyvp86g6tuzwwryfkpvua89fau728ctm",
        pubKey: "wasmpub1addwnpepqt0a4x3r8zluaamv780znvm8qd7jlq6ew2zah3fgmlwt3glmpkl5ct93fee",
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
        pubKey: "wasmpub1addwnpepqt3yh0e5j2v5fm86erxukrhztml296248pc9xutgdc4xqw2gefk4gne3ew6",
    },
    guardian_2: {
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
        address: "wasm1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ausut0d",
        pubKey: "wasmpub1addwnpepqv0ank7gamza23m4j6072gkz6202uzg5zntke6c5vxjwf08d0cnyg7kk8u5",
    },
    relayer_1: {
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
        address: "wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4",
        pubKey: "wasmpub1addwnpepqwvpfu2tjmxqm5kgfhrefumzzdxnxkuvy33j2tz5jtfwyxx45qh8kp3cyw9",
    },
    relayer_2: {
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
        address: "wasm1yjammmgqu62lz4sxk5seu7ml4fzdu7gkatgswc",
        pubKey: "wasmpub1addwnpepq28r2zt4h9qfy0yvzzwttf0l7429usex27k49x5ahjmnpfnc9dtpvr2gdwf",
    },
};
export const wasm_mainnet = {
    admin: {
        mnemonic: process.env.WASM_ADMIN_MNEMONIC,
        address: process.env.WASM_ADMIN_ADDRESS,
    },
};
export const wasm_testnet = { ...common };
export const wasm_localnet = { ...common };
