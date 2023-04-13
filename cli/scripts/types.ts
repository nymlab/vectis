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
                dir: contractsPath("proxy/schema"),
            },
            {
                name: "factory",
                dir: contractsPath("factory/schema"),
            },
            {
                name: "pluginRegistry",
                dir: contractsPath("plugin_registry/schema"),
            },
            {
                name: "cw3Flex",
                dir: downloadSchemaPath(typesFiles.cw3Flex),
            },
            {
                name: "cw4Group",
                dir: downloadSchemaPath(typesFiles.cw4Group),
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
