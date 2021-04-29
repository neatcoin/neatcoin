// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Neatcoin.
//
// Copyright (c) 2020 Wei Tang.
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

//! Variable storage pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_std::vec::Vec;
use sp_core::RuntimeDebug;
use frame_support::{
	decl_module, decl_storage, decl_event,
};
use frame_system::ensure_root;

pub trait Config: frame_system::Config {
	/// The overarching event type.
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

/// Variable value.
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Value {
	U32(u32),
	U64(u64),
	U128(u128),
	Bool(bool),
}

decl_storage! {
	trait Store for Module<T: Config> as Eras {
		/// Storage values.
		pub Values: map hasher(blake2_128_concat) Vec<u8> => Option<Value>;
	}
}

decl_event! {
	pub enum Event {
		/// Value set.
		ValueSet(Vec<u8>, Value),
		/// Value reset.
		ValueReset(Vec<u8>),
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 0]
		fn set_value(origin, key: Vec<u8>, value: Value) {
			ensure_root(origin)?;

			Values::insert(key.clone(), value.clone());
			Self::deposit_event(Event::ValueSet(key, value));
		}

		#[weight = 0]
		fn reset_value(origin, key: Vec<u8>) {
			ensure_root(origin)?;

			Values::remove(key.clone());
			Self::deposit_event(Event::ValueReset(key));
		}
	}
}
