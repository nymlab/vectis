import { coin, calculateFee, GasPrice } from "@cosmjs/stargate";
import { Coin } from "../interfaces/Factory.types";
import type { Chain } from "../config/chains";

/// These are just for testing
export const walletInitialFunds = (chain: Chain) => coin(1500000, chain.feeToken) as Coin;
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

/// These are the params deployed
export const walletCreationFee = (chain: Chain) => coin(0, chain.feeToken) as Coin;
export const pluginRegInstallFee = (chain: Chain) => coin(0, chain.feeToken) as Coin;
export const pluginRegRegistryFee = (chain: Chain) => coin(0, chain.feeToken) as Coin;
