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

use codec::{Encode, Decode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_core::RuntimeDebug;
use frame_support::parameter_types;
use crate::{
	AccountId, Runtime, Event, Registry, BlockNumber, Balances, Treasury,
	types::Balance,
	constants::{currency::UNITS, time::DAYS},
};

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

	fn root() -> Self { Self::Root }
	fn account(account: AccountId) -> Self { Self::Account(account) }
}

impl pallet_registry::Config for Runtime {
	type Ownership = Ownership;
	type Event = Event;
}

impl pallet_zone::Config for Runtime {
	type Ownership = Ownership;
	type Registry = Registry;
	type Event = Event;
}

parameter_types! {
	pub const Fee: Balance = 500 * UNITS;
	pub const Period: BlockNumber = 52 * 7 * DAYS;
	pub const FCFSOwnership: Ownership = Ownership::FCFS;
}

impl pallet_fcfs::Config for Runtime {
	type Ownership = Ownership;
	type FCFSOwnership = FCFSOwnership;
	type Registry = Registry;
	type Currency = Balances;
	type Fee = Fee;
	type Period = Period;
	type ChargeFee = Treasury;
	type Event = Event;
}
