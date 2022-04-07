import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin } from "@cosmjs/stargate";
import { sha256 } from "@cosmjs/crypto";
import { toBase64, toUtf8, toHex } from "@cosmjs/encoding";
import { assert } from "@cosmjs/utils";

import {
  addrPrefix,
  adminAddr,
  adminMnemonic,
  fixMultiSigCodePath,
  cw20CodePath,
  factoryCodePath,
  proxyCodePath,
  GovecCodePath,
  StakingCodePath,
  rpcEndPoint,
  userMnemonic,
  userAddr,
  relayer1Addr,
  relayer2Addr,
  guardian1Addr,
  guardian2Addr,
  relayer1Mnemonic,
  coinMinDenom,
} from "./util/config";

import {
  defaultUploadFee,
  defaultInstantiateFee,
  getContract,
  defaultWalletCreationFee,
  defaultExecuteFee,
  defaultRelayFee,
  walletFee,
  mnemonicToKeyPair,
  CreateWalletMsg,
  CreateGovernanceMsg,
  WalletInstance,
  FactoryInstance,
  MultisigInstance,
  StakingInstance,
  GovecInstance,
  WasmExecuteMsg,
  BankMsg,
  createSigningClient,
  createRelayTransaction,
} from "./util/tests";

describe("End to End testing: ", () => {
  let factory: FactoryInstance | undefined;
  let wallet: WalletInstance | undefined;
  let multisig: MultisigInstance | undefined;
  let staking: StakingInstance | undefined;
  let govec: GovecInstance | undefined;

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

    const govecCode= getContract(GovecCodePath!);
    const govecRes = await client.upload(
      adminAddr!,
      govecCode,
      defaultUploadFee
    );

    const stakeCode= getContract(StakingCodePath!);
    const stakeRes = await client.upload(
      adminAddr!,
      stakeCode,
      defaultUploadFee
    );

    const initialFund = coin(10000000, coinMinDenom!);
    const { contractAddress } = await client.instantiate(
      adminAddr!,
      factoryRes.codeId,
      {
        proxy_code_id: proxyRes.codeId,
        proxy_multisig_code_id: multisigRes.codeId,
		staking_code_id: stakeRes.codeId,
		govec_code_id: govecRes.codeId,
        addr_prefix: addrPrefix!,
		wallet_fee: walletFee, 
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
        addr_prefix: addrPrefix!,
      },
      address: contractAddress,
      initialFund: [initialFund],
    };

	staking = {
		address: null,
		codeId: stakeRes.codeId
	};

	govec = {
		address: null,
		codeId: govecRes.codeId
	};

    expect(factoryRes.originalChecksum).toEqual(toHex(sha256(factoryCode)));
    expect(proxyRes.originalChecksum).toEqual(toHex(sha256(proxyCode)));
    expect(multisigRes.originalChecksum).toEqual(toHex(sha256(multisigCode)));
    expect(govecRes.originalChecksum).toEqual(toHex(sha256(govecCode)));
    expect(stakeRes.originalChecksum).toEqual(toHex(sha256(stakeCode)));
    expect(factoryRes.compressedSize).toBeLessThan(factoryCode.length * 0.5);
    expect(proxyRes.compressedSize).toBeLessThan(proxyCode.length * 0.5);
    expect(factoryRes.codeId).toBeGreaterThanOrEqual(1);
    expect(proxyRes.codeId).toBeGreaterThanOrEqual(1);

    client.disconnect();
  });

  it("should have funds in factory account", async () => {
    assert(factory);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const fund = await client.getBalance(factory.address, coinMinDenom!);
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

  it("should create govec and staking contract with create_governance", async()=> {
	  assert(factory);
	  assert(staking);
	  const adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
	  const createGovernanceMsg: CreateGovernanceMsg  = {
		staking_options: { duration: null, code_id: staking.codeId },
		initial_balances: []
	  };
	  const res = await adminClient.execute(
		  adminAddr!,
		  factory.address,
		  {create_governance: createGovernanceMsg},
		  defaultInstantiateFee
	  )
	  // TODO verify events
  })

  it("creating new wallet with multisig guardians", async () => {
    assert(factory);
    const adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
    const userKeypair = await mnemonicToKeyPair(userMnemonic!);
    const walletInitialFunds = coin(1000, coinMinDenom!);
    const walletMultisigInitialFunds = coin(1, coinMinDenom!);

    const walletInitMsg: CreateWalletMsg = {
      user_pubkey: toBase64(userKeypair.pubkey),
      guardians: {
        addresses: [guardian1Addr!, guardian2Addr!],
        guardians_multisig: {
          threshold_absolute_count: 1,
          multisig_initial_funds: [walletMultisigInitialFunds],
        },
      },
      relayers: [relayer1Addr!, relayer2Addr!],
      proxy_initial_funds: [walletInitialFunds],
    };

    const newWalletRes = await adminClient.execute(
      adminAddr!,
      factory.address,
      { create_wallet: { create_wallet_msg: walletInitMsg } },
      defaultWalletCreationFee,
	  undefined,
	  //TODO: error when passing in multiple coins in the following array
	  [coin(1100, coinMinDenom!)] 
    );

    // wasm event is the last one
    const wasmEvent = newWalletRes.logs[0].events.length;
    const wasmEventAttributes = newWalletRes.logs[0].events[wasmEvent - 1].attributes;
    for (let i in wasmEventAttributes) {
      if (wasmEventAttributes[i].key == "multisig_address") {
        multisig = { address: wasmEventAttributes[i].value };
      }
      if (wasmEventAttributes[i].key == "proxy_address") {
        wallet = {
          address: wasmEventAttributes[i].value,
          instantiateMsg: walletInitMsg,
        };
      }
    }

    adminClient.disconnect();
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

  it("proxy wallet has correct fund", async () => {
    assert(factory);
    assert(wallet);
    assert(multisig);
    const client = await CosmWasmClient.connect(rpcEndPoint!);
    const post_wallet_fund = await client.getBalance(
      wallet.address,
      coinMinDenom!
    );
    const post_multisig_fund = await client.getBalance(
      multisig.address,
      coinMinDenom!
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

  it("user can send funds via execute", async () => {
    assert(factory);
    assert(wallet);
    const userClient = await createSigningClient(userMnemonic!, addrPrefix!);
    const sendAmount = coin(2, coinMinDenom!);
    const sendMsg: BankMsg = {
      bank: {
        send: {
          to_address: adminAddr!,
          amount: [sendAmount],
        },
      },
    };

    const initfund = await userClient.getBalance(wallet.address, coinMinDenom!);
    await userClient.execute(
      userAddr!,
      wallet.address,
      { execute: { msgs: [sendMsg] } },
      defaultExecuteFee
    );
    const postfund = await userClient.getBalance(wallet.address, coinMinDenom!);
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
    const sendAmount = coin(2, coinMinDenom!);
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
    const initfund = await relayerClient.getBalance(
      wallet.address,
      coinMinDenom!
    );
    await relayerClient.execute(
      relayer1Addr!,
      wallet.address,
      {
        relay: relayTransaction,
      },
      defaultExecuteFee
    );
    const postfund = await relayerClient.getBalance(
      wallet.address,
      coinMinDenom!
    );
    const diff = Number(initfund.amount) - Number(postfund.amount);
    expect(sendAmount.amount).toEqual(String(diff));
  });

  it("relayer can relay wasm message", async () => {
    assert(factory);
    assert(wallet);
    const adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
    const relayerClient = await createSigningClient(
      relayer1Mnemonic!,
      addrPrefix!
    );

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

    const relayTransaction = await createRelayTransaction(
      userMnemonic!,
      1,
      JSON.stringify(wasmMsg)
    );
    await relayerClient.execute(
      relayer1Addr!,
      wallet.address,
      {
        relay: relayTransaction,
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
