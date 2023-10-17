Spawn a local network using Zombienet and try out our XCM domain service.

# Prerequisites

You have configured your environment for Substrate development by installing [Rust and the Rust toolchain](https://docs.substrate.io/install/).

# Setup instructions

1. Make the bash script - `zombienet.sh` executable.

```cmd
chmod +x zombienet.sh
```

2. Install the polkadot, contracts-parachain, zombienet binaries.

```cmd
./zombienet.sh init
```

> **NOTE:** This can take a while (around 20+ minutes depending upon the system)...

3. Spawn the network

```cmd
./zombienet.sh spawn
```

This command spawns the following chains:

- Relay chain (ws_port: 9900)  
- Contracts parachain#1 (ws_port: 9910) {Hub}  
- Contracts parachain#2 (ws_port: 9920) {Spoke}  

4. Deploy the contracts

Open a new shell with the same working directory and run:

```cmd
cargo run
```

This will deploy the following contracts:

- **`Domain-service`** on parachain#1 (address: `5Dg8MLVcwDHAv5FjWBeeLute7M9yHXoic6oBYk97fpK2BXKx`)

- **`Xcm-handler`** on parachain#1 (address: `5DhGtfSDhZHzQKTjzy2NFEwEXBtHzMEx68WdwJmHWSM6MTZJ`)

- **`Xc-domain-service`** on parachain#2 (address: `5CAogDHwRT8pUkRBzBsPczKSMPULuh6Zg6zK3kQQzYtQUwYf`)

> **NOTE:** It can take some time (around 2-4 minutes) to complete the deployment.

5. Interacting with the contracts.

Open two `contracts-ui` page on your preferred browser, one for each chain. Use the following links:

- [Contracts-UI](https://contracts-ui.substrate.io/?rpc=ws://127.0.0.1:9910) for parachain#1

- [Contracts-UI](https://contracts-ui.substrate.io/?rpc=ws://127.0.0.1:9920) for parachain#2

Click on `Add New Contract` and choose the `Use On-chain Contract Address` method. Add the contracts on the respective `Contracts-UI` page. It will ask for the metadata which you can find in the [artefacts](./artefacts/) folder.

> **INFO:** You can skip this step for `Xcm-handler` contract.

## Faucet - Fund your account

You will need funds in your account for domain purchases/gas payment if not using pre-funded accounts like ALICE. Run the following command that will transfer 100 units of token to the specified addresses:

```cmd
cargo run -- fund <space-separated addresses>
```
