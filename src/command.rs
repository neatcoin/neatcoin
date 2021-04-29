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

use std::sync::Arc;
use futures::future::TryFutureExt;
use sc_cli::{SubstrateCli, RuntimeVersion, Role, ChainSpec};
use neatcoin_service::{chain_spec, IdentifyVariant, ChainVariant};
use crate::cli::{Cli, Subcommand};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(transparent)]
	NeatcoinService(#[from] neatcoin_service::Error),

	#[error(transparent)]
	SubstrateCli(#[from] sc_cli::Error),

	#[error(transparent)]
	SubstrateService(#[from] sc_service::Error),
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Neatcoin".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/neatcoin/neatcoin".into()
	}

	fn copyright_start_year() -> i32 {
		2021
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"vodka" | "testnet" => Box::new(chain_spec::vodka_config()?),
			"" | "neatcoin" | "mainnet" => Box::new(chain_spec::neatcoin_config()?),
			_path => return Err("Custom chain spec is not supported".into()),
		})
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		match spec.identify_variant() {
			ChainVariant::Neatcoin => &neatcoin_service::neatcoin_runtime::VERSION,
			ChainVariant::Vodka => &neatcoin_service::vodka_runtime::VERSION,
		}
	}
}

fn set_default_ss58_version(spec: &Box<dyn sc_service::ChainSpec>) {
	use sp_core::crypto::Ss58AddressFormat;

	let ss58_version = match spec.identify_variant() {
		ChainVariant::Neatcoin => Ss58AddressFormat::NeatcoinAccount,
		ChainVariant::Vodka => Ss58AddressFormat::SubstrateAccount,
	};

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Parses polkadot specific CLI arguments and run the service.
pub fn run() -> Result<(), Error> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)
				.map_err(Error::from)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.run_node_until_exit(move |config| async move {
				let role = config.role.clone();

				let task_manager = match role {
					Role::Light => neatcoin_service::build_light(config).map(|light| light.task_manager),
					_ => neatcoin_service::build_full(config).map(|full| full.task_manager),
				}?;
				Ok::<_, Error>(task_manager)
			})
		},
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			Ok(runner.sync_run(|config| {
				cmd.run(config.chain_spec, config.network)
			})?)
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)
				.map_err(Error::SubstrateCli)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let ops = neatcoin_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(Arc::new(ops.client), ops.import_queue).map_err(Error::SubstrateCli), ops.task_manager))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			Ok(runner.async_run(|mut config| {
				let ops = neatcoin_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(Arc::new(ops.client), config.database).map_err(Error::SubstrateCli), ops.task_manager))
			})?)
		},
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			Ok(runner.async_run(|mut config| {
				let ops = neatcoin_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(Arc::new(ops.client), config.chain_spec).map_err(Error::SubstrateCli), ops.task_manager))
			})?)
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			Ok(runner.async_run(|mut config| {
				let ops = neatcoin_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(Arc::new(ops.client), ops.import_queue).map_err(Error::SubstrateCli), ops.task_manager))
			})?)
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			Ok(runner.sync_run(|config| cmd.run(config.database))?)
		},
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			Ok(runner.async_run(|mut config| {
				let ops = neatcoin_service::new_chain_ops(&mut config)?;
				Ok((cmd.run(Arc::new(ops.client), ops.backend).map_err(Error::SubstrateCli), ops.task_manager))
			})?)
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			Ok(runner.sync_run(|config| {
				cmd.run::<neatcoin_service::neatcoin_runtime::Block, neatcoin_service::NeatcoinExecutor>(config)
					.map_err(|e| Error::SubstrateCli(e))
			})?)
		},
		Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;
			set_default_ss58_version(chain_spec);

			runner.async_run(|config| {
				use sc_service::TaskManager;
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager = TaskManager::new(
					config.task_executor.clone(),
					registry,
				).map_err(|e| Error::SubstrateService(sc_service::Error::Prometheus(e)))?;

				Ok((
					cmd.run::<
						service::kusama_runtime::Block,
						service::KusamaExecutor,
					>(config).map_err(Error::SubstrateCli),
					task_manager
				))
				// NOTE: we fetch only the block number from the block type, the chance of disparity
				// between kusama's and polkadot's block number is small enough to overlook this.
			})
		}
	}?;
	Ok(())
}
