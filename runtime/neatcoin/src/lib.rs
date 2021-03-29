// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Neatcoin.
//
// Copyright (c) 2021 Wei Tang.
//
// Neatcoin is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Neatcoin is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Neatcoin. If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

#[path = "../../common/primitives/mod.rs"]
mod primitives;
#[path = "../../common/configs.rs"]
mod configs;

use pallet_transaction_payment::CurrencyAdapter;
use crate::primitives::{
	SlowAdjustingFeeUpdate, CurrencyToVote, impls::DealWithFees,
	BlockHashCount, RocksDbWeight, BlockWeights, BlockLength, OffchainSolutionWeightLimit,
	AccountId, AccountIndex, Balance, BlockNumber, Signature, Hash, Moment, Nonce,
	BABE_GENESIS_EPOCH_CONFIG,
};

use sp_std::prelude::*;
use sp_core::u32_trait::{_1, _2, _3, _4, _5};
use codec::{Encode, Decode};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys, ModuleId, ApplyExtrinsicResult,
	KeyTypeId, Percent, Permill, Perbill, curve::PiecewiseLinear,
	transaction_validity::{TransactionValidity, TransactionSource, TransactionPriority},
	traits::{
		BlakeTwo256, Block as BlockT, OpaqueKeys, ConvertInto, AccountIdLookup,
		Extrinsic as ExtrinsicT, SaturatedConversion, Verify,
	},
};
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::RuntimeString;
use sp_version::RuntimeVersion;
use pallet_grandpa::{AuthorityId as GrandpaId, fg_primitives};
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use sp_core::OpaqueMetadata;
use sp_staking::SessionIndex;
use frame_support::{
	parameter_types, construct_runtime, RuntimeDebug,
	traits::{KeyOwnerProofSystem, Randomness, LockIdentifier, Filter},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureOneOf};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};
use pallet_session::historical as session_historical;
use static_assertions::const_assert;

#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;

use crate::VERSION;
use crate::primitives::constants::{time::*, currency::*, fee::*};
use frame_support::traits::InstanceFilter;

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Polkadot version identifier;
/// Runtime version (Polkadot).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("neatcoin"),
	impl_name: create_runtime_str!("neatcoin"),
	authoring_version: 0,
	spec_version: 0,
	impl_version: 0,
	apis: configs::API_VERSIONS,
	transaction_version: 0,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(call: &Call) -> bool {
		match call {
			// These modules are all allowed to be called by transactions:
			Call::Democracy(_) | Call::Council(_) | Call::TechnicalCommittee(_) |
			Call::TechnicalMembership(_) | Call::Treasury(_) | Call::ElectionsPhragmen(_) |
			Call::System(_) | Call::Scheduler(_) | Call::Indices(_) |
			Call::Babe(_) | Call::Timestamp(_) | Call::Balances(_) |
			Call::Authorship(_) | Call::Staking(_) | Call::Offences(_) |
			Call::Session(_) | Call::Grandpa(_) | Call::ImOnline(_) |
			Call::AuthorityDiscovery(_) |
			Call::Utility(_) | Call::Vesting(_) |
			Call::Identity(_) | Call::Proxy(_) | Call::Multisig(_) |
			Call::Bounties(_) | Call::Tips(_) | Call::ElectionProviderMultiPhase(_)
			=> true,
		}
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub grandpa: Grandpa,
		pub babe: Babe,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = crate::primitives::opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 31,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 1,

		// Must be before session.
		Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 2,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 32,

		// Consensus support.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 6,
		Staking: pallet_staking::{Pallet, Call, Storage, Config<T>, Event<T>} = 7,
		Offences: pallet_offences::{Pallet, Call, Storage, Event} = 8,
		Historical: session_historical::{Pallet} = 33,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 9,
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 11,
		ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 12,
		AuthorityDiscovery: pallet_authority_discovery::{Pallet, Call, Config} = 13,

		// Governance stuff.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config, Event<T>} = 14,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 15,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 16,
		ElectionsPhragmen: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 17,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 18,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 19,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 36,

		// Vesting.
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 25,
		Utility: pallet_utility::{Pallet, Call, Event} = 26,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 28,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 29,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 30,
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 34,
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 35,
	}
}
