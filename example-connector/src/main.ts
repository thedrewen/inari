import { WebSocket } from "ws";
import { configDotenv } from "dotenv";
import { AuthRequest, AuthResponse, ClientConnectorDefinition, ConnectionDefinition, ErrorResponse, MessageData, MessageKind, SyncConnectorStateRequest, SyncConnectorStateResponse } from "./type";
import { v4 as uuid } from "uuid";
import { readFileSync } from "fs";
import { readFile } from "fs/promises";

configDotenv();

class Connector {
    private ws_client: WebSocket;
    private pendingRequest = new Map<string, {res: any, rej: any}>();
    private clients: ConnectionDefinition[] = [];

    constructor () {
        this.ws_client = new WebSocket(process.env.INARI_URL as string);
        this.ws_client.on('open', async () => {
            console.log('WS > Connection Open !');
            const result : MessageData = await this.call('auth', { auth_key: process.env.INARI_API_KEY as string, channel: process.env.INARI_CHANNEL, is_connector: true } as AuthRequest);
            if (result.kind == 'error') {
                console.log(`WS > Authentification Error ! '${(result.data as ErrorResponse).details}'`);
                this.ws_client.close();
                return;
            }
            const data = result.data as AuthResponse;
            console.log(`Welcome on Inari ${data.app_version}. Max Distance set to ${data.max_distance} !`);
            await this.handle();
        });
        this.ws_client.on('close', () => {
            console.log('WS > Connection Close !');
            process.exit(1);
        });
        this.ws_client.on('message', (rawdata) => {
            try {
                const data: MessageData = JSON.parse(rawdata.toString());
                const v = this.pendingRequest.get(data.transaction_uuid);
                if (v) {
                    v.res(data);
                } else {
                    console.log(`WS > Unknow Message : ${JSON.stringify(data, null, 2)}`);
                }
            } catch (error) {}
        });
    }

    private async call(kind: MessageKind, data: any, withResponse: boolean = true): Promise<undefined | any> {
        return new Promise((res, rej) => {
            const transaction_uuid = uuid(); 
            if (withResponse) {
                this.pendingRequest.set(transaction_uuid, { res, rej });  
            }
            this.ws_client.send(JSON.stringify({ kind, data, transaction_uuid } as MessageData));
            if (!withResponse) {
                res(undefined);
            }
        });
    }

    private async handle() {
        setInterval(async () => {
            const json_data = JSON.parse((await readFile('./src/data.json', 'utf-8'))) as ClientConnectorDefinition[];
            this.clients = (await this.call('sync_connector_state', { clients: json_data } as SyncConnectorStateRequest) as { data: SyncConnectorStateResponse }).data.connections;
            console.log(JSON.stringify(this.clients, null, 2));
        }, 300); // Always 300ms;
    }
}

new Connector();