# Platform Model

```
Domain-A
    |- Domain-B
    |   |- App-B-1.1.0
    |   |   |- Actor-B-2.0.5
    |   |- App-A
    |   |   |-Actor-A-1.2.3
    |- App-A-1.0.0
        |- Actor-A-1.2.3
```

## Domain
- Domains contain sub-Domains
- Domains contain Apps

## App
- An App is an application binary. It's maps to a published crate.
- Apps are versioned
- Apps are composed of Actor(s)

## Actor
- An Actor represents an [Actix Actor](https://crates.io/crates/actix)
- Actors are versioned
- Actors are defined in library crates.

### Remote Actors
- Actors support remote messaging
- Messages can support multiple formats via [serde](https://crates.io/search?q=serde)
    - JSON, CBOR, Bincode, MessagePack
        - [Bincode](https://github.com/TyOverby/bincode) is the most efficient in terms of serialized bytes size - best choice for Rust apps
        - [MessagePack](https://msgpack.org/) is the next best choice in terms of effiency and there is much broader language support
        - [CBOR](http://cbor.io/) is more efficient than JSON and also has broad language support
        - JSON is the least efficient, but has the broadest support of all
- Remote actors can integrate via :
    - [tarpc](https://crates.io/crates/tarpc)
    - Kafka
    - NATS

## Automated Application Assemply
The goal is to automate application assembly.
- Users map actors to applications
- Users configure the actors per application.

# Platform Services

## Application Services
1. Application logging
1. Application event logging
1. Application metrics logging
1. Application metrics based monitoring and alerting service
1. Application health checks monitoring and alerting
1. Application configuration management
1. Application assembly
    - application binaries
    - docker images
1. Application release management
1. Application defect tracking
1. Application stack overflow
1. Application slack
1. Application security
1. Application reporting
1. Application dashboards

## Domain Service
1. Domain management
1. Domain security
1. User management
1. Domain stack overflow
1. Domain slack




