import * as protocol from './lib/protocol';
import WebSocket from 'ws';
import * as url from 'url';

const trace = (...args: any[]) => {}; // console.log(...args);

// A websocket with
// - typed 'message' event
// - typed send
// - convenience methods for client-server request-response patterns
export class VoodooClient extends WebSocket {
    constructor(address: string | url.URL, options?: WebSocket.ClientOptions) {
        super(address, options);

        super.on('message', function handle(message) {
            trace(`Received message: ${message}`);
            const m = JSON.parse(message.toString()) as protocol.ServerMessage;
            trace(`Message as string: ${m}`);
            switch (m.type) {
                case 'AssignParticipants':
                case 'HubAccountsCreated':
                case 'TransferComplete':
                case 'TransferError':
                    this.emit(m.type, m.value);
                    break;
                default: {
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
    exchange(
        sendMsg: protocol.ClientMessage,
        recvMsgType: protocol.ServerMessage['type'],
        timeoutMs: number = 5000,
    ) {
        return new Promise((resolve, reject) => {
            const exchangeType = `${sendMsg.type}-${recvMsgType}`;
            trace(`Beginning ${exchangeType} exchange`);
            const t = setTimeout(
                () => reject(`${exchangeType} exchange timed out after ${timeoutMs}ms`),
                timeoutMs
            );

            this.once(recvMsgType, (m: protocol.ServerMessage['value']) => {
                trace(`Received ${recvMsgType} message for ${exchangeType} exchange`);
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
        timeoutMs: number = 5000,
    ) {
        return this.exchange(
            {
                type: "CreateParticipants",
                value: participants,
            },
            "AssignParticipants",
            timeoutMs,
        );
    }

    createHubAccounts(
        currencies: protocol.Currency[],
        timeoutMs: number = 5000,
    ) {
        return this.exchange(
            {
                type: "CreateHubAccounts",
                value: currencies,
            },
            "HubAccountsCreated",
            timeoutMs,
        );
    }

    completeTransfers(
        transfers: protocol.TransferMessage[],
        timeoutMs: number = 5000,
    ) {
        return this.exchange(
            {
                type: "Transfers",
                value: transfers,
            },
            "TransferComplete",
            timeoutMs,
        );
    }
}

export * as protocol from './lib/protocol';
