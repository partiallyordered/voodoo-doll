# mojaloop-voodoo-client

A client for https://github.com/partiallyordered/voodoo-doll

Nodejs-only, due to dependency on the `ws` websockets library.

### Installation
```sh
npm i -S mojaloop-voodoo-client
```

### Usage
#### Typescript
```typescript
import type { ClientMessage } from 'mojaloop-voodoo-client';
import { VoodooClient, protocol } from 'mojaloop-voodoo-client';

// Port-forward voodoo-doll server to localhost:3030
const client = new VoodooClient('ws://localhost:3030/voodoo');

client.on('error', function incoming(error) {
    console.log('uh oh %s', error);
});

await client.connected();

const currencies: protocol.Currency[] = ['MMK'];
await client.createHubAccounts(currencies);

const accounts: protocol.AccountInitialization[] = [
  { currency: 'MMK', initial_position: '0', ndc: 10000 },
  { currency: 'MMK', initial_position: '0', ndc: 10000 },
];
const participants = await client.createParticipants(accounts);

const transfers: protocol.TransferMessage[] = [{
  msg_sender: participants[0].name,
  msg_recipient: participants[1].name,
  currency: 'MMK',
  amount: 10,
  transfer_id: uuidv4(),
}];
await client.completeTransfers(transfers);
```

#### Node
```javascript
const voodoo_client = require('mojaloop-voodoo-client');

// Port-forward voodoo-doll server to localhost:3030
const client = new voodoo_client.VoodooClient(`ws://localhost:3030/voodoo`);

client.on('open', function open() {
    const currencies = ['MMK'];
    const m = {
        type: 'CreateHubAccounts',
        value: currencies,
    };
    client.send(m);
});

client.on('message', function incoming(message) {
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
            client.send(m);
            break;
        };
        default: {
            console.log('Unhandled');
            break;
        }
    }
});

client.on('error', function incoming(error) {
    console.log('uh oh %s', error);
});
```
