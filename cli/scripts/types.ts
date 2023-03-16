import tsGenerator from "@cosmwasm/ts-codegen";
import { join } from "path";
import { areTypesSchemasDownloaded, downloadTypeSchema } from "../utils/fs";
import {
    cw4GroupSchemaLink,
    cw20StakeSchemaLink,
    proposalSingleSchemaLink,
    prePropSingleApprovalSchemaLink,
    cw3flexSchemaLink,
} from "../utils/constants";

const contractsPath = (path: string) => join(__dirname, "../../contracts/", path);
const downloadSchemaPath = (path: string) => join(__dirname, "../.cache/schemas/", path);
const outPath = join(__dirname, "../interfaces");

const typesFiles = {
    cw20Stake: "cw20-stake",
    daoProp: "dao-proposal-single",
    daoPreProp: "dao-pre-prepose-approval-single",
    cw3Flex: "cw3-flex-multisig",
    cw4Group: "cw4-group",
};

(async function downloadSchemas() {
    if (!areTypesSchemasDownloaded()) {
        console.log("downloading");
        await downloadTypeSchema(cw20StakeSchemaLink, typesFiles.cw20Stake, "schema.json");
        await downloadTypeSchema(proposalSingleSchemaLink, typesFiles.daoProp, "schema.json");
        await downloadTypeSchema(prePropSingleApprovalSchemaLink, typesFiles.daoPreProp, "schema.json");
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
            {
                name: "remoteFactory",
                dir: contractsPath("remote_factory/schema"),
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
            {
                name: "cw20Stake",
                dir: downloadSchemaPath(typesFiles.cw20Stake),
            },
            {
                name: "daoProposalSingle",
                dir: downloadSchemaPath(typesFiles.daoProp),
            },
            {
                name: "daoPreProposeApprovalSingel",
                dir: downloadSchemaPath(typesFiles.daoPreProp),
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
