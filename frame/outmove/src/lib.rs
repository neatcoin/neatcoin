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

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure,
};
use sp_std::{borrow::ToOwned, prelude::*};
use frame_system::ensure_signed;
use sp_runtime::AccountId32;
use omv::{
	primitives::{account_address::AccountAddress, language_storage::{ModuleId, StructTag, TypeTag}, identifier::Identifier, vm_status::StatusCode, gas_schedule::{CostTable, GasCost, GasUnits, GasAlgebra}},
	runtime::data_cache::RemoteCache,
	runtime::logging::NoContextLog,
	types::gas_schedule::{self, NativeCostIndex as N, CostStrategy},
	core::file_format::{
        Bytecode, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex,
        FunctionHandleIndex, FunctionInstantiationIndex, StructDefInstantiationIndex,
        StructDefinitionIndex,
    },
    core::file_format_common::instruction_key,
	core::errors::{VMResult, PartialVMResult, PartialVMError},
};

pub trait Config: frame_system::Config<AccountId=AccountId32> {
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
}

pub type RawIdentifier = Vec<u8>;
pub type RawStructTag = Vec<u8>;
pub type RawArgument = Vec<u8>;
pub type RawTypeTag = Vec<u8>;

decl_storage! {
	trait Store for Module<T: Config> as Outmove {
		Modules: double_map hasher(blake2_128_concat) AccountId32, hasher(blake2_128_concat) RawIdentifier => Option<Vec<u8>>;
		Resources: double_map hasher(blake2_128_concat) AccountId32, hasher(blake2_128_concat) RawStructTag => Option<Vec<u8>>;
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		InvalidModuleIdentifier,
		InvalidTransactionArgument,
		GasBudgetTooHigh,
		RunScriptFailed,
	}
}

decl_event!(
	pub enum Event {
		Dummy,
	}
);

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		fn publish(origin, identifier_raw: Vec<u8>, module_data: Vec<u8>) {
			let account_id = ensure_signed(origin)?;
			let identifier = Identifier::from_utf8(identifier_raw.clone()).map_err(|_| Error::<T>::InvalidModuleIdentifier)?;
			ensure!(Identifier::is_valid(identifier.as_str()), Error::<T>::InvalidModuleIdentifier);

			// TODO: reject backward-incompatible publishing.
			
			Modules::insert(account_id, identifier_raw, module_data);
		}

		#[weight = 0]
		fn run(origin, script: Vec<u8>, type_args: Vec<RawTypeTag>, raw_args: Vec<RawArgument>, gas_budget: Option<u64>) {
			let account_id = ensure_signed(origin)?;

			let mut typs = Vec::<TypeTag>::new();
			for typ in type_args {
				typs.push(omv::serialize::from_bytes(&typ).map_err(|_| Error::<T>::InvalidTransactionArgument)?);
			}

			let mut signer_addresses = Vec::new();
			signer_addresses.push(AccountAddress::new(account_id.into()));

			let mut vm = omv::runtime::move_vm::MoveVM::new();
			let table = genesis_gas_schedule();
			let mut cost_strategy = Self::get_cost_strategy(&table, gas_budget)?;
			let log_context = NoContextLog::new();

			let mut session = vm.new_session(&Self(core::marker::PhantomData));
			let res = session.execute_script(
				script,
				typs,
				raw_args,
				signer_addresses,
				&mut cost_strategy,
				&log_context,
			);

			ensure!(res.is_ok(), Error::<T>::RunScriptFailed);

			let (changeset, _) = session.finish().map_err(|_| Error::<T>::RunScriptFailed)?; // This is currently okay as we haven't committed any events.

			for (addr, account) in changeset.accounts {
				for (struct_tag, blob_opt) in account.resources {
					let address = AccountId32::new(addr.to_u8());
					let tag = &omv::serialize::to_bytes(&struct_tag).unwrap(); // TODO: handle this error.

					match blob_opt {
						Some(blob) => Resources::insert(address, tag, blob),
						None => Resources::remove(address, tag),
					}
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	fn get_cost_strategy(table: &CostTable, gas_budget: Option<u64>) -> Result<CostStrategy, Error<T>> {
		let cost_strategy = if let Some(gas_budget) = gas_budget {
			let max_gas_budget = u64::MAX
				.checked_div(table.gas_constants.gas_unit_scaling_factor)
				.unwrap();
			if gas_budget >= max_gas_budget {
				return Err(Error::<T>::GasBudgetTooHigh)
			}
			CostStrategy::transaction(table, GasUnits::new(gas_budget))
		} else {
			// no budget specified. use CostStrategy::system, which disables gas metering
			CostStrategy::system(table, GasUnits::new(0))
		};
		Ok(cost_strategy)
	}
}

impl<T: Config> RemoteCache for Module<T> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
		let address = AccountId32::new(module_id.address().to_u8());
		let identifier = module_id.name().to_owned().into_bytes();

		Ok(Modules::get(&address, &identifier))
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
		let address = AccountId32::new(address.to_u8());
		let tag = &omv::serialize::to_bytes(struct_tag).map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))?;

		Ok(Resources::get(&address, &tag))
    }
}

fn genesis_gas_schedule() -> CostTable {
	{
		use Bytecode::*;
		let mut instrs = vec![
			(MoveTo(StructDefinitionIndex::new(0)), GasCost::new(13, 1)),
			(
				MoveToGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(27, 1),
			),
			(
				MoveFrom(StructDefinitionIndex::new(0)),
				GasCost::new(459, 1),
			),
			(
				MoveFromGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(13, 1),
			),
			(BrTrue(0), GasCost::new(1, 1)),
			(WriteRef, GasCost::new(1, 1)),
			(Mul, GasCost::new(1, 1)),
			(MoveLoc(0), GasCost::new(1, 1)),
			(And, GasCost::new(1, 1)),
			(Pop, GasCost::new(1, 1)),
			(BitAnd, GasCost::new(2, 1)),
			(ReadRef, GasCost::new(1, 1)),
			(Sub, GasCost::new(1, 1)),
			(MutBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
			(
				MutBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
				GasCost::new(1, 1),
			),
			(ImmBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
			(
				ImmBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
				GasCost::new(1, 1),
			),
			(Add, GasCost::new(1, 1)),
			(CopyLoc(0), GasCost::new(1, 1)),
			(StLoc(0), GasCost::new(1, 1)),
			(Ret, GasCost::new(638, 1)),
			(Lt, GasCost::new(1, 1)),
			(LdU8(0), GasCost::new(1, 1)),
			(LdU64(0), GasCost::new(1, 1)),
			(LdU128(0), GasCost::new(1, 1)),
			(CastU8, GasCost::new(2, 1)),
			(CastU64, GasCost::new(1, 1)),
			(CastU128, GasCost::new(1, 1)),
			(Abort, GasCost::new(1, 1)),
			(MutBorrowLoc(0), GasCost::new(2, 1)),
			(ImmBorrowLoc(0), GasCost::new(1, 1)),
			(LdConst(ConstantPoolIndex::new(0)), GasCost::new(1, 1)),
			(Ge, GasCost::new(1, 1)),
			(Xor, GasCost::new(1, 1)),
			(Shl, GasCost::new(2, 1)),
			(Shr, GasCost::new(1, 1)),
			(Neq, GasCost::new(1, 1)),
			(Not, GasCost::new(1, 1)),
			(Call(FunctionHandleIndex::new(0)), GasCost::new(1132, 1)),
			(
				CallGeneric(FunctionInstantiationIndex::new(0)),
				GasCost::new(582, 1),
			),
			(Le, GasCost::new(2, 1)),
			(Branch(0), GasCost::new(1, 1)),
			(Unpack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
			(
				UnpackGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(2, 1),
			),
			(Or, GasCost::new(2, 1)),
			(LdFalse, GasCost::new(1, 1)),
			(LdTrue, GasCost::new(1, 1)),
			(Mod, GasCost::new(1, 1)),
			(BrFalse(0), GasCost::new(1, 1)),
			(Exists(StructDefinitionIndex::new(0)), GasCost::new(41, 1)),
			(
				ExistsGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(34, 1),
			),
			(BitOr, GasCost::new(2, 1)),
			(FreezeRef, GasCost::new(1, 1)),
			(
				MutBorrowGlobal(StructDefinitionIndex::new(0)),
				GasCost::new(21, 1),
			),
			(
				MutBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(15, 1),
			),
			(
				ImmBorrowGlobal(StructDefinitionIndex::new(0)),
				GasCost::new(23, 1),
			),
			(
				ImmBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(14, 1),
			),
			(Div, GasCost::new(3, 1)),
			(Eq, GasCost::new(1, 1)),
			(Gt, GasCost::new(1, 1)),
			(Pack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
			(
				PackGeneric(StructDefInstantiationIndex::new(0)),
				GasCost::new(2, 1),
			),
			(Nop, GasCost::new(1, 1)),
		];
		// Note that the DiemVM is expecting the table sorted by instruction order.
		instrs.sort_by_key(|cost| instruction_key(&cost.0));
	
		let mut native_table = vec![
			(N::SHA2_256, GasCost::new(21, 1)),
			(N::SHA3_256, GasCost::new(64, 1)),
			(N::ED25519_VERIFY, GasCost::new(61, 1)),
			(N::ED25519_THRESHOLD_VERIFY, GasCost::new(3351, 1)),
			(N::BCS_TO_BYTES, GasCost::new(181, 1)),
			(N::LENGTH, GasCost::new(98, 1)),
			(N::EMPTY, GasCost::new(84, 1)),
			(N::BORROW, GasCost::new(1334, 1)),
			(N::BORROW_MUT, GasCost::new(1902, 1)),
			(N::PUSH_BACK, GasCost::new(53, 1)),
			(N::POP_BACK, GasCost::new(227, 1)),
			(N::DESTROY_EMPTY, GasCost::new(572, 1)),
			(N::SWAP, GasCost::new(1436, 1)),
			(N::ED25519_VALIDATE_KEY, GasCost::new(26, 1)),
			(N::SIGNER_BORROW, GasCost::new(353, 1)),
			(N::CREATE_SIGNER, GasCost::new(24, 1)),
			(N::DESTROY_SIGNER, GasCost::new(212, 1)),
			(N::EMIT_EVENT, GasCost::new(52, 1)),
		];
		native_table.sort_by_key(|cost| cost.0 as u64);
		let raw_native_table = native_table
			.into_iter()
			.map(|(_, cost)| cost)
			.collect::<Vec<_>>();
		gas_schedule::new_from_instructions(instrs, raw_native_table)
	}
}