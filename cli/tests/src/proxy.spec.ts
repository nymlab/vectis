import {
    defaultExecuteFee,
    defaultInstantiateFee,
    defaultUploadFee,
    defaultRelayFee,
    defaultSendFee,
} from "@vectis/core/utils/fee";
import { assert } from "@cosmjs/utils";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { createRelayTransaction, createSigningClient, mnemonicToKeyPair } from "@vectis/core/utils/utils";
import {
    Addr,
    CosmosMsg_for_Empty as CosmosMsg,
    BankMsg,
    Coin,
    ProxyClient,
} from "@vectis/types/contracts/ProxyContract";
import { FactoryClient } from "@vectis/types/contracts/FactoryContract";
import { GovecClient } from "@vectis/types/contracts/GovecContract";
import { coin } from "@cosmjs/stargate";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { getContract } from "@vectis/core/utils/fs";

import {
    addrPrefix,
    adminAddr,
    adminMnemonic,
    coinMinDenom,
    cw20CodePath,
    guardian1Addr,
    guardian1Mnemonic,
    guardian2Addr,
    guardian2Mnemonic,
    relayer1Addr,
    relayer1Mnemonic,
    relayer2Addr,
    rpcEndPoint,
    userAddr,
    userMnemonic,
} from "@vectis/core/utils/constants";
import { createTestProxyWallets } from "./mocks/proxyWallet";
import { instantiateFactoryContract, instantiateGovec } from "@vectis/core/contracts";

/**
 * This suite tests Proxy contract methods
 */
describe("Proxy Suite: ", () => {
    let userClient: SigningCosmWasmClient;
    let guardianClient: SigningCosmWasmClient;
    let adminClient: SigningCosmWasmClient;
    let client: CosmWasmClient;
    let proxyWalletAddress: Addr;
    let proxyWalletMultisigAddress: Addr;

    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let guardianProxyClient: ProxyClient;

    beforeAll(async () => {
        const { factoryRes, govecRes, proxyRes, multisigRes } = await import("../../uploadInfo.json" as string);

        userClient = await createSigningClient(userMnemonic, addrPrefix);
        adminClient = await createSigningClient(adminMnemonic, addrPrefix);
        guardianClient = await createSigningClient(guardian1Mnemonic, addrPrefix);
        client = await CosmWasmClient.connect(rpcEndPoint);

        const { factoryAddr } = await instantiateFactoryContract(
            adminClient,
            factoryRes.codeId,
            proxyRes.codeId,
            multisigRes.codeId
        );

        factoryClient = new FactoryClient(adminClient, adminAddr, factoryAddr);

        const { govecAddr } = await instantiateGovec(adminClient, govecRes.codeId, factoryAddr);

        const govecClient = new GovecClient(adminClient, adminAddr, govecAddr);

        const { minter } = await govecClient.minter();
        expect(minter).toEqual(factoryAddr);

        await factoryClient.updateGovecAddr({ addr: govecAddr });

        const govec = await factoryClient.govecAddr();
        expect(govec).toEqual(govecAddr);

        const [walletAddr, walletMSAddr] = await createTestProxyWallets(factoryClient);
        proxyWalletAddress = walletAddr;
        proxyWalletMultisigAddress = walletMSAddr;

        proxyClient = new ProxyClient(userClient, userAddr, proxyWalletAddress);
        guardianProxyClient = new ProxyClient(guardianClient, guardian1Addr, proxyWalletAddress);
    });

    it("Should get correct info from proxy wallet", async () => {
        const info = await proxyClient.info();
        expect(info.guardians).toContain(guardian1Addr);
        expect(info.guardians).toContain(guardian2Addr);
        expect(info.relayers).toContain(relayer2Addr);
        expect(info.relayers).toContain(relayer1Addr);
        expect(info.is_frozen).toEqual(false);
        expect(info.multisig_address).toBeFalsy();
        expect(info.nonce).toEqual(0);
    });

    it("Should be able to use wallet to send funds as user", async () => {
        const sendAmount = coin(2, coinMinDenom);
        const sendMsg: BankMsg = {
            send: {
                to_address: adminAddr,
                amount: [sendAmount as Coin],
            },
        };

        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom);
        const adminBalanceBefore = await client.getBalance(adminAddr!, coinMinDenom);
        await proxyClient.execute({
            msgs: [
                {
                    bank: sendMsg,
                },
            ],
        });
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom);
        const adminBalanceAfter = await client.getBalance(adminAddr, coinMinDenom);

        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);
        const adminDiff = Number(adminBalanceBefore.amount) - Number(adminBalanceAfter.amount);

        expect(walletDiff).toEqual(Number(sendAmount.amount));
        expect(adminDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should be able to send funds to wallet as user", async () => {
        const sendAmount = coin(10_000, coinMinDenom);
        const userBalanceBefore = await client.getBalance(userAddr!, coinMinDenom!);
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, coinMinDenom);
        await userClient.sendTokens(userAddr, proxyWalletAddress, [sendAmount], defaultSendFee);
        const userBalanceAfter = await client.getBalance(userAddr!, coinMinDenom);
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, coinMinDenom);

        const userDiff = Number(userBalanceBefore.amount) - Number(userBalanceAfter.amount);
        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        expect(userDiff).toBeGreaterThanOrEqual(Number(sendAmount.amount));
        expect(walletDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should be able to freeze and unfreeze wallet as guardian", async () => {
        assert(guardianProxyClient, "guardianProxyClient is not defined");
        let is_frozen: boolean = false;

        // Freeze
        await guardianProxyClient.revertFreezeStatus();
        is_frozen = (await proxyClient.info()).is_frozen;
        expect(is_frozen).toBeTruthy();

        // Unfreeze
        await guardianProxyClient.revertFreezeStatus();
        is_frozen = (await proxyClient.info()).is_frozen;
        expect(is_frozen).toBeFalsy();
    });

    it("Shouldn't be able to perform operations if a wallet is frozen", async () => {
        assert(guardianProxyClient, "guardianProxyClient is not defined");

        // Freeze
        await guardianProxyClient.revertFreezeStatus();

        // Try to send a bank message
        try {
            const sendMsg: BankMsg = {
                send: {
                    to_address: adminAddr!,
                    amount: [coin(2, coinMinDenom!) as Coin],
                },
            };
            await proxyClient.execute({
                msgs: [
                    {
                        bank: sendMsg,
                    },
                ],
            });

            // Force test failure, function didn't throw :/
            expect(false).toBeTruthy();
        } catch (err) {
            expect(err).toBeInstanceOf(Error);
            expect((err as Error).message).toContain("Frozen");
        }

        // Unfreeze
        await guardianProxyClient.revertFreezeStatus();
    });

    it("Should rotate user key as guardian", async () => {
        assert(guardianProxyClient, "guardianProxyClient is not defined");

        // New owner is admin
        await guardianProxyClient.rotateUserKey({
            newUserAddress: adminAddr,
        });

        // User (old wallet owner) shouldn't have the wallet anymore
        const { wallets: userWallets } = await factoryClient.walletsOf({ user: userAddr! });
        expect(userWallets).not.toContain(proxyWalletAddress);

        // Admin (admin wallet owner) should have the wallet
        const { wallets: adminWallets } = await factoryClient.walletsOf({ user: adminAddr! });
        expect(adminWallets).toContain(proxyWalletAddress);

        // Shouldn't be able to perform operations as user since it's not his wallet anymore
        try {
            const sendMsg: BankMsg = {
                send: {
                    to_address: adminAddr!,
                    amount: [coin(2, coinMinDenom!) as Coin],
                },
            };
            await proxyClient.execute({
                msgs: [
                    {
                        bank: sendMsg,
                    },
                ],
            });

            // Force test failure, function didn't throw :/
            expect(false).toBeTruthy();
        } catch (err) {
            expect(err).toBeInstanceOf(Error);
            expect((err as Error).message).toContain("IsNotUser");
        }

        // Return wallet to the user
        await guardianProxyClient.rotateUserKey({
            newUserAddress: userAddr!,
        });
    });

    it("Should be able to rotate key of multisig wallet", async () => {
        const clientG1 = await createSigningClient(guardian1Mnemonic, addrPrefix);
        const clientG2 = await createSigningClient(guardian2Mnemonic, addrPrefix);

        try {
            const msProxyClient = new ProxyClient(userClient, userAddr!, proxyWalletMultisigAddress);
            const { multisig_address } = await msProxyClient.info();

            const rotateUserKey: CosmosMsg = {
                wasm: {
                    execute: {
                        contract_addr: proxyWalletMultisigAddress,
                        msg: toBase64(
                            toUtf8(
                                JSON.stringify({
                                    rotate_user_key: { new_user_address: adminAddr! },
                                })
                            )
                        ),
                        funds: [],
                    },
                },
            };
            const proposal: CwPropSingleExecuteMsg = {
                propose: {
                    title: "Rotate user key",
                    description: "Need to rotate user key",
                    msgs: [rotateUserKey],
                    latest: null,
                },
            };
            await clientG1.execute(guardian1Addr!, multisig_address!, proposal, defaultExecuteFee);

            // Should have proposal in the list
            const queryProps: ProposalQueryMsg = { list_proposals: {} };
            const { proposals } = await clientG1.queryContractSmart(multisig_address!, queryProps);
            const prop = proposals.find((p: any) => p.title === proposal.propose.title);
            expect(prop).toBeTruthy();
            const propId = prop.id;

            // At this point, since Guardian1 proposed, his vote is already YES
            // Now Guardian2 votes YES
            const voteYes: CwPropSingleExecuteMsg = {
                vote: {
                    proposal_id: propId,
                    vote: "yes",
                },
            };
            await clientG2.execute(guardian2Addr!, multisig_address!, voteYes, defaultExecuteFee);

            // Since threshold is 2, freezing should be approved and executed
            const executeFreeze: CwPropSingleExecuteMsg = {
                execute: {
                    proposal_id: propId,
                },
            };
            await clientG2.execute(guardian2Addr!, multisig_address!, executeFreeze, defaultExecuteFee);

            // At this point, the wallet should be frozen
            const { user_addr } = await msProxyClient.info();
            expect(user_addr).toEqual(adminAddr!);
        } catch (err) {
            throw err;
        } finally {
            clientG1.disconnect();
            clientG2.disconnect();
        }
    });

    it("Should be able to freeze multisig wallet", async () => {
        const clientG1 = await createSigningClient(guardian1Mnemonic!, addrPrefix!);
        const clientG2 = await createSigningClient(guardian2Mnemonic!, addrPrefix!);

        try {
            const msProxyClient = new ProxyClient(userClient, userAddr!, proxyWalletMultisigAddress);
            const { multisig_address } = await msProxyClient.info();

            // Propose freezing of multisig wallet
            const revertFreezeStatusMsg: CosmosMsg = {
                wasm: {
                    execute: {
                        contract_addr: proxyWalletMultisigAddress,
                        msg: toBase64(
                            toUtf8(
                                JSON.stringify({
                                    revert_freeze_status: {},
                                })
                            )
                        ),
                        funds: [],
                    },
                },
            };
            const proposal: CwPropSingleExecuteMsg = {
                propose: {
                    title: "Revert freeze status",
                    description: "Need to revert freeze status",
                    msgs: [revertFreezeStatusMsg],
                    latest: null,
                },
            };
            await clientG1.execute(guardian1Addr!, multisig_address!, proposal, defaultExecuteFee);

            // Should have proposal in the list
            const queryProps: ProposalQueryMsg = { list_proposals: {} };
            const { proposals } = await clientG1.queryContractSmart(multisig_address!, queryProps);
            const prop = proposals.find((p: any) => p.title === proposal.propose.title);
            expect(prop).toBeTruthy();
            const propId = prop.id;

            // At this point, since Guardian1 proposed, his vote is already YES
            // Now Guardian2 votes YES
            const voteYes: CwPropSingleExecuteMsg = {
                vote: {
                    proposal_id: propId,
                    vote: "yes",
                },
            };
            await clientG2.execute(guardian2Addr!, multisig_address!, voteYes, defaultExecuteFee);

            // Since threshold is 2, freezing should be approved and executed
            const executeFreeze: CwPropSingleExecuteMsg = {
                execute: {
                    proposal_id: propId,
                },
            };
            await clientG2.execute(guardian2Addr!, multisig_address!, executeFreeze, defaultExecuteFee);

            // At this point, the wallet should be frozen
            const { is_frozen } = await msProxyClient.info();
            expect(is_frozen).toBeTruthy();

            // TODO: Write the exact same thing but for unfreezing
        } catch (err) {
            throw err;
        } finally {
            clientG1.disconnect();
            clientG2.disconnect();
        }
    });

    it("Should relay bank message as a relayer", async () => {
        const relayerClient = await createSigningClient(relayer1Mnemonic!, addrPrefix!);
        // We should use userAddr here
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
        adminClient?.disconnect();
        userClient?.disconnect();
        guardianClient?.disconnect();
        client?.disconnect();
    });
});
