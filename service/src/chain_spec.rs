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

use std::{marker::PhantomData, collections::HashMap};
use sp_core::crypto::{Ss58Codec, Ss58AddressFormat};
use sp_runtime::Perbill;
use sc_chain_spec::ChainType;
use np_opaque::{AccountId, Balance};

pub type NeatcoinChainSpec = sc_service::GenericChainSpec<neatcoin_runtime::GenesisConfig>;
pub type VodkaChainSpec = sc_service::GenericChainSpec<vodka_runtime::GenesisConfig>;

pub fn neatcoin_config() -> Result<NeatcoinChainSpec, String> {
	let _wasm_binary = neatcoin_runtime::WASM_BINARY.ok_or("Neatcoin development wasm not available")?;
	let boot_nodes = vec![];

	Ok(NeatcoinChainSpec::from_genesis(
		"Neatcoin",
		"neatcoin",
		ChainType::Live,
		move || Default::default(),
		boot_nodes,
		None,
		Some("neatcoin"),
		None,
		Default::default()
	))
}

pub fn genesis_allocations() -> HashMap<AccountId, Balance> {
	let raw: HashMap<String, String> = serde_json::from_slice(include_bytes!("../res/genesis.json"))
		.expect("parse genesis.json failed");
	raw.into_iter().map(|(key, value)| {
		let (address, version) = AccountId::from_ss58check_with_version(&key)
			.expect("parse address failed");
		assert_eq!(version, Ss58AddressFormat::KulupuAccount);
		let balance = u128::from_str_radix(&value, 10).expect("parse balance failed");
		(address, balance)
	}).collect()
}

pub fn vodka_genesis(
	wasm_binary: &[u8],
	genesis_keys: Vec<(AccountId, vodka_runtime::SessionKeys)>,
	sudo_key: AccountId,
) -> vodka_runtime::GenesisConfig {
	vodka_runtime::GenesisConfig {
		frame_system: vodka_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: vodka_runtime::BalancesConfig {
			balances: genesis_allocations().into_iter().collect(),
		},
		pallet_indices: vodka_runtime::IndicesConfig {
			indices: vec![],
		},
		pallet_session: vodka_runtime::SessionConfig {
			keys: genesis_keys.clone().into_iter().map(|(account, keys)| {
				(account.clone(), account, keys)
			}).collect(),
		},
		pallet_staking: vodka_runtime::StakingConfig {
			validator_count: 17,
			minimum_validator_count: 7,
			stakers: vec![],
			invulnerables: vec![],
			force_era: pallet_staking::Forcing::ForceNone,
			canceled_payout: Default::default(),
			history_depth: Default::default(),
			slash_reward_fraction: Perbill::from_percent(10),
		},
		pallet_authority_discovery: vodka_runtime::AuthorityDiscoveryConfig {
			keys: vec![],
		},
		pallet_babe: vodka_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(vodka_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_bootstrap: vodka_runtime::BootstrapConfig {
			endoweds: genesis_keys.clone().into_iter().map(|(account, _)| account).collect(),
		},
		pallet_collective_Instance1: vodka_runtime::CouncilConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_collective_Instance2: vodka_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_contracts: vodka_runtime::ContractsConfig {
			current_schedule: Default::default(),
		},
		pallet_democracy: vodka_runtime::DemocracyConfig { },
		pallet_elections_phragmen: vodka_runtime::ElectionsPhragmenConfig {
			members: vec![],
		},
		pallet_eons: vodka_runtime::EonsConfig {
			past_eons: vec![],
		},
		pallet_grandpa: vodka_runtime::GrandpaConfig {
			authorities: vec![],
		},
		pallet_im_online: vodka_runtime::ImOnlineConfig {
			keys: vec![],
		},
		pallet_membership_Instance1: vodka_runtime::TechnicalMembershipConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_sudo: vodka_runtime::SudoConfig {
			key: sudo_key,
		},
		pallet_treasury: vodka_runtime::TreasuryConfig { },
		pallet_vesting: vodka_runtime::VestingConfig {
			vesting: vec![],
		},
	}
}

pub fn vodka_config() -> Result<VodkaChainSpec, String> {
	let _wasm_binary = vodka_runtime::WASM_BINARY.ok_or("Vodka development wasm not available")?;
	let boot_nodes = vec![];

	Ok(VodkaChainSpec::from_genesis(
		"Vodka",
		"vodka",
		ChainType::Live,
		move || Default::default(),
		boot_nodes,
		None,
		Some("vodka"),
		None,
		Default::default()
	))
}
