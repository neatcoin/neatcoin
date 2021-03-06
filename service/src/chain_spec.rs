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

use std::marker::PhantomData;
use indexmap::IndexMap;
use codec::Decode;
use sp_core::crypto::{Ss58Codec, Ss58AddressFormat};
use sp_runtime::Perbill;
use sc_chain_spec::ChainType;
use np_opaque::{AccountId, Balance};

pub type NeatcoinChainSpec = sc_service::GenericChainSpec<neatcoin_runtime::GenesisConfig>;
pub type VodkaChainSpec = sc_service::GenericChainSpec<vodka_runtime::GenesisConfig>;

pub fn build_genesis_allocations() -> IndexMap<AccountId, Balance> {
	let raw: IndexMap<String, String> = serde_json::from_slice(include_bytes!("../res/genesis.json"))
		.expect("parse genesis.json failed");
	raw.into_iter().map(|(key, value)| {
		let (address, version) = AccountId::from_ss58check_with_version(&key)
			.expect("parse address failed");
		assert_eq!(version, Ss58AddressFormat::KulupuAccount);
		let balance = u128::from_str_radix(&value, 10).expect("parse balance failed");
		(address, balance)
	}).collect()
}

pub fn build_neatcoin_genesis(
	wasm_binary: &[u8],
	genesis_keys: Vec<(AccountId, neatcoin_runtime::SessionKeys)>,
) -> neatcoin_runtime::GenesisConfig {
	neatcoin_runtime::GenesisConfig {
		frame_system: neatcoin_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: neatcoin_runtime::BalancesConfig {
			balances: build_genesis_allocations().into_iter().collect(),
		},
		pallet_indices: neatcoin_runtime::IndicesConfig {
			indices: vec![],
		},
		pallet_session: neatcoin_runtime::SessionConfig {
			keys: genesis_keys.clone().into_iter().map(|(account, keys)| {
				(account.clone(), account, keys)
			}).collect(),
		},
		pallet_staking: neatcoin_runtime::StakingConfig {
			validator_count: 17,
			minimum_validator_count: 7,
			stakers: vec![],
			invulnerables: vec![],
			force_era: pallet_staking::Forcing::ForceNone,
			canceled_payout: Default::default(),
			history_depth: Default::default(),
			slash_reward_fraction: Perbill::from_percent(10),
		},
		pallet_authority_discovery: neatcoin_runtime::AuthorityDiscoveryConfig {
			keys: vec![],
		},
		pallet_babe: neatcoin_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(neatcoin_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_bootstrap: neatcoin_runtime::BootstrapConfig {
			endoweds: genesis_keys.clone().into_iter().map(|(account, _)| account).collect(),
		},
		pallet_collective_Instance1: neatcoin_runtime::CouncilConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_collective_Instance2: neatcoin_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_democracy: neatcoin_runtime::DemocracyConfig { },
		pallet_elections_phragmen: neatcoin_runtime::ElectionsPhragmenConfig {
			members: vec![],
		},
		pallet_eons: neatcoin_runtime::EonsConfig {
			past_eons: vec![],
		},
		pallet_grandpa: neatcoin_runtime::GrandpaConfig {
			authorities: vec![],
		},
		pallet_im_online: neatcoin_runtime::ImOnlineConfig {
			keys: vec![],
		},
		pallet_membership_Instance1: neatcoin_runtime::TechnicalMembershipConfig {
			phantom: PhantomData,
			members: vec![],
		},
		pallet_treasury: neatcoin_runtime::TreasuryConfig { },
		pallet_vesting: neatcoin_runtime::VestingConfig {
			vesting: vec![],
		},
	}
}

pub fn build_neatcoin_config() -> Result<NeatcoinChainSpec, String> {
	let boot_nodes = vec![
		"/dns4/a.bootnode.neatcoin.org/tcp/26100/ws/p2p/12D3KooWJcQDt9NaXgJvkiQmWB6NHrvAJFybp6JwjKPDEgvnRAoM".parse().expect("parse bootnode failed")
	];

	Ok(NeatcoinChainSpec::from_genesis(
		"Neatcoin",
		"neatcoin",
		ChainType::Live,
		move || {
			let init_vals = {
				let raw: IndexMap<String, String> = serde_json::from_slice(include_bytes!("../res/neatcoin-initvals.json"))
					.expect("parse neatcoin-initvals.json failed");

				raw.into_iter().map(|(key, value)| {
					(
						AccountId::from_ss58check(&key).expect("parse address failed"),
						neatcoin_runtime::SessionKeys::decode(&mut &hex::decode(&value).expect("decode hex failed")[..]).expect("decode session keys failed")
					)
				}).collect()
			};

			build_neatcoin_genesis(
				include_bytes!("../res/neatcoin-0.wasm"),
				init_vals,
			)
		},
		boot_nodes,
		None,
		Some("neatcoin"),
		Some(serde_json::json!({
			"ss58Format": 48,
			"tokenDecimals": 12,
			"tokenSymbol": "NEAT"
		}).as_object().expect("Created an object").clone()),
		Default::default()
	))
}

pub fn neatcoin_config() -> Result<NeatcoinChainSpec, String> {
	NeatcoinChainSpec::from_json_bytes(&include_bytes!("../res/neatcoin-spec.json")[..])
}

pub fn build_vodka_genesis(
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
			balances: build_genesis_allocations().into_iter().collect(),
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
			minimum_validator_count: 2, // NOTE: should set to 7 for mainnet.
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

pub fn build_vodka_config() -> Result<VodkaChainSpec, String> {
	let boot_nodes = vec![
		"/dns4/a.vodka.bootnode.neatcoin.org/tcp/27100/ws/p2p/12D3KooWHkNKLcFaxqAjgyRLxm4SeTQH9L3XZ2QDHijsLfrjGaW7".parse().expect("parse bootnode failed")
	];

	Ok(VodkaChainSpec::from_genesis(
		"Vodka",
		"vodka",
		ChainType::Live,
		move || {
			let init_vals = {
				let raw: IndexMap<String, String> = serde_json::from_slice(include_bytes!("../res/vodka-initvals.json"))
					.expect("parse vodka-initvals.json failed");

				raw.into_iter().map(|(key, value)| {
					(
						AccountId::from_ss58check(&key).expect("parse address failed"),
						vodka_runtime::SessionKeys::decode(&mut &hex::decode(&value).expect("decode hex failed")[..]).expect("decode session keys failed")
					)
				}).collect()
			};

			build_vodka_genesis(
				include_bytes!("../res/vodka-0.wasm"),
				init_vals,
				AccountId::from_ss58check("5DjqKKzLzYHTzgMgG2mxtZaSojShWwp9N3qPhnWuRoL3sFeD").expect("parse address failed"),
			)
		},
		boot_nodes,
		None,
		Some("vodka"),
		Some(serde_json::json!({
			"ss58Format": 42,
			"tokenDecimals": 12,
			"tokenSymbol": "VODKA"
		}).as_object().expect("Created an object").clone()),
		Default::default()
	))
}

pub fn vodka_config() -> Result<VodkaChainSpec, String> {
	VodkaChainSpec::from_json_bytes(&include_bytes!("../res/vodka-spec.json")[..])
}
