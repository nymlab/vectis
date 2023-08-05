import { Secp256k1HdWallet } from "@cosmjs/amino";
import { Accounts, Account } from "../config/accounts";
import * as ConfigChains from "../config/chains";
import { Chains } from "../config/chains";
import { PrivateKey } from "@injectivelabs/sdk-ts";
import { testAccounts } from "../config/accounts/base";
import * as ConfigAccts from "../config/accounts/networks";
import { getAccountsPath } from "./fs";
import { writeToFile } from "./fs";

async function _getAcc(
    prefix: string,
    coinType: number,
    account: Accounts,
    accounts: Record<Accounts, string>
): Promise<Account> {
    if (coinType == 118) {
        const wallet = await Secp256k1HdWallet.fromMnemonic(accounts[account], {
            prefix,
        });
        const [{ address }] = await wallet.getAccounts();
        return { address, mnemonic: accounts[account] };
    } else if (coinType == 60) {
        const pk = PrivateKey.fromMnemonic(accounts[account]);
        const address = pk.toBech32();
        return { address, mnemonic: accounts[account] };
    } else {
        throw new Error("CoinType not supported");
    }
}

export async function getAcct(chain: Chains, account: Accounts): Promise<Account> {
    let prefix = ConfigChains[chain].addressPrefix;
    let coinType = ConfigChains[chain].coinType;

    // overwrites the testAccounts with env values if exists for the chain
    if (Object.entries(ConfigAccts[chain]).length > 0) {
        for (let [accName, account] of Object.entries(ConfigAccts[chain])) {
            testAccounts[accName as Accounts] = account.mnemonic!;
        }
    }
    return _getAcc(prefix, coinType, account, testAccounts);
}

export const generateAddrFromMnemonics = async (chain: Chains): Promise<void> => {
    let prefix = ConfigChains[chain].addressPrefix;
    let coinType = ConfigChains[chain].coinType;
    let accounts = {} as Record<Accounts, Account>;
    const _testAccounts = { ...testAccounts };

    // overwrites the _testAccounts with env values if exists for the chain
    if (Object.entries(ConfigAccts[chain]).length > 0) {
        for (let [accName, account] of Object.entries(ConfigAccts[chain])) {
            _testAccounts[accName as Accounts] = account.mnemonic!;
        }
    }

    for (const account of Object.keys(_testAccounts)) {
        const acc = await _getAcc(prefix, coinType, account as Accounts, _testAccounts);
        accounts[account as Accounts] = acc;
    }

    writeToFile(getAccountsPath(chain), JSON.stringify(accounts, null, 2));
};
