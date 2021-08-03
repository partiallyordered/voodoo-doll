import * as protocol from './lib/protocol';
import WebSocket from 'ws';

// A websocket with typed 'message' event, and typed send.
export class VoodooClient extends WebSocket {
    send(m: protocol.ClientMessage) {
        super.send(JSON.stringify(m));
    }

    on(event: string | symbol, listener: (this: WebSocket, ...args: any[]) => void): this {
        if (event === 'message') {
            super.on('message', function incoming(message) {
                const m = JSON.parse(message.toString()) as protocol.ServerMessage;
                listener.bind(this)(m);
            })
        } else {
            super.on(event, listener);
        }
        return this;
    }
}

export * as protocol from './lib/protocol';

// vim: sw=4 ts=4 et
