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

use sc_chain_spec::ChainType;

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
