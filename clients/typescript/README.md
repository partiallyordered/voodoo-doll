# voodoo-client

A client for https://github.com/partiallyordered/voodoo-doll

Nodejs-only, due to dependency on the `ws` websockets library.

### Installation
```sh
npm i -S voodoo-client
```

### Usage
#### Typescript
```typescript
import type { ClientMessage } from 'voodoo-client';
import { VoodooClient, protocol } from 'voodoo-client';

// Port-forward voodoo-doll server to localhost:3030
const ws = new VoodooClient(`ws://localhost:3030/voodoo`);

ws.on('open', function open() {
    const currencies: protocol.Currency[] = ["MMK"];
    const m: ClientMessage = {
        type: "CreateHubAccounts",
        value: currencies,
    };
    ws.send(m);
});

ws.on('message', function incoming(message: protocol.ServerMessage) {
    console.log('received: %s', message);
    switch (message.type) {
        case "HubAccountsCreated": {
            console.log("Creating participant accounts");
            const accounts: protocol.AccountInitialization[] = [
                { currency: "MMK", initial_position: "0", ndc: 10000 },
            ];
            const m: ClientMessage = {
                type: "CreateParticipants",
                value: accounts,
            };
            ws.send(m);
        };
        default: {
            console.log("Unhandled");
        }
    }
});

ws.on('error', function incoming(error) {
    console.log('uh oh %s', error);
});
```

#### Node
```javascript
const voodoo_client = require('voodoo-client');

// Port-forward voodoo-doll server to localhost:3030
const ws = new voodoo_client.VoodooClient(`ws://localhost:3030/voodoo`);

ws.on('open', function open() {
    const currencies = ['MMK'];
    const m = {
        type: 'CreateHubAccounts',
        value: currencies,
    };
    ws.send(m);
});

ws.on('message', function incoming(message) {
    console.log('received: %s', message);
    switch (message.type) {
        case 'HubAccountsCreated': {
            console.log('Creating participant accounts');
            const accounts = [
                { currency: 'MMK', initial_position: '0', ndc: 10000 },
            ];
            const m = {
                type: 'CreateParticipants',
                value: accounts,
            };
            ws.send(m);
        };
        default: {
            console.log('Unhandled');
        }
    }
});

ws.on('error', function incoming(error) {
    console.log('uh oh %s', error);
});
```
