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

//! Parachain runtime mock.

use super::{
    mock_msg_queue::pallet as mock_msg_queue, AllowNoteUnlockables, Balance,
    ForeignChainAliasAccount,
};

use core::marker::PhantomData;
use frame_support::{
    construct_runtime, ensure, parameter_types,
    traits::{
        AsEnsureOriginWithArg, Contains, ContainsPair, EnsureOrigin, EnsureOriginWithArg,
        Everything, EverythingBut, Nothing,
    },
    weights::{
        constants::{WEIGHT_PROOF_SIZE_PER_MB, WEIGHT_REF_TIME_PER_SECOND},
        Weight,
        IdentityFee
    },
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_core::{ConstU128, ConstU32, H256};
use sp_runtime::{
    testing::Header,
    traits::{Get, IdentityLookup},
    AccountId32,
};
use sp_std::{cell::RefCell, prelude::*};
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowExplicitUnpaidExecutionFrom, AllowTopLevelPaidExecutionFrom,
    AsPrefixedGeneralIndex, ConvertedConcreteId, CurrencyAdapter as XcmCurrencyAdapter,
    EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, FungiblesAdapter, IsConcrete,
    NativeAsset, NoChecking, NonFungiblesAdapter, ParentAsSuperuser, ParentIsPreset,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, WithComputedOrigin,
};
use xcm_executor::{
    traits::{AssetExchange, Convert, JustTry},
    Assets as HoldingAssets, Config, XcmExecutor,
};

pub type AccountId = AccountId32;
pub type AssetIdForAssets = u128;

pub type SovereignAccountOf = (
    ForeignChainAliasAccount<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
    ParentIsPreset<AccountId>,
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type HoldIdentifier = ();
    type FreezeIdentifier = ();
    type MaxHolds = ConstU32<0>;
    type MaxFreezes = ConstU32<0>;
}

parameter_types! {
    pub const AssetDeposit: u128 = 1_000_000;
    pub const MetadataDepositBase: u128 = 1_000_000;
    pub const MetadataDepositPerByte: u128 = 100_000;
    pub const AssetAccountDeposit: u128 = 1_000_000;
    pub const ApprovalDeposit: u128 = 1_000_000;
    pub const AssetsStringLimit: u32 = 50;
    pub const RemoveItemsLimit: u32 = 50;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = AssetIdForAssets;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type AssetAccountDeposit = AssetAccountDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
    type RemoveItemsLimit = RemoveItemsLimit;
    type AssetIdParameter = AssetIdForAssets;
    type CallbackHandle = ();
}

// `EnsureOriginWithArg` impl for `CreateOrigin` which allows only XCM origins
// which are the correct sovereign account.
pub struct ForeignCreators;
impl EnsureOriginWithArg<RuntimeOrigin, MultiLocation> for ForeignCreators {
    type Success = AccountId;

    fn try_origin(
        o: RuntimeOrigin,
        a: &MultiLocation,
    ) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
        let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())?;
        if !a.starts_with(&origin_location) {
            return Err(o);
        }
        SovereignAccountOf::convert(origin_location).map_err(|_| o)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin(a: &MultiLocation) -> Result<RuntimeOrigin, ()> {
        Ok(pallet_xcm::Origin::Xcm(a.clone()).into())
    }
}

impl pallet_uniques::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = u32;
    type ItemId = u32;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CollectionDeposit = ConstU128<1_000>;
    type ItemDeposit = ConstU128<1_000>;
    type MetadataDepositBase = ConstU128<1_000>;
    type AttributeDepositBase = ConstU128<1_000>;
    type DepositPerByte = ConstU128<1>;
    type StringLimit = ConstU32<64>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<128>;
    type Locker = ();
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
    pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
}

parameter_types! {
    pub const KsmLocation: MultiLocation = MultiLocation::parent();
    pub const TokenLocation: MultiLocation = Here.into_location();
    pub const RelayNetwork: NetworkId = ByGenesis([0; 32]);
    pub UniversalLocation: InteriorMultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
}

pub type XcmOriginToCallOrigin = (
    SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
    ParentAsSuperuser<RuntimeOrigin>,
    SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
    XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
    pub const XcmInstructionWeight: Weight = Weight::from_parts(1_000, 1_000);
    pub TokensPerSecondPerMegabyte: (AssetId, u128, u128) = (Concrete(Parent.into()), 1_000_000_000_000, 1024 * 1024);
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsIntoHolding: u32 = 64;
    pub ForeignPrefix: MultiLocation = (Parent,).into();
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
    pub TrustedLockPairs: (MultiLocation, MultiAssetFilter) =
    (Parent.into(), Wild(AllOf { id: Concrete(Parent.into()), fun: WildFungible }));
}

pub fn estimate_message_fee(number_of_instructions: u64) -> u128 {
    let weight = estimate_weight(number_of_instructions);

    estimate_fee_for_weight(weight)
}

pub fn estimate_weight(number_of_instructions: u64) -> Weight {
    XcmInstructionWeight::get().saturating_mul(number_of_instructions)
}

pub fn estimate_fee_for_weight(weight: Weight) -> u128 {
    let (_, units_per_second, units_per_mb) = TokensPerSecondPerMegabyte::get();

    units_per_second * (weight.ref_time() as u128) / (WEIGHT_REF_TIME_PER_SECOND as u128)
        + units_per_mb * (weight.proof_size() as u128) / (WEIGHT_PROOF_SIZE_PER_MB as u128)
}

pub type LocalBalancesTransactor =
    XcmCurrencyAdapter<Balances, IsConcrete<TokenLocation>, SovereignAccountOf, AccountId, ()>;

pub struct FromMultiLocationToAsset<MultiLocation, AssetId>(PhantomData<(MultiLocation, AssetId)>);
impl Convert<MultiLocation, AssetIdForAssets>
    for FromMultiLocationToAsset<MultiLocation, AssetIdForAssets>
{
    fn convert(value: MultiLocation) -> Result<AssetIdForAssets, MultiLocation> {
        match value {
            MultiLocation {
                parents: 1,
                interior: Here,
            } => Ok(0 as AssetIdForAssets),
            MultiLocation {
                parents: 1,
                interior: X1(Parachain(para_id)),
            } => Ok(para_id as AssetIdForAssets),
            _ => Err(value),
        }
    }
}

pub type ForeignAssetsTransactor = FungiblesAdapter<
    Assets,
    ConvertedConcreteId<
        AssetIdForAssets,
        Balance,
        FromMultiLocationToAsset<MultiLocation, AssetIdForAssets>,
        JustTry,
    >,
    SovereignAccountOf,
    AccountId,
    NoChecking,
    CheckingAccount,
>;

pub type ForeignUniquesTransactor = NonFungiblesAdapter<
    ForeignUniques,
    ConvertedConcreteId<u32, u32, AsPrefixedGeneralIndex<KsmLocation, u32, JustTry>, JustTry>,
    SovereignAccountOf,
    AccountId,
    NoChecking,
    (),
>;

/// Means for transacting assets on this chain
pub type AssetTransactors = (
    LocalBalancesTransactor,
    ForeignAssetsTransactor,
    ForeignUniquesTransactor,
);

pub struct ParentRelay;
impl Contains<MultiLocation> for ParentRelay {
    fn contains(location: &MultiLocation) -> bool {
        location.contains_parents_only(1)
    }
}

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;
pub type Barrier = WithComputedOrigin<
    (
        AllowNoteUnlockables,
        AllowExplicitUnpaidExecutionFrom<ParentRelay>,
        AllowTopLevelPaidExecutionFrom<Everything>,
    ),
    UniversalLocation,
    ConstU32<1>,
>;

parameter_types! {
    pub NftCollectionOne: MultiAssetFilter
        = Wild(AllOf { fun: WildNonFungible, id: Concrete((Parent, GeneralIndex(1)).into()) });
    pub NftCollectionOneForRelay: (MultiAssetFilter, MultiLocation)
        = (NftCollectionOne::get(), Parent.into());
    pub RelayNativeAsset: MultiAssetFilter = Wild(AllOf { fun: WildFungible, id: Concrete((Parent, Here).into()) });
    pub RelayNativeAssetForRelay: (MultiAssetFilter, MultiLocation) = (RelayNativeAsset::get(), Parent.into());
}
pub type TrustedTeleporters = (
    xcm_builder::Case<NftCollectionOneForRelay>,
    xcm_builder::Case<RelayNativeAssetForRelay>,
);
pub type TrustedReserves = EverythingBut<xcm_builder::Case<NftCollectionOneForRelay>>;

thread_local! {
    pub static EXCHANGE_ASSETS: RefCell<HoldingAssets> = RefCell::new(HoldingAssets::new());
}
pub fn set_exchange_assets(assets: impl Into<MultiAssets>) {
    EXCHANGE_ASSETS.with(|a| a.replace(assets.into().into()));
}
pub fn exchange_assets() -> MultiAssets {
    EXCHANGE_ASSETS.with(|a| a.borrow().clone().into())
}
/// Simple implementation of AssetExchange.
/// If maximal is true we take all assets in the exchange
/// for the assets we want to give.
/// If maximal is false, we take exactly what we want in return for all assets in give.
pub struct TestAssetExchange;
impl AssetExchange for TestAssetExchange {
    fn exchange_asset(
        _origin: Option<&MultiLocation>,
        give: HoldingAssets,
        want: &MultiAssets,
        maximal: bool,
    ) -> Result<HoldingAssets, HoldingAssets> {
        let mut have = EXCHANGE_ASSETS.with(|l| l.borrow().clone());
        ensure!(have.contains_assets(want), give);
        let get = if maximal {
            std::mem::replace(&mut have, HoldingAssets::new())
        } else {
            have.saturating_take(want.clone().into())
        };
        have.subsume_assets(give);
        EXCHANGE_ASSETS.with(|l| l.replace(have));
        Ok(get)
    }
}

pub struct XcmConfig;
impl Config for XcmConfig {
    type RuntimeCall = RuntimeCall;
    type XcmSender = XcmRouter;
    type AssetTransactor = AssetTransactors;
    type OriginConverter = XcmOriginToCallOrigin;
    type IsReserve = (NativeAsset, TrustedReserves);
    type IsTeleporter = TrustedTeleporters;
    type UniversalLocation = UniversalLocation;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<XcmInstructionWeight, RuntimeCall, MaxInstructions>;
    type Trader = FixedRateOfFungible<TokensPerSecondPerMegabyte, ()>;
    type ResponseHandler = ();
    type AssetTrap = PolkadotXcm;
    type AssetLocker = PolkadotXcm;
    type AssetExchanger = TestAssetExchange;
    type AssetClaims = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type PalletInstancesInfo = AllPalletsWithSystem;
    type FeeManager = ();
    type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
    type MessageExporter = ();
    type UniversalAliases = Nothing;
    type CallDispatcher = RuntimeCall;
    type SafeCallFilter = Everything;
}

impl mock_msg_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
    pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

pub struct TrustedLockerCase<T>(PhantomData<T>);
impl<T: Get<(MultiLocation, MultiAssetFilter)>> ContainsPair<MultiLocation, MultiAsset>
    for TrustedLockerCase<T>
{
    fn contains(origin: &MultiLocation, asset: &MultiAsset) -> bool {
        let (o, a) = T::get();
        a.matches(asset) && &o == origin
    }
}

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<XcmInstructionWeight, RuntimeCall, MaxInstructions>;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = IsConcrete<TokenLocation>;
    type TrustedLockers = TrustedLockerCase<TrustedLockPairs>;
    type SovereignAccountOf = SovereignAccountOf;
    type MaxLockers = ConstU32<8>;
    type WeightInfo = pallet_xcm::TestWeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    type ReachableDest = ReachableDest;
    type AdminOrigin = EnsureRoot<AccountId>;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = sp_core::ConstU64<1>;
	type WeightInfo = ();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = sp_core::ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
	pub const DepositPerItem: Balance = 32;
	pub const DepositPerByte: Balance = 1;
    pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;

	type CallFilter = frame_support::traits::Nothing;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
    type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = ();
    type Schedule = Schedule;
    type CallStack = [pallet_contracts::Frame<Self>; 14];
    type DepositPerByte = DepositPerByte;
    type DefaultDepositLimit = ConstU128<{ u128::MAX }>;
    type DepositPerItem = DepositPerItem;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 32 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
    type UnsafeUnstableInterface = sp_core::ConstBool<false>;
    type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment,
        MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
        Assets: pallet_assets,
        ForeignUniques: pallet_uniques,
        Contracts: pallet_contracts,
    }
);
