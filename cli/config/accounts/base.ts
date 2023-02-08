import { Secp256k1HdWallet } from "@cosmjs/amino";

const base = {
    admin: "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose",
    user: "useful guitar throw awesome later damage film tonight escape burger powder manage exact start title december impulse random similar eager smart absurd unaware enlist",
    guardian_1:
        "slim rely one tiny chapter job toilet vague moment inquiry abandon toe robust trust orchard oyster elephant jazz quantum shaft stairs polar drop gospel",
    guardian_2:
        "prepare tired ten whisper receive spider heavy differ mom web champion clever brass sight furnace cash march rice use nature ginger portion area million",
    relayer_1:
        "regret blur gas upon blossom illness exercise lamp combine monster draw inquiry borrow scrub achieve credit country donate stool develop kid amount flush wall",
    relayer_2:
        "material often similar patrol please flat van toast agree milk grass pause glow rhythm voyage reason potato sunset great govern pave decade critic lens",
};

const generate = async (): Promise<void> => {
    const prefix = "osmo";

    let accounts = {} as Record<keyof typeof base, { address: string; mnemonic: string }>;
    for (const [account, mnemonic] of Object.entries(base)) {
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix,
        });
        const [{ address }] = await wallet.getAccounts();
        accounts[account as keyof typeof base] = { address, mnemonic };
    }
    console.log(accounts);
};

generate();
