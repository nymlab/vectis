import { toBase64 } from "@cosmjs/encoding";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Coin, FactoryClient, Addr } from "@vectis/types/contracts/FactoryContract";
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
    uploadReportPath,
    userAddr,
    userMnemonic,
} from "@vectis/core/utils/constants";
import { defaultWalletCreationFee } from "@vectis/core/utils/fee";
import { createSigningClient, mnemonicToKeyPair } from "@vectis/core/services/cosmwasm";
import { FACTORY_INITIAL_FUND, instantiateFactoryContract } from "@vectis/core/services/factory";
import { instantiateGovec } from "@vectis/core/services/govec";

/**
 * This suite tests Factory contract methods
 */
describe("Factory Suite: ", () => {
    let adminClient: SigningCosmWasmClient;
    let client: CosmWasmClient;
    let proxyCodeId: number;

    let factoryClient: FactoryClient;
    let proxyWalletAddress: Addr;

    beforeAll(async () => {
        const { proxyRes, factoryRes, govecRes, multisigRes } = await import(uploadReportPath);
        proxyCodeId = proxyRes.codeId;

        adminClient = await createSigningClient(adminMnemonic, addrPrefix);
        client = await CosmWasmClient.connect(rpcEndPoint);

        const { factoryAddr } = await instantiateFactoryContract(
            adminClient,
            factoryRes.codeId,
            proxyCodeId,
            multisigRes.codeId,
            [FACTORY_INITIAL_FUND]
        );
        factoryClient = new FactoryClient(adminClient, adminAddr!, factoryAddr);

        const { govecAddr } = await instantiateGovec({
            client: adminClient,
            initial_balances: [],
            govecCodeId: govecRes.codeId as number,
            admin: factoryAddr,
            minter: factoryAddr,
        });

        await factoryClient.updateGovecAddr({ addr: govecAddr });
        const govec = await factoryClient.govecAddr();

        expect(govec).toEqual(govecAddr);
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

    it("Should create a new proxy wallet with multisig", async () => {
        const userKeypair = await mnemonicToKeyPair(userMnemonic!);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(testWalletInitialFunds.amount);

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
                    label: "initial label",
                },
            },
            defaultWalletCreationFee,
            undefined,
            [coin(totalFee.toString(), coinMinDenom!) as Coin]
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
