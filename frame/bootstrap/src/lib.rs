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

use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

pub trait Config: frame_system::Config {}

decl_storage! {
	trait Store for Module<T: Config> as Bootstrap {
		/// Bootstrapping endowed accounts.
		pub Endoweds get(fn endoweds): Vec<T::AccountId>;
	}
	add_extra_genesis {
		config(endoweds): Vec<T::AccountId>;
		build(|config: &GenesisConfig<T>| {
			Endoweds::<T>::set(config.endoweds.clone());
			for account in &config.endoweds {
				frame_system::Pallet::<T>::inc_providers(account);
			}
		})
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin { }
}
