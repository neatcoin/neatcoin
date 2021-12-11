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

use crate::{
	constants::{currency::UNITS, time::DAYS},
	types::Balance,
	AccountId, Balances, BlockNumber, Event, Registry, Runtime, Treasury,
};
use codec::{Decode, Encode};
use frame_support::parameter_types;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::RuntimeDebug;

#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Ownership {
	None,
	Root,
	FCFS,
	Account(AccountId),
}

impl Default for Ownership {
	fn default() -> Self {
		Self::None
	}
}

impl pallet_registry::Ownership for Ownership {
	type AccountId = AccountId;

	fn root() -> Self {
		Self::Root
	}
	fn account(account: AccountId) -> Self {
		Self::Account(account)
	}
}

impl pallet_registry::Config for Runtime {
	type Ownership = Ownership;
	type Event = Event;
	type WeightInfo = ();
}

impl pallet_zone::Config for Runtime {
	type Ownership = Ownership;
	type Registry = Registry;
	type Event = Event;
	type WeightInfo = ();
}

parameter_types! {
	pub const DefaultFee: Balance = 500 * UNITS;
	pub const Period: BlockNumber = 52 * 7 * DAYS;
	pub const CanRenewAfter: BlockNumber = 52 * 7 * DAYS;
	pub const FCFSOwnership: Ownership = Ownership::FCFS;
}

impl pallet_fcfs::Config for Runtime {
	type Ownership = Ownership;
	type FCFSOwnership = FCFSOwnership;
	type Registry = Registry;
	type Currency = Balances;
	type DefaultFee = DefaultFee;
	type Period = Period;
	type CanRenewAfter = CanRenewAfter;
	type ChargeFee = Treasury;
	type WeightInfo = ();
	type Event = Event;
}
