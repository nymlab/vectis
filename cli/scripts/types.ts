import tsGenerator from "@cosmwasm/ts-codegen";
import { join } from "path";

const contractsPath = (path: string) => join(__dirname, "../../contracts/", path);
const outPath = join(__dirname, "../interfaces");

tsGenerator({
    contracts: [
        {
            name: "proxy",
            dir: contractsPath("proxy/schema"),
        },
        {
            name: "factory",
            dir: contractsPath("factory/schema"),
        },
        {
            name: "govec",
            dir: contractsPath("govec/schema"),
        },
        {
            name: "daoTunnel",
            dir: contractsPath("dao_tunnel/schema"),
        },
        {
            name: "remoteTunnel",
            dir: contractsPath("remote_tunnel/schema"),
        },
    ],
    outPath,
    options: {
        bundle: {
            enabled: false,
        },
    },
}).then(() => console.log("Generated typescript interfaces for contracts"));
