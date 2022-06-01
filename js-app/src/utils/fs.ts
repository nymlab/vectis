import fs from "fs";
import path from "path";

export function getContract(path: string): Uint8Array {
    return fs.readFileSync(path);
}

export function getVectisContractPaths(basePath: string): any {
    const filePath = fs.readdirSync(basePath);
    const proxyCodePath = path.join(basePath, filePath.find((file) => file.includes("proxy"))!);
    const govecCodePath = path.join(basePath, filePath.find((file) => file.includes("govec"))!);
    const factoryCodePath = path.join(basePath, filePath.find((file) => file.includes("factory"))!);
    return { proxyCodePath, govecCodePath, factoryCodePath };
}

export function getDownloadContractsPath(basePath: string): any {
    const fixMultiSigCodePath = path.join(basePath, "cw3_fixed_multisig.wasm");
    const cw20CodePath = path.join(basePath, "cw20_base.wasm");
    const daoCodePath = path.join(basePath, "cw_core.wasm");
    const stakingCodePath = path.join(basePath, "stake_cw20.wasm");
    const voteCodePath = path.join(basePath, "cw20_staked_balance_voting.wasm");
    const proposalSingleCodePath = path.join(basePath, "cw_proposal_single.wasm");
    return { fixMultiSigCodePath, cw20CodePath, daoCodePath, stakingCodePath, voteCodePath, proposalSingleCodePath };
}

export function writeInFile<T>(fileName: string, content: T, encoding: string = "utf8"): void {
    fs.writeFileSync(path.join(__dirname, "..", "..", fileName), content, { encoding });
}
