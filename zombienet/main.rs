use subxt::ext::codec::Encode;
use subxt_signer::sr25519::{dev, Keypair};
use subxt::utils::AccountId32;

#[subxt::subxt(
    // runtime_metadata_url = "ws://127.0.0.1:9910"
    runtime_metadata_path = "metadata.scale",
    substitute_type(
        path = "sp_weights::weight_v2::Weight",
        with = "::subxt::utils::Static<::sp_weights::Weight>"
    )
)]
pub mod runtime {}

pub type ParachainClient = subxt::OnlineClient<subxt::SubstrateConfig>;

pub const TX_GAS: sp_weights::Weight = sp_weights::Weight::from_parts(10_000_000_000, 200_000);
pub const CUSTOM_WT: Option<(u64, u64)> = Some((10_000_000_000, 130_000));

fn get_selector(name: &str) -> [u8; 4] {
    let bytes = subxt::ext::sp_core::blake2_256(name.as_bytes());
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

fn sibling_account_account_id(para: u32, who: &AccountId32) -> AccountId32 {
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

async fn deploy_contract(
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

    let Some(instantiated) = events.find_first::<runtime::contracts::events::Instantiated>()? else {
        panic!("Failed to deploy the contract")
    };

    Ok(instantiated.contract)
}

async fn call_contract(
    client: &ParachainClient,
    contract: &AccountId32,
    caller: Keypair,
    msg: Vec<u8>,
    value: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    let call_tx = runtime::tx().contracts().call(
        contract.clone().into(), 
        value, 
        TX_GAS.into(), 
        None, 
        msg
    );

    client
        .tx()
        .sign_and_submit_then_watch_default(&call_tx, &caller)
        .await?
        .wait_for_finalized_success()
        .await?;

    Ok(())
}

async fn deploy_state_manager(
    client: &ParachainClient,
    admin: &AccountId32,
    handler: &AccountId32,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let code = std::fs::read("./artefacts/domain_service.wasm").expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, admin, handler).encode(); // (selector, admin, handler)

    deploy_contract(client, code, payload, dev::alice()).await
}

async fn deploy_xcm_handler(
    client: &ParachainClient,
    admin: &AccountId32,
    state_manager: &AccountId32,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let code = std::fs::read("./artefacts/xcm_handler.wasm").expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, admin, state_manager, CUSTOM_WT).encode(); // (selector, admin, state_manager, custom_wt)

    deploy_contract(client, code, payload, dev::alice()).await
}

async fn deploy_xc_contract(
    client: &ParachainClient,
    xcm_handler: &AccountId32,
    xcm_handler_soac: &AccountId32,
) -> Result<AccountId32, Box<dyn std::error::Error>> {
    let code =
        std::fs::read("./artefacts/xc_domain_service.wasm").expect("cound not find wasm blob");

    let sel_constructor = get_selector("new");
    let payload = (sel_constructor, xcm_handler, xcm_handler_soac, CUSTOM_WT).encode(); // (selector, xcm_handler, xcm_handler_soac, custom_wt)

    deploy_contract(client, code, payload, dev::alice()).await
}

async fn set_handler(
    client: &ParachainClient,
    state_manager: &AccountId32,
    xcm_handler: &AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    let sel_set_handler = get_selector("set_handler");
    let payload = (sel_set_handler, xcm_handler).encode(); // (selector, new_handler)

    call_contract(client, state_manager, dev::alice(), payload, 0).await
}

async fn add_xc_contract(
    client: &ParachainClient,
    xcm_handler: &AccountId32,
    xc_contract_soac: &AccountId32,
    origin_path: &(u8, Option<u32>, AccountId32)
) -> Result<(), Box<dyn std::error::Error>> {
    let sel_add_xc_contract = get_selector("add_xc_contract");
    let payload = (sel_add_xc_contract, xc_contract_soac, origin_path).encode(); // (selector, xc_contract_soac, origin_path)

    call_contract(client, xcm_handler, dev::alice(), payload, 0).await
}

async fn fund_address(
    client: &ParachainClient,
    addr: &AccountId32,
) -> Result<(), Box<dyn std::error::Error>> {
    let balance_transfer_tx = runtime::tx().balances().transfer_allow_death(addr.clone().into(), 1_000_000_000_000);

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

async fn setup(para_a: &ParachainClient, para_b: &ParachainClient) -> Result<(AccountId32, AccountId32, AccountId32), Box<dyn std::error::Error>> {
    let alice: AccountId32 = dev::alice().public_key().into();

    // 1. Deploy `domain_service: state-handler`
    let state_manager = deploy_state_manager(para_a, &alice, &alice).await?;
    println!("Domain-service deployed on ParaA with Address: {:}", state_manager);

    // 2A. Deploy `xcm_handler`
    let xcm_handler = deploy_xcm_handler(para_a, &alice, &state_manager).await?;
    let xcm_handler_soac = sibling_account_account_id(1, &xcm_handler);
    println!("Xcm-handler deployed on ParaA with Address: {:}", xcm_handler);

    // 2B. Update state_manager::set_handler
    set_handler(para_a, &state_manager, &xcm_handler).await?;
    println!("Linked the xcm-handler with domain-service successfully");

    // 3A. Deploy `xc_domain_service: xc-contract`
    let xc_contract = deploy_xc_contract(para_b, &xcm_handler, &xcm_handler_soac).await?;
    let xc_contract_soac = sibling_account_account_id(2, &xc_contract);
    println!("Xc-domain-service deployed on ParaB with Address: {:}", xc_contract);

    // 3B. Approve xc_contract on xcm_handler
    let origin_path = (1, Some(2), xc_contract.clone());  // (parent, Option<ParaId>, AccountId)
    add_xc_contract(para_a, &xcm_handler, &xc_contract_soac, &origin_path).await?;
    println!("ParaB's xc-domain-service approved with the Xcm-handler");

    // 4. Fund sovereign accounts for gas fee payment
    println!("Funding sovereign accounts...");
    fund_address(para_a, &xc_contract_soac).await?;
    fund_address(para_b, &xcm_handler_soac).await?;

    Ok((state_manager, xcm_handler, xc_contract))
}

#[tokio::main]
pub async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let para_a = subxt::OnlineClient::<subxt::SubstrateConfig>::from_url("ws://127.0.0.1:9910")
        .await
        .unwrap();

    let para_b = subxt::OnlineClient::<subxt::SubstrateConfig>::from_url("ws://127.0.0.1:9920")
        .await
        .unwrap();

    setup(&para_a, &para_b).await?;

    Ok(())
}
