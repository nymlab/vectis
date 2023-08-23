import tsGenerator from "@cosmwasm/ts-codegen";
import { join } from "path";

const contractsPath = (path: string) => join(__dirname, "../../contracts/", path);

const outPath = join(__dirname, "../interfaces");

export async function generateTypes() {
    tsGenerator({
        contracts: [
            {
                name: "proxy",
                dir: contractsPath("core/proxy/schema"),
            },
            {
                name: "factory",
                dir: contractsPath("core/factory/schema"),
            },
            {
                name: "pluginRegistry",
                dir: contractsPath("core/plugin_registry/schema"),
            },
        ],
        outPath,
        options: {
            bundle: {
                enabled: false,
            },
            client: {
                noImplicitOverride: true,
            },
        },
    }).then(() => console.log("Generated typescript interfaces for contracts"));
}
