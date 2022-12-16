import { coin } from "@cosmjs/stargate";
import { FactoryT, FactoryClient } from "../../interfaces";
import { hostAccounts, hostChain, remoteChain, remoteAccounts } from "../../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../../utils/fees";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<FactoryT.Addr[]> {
    const initial_funds = walletInitialFunds(hostChain);
    const { wallet_fee } = await factoryClient.fees();
    const totalFee: Number = Number(wallet_fee.amount) + Number(initial_funds.amount);
    let walletAddress: string | null;
    let walletMSAddress: string | null;

    let oldWalletRes = await factoryClient.unclaimedGovecWallets({});

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                controller_addr: hostAccounts.user.address,
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
    let oldAddr = oldWalletRes.wallets.map((s, e) => s[0]);
    let newAddr = walletRes.wallets.map((s, e) => s[0]);

    if (oldWalletRes.wallets.length == 0) {
        walletAddress = walletRes.wallets[0][0];
    } else {
        for (let e of newAddr) {
            if (!oldAddr.includes(e)) {
                walletAddress = e;
            }
        }
    }

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                controller_addr: hostAccounts.user.address,
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

    let MSwalletRes = await factoryClient.unclaimedGovecWallets({});
    let MSAddrs = MSwalletRes.wallets.map((s, e) => s[0]);

    for (let e of MSAddrs) {
        if (!newAddr.includes(e)) {
            walletMSAddress = e;
        }
    }

    return [walletAddress!, walletMSAddress!];
}

export async function createSingleProxyWallet(factoryClient: FactoryClient, chain: string): Promise<FactoryT.Addr> {
    const accounts = chain == "host" ? hostAccounts : remoteAccounts;
    const chains = chain == "host" ? hostChain : remoteChain;
    const initialFunds = walletInitialFunds(hostChain);
    const { wallet_fee } = await factoryClient.fees();
    const totalFee: Number = Number(wallet_fee.amount) + Number(initialFunds.amount);
    let walletAddress: string | null;
    let walletMSAddress: string | null;

    let oldWalletRes = await factoryClient.unclaimedGovecWallets({});

    await factoryClient.createWallet(
        {
            createWalletMsg: {
                controller_addr: accounts.user.address,
                guardians: {
                    addresses: [accounts.guardian_1.address, accounts.guardian_2.address],
                },
                relayers: [accounts.relayer_1.address, accounts.relayer_2.address],
                proxy_initial_funds: [initialFunds as FactoryT.Coin],
                label: "wallet",
            },
        },
        getDefaultWalletCreationFee(chains),
        undefined,
        [coin(totalFee.toString(), chains.feeToken) as FactoryT.Coin]
    );

    let walletRes = await factoryClient.unclaimedGovecWallets({});
    let oldAddr = oldWalletRes.wallets.map((s, e) => s[0]);
    let newAddr = walletRes.wallets.map((s, e) => s[0]);

    if (oldWalletRes.wallets.length == 0) {
        walletAddress = walletRes.wallets[0][0];
    } else {
        for (let e of newAddr) {
            if (!oldAddr.includes(e)) {
                walletAddress = e;
            }
        }
    }

    return walletAddress!;
}
