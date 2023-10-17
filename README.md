# xcm-domain-name-service

This repository contains the Polkadot XCM Domain Service, which is part of a larger project to enable cross-chain messaging (XCM) and domain services in the Polkadot ecosystem. This service allows you to manage domains and interact with contracts on different chains.

## Getting Started

Follow the steps below to set up the Polkadot XCM Domain Service on your local machine.

### Prerequisites

- Terminal
- Rust and Cargo installed
- Polkadot.js/apps installed for managing funds (if needed)

### Installation

1. **Clone the Repository**:

   Open your terminal and clone this repository:

   ```bash
   git clone https://github.com/your-username/zombienet-xcm-domain-service.git
   ```

2. **Navigate to the Service Folder**:

   Enter the `xcm-domain-service/zombienet` folder:

   ```bash
   cd xcm-domain-service/zombienet
   ```

3. **Set Permissions for the Script**:

   Run the following command to make the `zombienet.sh` script executable:

   ```bash
   chmod +x ./zombienet.sh
   ```

4. **Spawn ZombieNet**:

   Start the ZombieNet by running:

   ```bash
   ./zombienet.sh spawn
   ```

   **Note**: The first run can take 20+ minutes, so please be patient.

5. **Run the Cargo Application**:

   Open a new tab or shell, and run the following command inside the same folder:

   ```bash
   cargo run
   ```

   This will output three contract addresses: `domain-service`, `xcm-handler`, and `xc-domain-service`.

6. **Access Contracts**:

   You can access the contracts using contracts-ui. The first two contracts are deployed on chain A (<https://localhost:9910>), and the third one is on chain B (<https://localhost:9920>).

7. **Manage Funds**:

   If you need funds to buy a domain or cover fees, you can either transfer the balance from Alice using Polkadot.js/apps or run the following command in the `zombienet` folder:

   ```bash
   cargo run -- fund <addresses space separated>
   ```

   Replace `<addresses>` with the addresses you want to fund.

## License

This project is licensed under the [GNU General Public License 3.0](LICENSE) - see the [LICENSE](LICENSE) file for the full text of the license.
