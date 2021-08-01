import * as protocol from './lib/protocol';
import WebSocket from 'ws';
import * as dotenv from 'dotenv';
import { VoodooClient } from './index';

dotenv.config();

const ws = new VoodooClient(`ws://${process.env.VOODOO_URI}`);

ws.on('open', function open() {
    ws.send(protocol.ClientMessage [
        protocol.AccountInitialization
    ]);
});

ws.on('message', function incoming(message: protocol.ServerMessage) {
  console.log('received: %s', message);
});
