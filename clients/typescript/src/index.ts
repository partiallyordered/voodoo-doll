import * as protocol from './lib/protocol';
import WebSocket from 'ws';
import * as url from 'url';

const trace = (...args: any[]) => {}; // console.log(...args);

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
            switch (m.type) {
                case 'AssignParticipants':
                case 'HubAccountsCreated':
                case 'TransferComplete':
                case 'TransferError':
                case 'SettlementModelCreated':
                case 'SettlementWindowClosed':
                case 'SettlementWindowCloseFailed':
                case 'SettlementWindows':
                case 'Settlements':
                case 'NewSettlementCreated':
                    this.emit(m.type, m.value);
                    break;
                default: {
                    // Did you get a compile error here? This code is written such that if every
                    // case in the above switch state is not handled, compilation will fail. Why?
                    // Well, as a matter of fact, we can receive a message that is not of the type
                    // we're interested in, but when we receive a message of the type we _are_
                    // interested in, we want to be sure we've handled it and emitted it to our
                    // listeners as a correctly typed event.
                    const exhaustiveCheck: never = m;
                    throw new Error(`Unhandled message type: ${exhaustiveCheck}`);
                }
            }
        });
    }

    send(m: protocol.ClientMessage) {
        return super.send(JSON.stringify(m));
    }

    // Exchange a single request-response pair with the server
    exchange<ServerResponse extends protocol.ServerMessage['value']>(
        sendMsg: protocol.ClientMessage,
        recvMsgDiscriminator: protocol.ServerMessage['type'],
        timeoutMs: number = this.defaultTimeout,
    ): PromiseLike<ServerResponse> {
        return new Promise((resolve, reject) => {
            const exchangeType = `${sendMsg.type}-${recvMsgDiscriminator}`;
            trace(`Beginning ${exchangeType} exchange. Timeout after ${timeoutMs}ms.`);
            const t = setTimeout(
                () => reject(new Error(`${exchangeType} exchange timed out after ${timeoutMs}ms`)),
                timeoutMs
            );

            this.once(recvMsgDiscriminator, (m: ServerResponse) => {
                trace(`Received ${recvMsgDiscriminator} message for ${exchangeType} exchange`);
                clearTimeout(t);
                resolve(m);
            });

            trace(`Sending ${sendMsg.type} message for ${exchangeType} exchange`);
            this.send(sendMsg);
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
        return this.exchange<protocol.ClientParticipant[]>(
            {
                type: "CreateParticipants",
                value: participants,
            },
            "AssignParticipants",
            timeoutMs,
        );
    }

    createHubAccounts(
        accounts: protocol.HubAccount[],
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.HubAccount[]>(
            {
                type: "CreateHubAccounts",
                value: accounts,
            },
            "HubAccountsCreated",
            timeoutMs,
        );
    }

    completeTransfers(
        transfers: protocol.TransferMessage[],
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.TransferCompleteMessage>(
            {
                type: "Transfers",
                value: transfers,
            },
            "TransferComplete",
            timeoutMs,
        );
    }

    createSettlementModel(
        model: protocol.SettlementModel,
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.SettlementModelCreatedMessage>(
            {
                type: "CreateSettlementModel",
                value: model,
            },
            "SettlementModelCreated",
            timeoutMs,
        );
    }

    getSettlementWindows(
        params: protocol.GetSettlementWindows,
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.SettlementWindow[]>(
            {
                type: "GetSettlementWindows",
                value: params,
            },
            "SettlementWindows",
            timeoutMs,
        );
    }

    closeSettlementWindow(
        payload: protocol.SettlementWindowCloseMessage,
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.SettlementWindowId>(
            {
                type: "CloseSettlementWindow",
                value: payload,
            },
            "SettlementWindowClosed",
            timeoutMs,
        );
    }

    getSettlements(
        payload: protocol.GetSettlements,
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.Settlement[]>(
            {
                type: "GetSettlements",
                value: payload,
            },
            "Settlements",
            timeoutMs,
        );
    }

    createSettlement(
        payload: protocol.NewSettlement,
        timeoutMs: number = this.defaultTimeout,
    ) {
        return this.exchange<protocol.Settlement>(
            {
                type: "CreateSettlement",
                value: payload,
            },
            "NewSettlementCreated",
            timeoutMs,
        );
    }
}

export * as protocol from './lib/protocol';
