import path from "path";
import * as dotenv from "dotenv";
dotenv.config({ path: path.join(__dirname, "../../../.env") });

export const archway_mainnet = {
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.ARCH_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.ARCH_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.ARCH_COMMITTEE2_ADDRESS,
    },
};

export const aura_testnet = {
    admin: {
        mnemonic: process.env.TESTNET_AURA_ADMIN_MNEMONIC,
        address: process.env.TESTNET_AURA_ADMIN_ADDRESS,
    },
};

export const archway_testnet = {
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_ARCH_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_ARCH_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_ARCH_COMMITTEE2_ADDRESS,
    },
};

export const neutron_mainnet = {
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

export const stargaze_mainnet = {
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

export const juno_mainnet = {
    admin: {
        mnemonic: process.env.ADMIN_MNEMONIC,
        address: process.env.JUNO_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.COMMITTEE1_MNEMONIC,
        address: process.env.JUNO_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.COMMITTEE2_MNEMONIC,
        address: process.env.JUNO_COMMITTEE2_ADDRESS,
    },
};
export const juno_testnet = {
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_JUNO_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_JUNO_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_JUNO_COMMITTEE2_ADDRESS,
    },
    walletCreator: {
        mnemonic: process.env.TESTNET_WALLET_CREATOR_MNEMONIC,
        address: process.env.TESTNET_JUNO_WALLET_CREATOR_ADDRESS,
    },
};

export const osmosis_testnet = {
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

export const juno_localnet = {};

export const injective_testnet = {
    admin: {
        mnemonic: process.env.TESTNET_ADMIN_MNEMONIC,
        address: process.env.TESTNET_INJ_ADMIN_ADDRESS,
    },
    committee1: {
        mnemonic: process.env.TESTNET_COMMITTEE1_MNEMONIC,
        address: process.env.TESTNET_INJ_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.TESTNET_COMMITTEE2_MNEMONIC,
        address: process.env.TESTNET_INJ_COMMITTEE2_ADDRESS,
    },
};
