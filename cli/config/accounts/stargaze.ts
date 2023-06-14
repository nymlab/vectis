import path from "path";
import * as dotenv from "dotenv";
dotenv.config({ path: path.join(__dirname, "../../../.env") });

const common = {
    admin: {
        address: "star16g2rahf5846rxzp3fwlswy08fz8ccuwkcemxw5",
        mnemonic:
            "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
    },
    user: {
        address: "star1tcxyhajlzvdheqyackfzqcmmfcr760magww7sn",
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
    },
    guardian_1: {
        address: "star1qwwx8hsrhge9ptg4skrmux35zgna47pwyl0xm9",
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
    },
    guardian_2: {
        address: "star1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ahkrh32",
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
    },
    relayer_1: {
        address: "star1ucl9dulgww2trng0dmunj348vxneufu5cs2c6j",
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
    },
    relayer_2: {
        address: "star1yjammmgqu62lz4sxk5seu7ml4fzdu7gkkdhvsl",
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
    },
};

export const stargaze_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.STARGAZE_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.STARGAZE_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.STARGAZE_COMMITTEE2_ADDRESS,
    },
};
export const stargaze_testnet = {
    ...common,
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_STARGAZE_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_STARGAZE_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_STARGAZE_COMMITTEE2_ADDRESS,
    },
};
