import * as protocol from './lib/protocol';
import type { ClientMessage } from './lib/protocol';
import * as dotenv from 'dotenv';
import { VoodooClient } from './index';

dotenv.config();

const ws = new VoodooClient(`ws://${process.env.VOODOO_URI}/voodoo`);

ws.on('open', function open() {
    const accounts: protocol.AccountInitialization[] = [
        { currency: "MMK", initial_position: "0", ndc: 10000 },
    ];
    const m: ClientMessage = {
        type: "CreateParticipants",
        value: accounts,
    };
    ws.send(m);
});

ws.on('message', function incoming(message: protocol.ServerMessage) {
  console.log('received: %s', message);
});

ws.on('error', function incoming(error) {
  console.log('uh oh %s', error);
});
