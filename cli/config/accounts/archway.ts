import path from "path";
import * as dotenv from "dotenv";
dotenv.config({ path: path.join(__dirname, "../../../.env") });

const common = {
    admin: {
        mnemonic:
            "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
        address: "",
    },
    user: {
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
        address: "",
    },
    guardian_1: {
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
        address: "",
    },
    guardian_2: {
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
        address: "",
    },
    relayer_1: {
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
        address: "",
    },
    relayer_2: {
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
        address: "",
    },
    committee1: {
        mnemonic:
            "cave topple history exercise carpet crash answer correct one benefit fury tiger medal emerge canoe acquire pig chuckle mystery confirm alley security exit mixture",
        address: "",
    },
    committee2: {
        mnemonic:
            "divorce park goat subject cake arrive liar reward favorite shed market spot harsh garden wet general enlist limb chair message current grant curtain that",
        address: "",
    },
};

export const archway_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.ARCHWAY_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.ARCHWAY_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.ARCHWAY_COMMITTEE2_ADDRESS,
    },
};
export const archway_testnet = {
    ...common,
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_ARCHWAY_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_ARCHWAY_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_ARCHWAY_COMMITTEE2_ADDRESS,
    },
};
