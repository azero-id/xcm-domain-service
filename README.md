# XCM Domain Service (PoC)

This repository contains a working PoC of a cross-parachain domain name service consisting of multiple XCM-enabled ink! smart contracts. It's part of the delivery for our [W3F Grant](https://github.com/w3f/Grants-Program/pull/1733) ([M1 Delivery](https://github.com/w3f/Grant-Milestone-Delivery/pull/951), [M2 Delivery](https://github.com/w3f/Grant-Milestone-Delivery/pull/1030)).

The PoC domain name service implementation is a registry mapping between domain names, owning, and resolving addresses. Find a detailed [contract walkthrough](#7-interact-with-the-contracts) below and also check the inline ink! documentation for each function.

---

1. [Architecture](#architecture)
2. [Repository structure](#repository-structure)
3. [Getting started](#getting-started)
   1. [Prerequisites](#prerequisites)
   2. [Setup instructions](#setup-instructions)
   3. [Fund your account via Faucet](#fund-your-account-via-faucet)
4. [Unit tests via `xcm-simulator`](#unit-tests-via-xcm-simulator)
5. [Development environment](#development-environment)

---

## Architecture

![architecture](./Architecture.png)

Read more in the documents submitted as part of our [M2 Delivery](https://github.com/w3f/Grant-Milestone-Delivery/pull/1030).

## Repository structure

1. [`contracts/`](./contracts/): It contains the ink! smart contracts for `xcm-domain-service`.
2. [`src/`](./src/): It contains the mock `xcm-simulator` network and unit tests.
3. [`zombienet/`](./zombienet/): It allows you to deploy a live local network using `zombienet` where you can interact with the cross-chain enabled domain service.

## Getting started

Spawn a local network using Zombienet and experience the XCM domain service first-hand.

### Prerequisites

You have configured your environment for Substrate development by installing [Rust and the Rust toolchain](https://docs.substrate.io/install/).

### Setup instructions

#### 1. Clone the repo and navigate to the `zombienet/` folder:

```cmd
git clone https://github.com/azero-id/xcm-domain-service.git
cd xcm-domain-service/zombienet/
```

#### 2. Make the `zombienet.sh` bash script executable:

```cmd
chmod +x zombienet.sh
```

#### 3. Install the [polkadot](https://github.com/paritytech/polkadot/tree/release-v1.0.0), [contracts-parachain](https://github.com/azero-id/contracts-parachain) & [zombienet](https://github.com/paritytech/zombienet) binaries:

```cmd
./zombienet.sh init
```

> [!NOTE]  
> This can take a while (around 20+ minutes depending upon the system)… ☕

#### 4. Spawn the network

```cmd
./zombienet.sh spawn
```

This command spawns the following chains:

- **Relay Chain** (ws_port: `9900`)
- **Contracts Parachain #1** (ws_port: `9910`) {Hub}
- **Contracts Parachain #2** (ws_port: `9920`) {Spoke}

#### 5. Deploy the contracts

Open a new shell with the same working directory and run:

```cmd
cargo run
```

This will deploy the following contracts:

- **`Domain-service`** on Parachain #1 (address: `5Dg8MLVcwDHAv5FjWBeeLute7M9yHXoic6oBYk97fpK2BXKx`)
- **`Xcm-handler`** on Parachain #1 (address: `5DhGtfSDhZHzQKTjzy2NFEwEXBtHzMEx68WdwJmHWSM6MTZJ`)
- **`Xc-domain-service`** on Parachain #2 (address: `5CAogDHwRT8pUkRBzBsPczKSMPULuh6Zg6zK3kQQzYtQUwYf`)

> [!NOTE]  
> It can take some time (around 2-4 minutes) to complete the deployment.

#### 6. Fund your account(s)

You will need funds in your account for the domain registration and gas fees, if not using pre-funded accounts like `//Alice`. Run the following command that will transfer 100 token units to the specified addresses:

```cmd
cargo run -- fund <space-separated addresses>
```

#### 7. Interact with the contracts

Open two `contracts-ui` pages on your preferred browser, one for each chain. Use the following links:

- [Contracts-UI](https://contracts-ui.substrate.io/?rpc=ws://127.0.0.1:9910) for Parachain #1
- [Contracts-UI](https://contracts-ui.substrate.io/?rpc=ws://127.0.0.1:9920) for Parachain #2

Click on `Add New Contract` and choose the `Use On-chain Contract Address` method. Add the contracts on the respective `Contracts-UI` page. It will ask for the metadata which you can find in the [artefacts](./artefacts/) folder.

> [!NOTE]  
> You can skip this step for `Xcm-handler` contract.

Below you will find a sample order of interaction via `xc_domain_service` (Parachain #2):

1. **`register_name(name)`**: Register a domain. (fails if name is already claimed, refund not handled for simplicity).
2. **`get_owner(name) -> TicketId`**: Request for the owner address of the given name.
3. **`retrieve_owner(ticket_id)`**: Get the owner details associated with the `ticketId` (if valid).
4. **`set_address(name, multi_location)`**: Set the resolving address in `MultiLocation` format for the given `name`.
5. **`get_address(name) -> TicketId`**: Request for the resolving address details of the given name.
6. **`retrieve_address(ticket_id)`**: Get the address details associated with the TicketId (if valid).
7. **`transfer_name(name, to)`**: Transfer domain ownership

Alternatively, you can also interact with `Domain-service` on Parachain #1 directly (e.g. for double checking that the state changed consistently across chains).

> [!NOTE]  
> For more details refer to the inline documentation available for each contract message.

## Unit tests via `xcm-simulator`

You will first need to build the contracts (`domain_service`, `xcm_handler`, and `xc_domain_service`) [here](./contracts/). Then run the following command from the project root:

```cmd
cargo test
```

## Development environment

- Rust Stable: rustc 1.72.0 (5680fa18f 2023-08-23)
- Ink! v4.3.0
- Cargo-contract 4.0.0-alpha
- Relay/Para-chain nodes based on Polkadot release v1.0.0
- Zombienet v1.3.69
- System: Apple M2 Pro
