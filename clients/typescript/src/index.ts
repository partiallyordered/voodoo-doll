import * as protocol from './lib/protocol';
import WebSocket from 'ws';
import * as url from 'url';
import * as http from 'http';

// A websocket with typed 'message' event, and typed send.
export class VoodooClient extends WebSocket {
    // constructor(address: string | url.URL, options?: WebSocket.ClientOptions) {
    //     super(address, options);
    // }

    send(m: protocol.ClientMessage) {
        super.send(JSON.stringify(m));
    }

    // on(event: 'close', listener: (this: WebSocket, code: number, reason: string) => void): this;
    // on(event: 'error', listener: (this: WebSocket, err: Error) => void): this;
    // on(event: 'upgrade', listener: (this: WebSocket, request: http.IncomingMessage) => void): this;
    // on(event: 'message', listener: (this: WebSocket, data: protocol.ServerMessage) => void): this;
    // on(event: 'open' , listener: (this: WebSocket) => void): this;
    // on(event: 'ping' | 'pong', listener: (this: WebSocket, data: Buffer) => void): this;
    // on(event: 'unexpected-response', listener: (this: WebSocket, request: http.ClientRequest, response: http.IncomingMessage) => void): this;
    on(event: string | symbol, listener: (this: WebSocket, ...args: any[]) => void): this {
        // super.on(event, listener);
        if (event === 'message') {
            super.on('message', function incoming(message) {
                if (message.isBinary) {

                }
                // const m: protocol.ServerMessage = JSON.parse(message);
                const m = JSON.parse(message) as protocol.ServerMessage ;
                listener(this, m);
            })
        } else {
            super.on(event, listener);
        }
        return this;
    }
    // on(event: string | symbol, listener: (this: WebSocket, ...args: any[]) => void): this {
    //     if (event === 'message') {
    //
    //     }
    // }
    // on(event: 'message', listener: (this: WebSocket, data: protocol.ServerMessage) => void) {
    //
    // }
}

const ws = new WebSocket('ws://www.host.com/path');

ws.on('open', function open() {
  ws.send('something');
});

ws.on('message', function incoming(message) {
  console.log('received: %s', message);
});
