# Valence Domain Clients

This directory contains various utilities for interacting with chains from an off-chain client.

`src/` contains the following directories:
- `common` for defining common types and functionality that will apply to all domains
- `cosmos` for defining cosmos-sdk related types and traits
- `evm` for defining evm related types and traits

For example implementations, see the following files under `src/`:
- cosmos: `neutron.rs`, `osmosis.rs`
- evm: `ethereum.rs`

## Co-processor

#### CLI

To install the valence-coprocessor binary:

```shell
cargo install \
  --git https://github.com/timewave-computer/valence-domain-clients.git \
  --no-default-features \
  --features coprocessor-bin \
  --bin valence-coprocessor
```

You can check the available commands via:

```shell
valence-coprocessor --help
```
