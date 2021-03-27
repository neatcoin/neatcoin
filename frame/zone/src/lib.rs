// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Nomo.
//
// Copyright (c) 2019-2020 Wei Tang.
//
// Nomo is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Nomo is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Nomo. If not, see <http://www.gnu.org/licenses/>.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, EncodeLike, Decode};
use sp_std::{prelude::*, fmt::Debug};
use sp_runtime::RuntimeDebug;
use frame_support::{
	dispatch::DispatchResult, decl_module, decl_storage, decl_event, decl_error, ensure
};
use frame_system::{ensure_signed, ensure_root};
use primitive_types::H160;
use np_domain::{Name, NameHash, NameValue};

pub trait Config: frame_system::Config {
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

pub type RawIpv4 = u32;
pub type RawIpv6 = u128;

decl_storage! {
	trait Store for Module<T: Config> as Zone {
		As: map hasher(identity) NameHash => NameValue<Vec<RawIpv4>>;
		AAAAs: map hasher(identity) NameHash => NameValue<Vec<RawIpv6>>;
		NSs: map hasher(identity) NameHash => NameValue<Vec<Name>>;
		CNAMEs: map hasher(identity) NameHash => NameValue<Name>;
		MXs: map hasher(identity) NameHash => NameValue<(u16, Name)>;
		ExternICANNs: map hasher(identity) NameHash => NameValue<()>;
		ExternOpenNICs: map hasher(identity) NameHash => NameValue<()>;
		ExternHandshake: map hasher(identity) NameHash => NameValue<()>;
	}
}

decl_event! {
	pub enum Event {

	}
}

decl_error! {
	pub enum Error for Module<T: Config> {

	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
	}
}
