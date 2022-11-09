const common = {
    admin: {
        mnemonic:
            "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
        address: "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y",
    },
    user: {
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
        address: "juno1tcxyhajlzvdheqyackfzqcmmfcr760malxrvqr",
    },
    guardian_1: {
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
        address: "juno1qwwx8hsrhge9ptg4skrmux35zgna47pwnhz5t4",
    },
    guardian_2: {
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
        address: "juno1wk2r0jrhuskqmhc0gk6dcpmnz094sc2aq7w9p6",
    },
    relayer_1: {
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
        address: "juno1ucl9dulgww2trng0dmunj348vxneufu50c822z",
    },
    relayer_2: {
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
        address: "juno1yjammmgqu62lz4sxk5seu7ml4fzdu7gkp967q0",
    },
};

export const juno_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.JUNO_ADMIN_MNEMONIC,
        address: process.env.JUNO_ADMIN_ADDRESS,
    },
};
export const juno_testnet = { ...common };
export const juno_localnet = { ...common };
