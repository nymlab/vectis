import path from "path";
import * as dotenv from "dotenv";
dotenv.config({ path: path.join(__dirname, "../../../.env") });

const common = {
    admin: {
        mnemonic:
            "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
        address: "neutron16g2rahf5846rxzp3fwlswy08fz8ccuwkauudrl",
    },
    user: {
        mnemonic:
            "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
        address: "neutron1tcxyhajlzvdheqyackfzqcmmfcr760madtf4ac",
    },
    guardian_1: {
        mnemonic:
            "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
        address: "neutron1qwwx8hsrhge9ptg4skrmux35zgna47pwp6gdkw",
    },
    guardian_2: {
        mnemonic:
            "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
        address: "neutron1wk2r0jrhuskqmhc0gk6dcpmnz094sc2ajnyuup",
    },
    relayer_1: {
        mnemonic:
            "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
        address: "neutron1ucl9dulgww2trng0dmunj348vxneufu5a4dnhe",
    },
    relayer_2: {
        mnemonic:
            "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
        address: "neutron1yjammmgqu62lz4sxk5seu7ml4fzdu7gkngs8a5",
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

export const neutron_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.NEUTRON_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.NEUTRON_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.NEUTRON_COMMITTEE2_ADDRESS,
    },
};
export const neutron_testnet = {
    ...common,
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_NEUTRON_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_NEUTRON_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_NEUTRON_COMMITTEE2_ADDRESS,
    },
};
