import { coin, Event } from "@cosmjs/stargate";
import { FactoryT, FactoryClient } from "../../interfaces";
import { hostAccounts, hostChain, remoteChain, remoteAccounts } from "../../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../../utils/fees";
import { CWClient } from "../../clients";

export async function createTestProxyWallets(factoryClient: FactoryClient): Promise<FactoryT.Addr[]> {
    const { wallet_fee } = await factoryClient.fees();

    const proxy_initial_funds = walletInitialFunds(hostChain).amount == "0" ? [] : [walletInitialFunds(hostChain)];
    const totalFee: Number = Number(wallet_fee.amount) + Number(walletInitialFunds(hostChain).amount);
    let funds = totalFee == 0 ? undefined : [coin(totalFee.toString(), hostChain.feeToken) as FactoryT.Coin];
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
                proxy_initial_funds,
                label: "wallet",
            },
        },
        getDefaultWalletCreationFee(hostChain),
        undefined,
        funds
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
                proxy_initial_funds,
                label: "cronkitty test 1",
            },
        },
        getDefaultWalletCreationFee(hostChain),
        undefined,
        funds
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
    const proxy_initial_funds =
        walletInitialFunds(chains).amount == "0" ? [] : [walletInitialFunds(chains) as FactoryT.Coin];
    const { wallet_fee } = await factoryClient.fees();
    const totalFee: Number = Number(wallet_fee.amount) + Number(walletInitialFunds(hostChain).amount);
    let totalFeeToSend = totalFee == 0 ? undefined : [coin(totalFee.toString(), chains.feeToken) as FactoryT.Coin];

    console.log("initial funds: ", proxy_initial_funds);
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
                proxy_initial_funds,
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
