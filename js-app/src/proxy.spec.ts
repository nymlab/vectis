import { defaultExecuteFee, defaultInstantiateFee, defaultUploadFee } from "./util/fee";
import { assert } from "@cosmjs/utils";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { createRelayTransaction, createSigningClient, getContract, mnemonicToKeyPair } from "./util/utils";
import { deployFactoryContract } from "./util/contracts";
import { Addr, BankMsg, Coin, ProxyClient } from "../types/ProxyContract";
import { FactoryClient } from "../types/FactoryContract";
import { coin } from "@cosmjs/stargate";

import {
    addrPrefix,
    adminAddr,
    adminMnemonic,
    coinMinDenom,
    cw20CodePath,
    guardian1Addr,
    guardian2Addr,
    relayer1Addr,
    relayer1Mnemonic,
    relayer2Addr,
    rpcEndPoint,
    testWalletInitialFunds,
    userAddr,
    userMnemonic,
} from "./util/env";

/**
 * This suite tests Proxy contract methods
 */
describe("Proxy Suite: ", () => {
    let userClient: SigningCosmWasmClient;
    let adminClient: SigningCosmWasmClient;
    let client: CosmWasmClient;
    let proxyWalletAddress: Addr;

    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;

    async function createTestProxyWallet(): Promise<Addr> {
        assert(factoryClient);
        const userKeypair = await mnemonicToKeyPair(userMnemonic!);
        const walletCreationFee = await factoryClient.fee();

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_pubkey: toBase64(userKeypair.pubkey),
                    guardians: {
                        addresses: [guardian1Addr!, guardian2Addr!],
                        guardians_multisig: {
                            threshold_absolute_count: 1,
                            multisig_initial_funds: [],
                        },
                    },
                    relayers: [relayer1Addr!, relayer2Addr!],
                    proxy_initial_funds: [testWalletInitialFunds as Coin],
                },
            },
            Number(walletCreationFee.amount),
            undefined,
            [coin(1100, coinMinDenom!) as Coin]
        );

        const { wallets } = await factoryClient.walletsOf({ user: userAddr! });
        return wallets[0];
    }

    beforeAll(async () => {
        try {
            userClient = await createSigningClient(userMnemonic!, addrPrefix!);
            adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
            client = await CosmWasmClient.connect(rpcEndPoint!);
            const { contractAddress } = await deployFactoryContract(adminClient);

            factoryClient = new FactoryClient(userClient, userAddr!, contractAddress);
            proxyWalletAddress = await createTestProxyWallet();

            proxyClient = new ProxyClient(userClient, userAddr!, proxyWalletAddress);
        } catch (err) {
            console.error("Failed to load scenario!", err);
        }
    });

    beforeEach(() => {
        assert(userClient);
        assert(adminClient);
        assert(client);
        assert(proxyWalletAddress);
        assert(factoryClient);
        assert(proxyClient);
    });

    it("Should get correct info from proxy wallet", async () => {
        const info = await proxyClient.info();
        expect(info.guardians).toEqual([guardian1Addr!, guardian2Addr!]);
        expect(info.relayers).toEqual([relayer1Addr!, relayer2Addr!]);
        expect(info.is_frozen).toEqual(false);
        expect(info.multisig_code_id).toBeGreaterThanOrEqual(1);
        expect(info.multisig_address).toBeTruthy();
        expect(info.nonce).toBeGreaterThan(0);
    });

    it("User can use wallet to send funds", async () => {
        const sendAmount = coin(2, coinMinDenom!);
        const sendMsg: BankMsg = {
            send: {
                to_address: adminAddr!,
                amount: [sendAmount as Coin],
            },
        };

        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        const adminBalanceBefore = await client.getBalance(adminAddr!, coinMinDenom!);
        await proxyClient.execute({
            msgs: [
                {
                    bank: sendMsg,
                },
            ],
        });
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        const adminBalanceAfter = await client.getBalance(adminAddr!, coinMinDenom!);

        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);
        const adminDiff = Number(adminBalanceBefore.amount) - Number(adminBalanceAfter.amount);

        expect(walletDiff).toEqual(Number(sendAmount.amount));
        expect(adminDiff).toEqual(-Number(sendAmount.amount));
    });

    it("User can send funds to wallet", async () => {
        const sendAmount = coin(10, coinMinDenom!);
        const sendMsg: BankMsg = {
            send: {
                to_address: proxyWalletAddress,
                amount: [sendAmount as Coin],
            },
        };

        const userBalanceBefore = await client.getBalance(userAddr!, coinMinDenom!);
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        await proxyClient.execute({
            msgs: [
                {
                    bank: sendMsg,
                },
            ],
        });
        const userBalanceAfter = await client.getBalance(userAddr!, coinMinDenom!);
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom!);

        const userDiff = Number(userBalanceBefore.amount) - Number(userBalanceAfter.amount);
        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        expect(userDiff).toEqual(Number(sendAmount.amount));
        expect(walletDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should relay bank message as a relayer", async () => {
        const relayerClient = await createSigningClient(relayer1Mnemonic!, addrPrefix!);
        const relayerProxyClient = new ProxyClient(relayerClient, relayer1Addr!, proxyWalletAddress);

        const sendAmount = coin(10, coinMinDenom!);
        const sendMsg: BankMsg = {
            send: {
                to_address: adminAddr!,
                amount: [sendAmount as Coin],
            },
        };

        const relayTransaction = await createRelayTransaction(userMnemonic!, 0, JSON.stringify(sendMsg));
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom!);

        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            defaultExecuteFee
        );

        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        const diff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        expect(sendAmount.amount).toEqual(String(diff));
        relayerClient.disconnect();
    });

    it("Should relay WASM message as a relayer", async () => {
        const relayerClient = await createSigningClient(relayer1Mnemonic!, addrPrefix!);
        const relayerProxyClient = new ProxyClient(relayerClient, relayer1Addr!, proxyWalletAddress);

        // Instantiate a new CW20 contract giving the wallet some funds
        const cw20Code = getContract(cw20CodePath!);
        const cw20Res = await adminClient.upload(adminAddr!, cw20Code, defaultUploadFee);

        const initAmount = "1000";
        const cw20contract = await adminClient.instantiate(
            adminAddr!,
            cw20Res.codeId,
            {
                name: "scw-test",
                symbol: "scw",
                decimals: 10,
                initial_balances: [{ address: proxyWalletAddress, amount: initAmount }],
                mint: null,
                marketing: null,
            },
            "SCW Test CW20",
            defaultInstantiateFee,
            {
                funds: [],
            }
        );
        const transferAmount = "100";
        const transferMsg = {
            transfer: { recipient: guardian1Addr!, amount: transferAmount },
        };

        const wasmMsg = {
            wasm: {
                execute: {
                    contract_addr: cw20contract.contractAddress,
                    msg: toBase64(toUtf8(JSON.stringify(transferMsg))),
                    funds: [],
                },
            },
        };

        const relayTransaction = await createRelayTransaction(userMnemonic!, 1, JSON.stringify(wasmMsg));
        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            defaultExecuteFee
        );

        const postFund = await relayerClient.queryContractSmart(cw20contract.contractAddress, {
            balance: { address: proxyWalletAddress },
        });

        expect(postFund.balance).toEqual("900");
        relayerClient.disconnect();
    });

    afterAll(() => {
        userClient?.disconnect();
        client?.disconnect();
    });
});
