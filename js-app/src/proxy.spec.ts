import {
    defaultExecuteFee,
    defaultInstantiateFee,
    defaultUploadFee,
    defaultWalletCreationFee,
    defaultRelayFee,
    defaultSendFee,
} from "./util/fee";
import { assert } from "@cosmjs/utils";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { createRelayTransaction, createSigningClient, getContract, mnemonicToKeyPair } from "./util/utils";
import { uploadContracts, instantiateFactoryContract, instantiateGovec } from "./util/contracts";
import { Addr, CosmosMsg_for_Empty as CosmosMsg, BankMsg, Coin, ProxyClient } from "../types/ProxyContract";
import { FactoryClient } from "../types/FactoryContract";
import { GovecClient } from "../types/GovecContract";
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
        const totalFee: Number = Number(walletCreationFee.amount) + Number(testWalletInitialFunds.amount);

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_pubkey: toBase64(userKeypair.pubkey),
                    guardians: {
                        addresses: [guardian1Addr!, guardian2Addr!],
                        // guardians_multisig: null,
                        guardians_multisig: {
                            threshold_absolute_count: 1,
                            multisig_initial_funds: [],
                        },
                    },
                    relayers: [relayer1Addr!, relayer2Addr!],
                    proxy_initial_funds: [testWalletInitialFunds as Coin],
                },
            },
            defaultWalletCreationFee,
            undefined,
            [coin(totalFee.toString(), coinMinDenom!) as Coin]
        );

        const { wallets } = await factoryClient.walletsOf({ user: userAddr! });
        return wallets[0];
    }

    beforeAll(async () => {
        try {
            userClient = await createSigningClient(userMnemonic!, addrPrefix!);
            adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
            client = await CosmWasmClient.connect(rpcEndPoint!);
            const { factoryRes, proxyRes, multisigRes, govecRes } = await uploadContracts(adminClient);
            const { factoryAddr } = await instantiateFactoryContract(
                adminClient,
                factoryRes.codeId,
                proxyRes.codeId,
                multisigRes.codeId
            );
            factoryClient = new FactoryClient(adminClient, adminAddr!, factoryAddr);
            const { govecAddr } = await instantiateGovec(adminClient, govecRes.codeId, factoryAddr);
            const govecClient = new GovecClient(adminClient, adminAddr!, govecAddr);
            const minter = await govecClient.minter();
            expect(minter.minter).toEqual(factoryAddr);

            await factoryClient.updateGovecAddr({ addr: govecAddr });

            let govec = await factoryClient.govecAddr();
            expect(govec).toEqual(govecAddr);

            proxyWalletAddress = await createTestProxyWallet();

            proxyClient = new ProxyClient(userClient, userAddr!, proxyWalletAddress);
            const info = await proxyClient.info();
            expect(info.nonce).toEqual(0);
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
        expect(info.guardians).toContain(guardian1Addr!);
        expect(info.guardians).toContain(guardian2Addr!);
        expect(info.relayers).toContain(relayer2Addr!);
        expect(info.relayers).toContain(relayer1Addr!);
        expect(info.is_frozen).toEqual(false);
        expect(info.multisig_address).toBeDefined();
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
        const sendAmount = coin(10_000, coinMinDenom!);
        const userBalanceBefore = await client.getBalance(userAddr!, coinMinDenom!);
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        await userClient.sendTokens(userAddr!, proxyWalletAddress, [sendAmount], defaultSendFee);
        const userBalanceAfter = await client.getBalance(userAddr!, coinMinDenom!);
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom!);

        const userDiff = Number(userBalanceBefore.amount) - Number(userBalanceAfter.amount);
        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        // Gas price
        expect(userDiff).toBeGreaterThanOrEqual(Number(sendAmount.amount));
        expect(walletDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should relay bank message as a relayer", async () => {
        const relayerClient = await createSigningClient(relayer1Mnemonic!, addrPrefix!);
        const relayerProxyClient = new ProxyClient(relayerClient, relayer1Addr!, proxyWalletAddress);
        const info = await relayerProxyClient.info();

        const sendAmount = coin(10_000, coinMinDenom!);
        const sendMsg: CosmosMsg = {
            bank: {
                send: {
                    to_address: adminAddr!,
                    amount: [sendAmount as Coin],
                },
            },
        };

        const relayTransaction = await createRelayTransaction(userMnemonic!, info.nonce, JSON.stringify(sendMsg));
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom!);

        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            defaultRelayFee
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

        const cosmosWasmMsg: CosmosMsg = {
            wasm: {
                execute: {
                    contract_addr: cw20contract.contractAddress,
                    msg: toBase64(toUtf8(JSON.stringify(transferMsg))),
                    funds: [],
                },
            },
        };

        const relayTransaction = await createRelayTransaction(userMnemonic!, 1, JSON.stringify(cosmosWasmMsg));
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
