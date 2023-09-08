// Copyright Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.
#![allow(dead_code)]

pub mod mock_msg_queue;
pub mod runtimes;

pub use runtimes::{parachain, relay_chain};

use frame_support::{sp_tracing, traits::GenesisBuild};
use xcm::prelude::*;
use xcm_executor::traits::Convert;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

// Accounts
pub const ADMIN: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([99u8; 32]);
pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([111u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([222u8; 32]);

// Balances
pub type Balance = u128;
pub const UNITS: Balance = 10_000_000_000;
pub const CENTS: Balance = UNITS / 100; // 100_000_000
pub const INITIAL_BALANCE: u128 = 1000 * UNITS;

decl_test_parachain! {
    pub struct ParaA {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::MsgQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(1),
    }
}

decl_test_parachain! {
    pub struct ParaB {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::MsgQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(2),
    }
}

decl_test_parachain! {
    pub struct ParaC {
        Runtime = parachain::Runtime,
        XcmpMessageHandler = parachain::MsgQueue,
        DmpMessageHandler = parachain::MsgQueue,
        new_ext = para_ext(2),
    }
}

decl_test_relay_chain! {
    pub struct Relay {
        Runtime = relay_chain::Runtime,
        XcmConfig = relay_chain::XcmConfig,
        new_ext = relay_ext(),
    }
}

decl_test_network! {
    pub struct MockNet {
        relay_chain = Relay,
        parachains = vec![
            (1, ParaA),
            (2, ParaB),
            (3, ParaC),
        ],
    }
}

pub type RelaychainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type RelaychainBalances = pallet_balances::Pallet<relay_chain::Runtime>;
pub type ParachainBalances = pallet_balances::Pallet<parachain::Runtime>;
pub type ParachainAssets = pallet_assets::Pallet<parachain::Runtime>;
pub type ParachainContracts = pallet_contracts::Pallet<parachain::Runtime>;

pub fn relay_sovereign_account_id() -> parachain::AccountId {
    let location = (Parent,);
    parachain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn parachain_sovereign_account_id(para: u32) -> relay_chain::AccountId {
    let location = (Parachain(para),);
    relay_chain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn parachain_account_sovereign_account_id(
    para: u32,
    who: sp_runtime::AccountId32,
) -> relay_chain::AccountId {
    let location = (
        Parachain(para),
        AccountId32 {
            network: Some(relay_chain::RelayNetwork::get()),
            id: who.into(),
        },
    );
    relay_chain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn sibling_sovereign_account_id(para: u32) -> parachain::AccountId {
    let location = (Parent, Parachain(para));
    parachain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn sibling_account_sovereign_account_id(
    para: u32,
    who: sp_runtime::AccountId32,
) -> parachain::AccountId {
    let location = (
        Parent,
        Parachain(para),
        AccountId32 {
            network: None,
            id: who.into(),
        },
    );
    parachain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn relay_account_sovereign_account_id(who: sp_runtime::AccountId32) -> parachain::AccountId {
    let location = (
        Parent,
        AccountId32 {
            network: None,
            id: who.into(),
        },
    );
    parachain::SovereignAccountOf::convert(location.into()).unwrap()
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
    use parachain::{MsgQueue, Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();
    let other_para_ids = match para_id {
        1 => [2, 3],
        2 => [1, 3],
        3 => [1, 2],
        _ => panic!("No parachain exists with para_id = {para_id}"),
    };

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (relay_sovereign_account_id(), INITIAL_BALANCE),
            (BOB, INITIAL_BALANCE),
        ]
        .into_iter()
        .chain(other_para_ids.iter().map(
            // Initial balance of native token for ALICE on all sibling sovereign accounts
            |&para_id| {
                (
                    sibling_account_sovereign_account_id(para_id, ALICE),
                    INITIAL_BALANCE,
                )
            },
        ))
        .chain(other_para_ids.iter().map(
            // Initial balance of native token all sibling sovereign accounts
            |&para_id| (sibling_sovereign_account_id(para_id), INITIAL_BALANCE),
        ))
        .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_assets::GenesisConfig::<Runtime> {
        assets: vec![
            (0u128, ADMIN, false, 1u128), // Create derivative asset for relay's native token
        ]
        .into_iter()
        .chain(
            other_para_ids
                .iter()
                .map(|&para_id| (para_id as u128, ADMIN, false, 1u128)),
        ) // Derivative assets for the other parachains' native tokens
        .collect(),
        metadata: Default::default(),
        accounts: vec![
            (0u128, ALICE, INITIAL_BALANCE),
            (0u128, relay_sovereign_account_id(), INITIAL_BALANCE),
        ]
        .into_iter()
        .chain(
            other_para_ids
                .iter()
                .map(|&para_id| (para_id as u128, ALICE, INITIAL_BALANCE)),
        ) // Initial balance for derivatives of other parachains' tokens
        .chain(other_para_ids.iter().map(|&para_id| {
            (
                0u128,
                sibling_account_sovereign_account_id(para_id, ALICE),
                INITIAL_BALANCE,
            )
        })) // Initial balance for sovereign accounts (for fee payment)
        .chain(other_para_ids.iter().map(|&para_id| {
            (
                0u128,
                sibling_sovereign_account_id(para_id),
                INITIAL_BALANCE,
            )
        })) // Initial balance for sovereign accounts (for fee payment)
        .collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        sp_tracing::try_init_simple();
        System::set_block_number(1);
        MsgQueue::set_para_id(para_id.into());
    });
    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use relay_chain::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (parachain_sovereign_account_id(1), INITIAL_BALANCE),
            (parachain_sovereign_account_id(2), INITIAL_BALANCE),
            (parachain_sovereign_account_id(3), INITIAL_BALANCE),
            (
                parachain_account_sovereign_account_id(1, ALICE),
                INITIAL_BALANCE,
            ),
            (
                parachain_account_sovereign_account_id(2, ALICE),
                INITIAL_BALANCE,
            ),
            (
                parachain_account_sovereign_account_id(3, ALICE),
                INITIAL_BALANCE,
            ),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

pub fn print_para_events() {
    use parachain::System;
    System::events()
        .iter()
        .for_each(|r| println!(">>> {:?}", r.event));

    System::reset_events();
}

pub fn print_relay_events() {
    use relay_chain::System;
    System::events()
        .iter()
        .for_each(|r| println!(">>> {:?}", r.event));

    System::reset_events();
}

pub fn relay_successful_execution() -> bool {
    use relay_chain::System;
    System::events().iter().any(|e| match &e.event {
        relay_chain::RuntimeEvent::ParasUmp(
            polkadot_runtime_parachains::ump::Event::ExecutedUpward(_, outcome),
        ) => outcome.clone().ensure_complete().is_ok(),
        _ => false,
    })
}
