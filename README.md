# voodoo-doll
An in-cluster Mojaloop participant simulator intended for temporary deployment by mojo

## Build
Also generates Kubernetes manifests in the `kubernetes` directory.
```sh
cargo b --all-features
```

## Generate Typescript protocol
```sh
cargo t --features=typescript_types
```

### TODO
- How to handle multiple k8s API versions?
- Is it possible to detect the k8s API version?
- k8s-openapi should be a sibling dependency. How do other libs do this?
- Basic deployment smoke test of the k8s manifests
- Change pod to deployment, easier to change version, especially with Skaffold
- Document client features, and required features
- Extend clients to support
  - convenient deployment/detection of voodoo-doll
  - convenient port-forwarding mechanisms, preferably including to a stream abstraction available
      within the language
- A release process that
  - Puts the GH revision in the built artifacts
  - Sets all the built artifact versions equal
  - Allows only semver releases
  - Either
    1. prevents code with unchanged versions being merged
        - Potentially annoying for things that don't change build outputs, e.g. a CI configuration
            change.
        - Means that all changes to the repo will be versioned. So if there are accidental changes,
            e.g. the CI configuration change actually results in an unforeseen change in an artifact,
            this change will be versioned somehow or other.
    2. modifies the versions at release
        - Easier to manage
        - Potentially annoying to implement, because the release has to modify HEAD before building
            artifacts, to keep versions aligned with the code they're built from.
- Versioned protocol. This is useful because it becomes possible to support older clients with
    newer versions of voodoo-doll. So, for example, the client can simply check "is the server
    version at least as new as me?". It should also mean that if a client detects a server version
    that's too old, it can simply update the server version, with no/minimal risk of disrupting
    other clients. *But* if there's any state in the server itself (there is) this could be
    problematic. Can we store our config using k8s API? Or should this service share config with
    itself using raft or something?
- Implement a timeout for the service to shut itself down.
- Make sure to correctly handle sigterm, sigkill, etc.
- It should be possible to implement more complex scenarios as a state machine. Each event could
    have some outputs, e.g. a quote could return the quote ID and transaction ID as outputs. These
    could be referenced in later events, by type (i.e. Event::QuoteRequest) or by ID (0).
    events(Event::QuoteRequest).quote_id or events(0).quote_id. We could therefore support
    arbitrarily complex scenarios with a generalised state machine. Moreover, we could subsequently
    expose this to the user so they can access this functionality. Maybe. Do we want this? Or do we
    just want the user to handle their own state transitions? Isn't that kind of the point of this
    service? We could help them out with good client libs.
- Allow a client to "take ownership" of a participant, or participants
- Allow a client to create participants on-demand
- Allow a client to subscribe to all received FSPIOP messages for a participant or participants
