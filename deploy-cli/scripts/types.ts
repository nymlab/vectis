import tsGenerator from "@cosmwasm/ts-codegen";
import { join } from "path";
import { areTypesSchemasDownloaded, downloadTypeSchema } from "../utils/fs";
import { cw4GroupSchemaLink, cw3flexSchemaLink } from "../utils/constants";

const contractsPath = (path: string) => join(__dirname, "../../contracts/", path);
const downloadSchemaPath = (path: string) => join(__dirname, "../.cache/schemas/", path);
const outPath = join(__dirname, "../interfaces");

const typesFiles = {
    cw3Flex: "cw3-flex-multisig",
    cw4Group: "cw4-group",
};

(async function downloadSchemas() {
    if (!areTypesSchemasDownloaded()) {
        console.log("downloading schemas");
        await downloadTypeSchema(cw3flexSchemaLink, typesFiles.cw3Flex, "schema.json");
        await downloadTypeSchema(cw4GroupSchemaLink, typesFiles.cw4Group, "schema.json");
    }

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
})();
