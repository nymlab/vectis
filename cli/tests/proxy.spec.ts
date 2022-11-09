import { assert } from "@cosmjs/utils";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { Addr, CosmosMsgForEmpty as CosmosMsg, BankMsg, Coin } from "../interfaces/Proxy.types";
import { FactoryClient } from "../clients";
import { coin } from "@cosmjs/stargate";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { getContract } from "../utils/fs";

import { cw20CodePath, deployReportPath, hostAccounts, hostChain } from "../utils/constants";
import { createTestProxyWallets } from "./mocks/proxyWallet";
import { CWClient } from "../clients";
import { getDefaultRelayFee, getDefaultSendFee, getDefaultUploadFee } from "../utils/fees";
import { ProxyClient } from "../interfaces";
import { delay } from "../utils/promises";

/**
 * This suite tests Proxy contract methods
 */
describe("Proxy Suite: ", () => {
    let hostUserClient: CWClient;
    let hostGuardianClient: CWClient;
    let hostAdminClient: CWClient;
    let hostClient: CosmWasmClient;
    let proxyWalletAddress: Addr;
    let proxyWalletMultisigAddress: Addr;

    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let guardianProxyClient: ProxyClient;

    beforeAll(async () => {
        const { factoryAddr } = await import(deployReportPath);

        hostUserClient = await CWClient.connectHostWithAccount("user");
        hostAdminClient = await CWClient.connectHostWithAccount("admin");
        hostGuardianClient = await CWClient.connectHostWithAccount("guardian_1");
        hostClient = await CosmWasmClient.connect(hostChain.rpcUrl);

        factoryClient = new FactoryClient(hostAdminClient, hostAdminClient.sender, factoryAddr);

        const [walletAddr, walletMSAddr] = await createTestProxyWallets(factoryClient);
        console.log(walletAddr, walletMSAddr);
        proxyWalletAddress = walletAddr;
        proxyWalletMultisigAddress = walletMSAddr;

        proxyClient = new ProxyClient(hostUserClient, hostUserClient.sender, proxyWalletAddress);
        guardianProxyClient = new ProxyClient(hostGuardianClient, hostGuardianClient.sender, proxyWalletAddress);
    });

    it("Should get correct info from proxy wallet", async () => {
        const info = await proxyClient.info();
        expect(info.guardians).toContain(hostAccounts.guardian_1.address);
        expect(info.guardians).toContain(hostAccounts.guardian_2.address);
        expect(info.relayers).toContain(hostAccounts.relayer_1.address);
        expect(info.relayers).toContain(hostAccounts.relayer_2.address);
        expect(info.is_frozen).toEqual(false);
        expect(info.multisig_address).toBeFalsy();
        expect(info.nonce).toEqual(0);
    });

    it("Should be able to use wallet to send funds as user", async () => {
        const sendAmount = coin(2, hostChain.feeToken);
        const sendMsg: BankMsg = {
            send: {
                to_address: hostAccounts.admin.address,
                amount: [sendAmount as Coin],
            },
        };

        const walletBalanceBefore = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken);
        const adminBalanceBefore = await hostClient.getBalance(hostAccounts.admin.address, hostChain.feeToken);
        await proxyClient.execute({
            msgs: [
                {
                    bank: sendMsg,
                },
            ],
        });
        const walletBalanceAfter = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken);
        const adminBalanceAfter = await hostClient.getBalance(hostAccounts.admin.address, hostChain.feeToken);

        const walletDiff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);
        const adminDiff = Number(adminBalanceBefore.amount) - Number(adminBalanceAfter.amount);

        expect(walletDiff).toEqual(Number(sendAmount.amount));
        expect(adminDiff).toEqual(-Number(sendAmount.amount));
    });

    it("Should be able to send funds to wallet as user", async () => {
        const sendAmount = coin(10_000, hostChain.feeToken);
        const userBalanceBefore = await hostClient.getBalance(hostAccounts.user.address, hostChain.feeToken!);
        const walletBalanceBefore = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken);
        await hostUserClient.sendTokens(
            hostAccounts.user.address,
            proxyWalletAddress,
            [sendAmount],
            getDefaultSendFee(hostChain)
        );
        const userBalanceAfter = await hostClient.getBalance(hostAccounts.user.address, hostChain.feeToken);
        const walletBalanceAfter = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken);

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
                    to_address: hostAccounts.admin.address,
                    amount: [coin(2, hostChain.feeToken!) as Coin],
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
            newUserAddress: hostAccounts.admin.address,
        });

        /*  // User (old wallet owner) shouldn't have the wallet anymore
        const { wallets: userWallets } = await factoryClient.walletsOf({ user: hostAccounts.user.address! });
        expect(userWallets).not.toContain(proxyWalletAddress);

        // Admin (admin wallet owner) should have the wallet
        const { wallets: adminWallets } = await factoryClient.walletsOf({ user: hostAccounts.admin.address });
        expect(adminWallets).toContain(proxyWalletAddress);
 */
        // Shouldn't be able to perform operations as user since it's not his wallet anymore
        try {
            const sendMsg: BankMsg = {
                send: {
                    to_address: hostAccounts.admin.address,
                    amount: [coin(2, hostChain.feeToken!) as Coin],
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
            newUserAddress: hostAccounts.user.address,
        });
    });

    it("Should be able to rotate key of multisig wallet", async () => {
        const clientG1 = await CWClient.connectHostWithAccount("guardian_1");
        const clientG2 = await CWClient.connectHostWithAccount("guardian_2");

        try {
            const msProxyClient = new ProxyClient(
                hostUserClient,
                hostAccounts.user.address!,
                proxyWalletMultisigAddress
            );
            const { multisig_address } = await msProxyClient.info();

            const rotateUserKey: CosmosMsg = {
                wasm: {
                    execute: {
                        contract_addr: proxyWalletMultisigAddress,
                        msg: toBase64(
                            toUtf8(
                                JSON.stringify({
                                    rotate_user_key: {
                                        new_user_address: hostAccounts.admin.address,
                                    },
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
            await clientG1.execute(clientG1.sender, multisig_address!, proposal, getDefaultRelayFee(hostChain));

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
            expect(user_addr).toEqual(hostAccounts.admin.address);
        } catch (err) {
            throw err;
        } finally {
            clientG1.disconnect();
            clientG2.disconnect();
        }
    });

    it("Should be able to freeze multisig wallet", async () => {
        const clientG1 = await CWClient.connectHostWithAccount("guardian_1");
        const clientG2 = await CWClient.connectHostWithAccount("guardian_2");

        try {
            const msProxyClient = new ProxyClient(hostUserClient, hostUserClient.sender, proxyWalletMultisigAddress);
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
            await clientG1.execute(clientG1.sender, multisig_address!, proposal, getDefaultRelayFee(hostChain));

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
        const relayerClient = await CWClient.connectHostWithAccount("relayer_1");
        // We should use hostAccounts.user.address here
        const relayerProxyClient = new ProxyClient(relayerClient, relayerClient.sender, proxyWalletAddress);
        const info = await relayerProxyClient.info();

        const sendAmount = coin(10_000, hostChain.feeToken!);
        const sendMsg: CosmosMsg = {
            bank: {
                send: {
                    to_address: hostAccounts.admin.address,
                    amount: [sendAmount as Coin],
                },
            },
        };

        const relayTransaction = await CWClient.createRelayTransaction(
            hostAccounts.user.mnemonic,
            info.nonce,
            JSON.stringify(sendMsg)
        );
        const walletBalanceBefore = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken!);

        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            "auto"
        );

        const walletBalanceAfter = await hostClient.getBalance(proxyWalletAddress, hostChain.feeToken!);
        const diff = Number(walletBalanceBefore.amount) - Number(walletBalanceAfter.amount);

        expect(sendAmount.amount).toEqual(String(diff));
        relayerClient.disconnect();
    });

    it("Should relay WASM message as a relayer", async () => {
        const relayerClient = await CWClient.connectHostWithAccount("relayer_1");
        const relayerProxyClient = new ProxyClient(relayerClient, relayerClient.sender, proxyWalletAddress);

        // Instantiate a new CW20 contract giving the wallet some funds
        const cw20Code = getContract(cw20CodePath!);
        const cw20Res = await hostAdminClient.upload(
            hostAccounts.admin.address,
            cw20Code,
            getDefaultUploadFee(hostChain)
        );

        const initAmount = "1000";
        const cw20contract = await hostAdminClient.instantiate(
            hostAccounts.admin.address,
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
            transfer: {
                recipient: hostAccounts.guardian_1.address,
                amount: transferAmount,
            },
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
            hostAccounts.user.mnemonic,
            1,
            JSON.stringify(cosmosWasmMsg)
        );
        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            "auto"
        );
        await delay(5000);

        const postFund = await relayerClient.queryContractSmart(cw20contract.contractAddress, {
            balance: { address: proxyWalletAddress },
        });

        expect(postFund.balance).toEqual("900");
        relayerClient.disconnect();
    });

    afterAll(() => {
        hostAdminClient?.disconnect();
        hostUserClient?.disconnect();
        hostGuardianClient?.disconnect();
        hostClient?.disconnect();
    });
});
