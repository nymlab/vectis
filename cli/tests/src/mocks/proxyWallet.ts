import { toBase64 } from "@cosmjs/encoding";
import {
    coinMinDenom,
    guardian1Addr,
    guardian2Addr,
    relayer1Addr,
    relayer2Addr,
    testWalletInitialFunds,
    userAddr,
    userMnemonic,
} from "@vectis/core/utils/constants";
import { defaultWalletCreationFee } from "@vectis/core/utils/fee";
import { mnemonicToKeyPair } from "@vectis/core/services/cosmwasm";
import { Addr, Coin, FactoryClient } from "@vectis/types/contracts/FactoryContract";
import { coin } from "@cosmjs/stargate";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<Addr[]> {
    const walletCreationFee = await factoryClient.fee();
    const totalFee: Number = Number(walletCreationFee.amount) + Number(testWalletInitialFunds.amount);

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: userAddr,
                guardians: {
                    addresses: [guardian1Addr, guardian2Addr],
                },
                relayers: [relayer1Addr, relayer2Addr],
                proxy_initial_funds: [testWalletInitialFunds as Coin],
                label: "initial label",
            },
        },
        defaultWalletCreationFee,
        undefined,
        [coin(totalFee.toString(), coinMinDenom) as Coin]
    );

    const [walletAddress] = (await factoryClient.walletsOf({ user: userAddr! })).wallets;

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: userAddr,
                guardians: {
                    addresses: [guardian1Addr, guardian2Addr],
                    guardians_multisig: {
                        threshold_absolute_count: 2,
                        multisig_initial_funds: [],
                    },
                },
                relayers: [relayer1Addr, relayer2Addr],
                proxy_initial_funds: [testWalletInitialFunds as Coin],
                label: "initial label",
            },
        },
        defaultWalletCreationFee,
        undefined,
        [coin(totalFee.toString(), coinMinDenom) as Coin]
    );

    const walletMSAddress = (await factoryClient.walletsOf({ user: userAddr })).wallets.find(
        (w) => w !== walletAddress
    );
    return [walletAddress, walletMSAddress!];
}
