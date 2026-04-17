# fscl-core

Core domain and application crate for FSCL.

It contains:

- domain types such as `ResourceId`, `Component`, and `DomainEvent`
- application workflows such as `ComponentLifecycleUow`
- ports for repository, unit-of-work, and event publishing
- a small bridge to `fscl-messaging` for building shared event envelopes

## Dependencies

```text
fscl-core
    |
    +--> fscl-messaging

view services
    |
    +--> fscl-core
```

`fscl-core` depends on `fscl-messaging`  because it exposes a bridge helper in `core::messaging` for constructing `EventEnvelope` values from core identifiers.

## Split

```text
fscl-core        -> domain model, workflows, ports
fscl-messaging   -> wire contract, outbox contract, schema constants
service crates   -> persistence, transport adapters, runtime config
```

`fscl-core` should not own deployment configuration. It only exposes reusable code contracts.

## Config

`fscl-core` has no runtime env contract of its own.

Relevant exported messaging constants come from `fscl-messaging`:

- `OUTBOX_NOTIFY_CHANNEL`: fixed outbox notify channel contract
- `OUTBOX_TABLE`: outbox table name contract
- `OUTBOX_SCHEMA_VERSION`: schema compatibility marker

These are code-level contract values, not deployment-time settings.

## Dev Setup

```sh
cd fscl-core
cargo test
```

No `.env` file is required for this crate.

## K8s Setup

Scaffold only: `fscl-core` is not deployed directly to Kubernetes.

It is consumed by deployed binaries such as:

- `fscl-process-svc`
- `fscl-outbox-publisher`
