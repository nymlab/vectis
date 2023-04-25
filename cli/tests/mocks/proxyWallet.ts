import { coin, Event } from "@cosmjs/stargate";
import { FactoryT, FactoryClient } from "../../interfaces";
import { hostAccounts, hostChain, remoteChain, remoteAccounts } from "../../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../../utils/fees";
import { CWClient } from "../../clients";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<FactoryT.Addr[]> {
    const initial_funds = walletInitialFunds(hostChain);
    const { wallet_fee } = await factoryClient.fees();

    const totalFee: Number = Number(wallet_fee.amount) + Number(initial_funds.amount);
    let walletAddress: string | null;
    let walletMSAddress: string | null;

    let res = await factoryClient.createWallet(
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

    walletAddress = CWClient.getContractAddrFromResult(res, "_contract_address");

    res = await factoryClient.createWallet(
        {
            createWalletMsg: {
                controller_addr: hostAccounts.user.address,
                guardians: {
                    addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                    guardians_multisig: {
                        threshold_absolute_count: 2,
                    },
                },
                relayers: [hostAccounts.relayer_1.address, hostAccounts.relayer_2.address],
                proxy_initial_funds: [initial_funds as FactoryT.Coin],
                label: "cronkitty test 1",
            },
        },
        getDefaultWalletCreationFee(hostChain),
        undefined,
        [coin(totalFee.toString(), hostChain.feeToken) as FactoryT.Coin]
    );

    walletMSAddress = CWClient.getContractAddrFromEvent(
        res,
        "wasm-vectis.proxy.v1.MsgInstantiate",
        "_contract_address"
    );

    return [walletAddress!, walletMSAddress!];
}

export async function createSingleProxyWallet(factoryClient: FactoryClient, chain: string): Promise<FactoryT.Addr> {
    const accounts = chain == "host" ? hostAccounts : remoteAccounts;
    const chains = chain == "host" ? hostChain : remoteChain;
    const initialFunds = walletInitialFunds(chains).amount == "0" ? [] : [walletInitialFunds(chains) as FactoryT.Coin];
    const { wallet_fee } = await factoryClient.fees();
    const totalFee: Number = initialFunds.length == 1 ? Number(initialFunds[0].amount) + Number(wallet_fee.amount) : 0;
    let totalFeeToSend = totalFee == 0 ? undefined : [coin(totalFee.toString(), chains.feeToken) as FactoryT.Coin];

    console.log("initial funds: ", initialFunds);
    console.log("total fees: ", totalFeeToSend);

    let walletAddress: string | null;

    let res = await factoryClient.createWallet(
        {
            createWalletMsg: {
                controller_addr: accounts.user.address,
                guardians: {
                    addresses: [accounts.guardian_1.address, accounts.guardian_2.address],
                },
                relayers: [accounts.relayer_1.address, accounts.relayer_2.address],
                proxy_initial_funds: initialFunds,
                label: "wallet",
            },
        },
        getDefaultWalletCreationFee(chains),
        undefined,
        totalFeeToSend
    );

    walletAddress = CWClient.getContractAddrFromEvent(res, "wasm-vectis.proxy.v1.MsgInstantiate", "_contract_address");
    return walletAddress!;
}

function get_multisig_from_event(events: Event[]): string {
    let event = events.find((ev) => ev.type == "wasm.vectis.proxy.v1.MsgInstantiate");
    let attr = event?.attributes.find((at) => at.key == "vectis.proxy.v1.MsgReplyMultisigInstantiate");
    return attr!.value;
}
