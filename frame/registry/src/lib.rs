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

mod benchmarking;
mod default_weights;

use codec::{Decode, Encode, EncodeLike};
use frame_support::{dispatch::DispatchResult, ensure, weights::Weight};
use frame_system::ensure_root;
use np_domain::{Name, NameHash, NameValue};
use scale_info::TypeInfo;
use sp_std::{fmt::Debug, prelude::*};

pub use pallet::*;

pub trait Ownership:
	Encode + Decode + EncodeLike + Default + Eq + Debug + Clone + TypeInfo
{
	type AccountId;

	/// Explictly owned by root.
	fn root() -> Self;
	/// Owned by a specific account.
	fn account(account: Self::AccountId) -> Self;
}

pub trait Registry {
	type Ownership: Ownership;

	fn set_ownership_as(
		as_ownership: &Self::Ownership,
		name: Name,
		ownership: Option<Self::Ownership>,
	) -> DispatchResult;
	fn set_ownership_unchecked(name: Name, ownership: Option<Self::Ownership>);
	fn can_set_ownership(as_ownership: &Self::Ownership, name: &Name) -> bool;
	fn ensure_can_set_ownership(as_ownership: &Self::Ownership, name: &Name) -> DispatchResult;
	fn owner(name: &Name) -> Option<Self::Ownership>;

	fn parent_owner(name: &Name) -> Option<Self::Ownership> {
		name.parent().and_then(|parent| Self::owner(&parent))
	}
	fn effective_owner(name: &Name) -> Option<Self::Ownership> {
		let mut current = Some(name.clone());
		while let Some(check) = current {
			if let Some(ret) = Self::owner(&check) {
				return Some(ret);
			}
			current = check.parent();
		}
		None
	}
	fn is_owned(as_ownership: &Self::Ownership, name: &Name) -> bool {
		let ownership = Self::owner(name);
		ownership.as_ref() == Some(as_ownership)
	}
	fn is_parent_owned(as_ownership: &Self::Ownership, name: &Name) -> bool {
		let ownership = Self::parent_owner(name);
		ownership.as_ref() == Some(as_ownership)
	}
	fn is_effective_owned(as_ownership: &Self::Ownership, name: &Name) -> bool {
		let ownership = Self::effective_owner(name);
		ownership.as_ref() == Some(as_ownership)
	}
}

pub trait WeightInfo {
	fn force_set_ownership() -> Weight;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Ownership: Ownership<AccountId = Self::AccountId>;
		type WeightInfo: WeightInfo;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::force_set_ownership())]
		pub fn force_set_ownership(
			origin: OriginFor<T>,
			name: Name,
			ownership: Option<T::Ownership>,
		) -> DispatchResult {
			ensure_root(origin)?;

			<Self as Registry>::set_ownership_as(&Ownership::root(), name, ownership)?;

			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OwnershipSet(Name, Option<T::Ownership>),
	}

	#[pallet::error]
	pub enum Error<T> {
		OwnershipMismatch,
		AttemptToSetRootOwnership,
	}

	#[pallet::storage]
	pub(super) type Ownerships<T: Config> =
		StorageMap<_, Identity, NameHash, NameValue<T::Ownership>, ValueQuery>;
}

impl<T: Config> Registry for Pallet<T> {
	type Ownership = T::Ownership;

	fn ensure_can_set_ownership(as_ownership: &T::Ownership, name: &Name) -> DispatchResult {
		ensure!(!name.is_root(), Error::<T>::AttemptToSetRootOwnership);

		if as_ownership != &T::Ownership::root() {
			let parent = name.parent().ok_or(Error::<T>::OwnershipMismatch)?;
			let parent_ownership = Ownerships::<T>::get(&parent.hash()).into_value();

			ensure!(
				parent_ownership.as_ref() == Some(as_ownership),
				Error::<T>::OwnershipMismatch
			);
		}

		Ok(())
	}

	fn set_ownership_unchecked(name: Name, ownership: Option<T::Ownership>) {
		if let Some(ownership) = ownership.clone() {
			Ownerships::<T>::insert(
				name.hash(),
				NameValue::some(name.clone(), ownership.clone()),
			);
		} else {
			Ownerships::<T>::remove(name.hash());
		}

		Self::deposit_event(Event::<T>::OwnershipSet(name, ownership));
	}

	fn set_ownership_as(
		as_ownership: &T::Ownership,
		name: Name,
		ownership: Option<T::Ownership>,
	) -> DispatchResult {
		Self::ensure_can_set_ownership(as_ownership, &name)?;
		Self::set_ownership_unchecked(name, ownership);

		Ok(())
	}

	fn can_set_ownership(as_ownership: &T::Ownership, name: &Name) -> bool {
		if name.is_root() {
			return false;
		}

		if as_ownership == &T::Ownership::root() {
			return true;
		}

		let parent = match name.parent() {
			Some(parent) => parent,
			None => return false,
		};
		let parent_ownership = Ownerships::<T>::get(&parent.hash()).into_value();

		parent_ownership.as_ref() == Some(as_ownership)
	}

	fn owner(name: &Name) -> Option<T::Ownership> {
		Ownerships::<T>::get(&name.hash()).into_value()
	}
}
