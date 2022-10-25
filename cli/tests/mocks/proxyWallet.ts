import { coin } from "@cosmjs/stargate";
import { FactoryT, FactoryClient } from "@vectis/types";
import { defaultWalletCreationFee, HOST_ACCOUNTS, HOST_CHAIN, walletInitialFunds } from "./constants";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<FactoryT.Addr[]> {
    const initial_funds = walletInitialFunds(HOST_CHAIN);
    const walletCreationFee = await factoryClient.fee();
    const totalFee: Number = Number(walletCreationFee.amount) + Number(initial_funds.amount);

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: HOST_ACCOUNTS.user.address,
                guardians: {
                    addresses: [HOST_ACCOUNTS.guardian_1.address, HOST_ACCOUNTS.guardian_2.address],
                },
                relayers: [HOST_ACCOUNTS.relayer_1.address, HOST_ACCOUNTS.relayer_2.address],
                proxy_initial_funds: [initial_funds as FactoryT.Coin],
                label: "wallet",
            },
        },
        defaultWalletCreationFee,
        undefined,
        [coin(totalFee.toString(), HOST_CHAIN.feeToken) as FactoryT.Coin]
    );

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: HOST_ACCOUNTS.user.address,
                guardians: {
                    addresses: [HOST_ACCOUNTS.guardian_1.address, HOST_ACCOUNTS.guardian_2.address],
                    guardians_multisig: {
                        threshold_absolute_count: 2,
                        multisig_initial_funds: [],
                    },
                },
                relayers: [HOST_ACCOUNTS.relayer_1.address, HOST_ACCOUNTS.relayer_2.address],
                proxy_initial_funds: [initial_funds as FactoryT.Coin],
                label: "wallet-multisig",
            },
        },
        defaultWalletCreationFee,
        undefined,
        [coin(totalFee.toString(), HOST_CHAIN.feeToken) as FactoryT.Coin]
    );

    const { wallets } = await factoryClient.unclaimedGovecWallets({});
    const walletAddress = wallets[0][0];
    const walletMSAddress = wallets[1][0];

    return [walletAddress, walletMSAddress];
}
