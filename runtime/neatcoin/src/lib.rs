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

mod config;

#[path = "../../common/api.rs"]
mod api;
#[path = "../../common/constants.rs"]
mod constants;
#[path = "../../common/impls.rs"]
mod impls;
#[path = "../../common/types.rs"]
mod types;

use sp_std::prelude::*;
use sp_runtime::{create_runtime_str, impl_opaque_keys};
use sp_version::RuntimeVersion;
#[cfg(any(feature = "std", test))]
use sp_version::NativeVersion;
use frame_support::construct_runtime;
use pallet_session::historical as session_historical;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
#[cfg(feature = "std")]
pub use crate::api::{api::dispatch, RuntimeApi};
pub use crate::types::{
	opaque, BlockNumber, Moment, Signature, AccountPublic, AccountId, AccountIndex,
	Hash, Nonce, Address, Header, Block, SignedBlock, BlockId, SignedExtra,
	UncheckedExtrinsic, CheckedExtrinsic, SignedPayload,
};

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub const SS58_PREFIX: u8 = 48;

// Polkadot version identifier;
/// Runtime version (Polkadot).
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("neatcoin"),
	impl_name: create_runtime_str!("neatcoin"),
	authoring_version: 0,
	spec_version: 0,
	impl_version: 0,
	apis: crate::api::PRUNTIME_API_VERSIONS,
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

impl_opaque_keys! {
	pub struct SessionKeys {
		pub grandpa: Grandpa,
		pub babe: Babe,
		pub im_online: ImOnline,
		pub authority_discovery: AuthorityDiscovery,
	}
}

pub type CouncilCollectiveInstance = pallet_collective::Instance1;
pub type TechnicalCollectiveInstance = pallet_collective::Instance2;
pub type TechnicalMembershipInstance = pallet_membership::Instance1;

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = crate::types::opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Basic.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 1,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,

		// Bootstrap
		Bootstrap: pallet_bootstrap::{Pallet, Storage, Config<T>} = 100,

		// Must be before session.
		Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 3,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 4,
		Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,

		// Consensus support.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 8,
		Staking: pallet_staking::{Pallet, Call, Storage, Config<T>, Event<T>} = 9,
		Offences: pallet_offences::{Pallet, Call, Storage, Event} = 10,
		Historical: session_historical::{Pallet} = 11,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 12,
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 13,
		ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 14,
		AuthorityDiscovery: pallet_authority_discovery::{Pallet, Call, Config} = 15,

		// Governance stuff.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config, Event<T>} = 16,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 17,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 18,
		ElectionsPhragmen: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 19,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 21,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 22,

		// Utilities.
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 23,
		Utility: pallet_utility::{Pallet, Call, Event} = 24,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 25,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 26,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 27,
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 28,
		Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 29,
		Eons: pallet_eons::{Pallet, Call, Storage, Config<T>} = 30,
		Variables: pallet_variables::{Pallet, Call, Storage, Event} = 31,
		AtomicSwap: pallet_atomic_swap::{Pallet, Call, Storage, Event<T>} = 32,

		// Contracts
		Contracts: pallet_contracts::{Pallet, Call, Config<T>, Storage, Event<T>} = 33,

		// Nomo
		Registry: pallet_registry::{Pallet, Call, Storage, Event<T>} = 34,
		Zone: pallet_zone::{Pallet, Call, Storage, Event} = 35,
		FCFS: pallet_fcfs::{Pallet, Call, Storage, Event<T>} = 36,
	}
}

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	crate::types::Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;
