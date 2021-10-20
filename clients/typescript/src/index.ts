import { strict as assert } from 'assert';
import * as protocol from './lib/protocol';
import WebSocket from 'ws';
import * as url from 'url';
import { v4 as uuidv4 } from 'uuid';

const trace = (..._args: any[]) => ({}); // console.log(..._args);

const DEFAULT_TIMEOUT: number = 5000;

export interface VoodooClientOptions extends WebSocket.ClientOptions {
    defaultTimeout?: number,
}

// A websocket with
// - typed 'message' event
// - typed send
// - convenience methods for client-server request-response patterns
export class VoodooClient extends WebSocket {
    defaultTimeout: number;

    constructor(address: string | url.URL, options?: VoodooClientOptions) {
        super(address, options);

        this.defaultTimeout = options?.defaultTimeout || DEFAULT_TIMEOUT;

        super.on('message', function handle(message) {
            trace(`Received message: ${message}`);
            const m = JSON.parse(message.toString()) as protocol.ServerMessage;
            trace(`Message as string: ${m}`);
            switch (m.content.type) {
                case 'TransferPrepare':
                case 'TransferComplete':
                case 'TransferError':
                case 'AssignParticipants':
                case 'HubAccountsCreated':
                case 'SettlementModelCreated':
                case 'SettlementWindowClosed':
                case 'SettlementWindowCloseFailed':
                case 'SettlementWindows':
                case 'Settlements':
                case 'NewSettlementCreated':
                    this.emit(m.content.type, m);
                    this.emit(m.id, m);
                    break;
                default: {
                    // Did you get a compile error here? This code is written such that if every
                    // case in the above switch state is not handled, compilation will fail. Why?
                    // Well, as a matter of fact, we can receive a message that is not of the type
                    // we're interested in, but when we receive a message of the type we _are_
                    // interested in, we want to be sure we've handled it and emitted it to our
                    // listeners as a correctly typed event.
                    const exhaustiveCheck: never = m.content;
                    throw new Error(`Unhandled message type: ${exhaustiveCheck}`);
                }
            }
        });
    }

    send(m: protocol.ClientMessage) {
        return super.send(JSON.stringify(m));
    }

    // Exchange a single request-response pair with the server
    exchange(
        sendMsg: protocol.Request,
        timeoutMs: number = this.defaultTimeout,
    ): PromiseLike<protocol.ServerMessage> {
        return new Promise((resolve, reject) => {
            const id = uuidv4();
            trace(`Beginning ${sendMsg.type} request: ${id}. Timeout after ${timeoutMs}ms.`);
            const t = setTimeout(
                () => reject(new Error(`Request ${id} timed out after ${timeoutMs}ms`)),
                timeoutMs
            );

            this.once(id, (m: protocol.ServerMessage) => {
                trace(`Received notification for request id ${id}`);
                clearTimeout(t);
                resolve(m);
            });

            trace(`Sending request: ${sendMsg}`);
            this.send({
                id,
                content: sendMsg,
            });
        });
    }

    connected() {
        return new Promise((resolve) => {
            if (this.OPEN === this.readyState) {
                trace('Already connected');
                resolve(undefined);
            } else {
                trace('Waiting for connection');
                this.once('open', () => resolve(undefined));
            }
        });
    }

    createParticipants(
        participants: protocol.AccountInitialization[],
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange(
            {
                type: "CreateParticipants",
                value: participants,
            },
            timeoutMs,
        );
    }

    createHubAccounts(
        accounts: protocol.HubAccount[],
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange(
            {
                type: "CreateHubAccounts",
                value: accounts,
            },
            timeoutMs,
        );
    }

    completeTransfers(
        transfers: protocol.TransferMessage[],
        timeoutMs: number = this.defaultTimeout,
    ) {
        const sendMsg: protocol.Request = {
            type: 'Transfers',
            value: transfers,
        };
        return new Promise((resolve, reject) => {
            const id = uuidv4();
            trace(`Beginning ${sendMsg.type} request: ${id}. Timeout after ${timeoutMs}ms.`);
            const t = setTimeout(
                () => reject(new Error(`Request ${id} timed out after ${timeoutMs}ms`)),
                timeoutMs
            );

            const listener = (m: protocol.ServerMessage) => {
                if (m.content.type !== 'TransferPrepare') {
                    trace(`Received notification for request id ${id}`);
                    clearTimeout(t);
                    resolve(m);
                    this.off(id, listener);
                }
            };

            this.on(id, listener);

            trace(`Sending request: ${sendMsg}`);
            this.send({
                id,
                content: sendMsg,
            });
        });
    }

    async createSettlementModel(
        model: protocol.SettlementModel,
        timeoutMs: number = this.defaultTimeout,
    ) {
        const response = await this.exchange(
            {
                type: "CreateSettlementModel",
                value: model,
            },
            timeoutMs,
        );
        if (response.content.type === 'SettlementModelCreated') {
            return Promise.reject(response.content);
        }
        return response.content.value;
    }

    async getSettlementWindows(
        params: protocol.GetSettlementWindows,
        timeoutMs: number = this.defaultTimeout,
    ) {
        const response = await this.exchange(
            {
                type: "GetSettlementWindows",
                value: params,
            },
            timeoutMs,
        );
        if (response.content.type === 'SettlementWindows') {
            return Promise.reject(response.content);
        }
        return response.content.value;
    }

    async closeSettlementWindow(
        payload: protocol.SettlementWindowCloseMessage,
        timeoutMs: number = this.defaultTimeout,
    ) {
        const response = await this.exchange(
            {
                type: "CloseSettlementWindow",
                value: payload,
            },
            timeoutMs,
        );
        if (response.content.type === 'SettlementWindowClosed') {
            return Promise.reject(response.content);
        }
        return response.content.value;
    }

    async getSettlements(
        payload: protocol.GetSettlements,
        timeoutMs: number = this.defaultTimeout,
    ) {
        const response = await this.exchange(
            {
                type: "GetSettlements",
                value: payload,
            },
            timeoutMs,
        );
        if (response.content.type === 'SettlementWindows') {
            return Promise.reject(response.content);
        }
        return response.content.value;
    }

    async createSettlement(
        payload: protocol.NewSettlement,
        timeoutMs: number = this.defaultTimeout,
    ) {
        const response = await this.exchange(
            {
                type: "CreateSettlement",
                value: payload,
            },
            timeoutMs,
        );
        if (response.content.type === 'NewSettlementCreated') {
            return Promise.reject(response.content);
        }
        return response.content.value;
    }
}

export * as protocol from './lib/protocol';
