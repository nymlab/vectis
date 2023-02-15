import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import assert from "assert";
import { Addr, CosmosMsgForEmpty as CosmosMsg, BankMsg, Coin } from "../interfaces/Proxy.types";
import { GovecClient, FactoryClient, RelayerClient } from "../clients";
import { coin } from "@cosmjs/stargate";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/DaoProposalSingle.v2";
import { Vote } from "@dao-dao/types/contracts/DaoProposalSingle.common";
import { deployReportPath, hostAccounts, hostChain, remoteChain } from "../utils/constants";
import { createTestProxyWallets } from "./mocks/proxyWallet";
import { CWClient } from "../clients";
import { getDefaultRelayFee, getDefaultSendFee, getDefaultUploadFee } from "../utils/fees";
import { ProxyClient } from "../interfaces";
import { ProxyClient as ProxyHostClient } from "../clients";
import { randomAddress } from "@confio/relayer/build/lib/helpers";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { toCosmosMsg } from "../utils/enconding";
import { QueryMsg as prePropQueryMsg } from "../interfaces/DaoPreProposeApprovalSingel.types";

/**
 * This suite tests Proxy contract methods
 */
describe("Proxy Suite: ", () => {
    let hostUserClient: CWClient;
    let remoteUserClient: CWClient;
    let hostGuardianClient: CWClient;
    let hostAdminClient: CWClient;
    let hostClient: CosmWasmClient;
    let proxyWalletAddress: Addr;
    let proxyWalletMultisigAddress: Addr;
    let addrs: VectisDaoContractsAddrs;

    let govecClient: GovecClient;
    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let proxyHostClient: ProxyHostClient;
    let msProxyClient: ProxyClient;
    let guardianProxyClient: ProxyClient;
    const relayerClient = new RelayerClient();

    beforeAll(async () => {
        addrs = await import(deployReportPath);
        await relayerClient.connect();
        await relayerClient.loadChannels();

        hostUserClient = await CWClient.connectHostWithAccount("user");
        remoteUserClient = await CWClient.connectRemoteWithAccount("user");
        hostAdminClient = await CWClient.connectHostWithAccount("admin");
        hostGuardianClient = await CWClient.connectHostWithAccount("guardian_1");
        hostClient = await CosmWasmClient.connect(hostChain.rpcUrl);
        govecClient = new GovecClient(hostUserClient, hostUserClient.sender, addrs.govecAddr);

        factoryClient = new FactoryClient(hostAdminClient, hostAdminClient.sender, addrs.factoryAddr);

        const [walletAddr, walletMSAddr] = await createTestProxyWallets(factoryClient);
        proxyWalletAddress = walletAddr;
        proxyWalletMultisigAddress = walletMSAddr;

        proxyClient = new ProxyClient(hostUserClient, hostUserClient.sender, proxyWalletAddress);
        proxyHostClient = new ProxyHostClient(hostUserClient, hostUserClient.sender, proxyWalletAddress);
        msProxyClient = new ProxyClient(hostUserClient, hostUserClient.sender, proxyWalletMultisigAddress);
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

    it("should be able to add or remove a relayer", async () => {
        const relayer3 = randomAddress("juno");
        await proxyClient.addRelayer({ newRelayerAddress: relayer3 });

        const { relayers: relayersBefore } = await proxyClient.info();

        expect(relayersBefore).toContain(relayer3);

        await proxyClient.removeRelayer({ relayerAddress: relayer3 });

        const { relayers: relayersAfter } = await proxyClient.info();

        expect(relayersAfter).not.toContain(relayer3);
    });

    it("should be able to do ibc transfer", async () => {
        const amountToSend = 1e7;
        const targetAddress = randomAddress(remoteChain.addressPrefix);
        const funds = [coin(amountToSend, hostChain.feeToken) as Coin];

        const { amount: balanceBefore } = await remoteUserClient.getBalance(targetAddress, relayerClient.denoms.src);

        await proxyClient.execute(
            {
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: addrs.daoTunnelAddr,
                                msg: toCosmosMsg({
                                    ibc_transfer: {
                                        receiver: {
                                            connection_id: relayerClient.connections.hostConnection,
                                            addr: targetAddress,
                                        },
                                    },
                                }),
                                funds,
                            },
                        },
                    },
                ],
            },
            "auto",
            undefined,
            funds
        );

        await relayerClient.relayAll();

        const { amount: balanceAfter } = await remoteUserClient.getBalance(targetAddress, relayerClient.denoms.src);
        expect(+balanceAfter).toEqual(+balanceBefore + +amountToSend);
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

    it("should let user change their label", async () => {
        const newLabel = "test-label";
        await proxyClient.updateLabel({ newLabel });

        const info = await proxyClient.info();
        expect(info.label).toEqual(newLabel);
    });

    it("should accept request for updating guardians", async () => {
        const newGuardians = [hostAccounts.guardian_1.address];
        let res = await proxyClient.guardiansUpdateRequest();
        expect(res).toEqual(null);

        await proxyClient.requestUpdateGuardians({
            request: { guardians: { addresses: newGuardians } },
        });

        res = await proxyClient.guardiansUpdateRequest();
        expect(res?.guardians.addresses).toEqual(newGuardians);
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

    it("Should ootate user key as guardian", async () => {
        assert(guardianProxyClient, "guardianProxyClient is not defined");

        // New owner is admin
        await guardianProxyClient.rotateControllerKey({
            newControllerAddress: hostAccounts.admin.address,
        });

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
            expect((err as Error).message).toContain("IsNotController");
        }

        // Return wallet to the user
        await guardianProxyClient.rotateControllerKey({
            newControllerAddress: hostAccounts.user.address,
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
            const info = await msProxyClient.info();
            const multisig_address = info.multisig_address;
            expect(multisig_address).toBeDefined();

            const rotateUserKey: CosmosMsg = {
                wasm: {
                    execute: {
                        contract_addr: proxyWalletMultisigAddress,
                        msg: toBase64(
                            toUtf8(
                                JSON.stringify({
                                    rotate_controller_key: {
                                        new_controller_address: hostAccounts.admin.address,
                                    },
                                })
                            )
                        ),
                        funds: [],
                    },
                },
            };
            const proposal = {
                propose: {
                    title: "Rotate user key",
                    description: "Need to rotate user key",
                    msgs: [rotateUserKey],
                    latest: null,
                },
            };
            const res = await clientG1.execute(
                clientG1.sender,
                multisig_address!,
                proposal,
                getDefaultRelayFee(hostChain)
            );

            // Should have proposal in the list
            const queryProps: ProposalQueryMsg = { list_proposals: {} };
            const { proposals } = await clientG1.queryContractSmart(multisig_address!, queryProps);
            const prop = proposals.find((p: any) => p.title === proposal.propose.title);
            expect(prop).toBeTruthy();
            const propId = prop.id;

            // At this point, since Guardian1 proposed, his vote is already YES
            // Now Guardian2 votes YES
            const voteYes = {
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
            const { controller_addr } = await msProxyClient.info();
            expect(controller_addr).toEqual(hostAccounts.admin.address);
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
            const voteYes = {
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

    it("Should relay WASM message as a relayer - mint govec", async () => {
        const relayerClient = await CWClient.connectHostWithAccount("relayer_1");
        const relayerProxyClient = new ProxyClient(relayerClient, relayerClient.sender, proxyWalletAddress);
        const info = await relayerProxyClient.info();

        const { claim_fee } = await factoryClient.fees();

        const cosmosWasmMsg: CosmosMsg = {
            wasm: {
                execute: {
                    contract_addr: factoryClient.contractAddress,
                    msg: toBase64(toUtf8(JSON.stringify({ claim_govec: {} }))),
                    funds: [claim_fee],
                },
            },
        };

        const relayTransaction = await CWClient.createRelayTransaction(
            hostAccounts.user.mnemonic,
            info.nonce,
            JSON.stringify(cosmosWasmMsg)
        );

        await relayerProxyClient.relay(
            {
                transaction: relayTransaction,
            },
            "auto"
        );

        const mintAmount = await govecClient.mintAmount();
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe(mintAmount);
        relayerClient.disconnect();
    });

    it("Should be able to do pre-proposal", async () => {
        const msg: CosmosMsg = {
            bank: {
                send: {
                    from_address: addrs.daoAddr,
                    to_address: addrs.daoTunnelAddr,
                    amount: [coin("100000", hostChain.feeToken) as Coin],
                },
            },
        };

        // order is desending
        let queryMsg: prePropQueryMsg = { query_extension: { msg: { pending_proposals: {} } } };
        let previousPreProposals = await hostUserClient.queryContractSmart(addrs.preproposalAddr, queryMsg);

        let res = await proxyHostClient.createPreProposal(
            addrs.preproposalAddr,
            "title_proposal",
            "description_proposal",
            [msg]
        );
        let currentPreProposals = await hostUserClient.queryContractSmart(addrs.preproposalAddr, queryMsg);
        const { approval_id, proposer, msg: prepmsg, deposit } = currentPreProposals[0];

        expect(previousPreProposals.length + 1).toBe(currentPreProposals.length);
        expect(proposer).toBe(proxyHostClient.contractAddress);
    });

    afterAll(() => {
        hostAdminClient?.disconnect();
        hostUserClient?.disconnect();
        hostGuardianClient?.disconnect();
        hostClient?.disconnect();
    });
});
