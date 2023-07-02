import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC } from "../interfaces";

class ProxyClient extends ProxyC {
    constructor(cw: CWClient, sender: string, contractAddr: string) {
        super(cw.client, sender, contractAddr);
    }
}
