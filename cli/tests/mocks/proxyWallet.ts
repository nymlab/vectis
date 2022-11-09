import { coin } from "@cosmjs/stargate";
import { FactoryT, FactoryClient } from "../../interfaces";
import { hostAccounts, hostChain } from "../../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../../utils/fees";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<FactoryT.Addr[]> {
    const initial_funds = walletInitialFunds(hostChain);
    const walletCreationFee = await factoryClient.fee();
    const totalFee: Number = Number(walletCreationFee.amount) + Number(initial_funds.amount);

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: hostAccounts.user.address,
                guardians: {
                    addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                },
                relayers: [hostAccounts.relayer_1.address, hostAccounts.relayer_2.address],
                proxy_initial_funds: [initial_funds as FactoryT.Coin],
                label: "wallet",
            },
        },
        getDefaultWalletCreationFee(hostChain),
        undefined,
        [coin(totalFee.toString(), hostChain.feeToken) as FactoryT.Coin]
    );

    let walletRes = await factoryClient.unclaimedGovecWallets({});
    const walletAddress = walletRes.wallets[0][0];

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                user_addr: hostAccounts.user.address,
                guardians: {
                    addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                    guardians_multisig: {
                        threshold_absolute_count: 2,
                        multisig_initial_funds: [],
                    },
                },
                relayers: [hostAccounts.relayer_1.address, hostAccounts.relayer_2.address],
                proxy_initial_funds: [initial_funds as FactoryT.Coin],
                label: "wallet-multisig",
            },
        },
        getDefaultWalletCreationFee(hostChain),
        undefined,
        [coin(totalFee.toString(), hostChain.feeToken) as FactoryT.Coin]
    );

    walletRes = await factoryClient.unclaimedGovecWallets({});

    const walletMSAddress = walletRes.wallets[0][0];

    return [walletAddress, walletMSAddress];
}
