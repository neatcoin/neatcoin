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
use frame_support::{
	dispatch::DispatchResult, decl_module, decl_storage, decl_event, decl_error, ensure
};
use frame_system::{ensure_signed, ensure_root};
use np_domain::{Name, NameHash, NameValue};

pub trait Ownership: Encode + Decode + EncodeLike + Default + Eq + Debug + Clone {
	type AccountId;

	/// Explictly owned by root.
	fn root() -> Self;
	/// Owned by a specific account.
	fn account(account: Self::AccountId) -> Self;
}

pub trait Config: pallet_balances::Config {
	type Ownership: Ownership<AccountId=Self::AccountId>;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

pub trait Registry {
	type Ownership: Ownership;

	fn set_ownership_as(
		as_ownership: &Self::Ownership,
		name: Name,
		ownership: Option<Self::Ownership>,
	) -> DispatchResult;
	fn can_set_ownership(
		as_ownership: &Self::Ownership,
		name: &Name,
	) -> bool;
	fn is_owned(
		as_ownership: &Self::Ownership,
		name: &Name,
	) -> bool;
}

decl_storage! {
	trait Store for Module<T: Config> as Registry {
		Ownerships: map hasher(identity) NameHash => NameValue<T::Ownership>;
	}
}

decl_event! {
	pub enum Event<T> where Ownership = <T as crate::Config>::Ownership {
		OwnershipSet(Name, Option<Ownership>),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		OwnershipMismatch,
		AttemptToSetRootOwnership,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		fn set_ownership(origin, name: Name, ownership: Option<T::Ownership>) {
			let owner = ensure_signed(origin)?;

			<Self as Registry>::set_ownership_as(&Ownership::account(owner), name, ownership)?;
		}

		#[weight = 0]
		fn force_set_ownership(origin, name: Name, ownership: Option<T::Ownership>) {
			ensure_root(origin)?;

			<Self as Registry>::set_ownership_as(&Ownership::root(), name, ownership)?;
		}
	}
}

impl<T: Config> Registry for Module<T> {
	type Ownership = T::Ownership;

	fn set_ownership_as(as_ownership: &T::Ownership, name: Name, ownership: Option<T::Ownership>) -> DispatchResult {
		ensure!(!name.is_root(), Error::<T>::AttemptToSetRootOwnership);

		if as_ownership != &T::Ownership::root() {
			let parent = name.parent().ok_or(Error::<T>::OwnershipMismatch)?;
			let parent_ownership = Ownerships::<T>::get(&parent.hash()).into_value();

			ensure!(parent_ownership.as_ref() == Some(as_ownership), Error::<T>::OwnershipMismatch);
		}

		if let Some(ownership) = ownership.clone() {
			Ownerships::<T>::insert(name.hash(), NameValue::some(name.clone(), ownership.clone()));
		} else {
			Ownerships::<T>::remove(name.hash());
		}

		Self::deposit_event(Event::<T>::OwnershipSet(name, ownership));

		Ok(())
	}

	fn can_set_ownership(as_ownership: &T::Ownership, name: &Name) -> bool {
		if name.is_root() {
			return false
		}

		if as_ownership == &T::Ownership::root() {
			return true
		}

		let parent = match name.parent() {
			Some(parent) => parent,
			None => return false,
		};
		let parent_ownership = Ownerships::<T>::get(&parent.hash()).into_value();

		parent_ownership.as_ref() == Some(as_ownership)
	}

	fn is_owned(as_ownership: &T::Ownership, name: &Name) -> bool {
		let ownership = Ownerships::<T>::get(&name.hash()).into_value();

		ownership.as_ref() == Some(as_ownership)
	}
}
