import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin } from "@cosmjs/stargate";
import { sha256 } from "@cosmjs/crypto";
import { toBase64, toUtf8, toHex} from "@cosmjs/encoding";
import { assert } from "@cosmjs/utils";

import {
  addrPrefix,
  adminAddr,
  adminMnemonic,
  fixMultiSigCodePath,
  cw20CodePath,
  factoryCodePath,
  proxyCodePath,
  rpcEndPoint,
  userMnemonic,
  userAddr,
  relayer1Addr,
  relayer2Addr,
  guardian1Addr,
  guardian2Addr,
  relayer1Mnemonic,
} from "./util/config";
import {
  defaultUploadFee,
  defaultInstantiateFee,
  getContract,
  defaultWalletCreationFee,
  defaultExecuteFee,
  defaultRelayFee,
  mnemonicToKeyPair,
  CreateWalletMsg,
  WalletInstance,
  FactoryInstance,
  MultisigInstance,
  WasmExecuteMsg,
  BankMsg,
  createSigningClient,
  createRelayTransaction,
} from "./util/tests";

describe("End to End testing: ", () => {
  let factory: FactoryInstance | undefined;
  let wallet: WalletInstance | undefined;
  let multisig: MultisigInstance | undefined;

  it("should store contracts", async () => {
    const client = await createSigningClient(adminMnemonic!, addrPrefix!);
    const factoryCode = getContract(factoryCodePath!);
    const factoryRes = await client.upload(
      adminAddr!,
      factoryCode,
      defaultUploadFee
    );

    const proxyCode = getContract(proxyCodePath!);
    const proxyRes = await client.upload(
      adminAddr!,
      proxyCode,
      defaultUploadFee
    );

    const multisigCode = getContract(fixMultiSigCodePath!);
    const multisigRes = await client.upload(
      adminAddr!,
      multisigCode,
      defaultUploadFee
    );

    const initialFund = coin(10000000, "ucosm");
    const { contractAddress } = await client.instantiate(
      adminAddr!,
      factoryRes.codeId,
      {
        proxy_code_id: proxyRes.codeId,
        proxy_multisig_code_id: multisigRes.codeId,
		addr_prefix: "wasm"
      },
      "wallet factory",
      defaultInstantiateFee,
      {
        funds: [initialFund],
      }
    );
    factory = {
      instantiateMsg: {
        proxy_code_id: proxyRes.codeId,
        proxy_multisig_code_id: multisigRes.codeId,
		addr_prefix: "wasm"
      },
      address: contractAddress,
      initialFund: [initialFund],
    };

    expect(factoryRes.originalChecksum).toEqual(toHex(sha256(factoryCode)));
    expect(proxyRes.originalChecksum).toEqual(toHex(sha256(proxyCode)));
    expect(multisigRes.originalChecksum).toEqual(toHex(sha256(multisigCode)));
    expect(factoryRes.compressedSize).toBeLessThan(factoryCode.length * 0.5);
    expect(proxyRes.compressedSize).toBeLessThan(proxyCode.length * 0.5);
    expect(factoryRes.codeId).toBeGreaterThanOrEqual(1);
    expect(proxyRes.codeId).toBeGreaterThanOrEqual(1);

    client.disconnect();
  });

  it("should have funds in factory account", async () => {
    assert(factory);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const fund = await client.getBalance(factory.address, "ucosm");
    expect(fund.amount).toEqual(factory.initialFund[0].amount);
    expect(fund.denom).toEqual(factory.initialFund[0].denom);
    client.disconnect();
  });

  it("should store the proxy code id in the factory", async () => {
    assert(factory);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const codeId = await client.queryContractSmart(factory.address, {
      proxy_code_id: {},
    });
    expect(codeId).toEqual(factory.instantiateMsg.proxy_code_id);
    client.disconnect();
  });

  it("creating new wallet with multisig guardians", async () => {
    assert(factory);
    const adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
	const userKeypair = await mnemonicToKeyPair(userMnemonic!);
    const walletInitialFunds = [coin(1000, "ucosm")];
    const walletMultisigInitialFunds = [coin(100, "ucosm")];

    const walletInitMsg: CreateWalletMsg = {
      user_pubkey: toBase64(userKeypair.pubkey),
      guardians: {
        addresses: [guardian1Addr!, guardian2Addr!],
        guardians_multisig: {
          threshold_absolute_count: 1,
          multisig_initial_funds: walletMultisigInitialFunds,
        },
      },
      relayers: [relayer1Addr!, relayer2Addr!],
      proxy_initial_funds: walletInitialFunds,
    };

    const newWalletRes = await adminClient.execute(
      adminAddr!,
      factory.address,
      { create_wallet: { create_wallet_msg: walletInitMsg } },
      defaultWalletCreationFee
    );
    // wasm event is the last one
    const wasmEvent = newWalletRes.logs[0].events.length;
    // attributes[0] proxy contract address
    // attributes[1] action: Multisig Stored
    // attributes[2] action: Multisig Stored Address
    const newMultisigAddr =
      newWalletRes.logs[0].events[wasmEvent - 1].attributes[2].value;
    // attributes[3] facoty contract address
    // attributes[4] action: Proxy Stored
    // attributes[5] action: Proxy Stored Address
    const newProxyAddr =
      newWalletRes.logs[0].events[wasmEvent - 1].attributes[5].value;
    wallet = { address: newProxyAddr, instantiateMsg: walletInitMsg };
    multisig = { address: newMultisigAddr };
    adminClient.disconnect();
  });

  it("proxy wallet has correct fund", async () => {
    assert(factory);
    assert(wallet);
    assert(multisig);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const post_wallet_fund = await client.getBalance(wallet.address, "ucosm");
    const post_multisig_fund = await client.getBalance(
      multisig.address,
      "ucosm"
    );
    const wallet_fund = wallet.instantiateMsg.proxy_initial_funds[0].amount;
    const multisig_fund =
      wallet.instantiateMsg.guardians.guardians_multisig!
        .multisig_initial_funds[0].amount;
    expect(Number(post_wallet_fund.amount)).toEqual(
      Number(wallet_fund) - Number(multisig_fund)
    );
    expect(Number(post_multisig_fund.amount)).toEqual(Number(multisig_fund));
    client.disconnect();
  });

  it("factory stores proxy wallet address", async () => {
    assert(factory);
    assert(wallet);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const storedAddrs = await client.queryContractSmart(factory.address, {
      wallets: {},
    });
    expect(storedAddrs.wallets[0]).toEqual(wallet.address);
    client.disconnect();
  });

  it("user can send funds via execute", async () => {
    assert(factory);
    assert(wallet);
    const userClient = await createSigningClient(
	  userMnemonic!,
      addrPrefix!
    );
    const sendAmount = coin(2, "ucosm");
    const sendMsg: BankMsg = {
      bank: {
        send: {
          to_address: adminAddr!,
          amount: [sendAmount],
        },
      },
    };

    const initfund = await userClient.getBalance(wallet.address, "ucosm");
    await userClient.execute(
      userAddr!,
      wallet.address,
      { execute: { msgs: [sendMsg] } },
      defaultExecuteFee
    );
    const postfund = await userClient.getBalance(wallet.address, "ucosm");
    const diff = Number(initfund.amount) - Number(postfund.amount);
    expect(sendAmount.amount).toEqual(String(diff));
  });

  it("relayer can relay bank message", async () => {
    assert(factory);
    assert(wallet);
    const relayerClient = await createSigningClient(
      relayer1Mnemonic!,
      addrPrefix!
    );
    const sendAmount = coin(2, "ucosm");
    const sendMsg: BankMsg = {
      bank: {
        send: {
          to_address: adminAddr!,
          amount: [sendAmount],
        },
      },
    };

    const relayTransaction = await createRelayTransaction(
      userMnemonic!,
      0,
      JSON.stringify(sendMsg)
    );
    const initfund = await relayerClient.getBalance(wallet.address, "ucosm");
    await relayerClient.execute(
      relayer1Addr!,
      wallet.address,
      {
        relay: relayTransaction,
      },
      defaultExecuteFee
    );
    const postfund = await relayerClient.getBalance(wallet.address, "ucosm");
    const diff = Number(initfund.amount) - Number(postfund.amount);
    expect(sendAmount.amount).toEqual(String(diff));
  });

  it("relayer can relay wasm message", async () => {
    assert(factory);
    assert(wallet);
	const adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
	const relayerClient = await createSigningClient(relayer1Mnemonic!, addrPrefix!);

    // instantiate a new cw20 contract giving wallet some funds
    const cw20Code = getContract(cw20CodePath!);
    const cw20Res = await adminClient.upload(
      adminAddr!,
      cw20Code,
      defaultUploadFee
    );

    const initAmount = "1000";
    const cw20contract = await adminClient.instantiate(
      adminAddr!,
      cw20Res.codeId,
      {
        name: "scw-test",
        symbol: "scw",
        decimals: 10,
        initial_balances: [{ address: wallet.address, amount: initAmount }],
        mint: null,
        marketing: null,
      },
      "scw test cw20",
      defaultInstantiateFee,
      {
        funds: [],
      }
    );
    const transferAmount = "100";
    const transferMsg = {
      transfer: { recipient: guardian1Addr!, amount: transferAmount },
    };

    const wasmMsg: WasmExecuteMsg = {
      wasm: {
        execute: {
          contract_addr: cw20contract.contractAddress,
          msg: toBase64(toUtf8(JSON.stringify(transferMsg))),
          funds: [],
        },
      },
    };

	const relayTransaction = await createRelayTransaction(userMnemonic!, 1, JSON.stringify(wasmMsg));
    await relayerClient.execute(
      relayer1Addr!,
      wallet.address,
      {
        relay: relayTransaction
      },
      defaultRelayFee
    );
    const postfund = await relayerClient.queryContractSmart(
      cw20contract.contractAddress,
      { balance: { address: wallet.address! } }
    );

    expect(postfund.balance).toEqual("900");
  });
});
