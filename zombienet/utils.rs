use super::*;

pub const TX_GAS: sp_weights::Weight = sp_weights::Weight::from_parts(10_000_000_000, 200_000);

pub async fn deploy_contract(
    client: &ParachainClient,
    code: Vec<u8>,
    payload: Vec<u8>,
    deployer: Keypair,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let deploy_tx = runtime::tx().contracts().instantiate_with_code(
        0,
        TX_GAS.into(),
        None,
        code,
        payload,
        vec![],
    );

    let events = client
        .tx()
        .sign_and_submit_then_watch_default(&deploy_tx, &deployer)
        .await?
        .wait_for_finalized_success()
        .await?;

    let Some(instantiated) = events.find_first::<runtime::contracts::events::Instantiated>()?
    else {
        panic!("Failed to deploy the contract")
    };

    Ok(instantiated.contract)
}

pub async fn call_contract(
    client: &ParachainClient,
    contract: &AccountId32,
    caller: Keypair,
    msg: Vec<u8>,
    value: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    let call_tx =
        runtime::tx()
            .contracts()
            .call(contract.clone().into(), value, TX_GAS.into(), None, msg);

    client
        .tx()
        .sign_and_submit_then_watch_default(&call_tx, &caller)
        .await?
        .wait_for_finalized_success()
        .await?;

    Ok(())
}

pub async fn fund_address(
    client: &ParachainClient,
    addr: &AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    let balance_transfer_tx = runtime::tx()
        .balances()
        .transfer_allow_death(addr.clone().into(), 100_000_000_000_000);

    let from = dev::alice();

    let events = client
        .tx()
        .sign_and_submit_then_watch_default(&balance_transfer_tx, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

    let _transfer_event = events.find_first::<runtime::balances::events::Transfer>()?;

    Ok(())
}

pub async fn fund_user(
    para_a: &ParachainClient,
    para_b: &ParachainClient,
    user: &AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Sending funds to the following user: {:}", user);

    println!("Funding user on chain A");
    fund_address(para_a, user).await?;

    println!("Funding user on chain B");
    fund_address(para_b, user).await?;

    let chain_b_soac = sibling_account_account_id(2, user);
    println!("Funding sovereign account on chain A: {:}", chain_b_soac);
    fund_address(para_a, &chain_b_soac).await?;

    let chain_a_soac = sibling_account_account_id(1, user);
    println!("Funding sovereign account on chain B: {:}", chain_a_soac);
    fund_address(para_b, &chain_a_soac).await?;

    Ok(())
}

pub fn get_selector(name: &str) -> [u8; 4] {
    let bytes = subxt::ext::sp_core::blake2_256(name.as_bytes());
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

pub fn sibling_account_account_id(para: u32, who: &AccountId32) -> AccountId32 {
    let location: xcm::v3::MultiLocation = (
        xcm::v3::Parent,
        xcm::v3::prelude::Parachain(para),
        xcm::v3::prelude::AccountId32 {
            network: None, // Ensure network matches the runtime
            id: who.0,
        },
    )
        .into();

    // based on Account32Hash<(), AccountId>
    ("multiloc", location)
        .using_encoded(subxt::ext::sp_core::blake2_256)
        .into()
}
