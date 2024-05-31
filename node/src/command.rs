// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use academy_pow_runtime::Block;
use multi_pow::{ForkingConfig, MaxiPosition};
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use sp_core::sr25519::Public;

use crate::{
    chain_spec::{self, ForkingExtensions},
    cli::{Cli, Subcommand},
    service,
};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Academy PoW Chain".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn executable_name() -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn support_url() -> String {
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2019
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "" => Box::new(chain_spec::ChainSpec::from_json_bytes(
                &include_bytes!("../../spec.json")[..],
            )?),
            "dev" => Box::new(chain_spec::development_config()?),
            "local" => Box::new(chain_spec::testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(&cmd.base)?;
            runner.sync_run(|config| cmd.base.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config, ForkingConfig::Manual)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config, ForkingConfig::Manual)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config, ForkingConfig::Manual)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config, ForkingConfig::Manual)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config, ForkingConfig::Manual)?;
                Ok((cmd.run(client, backend, None), task_manager))
            })
        }
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        None => {
            // Get the mining account from the cli
            let bytes: [u8; 32] = cli.pow.public_key_bytes(cli.run.get_keyring());
            let sr25519_public_key = Public(bytes);

            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                // Get the forking information from the chain spec extension.
                // Convert it to a strong type, and fill in the proper maxi position if they are following mining.
                let forking_extension = ForkingExtensions::try_get(&*config.chain_spec)
                    .expect("Should be able to get the fork config from the extension");
                let forking_config = match ForkingConfig::from(forking_extension) {
                    ForkingConfig::Automatic(fork_heights, MaxiPosition::FollowMining) => {
                        let maxi_position = match cli.pow.mining_algo {
                            multi_pow::SupportedHashes::Md5 => MaxiPosition::NoMaxi,
                            multi_pow::SupportedHashes::Sha3 => MaxiPosition::Sha3Maxi,
                            multi_pow::SupportedHashes::Keccak => MaxiPosition::KeccakMaxi,
                        };
                        ForkingConfig::Automatic(fork_heights, maxi_position)
                    }
                    old_config => old_config,
                };

                service::new_full(
                    config,
                    forking_config,
                    //TODO Combine the following three fields into a MiningConfig analogous to the ForkingConfig
                    sr25519_public_key,
                    cli.pow.instant_seal,
                    cli.pow.mining_algo,
                )
                .map_err(sc_cli::Error::Service)
            })
        }
    }
}
