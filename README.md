# voodoo-doll
An in-cluster Mojaloop participant simulator intended for temporary deployment by mojo

### TODO
- It should be possible to implement more complex scenarios as a state machine. Each event could
    have some outputs, e.g. a quote could return the quote ID and transaction ID as outputs. These
    could be referenced in later events, by type (i.e. Event::QuoteRequest) or by ID (0).
    events(Event::QuoteRequest).quote_id or events(0).quote_id. We could therefore support
    arbitrarily complex scenarios with a generalised state machine. Moreover, we could subsequently
    expose this to the user so they can access this functionality. Maybe. Do we want this?
- Allow a client to "take ownership" of a participant, or participants
- Allow a client to create participants on-demand
- Allow a client to subscribe to all received FSPIOP messages for a participant or participants
