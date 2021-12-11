// This file is part of Neatcoin.
//
// Copyright (c) 2021 Wei Tang.
//
// Neatcoin is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by
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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use np_domain::{Label, Name};
use sp_runtime::traits::{UniqueSaturatedFrom, One};

benchmarks! {
	register {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let fcfs_name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
		]);
		T::Currency::deposit_creating(&caller, UniqueSaturatedFrom::unique_saturated_from(2000_000_000_000_000u128));
		T::Registry::set_ownership_unchecked(fcfs_name, Some(T::FCFSOwnership::get()));
	}: _(RawOrigin::Signed(caller.clone()), name.clone())
	verify {
		assert_eq!(T::Registry::owner(&name), Some(T::Ownership::account(caller)));
	}

	renew {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let fcfs_name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
		]);
		T::Currency::deposit_creating(&caller, UniqueSaturatedFrom::unique_saturated_from(2000_000_000_000_000u128));
		T::Registry::set_ownership_unchecked(fcfs_name, Some(T::FCFSOwnership::get()));
		crate::Call::<T>::register { name: name.clone() }.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into()).unwrap();
		frame_system::Pallet::<T>::set_block_number(From::from(26u32 * 7 * 14400));
		let current_expire = Renewals::<T>::get(&name.hash()).into_value().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), name.clone())
	verify {
		let new_expire = Renewals::<T>::get(&name.hash()).into_value().unwrap();
		assert!(new_expire.expire_at > current_expire.expire_at);
	}

	release_expired {
		let caller: T::AccountId = whitelisted_caller();
		let name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
			Label::try_from(b"testname".to_vec()).unwrap(),
		]);
		let fcfs_name = Name(vec![
			Label::try_from(b"neatuser".to_vec()).unwrap(),
		]);
		T::Currency::deposit_creating(&caller, UniqueSaturatedFrom::unique_saturated_from(2000_000_000_000_000u128));
		T::Registry::set_ownership_unchecked(fcfs_name, Some(T::FCFSOwnership::get()));
		crate::Call::<T>::register { name: name.clone() }.dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into()).unwrap();
		assert_eq!(T::Registry::owner(&name), Some(T::Ownership::account(caller.clone())));
		frame_system::Pallet::<T>::set_block_number(From::from(53u32 * 7 * 14400));
	}: _(RawOrigin::Signed(caller.clone()), name.clone())
	verify {
		assert_eq!(T::Registry::owner(&name), None);
	}

	set_fee {
		let fee = One::one();
	}: _(RawOrigin::Root, fee)
}
