import { juno_localnet as junoAccounts, wasm_localnet as wasmAccounts } from "@vectis/core/config/accounts";
import * as CHAINS from "@vectis/core/config/chains";
import { coin, calculateFee, GasPrice } from "@cosmjs/stargate";
import { Coin } from "@vectis/types/contracts/Factory.types";
import { Chain } from "@vectis/core/interfaces/chain";

export const HOST_CHAIN = CHAINS.juno_localnet;
export const REMOTE_CHAIN = CHAINS.wasm_localnet;
export const HOST_ACCOUNTS = junoAccounts;
export const REMOTE_ACCOUNTS = wasmAccounts;
export const getDefaultGasPrice = (chain: Chain) => GasPrice.fromString(chain.gasPrice + chain.feeToken);

export const getInitialFactoryBalance = (chain: Chain) => coin(1_000_000, chain.feeToken) as Coin;

export const getDefaultUploadFee = (chain: Chain) => calculateFee(55_500_000, getDefaultGasPrice(chain));
export const getDefaultInstantiateFee = (chain: Chain) => calculateFee(1_500_000, getDefaultGasPrice(chain));
export const getDefaultSendFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));
export const getDefaultExecuteFee = (chain: Chain) => calculateFee(1_200_000, getDefaultGasPrice(chain));
export const getDefaultRelayFee = (chain: Chain) => calculateFee(1_400_000, getDefaultGasPrice(chain));
export const getDefaultWalletCreationFee = (chain: Chain) => calculateFee(1_500_000, getDefaultGasPrice(chain));
export const getDefaultMigrateFee = (chain: Chain) => calculateFee(1_200_000, getDefaultGasPrice(chain));
export const getDefaultUpdateAdminFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));
export const getDefaultClearAdminFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));

export const walletInitialFunds = (chain: Chain) => coin(1000000, chain.feeToken) as Coin;
