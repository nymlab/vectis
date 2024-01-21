import { coin, calculateFee, GasPrice } from "@cosmjs/stargate";
import { FactoryTypes as FactoryT, PluginregistryTypes as PluginRegT } from "../clients/contracts";
import type { Chain } from "./chains";

export const getDefaultGasPrice = (chain: Chain) => GasPrice.fromString(chain.gasPrice + chain.feeToken);

// General wasm action config
export const getDefaultUploadFee = (chain: Chain) => calculateFee(55_500_000, getDefaultGasPrice(chain));
export const getDefaultInstantiateFee = (chain: Chain) => calculateFee(1_500_000, getDefaultGasPrice(chain));
export const getDefaultSendFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));
export const getDefaultExecuteFee = (chain: Chain) => calculateFee(1_200_000, getDefaultGasPrice(chain));
export const getDefaultMigrateFee = (chain: Chain) => calculateFee(1_200_000, getDefaultGasPrice(chain));
export const getDefaultUpdateAdminFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));
export const getDefaultClearAdminFee = (chain: Chain) => calculateFee(800_000, getDefaultGasPrice(chain));

/// Vectis configs
export const getDefaultVectisWalletCreationFee = (chain: Chain) => calculateFee(1_500_000, getDefaultGasPrice(chain));

// These are the params deployed in the Vectis Contracts
// Factory
export const walletCreationFee = (chain: Chain) => coin(0, chain.feeToken) as FactoryT.Coin;
// Pluing Registry
export const pluginFreeTierFee = (chain: Chain) => coin(0, chain.feeToken) as PluginRegT.Coin;
export const pluginRegSubscriptionFee = (chain: Chain) => coin(0, chain.feeToken) as PluginRegT.Coin;
export const pluginRegRegistryFee = (chain: Chain) => coin(0, chain.feeToken) as PluginRegT.Coin;
