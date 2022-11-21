import { CWClient } from "../../clients";

export const generateRandomAddress = async (prefix: string) => {
    const account = await CWClient.generateRandomAccount(prefix);
    const [{ address }] = await account.getAccounts();
    return address;
};
