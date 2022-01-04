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

use codec::Decode;
use indexmap::IndexMap;
use np_opaque::{AccountId, Balance};
use sc_chain_spec::{ChainSpecExtension, ChainType};
use serde::{Deserialize, Serialize};
use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_runtime::Perbill;
use std::convert::TryFrom;
use std::marker::PhantomData;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// The light sync state.
	///
	/// This value will be set by the `sync-state rpc` implementation.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

pub type NeatcoinChainSpec =
	sc_service::GenericChainSpec<neatcoin_runtime::GenesisConfig, Extensions>;
pub type VodkaChainSpec = sc_service::GenericChainSpec<vodka_runtime::GenesisConfig, Extensions>;

pub fn build_genesis_allocations() -> IndexMap<AccountId, Balance> {
	let raw: IndexMap<String, String> =
		serde_json::from_slice(include_bytes!("../res/genesis.json"))
			.expect("parse genesis.json failed");
	raw.into_iter()
		.map(|(key, value)| {
			let (address, version) =
				AccountId::from_ss58check_with_version(&key).expect("parse address failed");
			assert_eq!(
				Ss58AddressFormatRegistry::try_from(version),
				Ok(Ss58AddressFormatRegistry::KulupuAccount)
			);
			let balance = u128::from_str_radix(&value, 10).expect("parse balance failed");
			(address, balance)
		})
		.collect()
}

pub fn build_neatcoin_genesis(
	wasm_binary: &[u8],
	genesis_keys: Vec<(AccountId, neatcoin_runtime::SessionKeys)>,
) -> neatcoin_runtime::GenesisConfig {
	neatcoin_runtime::GenesisConfig {
		system: neatcoin_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: neatcoin_runtime::BalancesConfig {
			balances: build_genesis_allocations().into_iter().collect(),
		},
		indices: neatcoin_runtime::IndicesConfig { indices: vec![] },
		session: neatcoin_runtime::SessionConfig {
			keys: genesis_keys
				.clone()
				.into_iter()
				.map(|(account, keys)| (account.clone(), account, keys))
				.collect(),
		},
		staking: neatcoin_runtime::StakingConfig {
			validator_count: 17,
			minimum_validator_count: 7,
			stakers: vec![],
			invulnerables: vec![],
			force_era: pallet_staking::Forcing::ForceNone,
			canceled_payout: Default::default(),
			history_depth: Default::default(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		authority_discovery: neatcoin_runtime::AuthorityDiscoveryConfig { keys: vec![] },
		babe: neatcoin_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(neatcoin_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		bootstrap: neatcoin_runtime::BootstrapConfig {
			endoweds: genesis_keys
				.clone()
				.into_iter()
				.map(|(account, _)| account)
				.collect(),
		},
		council: neatcoin_runtime::CouncilConfig {
			phantom: PhantomData,
			members: vec![],
		},
		technical_committee: neatcoin_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData,
			members: vec![],
		},
		democracy: neatcoin_runtime::DemocracyConfig::default(),
		elections_phragmen: neatcoin_runtime::ElectionsPhragmenConfig { members: vec![] },
		eons: neatcoin_runtime::EonsConfig { past_eons: vec![] },
		grandpa: neatcoin_runtime::GrandpaConfig {
			authorities: vec![],
		},
		im_online: neatcoin_runtime::ImOnlineConfig { keys: vec![] },
		technical_membership: neatcoin_runtime::TechnicalMembershipConfig {
			phantom: PhantomData,
			members: vec![],
		},
		treasury: neatcoin_runtime::TreasuryConfig {},
		vesting: neatcoin_runtime::VestingConfig { vesting: vec![] },
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
				let raw: IndexMap<String, String> =
					serde_json::from_slice(include_bytes!("../res/neatcoin-initvals.json"))
						.expect("parse neatcoin-initvals.json failed");

				raw.into_iter()
					.map(|(key, value)| {
						(
							AccountId::from_ss58check(&key).expect("parse address failed"),
							neatcoin_runtime::SessionKeys::decode(
								&mut &hex::decode(&value).expect("decode hex failed")[..],
							)
							.expect("decode session keys failed"),
						)
					})
					.collect()
			};

			build_neatcoin_genesis(include_bytes!("../res/neatcoin-0.wasm"), init_vals)
		},
		boot_nodes,
		None,
		Some("neatcoin"),
		Some(
			serde_json::json!({
				"ss58Format": 48,
				"tokenDecimals": 12,
				"tokenSymbol": "NEAT"
			})
			.as_object()
			.expect("Created an object")
			.clone(),
		),
		Default::default(),
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
		system: vodka_runtime::SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: vodka_runtime::BalancesConfig {
			balances: build_genesis_allocations().into_iter().collect(),
		},
		indices: vodka_runtime::IndicesConfig { indices: vec![] },
		session: vodka_runtime::SessionConfig {
			keys: genesis_keys
				.clone()
				.into_iter()
				.map(|(account, keys)| (account.clone(), account, keys))
				.collect(),
		},
		staking: vodka_runtime::StakingConfig {
			validator_count: 17,
			minimum_validator_count: 2, // NOTE: should set to 7 for mainnet.
			stakers: vec![],
			invulnerables: vec![],
			force_era: pallet_staking::Forcing::ForceNone,
			canceled_payout: Default::default(),
			history_depth: Default::default(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		authority_discovery: vodka_runtime::AuthorityDiscoveryConfig { keys: vec![] },
		babe: vodka_runtime::BabeConfig {
			authorities: vec![],
			epoch_config: Some(vodka_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		bootstrap: vodka_runtime::BootstrapConfig {
			endoweds: genesis_keys
				.clone()
				.into_iter()
				.map(|(account, _)| account)
				.collect(),
		},
		council: vodka_runtime::CouncilConfig {
			phantom: PhantomData,
			members: vec![],
		},
		technical_committee: vodka_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData,
			members: vec![],
		},
		democracy: vodka_runtime::DemocracyConfig::default(),
		elections_phragmen: vodka_runtime::ElectionsPhragmenConfig { members: vec![] },
		eons: vodka_runtime::EonsConfig { past_eons: vec![] },
		grandpa: vodka_runtime::GrandpaConfig {
			authorities: vec![],
		},
		im_online: vodka_runtime::ImOnlineConfig { keys: vec![] },
		technical_membership: vodka_runtime::TechnicalMembershipConfig {
			phantom: PhantomData,
			members: vec![],
		},
		sudo: vodka_runtime::SudoConfig { key: sudo_key },
		treasury: vodka_runtime::TreasuryConfig {},
		vesting: vodka_runtime::VestingConfig { vesting: vec![] },
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
				let raw: IndexMap<String, String> =
					serde_json::from_slice(include_bytes!("../res/vodka-initvals.json"))
						.expect("parse vodka-initvals.json failed");

				raw.into_iter()
					.map(|(key, value)| {
						(
							AccountId::from_ss58check(&key).expect("parse address failed"),
							vodka_runtime::SessionKeys::decode(
								&mut &hex::decode(&value).expect("decode hex failed")[..],
							)
							.expect("decode session keys failed"),
						)
					})
					.collect()
			};

			build_vodka_genesis(
				include_bytes!("../res/vodka-0.wasm"),
				init_vals,
				AccountId::from_ss58check("5DjqKKzLzYHTzgMgG2mxtZaSojShWwp9N3qPhnWuRoL3sFeD")
					.expect("parse address failed"),
			)
		},
		boot_nodes,
		None,
		Some("vodka"),
		Some(
			serde_json::json!({
				"ss58Format": 42,
				"tokenDecimals": 12,
				"tokenSymbol": "VODKA"
			})
			.as_object()
			.expect("Created an object")
			.clone(),
		),
		Default::default(),
	))
}

pub fn vodka_config() -> Result<VodkaChainSpec, String> {
	VodkaChainSpec::from_json_bytes(&include_bytes!("../res/vodka-spec.json")[..])
}

pub fn development_config() -> Result<NeatcoinChainSpec, String> {
	let wasm_binary =
		neatcoin_runtime::WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(NeatcoinChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		move || {
			build_neatcoin_genesis(
				wasm_binary,
				vec![(Default::default(), neatcoin_runtime::SessionKeys::default())],
			)
		},
		vec![],
		None,
		None,
		None,
		Default::default(),
	))
}
