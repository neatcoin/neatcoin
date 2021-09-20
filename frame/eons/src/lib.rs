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

//! Era information recording.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
pub struct Eon<H> {
	/// Genesis block hash of the eon.
	pub genesis_block_hash: H,
	/// Final block hash.
	pub final_block_hash: H,
	/// Final state root.
	pub final_state_root: H,
}

pub trait Config: frame_system::Config {}

decl_storage! {
	trait Store for Module<T: Config> as Eons {
		/// Past eons.
		pub PastEons get(fn past_eons) config(past_eons): Vec<Eon<T::Hash>>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin { }
}
