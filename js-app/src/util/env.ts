import { coin } from "@cosmjs/stargate";
import * as dotenv from "dotenv";

dotenv.config();
console.log("Test for: ", process.env.NODE_ENV);
const path = `${__dirname}/../../../../.env.${process.env.NODE_ENV?.replace("-", ".") ?? "dev"}`;
dotenv.config({ path });

export const gasPrice = process.env.GAS_PRICE;
export const coinDenom = process.env.COIN_DENOM;
export const coinMinDenom = process.env.COIN_MIN_DENOM;
export const rpcEndPoint = process.env.RPC;
export const chainId = process.env.CHAIN_ID;
export const adminMnemonic = process.env.ADMIN_MNEMONIC;
export const adminAddr = process.env.ADMIN_ADDR;
export const userMnemonic = process.env.USER_MNEMONIC;
export const userAddr = process.env.USER_ADDR;
export const guardian1Mnemonic = process.env.GUARDIAN1_MNEMONIC;
export const guardian1Addr = process.env.GUARDIAN1_ADDR;
export const guardian2Mnemonic = process.env.GUARDIAN2_MNEMONIC;
export const guardian2Addr = process.env.GUARDIAN2_ADDR;
export const relayer1Mnemonic = process.env.RELAYER1_MNEMONIC;
export const relayer1Addr = process.env.RELAYER1_ADDR;
export const relayer2Mnemonic = process.env.RELAYER2_MNEMONIC;
export const relayer2Addr = process.env.RELAYER2_ADDR;
export const addrPrefix = process.env.ACCT_PREFIX;
export const fixMultiSigCodePath = process.env.FIXEDMULTISIG_CODE_PATH;
export const cw20CodePath = process.env.CW20_CODE_PATH;
export const factoryCodePath = process.env.FACTORY_CODE_PATH;
export const proxyCodePath = process.env.PROXY_CODE_PATH;
export const daoCodePath = process.env.DAO_CODE_PATH;
export const stakingCodePath = process.env.STAKING_CODE_PATH;
export const govecCodePath = process.env.GOVEC_CODE_PATH;
export const voteCodePath = process.env.VOTE_CODE_PATH;
export const proposalSingleCodePath = process.env.PROP_CODE_PATH;

export const testWalletInitialFunds = coin(5_000_000, coinMinDenom!);
