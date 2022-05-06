import { assert } from "@cosmjs/utils";
import { toBase64 } from "@cosmjs/encoding";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { deployFactoryContract, FACTORY_INITIAL_FUND } from "./util/contracts";
import { createSigningClient, mnemonicToKeyPair } from "./util/utils";
import { Coin, FactoryClient, Addr } from "../types/FactoryContract";
import { coin } from "@cosmjs/stargate";

import {
    addrPrefix,
    adminAddr,
    adminMnemonic,
    coinMinDenom,
    guardian1Addr,
    guardian2Addr,
    relayer1Addr,
    relayer2Addr,
    rpcEndPoint,
    testWalletInitialFunds,
    userAddr,
    userMnemonic,
} from "./util/env";

/**
 * This suite tests Factory contract methods
 */
describe("Factory Suite: ", () => {
    let adminClient: SigningCosmWasmClient;
    let client: CosmWasmClient;
    let proxyCodeId: number;
    let stakingCodeId: number;

    let factoryClient: FactoryClient;
    let proxyWalletAddress: Addr;

    beforeAll(async () => {
        adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
        client = await CosmWasmClient.connect(rpcEndPoint!);
        const { contractAddress, proxyRes, stakingRes } = await deployFactoryContract(adminClient);
        proxyCodeId = proxyRes.codeId;
        stakingCodeId = stakingRes.codeId;

        factoryClient = new FactoryClient(adminClient, adminAddr!, contractAddress);
    });

    beforeEach(() => {
        assert(adminClient);
        assert(client);
        assert(factoryClient);
    });

    it("Should have correct funds in Factory contract", async () => {
        const fund = await client.getBalance(factoryClient.contractAddress, coinMinDenom!);
        expect(fund).toEqual(FACTORY_INITIAL_FUND);
    });

    it("Should store Proxy code id in Factory contract", async () => {
        const codeId = await client.queryContractSmart(factoryClient.contractAddress, {
            code_id: { ty: "Proxy" },
        });
        expect(codeId).toEqual(proxyCodeId);
    });

    it("Should create Governance and Staking contracts from Factory contract", async () => {
        const res = await factoryClient.createGovernance({
            stakingOptions: { duration: null, code_id: stakingCodeId },
            initialBalances: [],
        });
        expect(res).toBeTruthy();
        // TODO: Verify events
    });

    it("Should create a new proxy wallet with multisig", async () => {
        const userKeypair = await mnemonicToKeyPair(userMnemonic!);
        const walletCreationFee = await factoryClient.fee();

        const newWalletRes = await factoryClient.createWallet(
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

        const wasmEvent = newWalletRes.logs[0].events.length;
        const wasmEventAttributes = newWalletRes.logs[0].events[wasmEvent - 1].attributes;
        proxyWalletAddress = wasmEventAttributes.find((event) => event.key === "proxy_address")?.value!;

        expect(proxyWalletAddress).toBeTruthy();
    });

    it("Should find created proxy wallet in Factory contract", async () => {
        // By calling 'wallets'
        const { wallets } = await factoryClient.wallets({});
        expect(wallets.length).toEqual(1);
        expect(wallets[0]).toEqual(proxyWalletAddress);

        // By calling 'wallets_of' with user address
        const { wallets: userWallets } = await factoryClient.walletsOf({ user: userAddr! });
        expect(userWallets.length).toEqual(1);
        expect(userWallets[0]).toEqual(proxyWalletAddress);
    });

    it("Should get correct balance in proxy wallet", async () => {
        const balance = await client.getBalance(proxyWalletAddress, coinMinDenom!);
        expect(balance).toEqual(testWalletInitialFunds);
    });

    afterAll(() => {
        adminClient?.disconnect();
        client?.disconnect();
    });
});
