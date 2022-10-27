import { assert } from "@cosmjs/utils";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { Addr, CosmosMsgForEmpty as CosmosMsg, BankMsg, Coin } from "@vectis/types/contracts/Proxy.types";
import { FactoryClient, GovecClient } from "@vectis/core/clients";
import { coin } from "@cosmjs/stargate";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { getContract } from "@vectis/core/utils/fs";

import { cw20CodePath, uploadReportPath } from "@vectis/core/utils/constants";
import { createTestProxyWallets } from "./mocks/proxyWallet";
import { CWClient } from "@vectis/core/clients";
import {
    getDefaultRelayFee,
    getDefaultSendFee,
    getDefaultUploadFee,
    HOST_ACCOUNTS,
    HOST_CHAIN,
    getInitialFactoryBalance,
} from "./mocks/constants";
import { ProxyClient } from "@vectis/types";

/**
 * This suite tests Proxy contract methods
 */
describe("Proxy Suite: ", () => {
    let userClient: CWClient;
    let guardianClient: CWClient;
    let adminClient: CWClient;
    let client: CosmWasmClient;
    let proxyWalletAddress: Addr;
    let proxyWalletMultisigAddress: Addr;

    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let guardianProxyClient: ProxyClient;

    beforeAll(async () => {
        const { host } = await import(uploadReportPath);
        const { factoryRes, govecRes, proxyRes, multisigRes } = host;

        userClient = await CWClient.connectWithAccount("juno_localnet", "user");
        adminClient = await CWClient.connectWithAccount("juno_localnet", "admin");
        guardianClient = await CWClient.connectWithAccount("juno_localnet", "guardian_1");
        client = await CosmWasmClient.connect(HOST_CHAIN.rpcUrl);

        factoryClient = await FactoryClient.instantiate(
            adminClient,
            factoryRes.codeId,
            FactoryClient.createFactoryInstMsg("juno_localnet", proxyRes.codeId, multisigRes.codeId),
            [getInitialFactoryBalance(HOST_CHAIN)]
        );

        const govecClient = await GovecClient.instantiate(adminClient, govecRes.codeId, {
            factory: factoryClient.contractAddress,
            initial_balances: [],
        });

        await factoryClient.updateGovecAddr({ addr: govecClient.contractAddress });

        const govec = await factoryClient.govecAddr();
        expect(govec).toEqual(govecClient.contractAddress);

        const [walletAddr, walletMSAddr] = await createTestProxyWallets(factoryClient);
        proxyWalletAddress = walletAddr;
        proxyWalletMultisigAddress = walletMSAddr;

        proxyClient = new ProxyClient(userClient, userClient.sender, proxyWalletAddress);
        guardianProxyClient = new ProxyClient(guardianClient, guardianClient.sender, proxyWalletAddress);
    });

    it("Should get correct info from proxy wallet", async () => {
        const info = await proxyClient.info();
        expect(info.guardians).toContain(HOST_ACCOUNTS.guardian_1.address);
        expect(info.guardians).toContain(HOST_ACCOUNTS.guardian_2.address);
        expect(info.relayers).toContain(HOST_ACCOUNTS.relayer_1.address);
        expect(info.relayers).toContain(HOST_ACCOUNTS.relayer_2.address);
        expect(info.is_frozen).toEqual(false);
        expect(info.multisig_address).toBeFalsy();
        expect(info.nonce).toEqual(0);
    });

    it("Should be able to use wallet to send funds as user", async () => {
        const sendAmount = coin(2, HOST_CHAIN.feeToken);
        const sendMsg: BankMsg = {
            send: {
                to_address: HOST_ACCOUNTS.admin.address,
                amount: [sendAmount as Coin],
            },
        };

        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken);
        const adminBalanceBefore = await client.getBalance(HOST_ACCOUNTS.admin.address, HOST_CHAIN.feeToken);
        await proxyClient.execute({
            msgs: [
                {
                    bank: sendMsg,
                },
            ],
        });
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken);
        const adminBalanceAfter = await client.getBalance(HOST_ACCOUNTS.admin.address, HOST_CHAIN.feeToken);

        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);
        const adminDiff = Number(adminBalanceBefore.amount) - Number(adminBalanceAfter.amount);

        expect(walletDiff).toEqual(Number(sendAmount.amount));
        expect(adminDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should be able to send funds to wallet as user", async () => {
        const sendAmount = coin(10_000, HOST_CHAIN.feeToken);
        const userBalanceBefore = await client.getBalance(HOST_ACCOUNTS.user.address, HOST_CHAIN.feeToken!);
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken);
        await userClient.sendTokens(
            HOST_ACCOUNTS.user.address,
            proxyWalletAddress,
            [sendAmount],
            getDefaultSendFee(HOST_CHAIN)
        );
        const userBalanceAfter = await client.getBalance(HOST_ACCOUNTS.user.address, HOST_CHAIN.feeToken);
        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken);

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
                    to_address: HOST_ACCOUNTS.admin.address,
                    amount: [coin(2, HOST_CHAIN.feeToken!) as Coin],
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
            newUserAddress: HOST_ACCOUNTS.admin.address,
        });

        /*  // User (old wallet owner) shouldn't have the wallet anymore
        const { wallets: userWallets } = await factoryClient.walletsOf({ user: HOST_ACCOUNTS.user.address! });
        expect(userWallets).not.toContain(proxyWalletAddress);

        // Admin (admin wallet owner) should have the wallet
        const { wallets: adminWallets } = await factoryClient.walletsOf({ user: HOST_ACCOUNTS.admin.address });
        expect(adminWallets).toContain(proxyWalletAddress);
 */
        // Shouldn't be able to perform operations as user since it's not his wallet anymore
        try {
            const sendMsg: BankMsg = {
                send: {
                    to_address: HOST_ACCOUNTS.admin.address,
                    amount: [coin(2, HOST_CHAIN.feeToken!) as Coin],
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
            newUserAddress: HOST_ACCOUNTS.user.address,
        });
    });

    it("Should be able to rotate key of multisig wallet", async () => {
        const clientG1 = await CWClient.connectWithAccount("juno_localnet", "guardian_1");
        const clientG2 = await CWClient.connectWithAccount("juno_localnet", "guardian_2");

        try {
            const msProxyClient = new ProxyClient(userClient, HOST_ACCOUNTS.user.address!, proxyWalletMultisigAddress);
            const { multisig_address } = await msProxyClient.info();

            const rotateUserKey: CosmosMsg = {
                wasm: {
                    execute: {
                        contract_addr: proxyWalletMultisigAddress,
                        msg: toBase64(
                            toUtf8(
                                JSON.stringify({
                                    rotate_user_key: { new_user_address: HOST_ACCOUNTS.admin.address },
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
            await clientG1.execute(clientG1.sender, multisig_address!, proposal, getDefaultRelayFee(HOST_CHAIN));

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
            await clientG2.execute(clientG2.sender, multisig_address!, voteYes, "auto");

            // Since threshold is 2, freezing should be approved and executed
            const executeFreeze: CwPropSingleExecuteMsg = {
                execute: {
                    proposal_id: propId,
                },
            };
            await clientG2.execute(clientG2.sender, multisig_address!, executeFreeze, "auto");

            // At this point, the wallet should be frozen
            const { user_addr } = await msProxyClient.info();
            expect(user_addr).toEqual(HOST_ACCOUNTS.admin.address);
        } catch (err) {
            throw err;
        } finally {
            clientG1.disconnect();
            clientG2.disconnect();
        }
    });

    it("Should be able to freeze multisig wallet", async () => {
        const clientG1 = await CWClient.connectWithAccount("juno_localnet", "guardian_1");
        const clientG2 = await CWClient.connectWithAccount("juno_localnet", "guardian_2");

        try {
            const msProxyClient = new ProxyClient(userClient, userClient.sender, proxyWalletMultisigAddress);
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
            await clientG1.execute(clientG1.sender, multisig_address!, proposal, getDefaultRelayFee(HOST_CHAIN));

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
            await clientG2.execute(clientG2.sender, multisig_address!, voteYes, "auto");

            // Since threshold is 2, freezing should be approved and executed
            const executeFreeze: CwPropSingleExecuteMsg = {
                execute: {
                    proposal_id: propId,
                },
            };
            await clientG2.execute(clientG2.sender, multisig_address!, executeFreeze, "auto");

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
        const relayerClient = await CWClient.connectWithAccount("juno_localnet", "relayer_1");
        // We should use HOST_ACCOUNTS.user.address here
        const relayerProxyClient = new ProxyClient(relayerClient, relayerClient.sender, proxyWalletAddress);
        const info = await relayerProxyClient.info();

        const sendAmount = coin(10_000, HOST_CHAIN.feeToken!);
        const sendMsg: CosmosMsg = {
            bank: {
                send: {
                    to_address: HOST_ACCOUNTS.admin.address,
                    amount: [sendAmount as Coin],
                },
            },
        };

        const relayTransaction = await CWClient.createRelayTransaction(
            HOST_ACCOUNTS.user.mnemonic,
            info.nonce,
            JSON.stringify(sendMsg)
        );
        const walletBalanceBefore = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken!);

        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            "auto"
        );

        const walletBalanceAfter = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken!);
        const diff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        expect(sendAmount.amount).toEqual(String(diff));
        relayerClient.disconnect();
    });

    it("Should relay WASM message as a relayer", async () => {
        const relayerClient = await CWClient.connectWithAccount("juno_localnet", "relayer_1");
        const relayerProxyClient = new ProxyClient(relayerClient, relayerClient.sender, proxyWalletAddress);

        // Instantiate a new CW20 contract giving the wallet some funds
        const cw20Code = getContract(cw20CodePath!);
        const cw20Res = await adminClient.upload(
            HOST_ACCOUNTS.admin.address,
            cw20Code,
            getDefaultUploadFee(HOST_CHAIN)
        );

        const initAmount = "1000";
        const cw20contract = await adminClient.instantiate(
            HOST_ACCOUNTS.admin.address,
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
            "auto",
            {
                funds: [],
            }
        );
        const transferAmount = "100";
        const transferMsg = {
            transfer: { recipient: HOST_ACCOUNTS.guardian_1.address, amount: transferAmount },
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

        const relayTransaction = await CWClient.createRelayTransaction(
            HOST_ACCOUNTS.user.mnemonic,
            1,
            JSON.stringify(cosmosWasmMsg)
        );
        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            "auto"
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
