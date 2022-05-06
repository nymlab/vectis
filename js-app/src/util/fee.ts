import { calculateFee, coin, GasPrice } from "@cosmjs/stargate";
import { coinMinDenom, gasPrice } from "./env";

export const defaultGasPrice = GasPrice.fromString(gasPrice!);
export const defaultUploadFee = calculateFee(55_500_000, defaultGasPrice);
export const defaultInstantiateFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultSendFee = calculateFee(800_000, defaultGasPrice);
export const defaultExecuteFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultRelayFee = calculateFee(1_400_000, defaultGasPrice);
export const defaultWalletCreationFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultMigrateFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultUpdateAdminFee = calculateFee(800_000, defaultGasPrice);
export const defaultClearAdminFee = calculateFee(800_000, defaultGasPrice);
export const walletFee = coin(100, coinMinDenom!);
