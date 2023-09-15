export enum FactoryActions {
    "updateSupportedChains",
    "updateSupportedProxy",
}

export type FactoryAction = keyof typeof FactoryActions;
