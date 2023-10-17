# XCM domain service (POC)

TODO

## Repository structure

1. [contracts](./contracts/): It contains the ink! smart contracts for xcm-domain-service.
2. [src](./src/): It contains the mock xcm-simulator network and unit tests.
3. [zombienet](./zombienet/): It allows you to deploy a live local network using zombienet where you can interact with the cross-chain enabled domain service!

## Environment used in development

- Rust Stable: rustc 1.72.0 (5680fa18f 2023-08-23)
- Ink! v4.3.0
- Cargo-contract 4.0.0-alpha
- Relay/Para-chain nodes based on Polkadot release v1.0.0
- Zombienet v1.3.69
- System: Apple M2 Pro