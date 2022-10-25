import { juno_localnet as junoAccounts, wasm_localnet as wasmAccounts } from "@vectis/core/config/accounts";
import { juno_localnet as hostChain, wasm_localnet as remoteChain } from "@vectis/core/config/chains";
import { coin, calculateFee, GasPrice } from "@cosmjs/stargate";
import { Coin } from "@vectis/types/contracts/Factory.types";
import { Chain } from "@vectis/core/interfaces/chain";

export const HOST_CHAIN = hostChain;
export const REMOTE_CHAIN = remoteChain;
export const HOST_ACCOUNTS = junoAccounts;
export const REMOTE_ACCOUNTS = wasmAccounts;
export const INITIAL_FACTORY_BALANCE = coin(1_000_000, HOST_CHAIN.feeToken) as Coin;

export const defaultGasPrice = GasPrice.fromString(HOST_CHAIN.gasPrice + HOST_CHAIN.feeToken);
export const defaultUploadFee = calculateFee(55_500_000, defaultGasPrice);
export const defaultInstantiateFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultSendFee = calculateFee(800_000, defaultGasPrice);
export const defaultExecuteFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultRelayFee = calculateFee(1_400_000, defaultGasPrice);
export const defaultWalletCreationFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultMigrateFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultUpdateAdminFee = calculateFee(800_000, defaultGasPrice);
export const defaultClearAdminFee = calculateFee(800_000, defaultGasPrice);

export const walletInitialFunds = (chain: Chain) => coin(1000000, chain.feeToken) as Coin;
