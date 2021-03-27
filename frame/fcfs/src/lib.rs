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

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use codec::{Encode, EncodeLike, Decode};
use sp_std::{prelude::*, fmt::Debug, cmp};
use sp_runtime::RuntimeDebug;
use frame_support::{
	dispatch::DispatchResult, decl_module, decl_storage, decl_event, decl_error, ensure,
	traits::{Get, Currency, OnUnbalanced, WithdrawReasons, ExistenceRequirement},
};
use frame_system::{ensure_signed, ensure_root};
use primitive_types::H160;
use np_domain::{Name, NameHash, NameValue};
use pallet_registry::{Registry, Ownership};

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub trait Config: frame_system::Config {
	type Ownership: Ownership<AccountId=Self::AccountId>;
	type FCFSOwnership: Get<Self::Ownership>;
	type Registry: Registry<Ownership=Self::Ownership>;
	type Currency: Currency<Self::AccountId>;
	type Fee: Get<BalanceOf<Self>>;
	type Period: Get<Self::BlockNumber>;
	type ChargeFee: OnUnbalanced<NegativeImbalanceOf<Self>>;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Eq, PartialEq, Clone, Encode, Decode, Debug)]
pub struct RenewalInfo<T: Config> {
	pub expire_at: T::BlockNumber,
	pub fee: BalanceOf<T>,
}

decl_storage! {
	trait Store for Module<T: Config> as FCFS {
		Renewals: map hasher(identity) NameHash => NameValue<RenewalInfo<T>>;
	}
}

decl_event! {
	pub enum Event<T> where BlockNumber = <T as frame_system::Config>::BlockNumber {
		Registered(Name, BlockNumber),
		Renewed(Name, BlockNumber),
		Expired(Name),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		OwnershipMismatch,
		NotAllowedRegister,
		AlreadyRegistered,
		RenewalInfoMissing,
		NotExpired,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		fn register(origin, name: Name) {
			let sender = ensure_signed(origin)?;

			ensure!(T::Registry::owner(&name).is_none(), Error::<T>::AlreadyRegistered);
			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let fee = T::Fee::get();
			let period = T::Period::get();
			let expire_at = frame_system::Module::<T>::block_number() + period;
			let info = RenewalInfo::<T> { fee, expire_at };

			let imbalance = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			)?;

			T::Registry::set_ownership_unchecked(name.clone(), Some(T::Ownership::account(sender)));
			Renewals::<T>::insert(name.hash(), NameValue::some(name.clone(), info));

			T::ChargeFee::on_unbalanced(imbalance);
			Self::deposit_event(Event::<T>::Registered(name, expire_at));
		}

		#[weight = 0]
		fn renew(origin, name: Name) {
			let sender = ensure_signed(origin)?;

			ensure!(T::Registry::owner(&name) == Some(T::Ownership::account(sender.clone())), Error::<T>::OwnershipMismatch);
			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let mut info = Renewals::<T>::get(&name.hash()).into_value().ok_or(Error::<T>::RenewalInfoMissing)?;
			let fee = cmp::min(info.fee, T::Fee::get());

			let imbalance = T::Currency::withdraw(
				&sender,
				fee,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			)?;

			info.expire_at += T::Period::get();
			let expire_at = info.expire_at;

			Renewals::<T>::insert(name.hash(), NameValue::some(name.clone(), info));

			T::ChargeFee::on_unbalanced(imbalance);
			Self::deposit_event(Event::<T>::Renewed(name, expire_at));
		}

		#[weight = 0]
		fn release_expired(origin, name: Name) {
			ensure_signed(origin)?;

			T::Registry::ensure_can_set_ownership(&T::FCFSOwnership::get(), &name)?;

			let info = Renewals::<T>::get(&name.hash()).into_value().ok_or(Error::<T>::RenewalInfoMissing)?;
			ensure!(frame_system::Module::<T>::block_number() > info.expire_at, Error::<T>::NotExpired);

			T::Registry::set_ownership_unchecked(name.clone(), None);
			Renewals::<T>::remove(&name.hash());

			Self::deposit_event(Event::<T>::Expired(name));
		}
	}
}
