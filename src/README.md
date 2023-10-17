Here you will find the mock network based on [Polkadot-sdk/xcm-simulator example network](https://github.com/paritytech/polkadot-sdk/tree/release-polkadot-v1.1.0/polkadot/xcm/xcm-simulator/example) v1.1.0.

# Run unit tests

You will first need to build the contracts (`domain_service`, `xcm_handler`, and `xc_domain_service`) [here](../contracts/). Then run the following command:

```cmd
cargo test
```