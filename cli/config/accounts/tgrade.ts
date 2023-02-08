const common = {
    admin: {
        mnemonic:
            "end wheat debris decrease nation bike giraffe used trade agent right above drift arctic case raccoon improve save intact item shove elite possible magnet ",
        address: "tgrade1j7ehm9s8vyad7t8c96nakwzu5jsjf3rqzpjjdk",
    },
    user: {
        mnemonic:
            "critic include various armed flock huge mention pause thought flat all museum target outside resist system online patrol they minute benefit doll plunge among",
        address: "tgrade1skj0sn5zw78jr7n6zq7wucf7845v2aq6aylmtu",
    },
    guardian_1: {
        mnemonic:
            "blouse wolf embody start identify crucial shuffle maple assume pilot range core lemon scrub black gadget hand hurt embrace cannon when female assault whisper",
        address: "tgrade16fslwyn2mjlsrgz256pxtagkjl5r2qvpnk9zyw",
    },
    guardian_2: {
        mnemonic:
            "bridge fever chronic glue differ license else nation chase initial more viable proud beyond truly someone promote job release bronze swamp such spray antique ",
        address: "tgrade1lh48mdd4msqhdmwcmnxecu589xlmd9hmmz62xf",
    },
    relayer_1: {
        mnemonic:
            "tray list genuine praise relief beauty logic slight food custom possible dilemma tortoise lunar flee wine radio hurdle switch gold cheese mimic viable enforce ",
        address: "tgrade1kexx9x0k3kv25pmz673jdt2f4j2h47dhh5a9tn",
    },
    relayer_2: {
        mnemonic:
            "glory humble barrel attack mind relief opinion excess surface media setup approve dynamic weird industry matrix hero later detail crush scene carpet hope ginger ",
        address: "tgrade1g02slnahvg29csd3ealu86yhqpcps6d6wfm79l",
    },
    committee1: {
        mnemonic:
            "cave topple history exercise carpet crash answer correct one benefit fury tiger medal emerge canoe acquire pig chuckle mystery confirm alley security exit mixture",
        address: "tgrade1dfd5vtxy2ty5gqqv0cs2z23pfucnpym9t2tz9q",
    },
    committee2: {
        mnemonic:
            "divorce park goat subject cake arrive liar reward favorite shed market spot harsh garden wet general enlist limb chair message current grant curtain that",
        address: "tgrade1ndxfpxzxg267ujpc6wwhw9fs2rvgfh06lgf4rc",
    },
};

export const tgrade_mainnet = {
    ...common,
    admin: {
        mnemonic: process.env.TGRADE_ADMIN_MNEMONIC as string,
        address: process.env.TGRADE_ADMIN_ADDRESS as string,
    },
    committee1: {
        mnemonic: process.env.WASM_COMMITTEE1_MNEMONIC,
        address: process.env.WASM_COMMITTEE1_ADDRESS,
    },
    committee2: {
        mnemonic: process.env.WASM_COMMITTEE2_MNEMONIC,
        address: process.env.WASM_COMMITTEE2_ADDRESS,
    },
};
export const tgrade_testnet = { ...common };
export const tgrade_localnet = { ...common };
