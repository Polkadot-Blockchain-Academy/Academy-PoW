use academy_pow_runtime::AccountId;
use sc_service::ChainType;
use sp_core::crypto::Ss58Codec;
use std::convert::TryInto;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: RunCmd,
}

#[derive(Debug, clap::Parser)]
pub struct RunCmd {
    #[clap(flatten)]
    pub base: sc_cli::RunCmd,

    /// Miner's SR25519 public key for block rewards
    #[clap(long, value_parser = parse_sr25519_public_key)]
    pub sr25519_public_key: Option<sp_core::sr25519::Public>,

    /// whether to use instant seal
    #[clap(long, default_value = "false")]
    pub instant_seal: bool,
}

#[derive(Debug, clap::Parser)]
pub struct BuildSpecCmd {
    #[clap(flatten)]
    pub base: sc_cli::BuildSpecCmd,

    /// Chain name.
    #[arg(long, default_value = "Academy PoW")]
    pub chain_name: String,

    /// Chain ID is a short identifier of the chain
    #[arg(long, value_name = "ID", default_value = "academy_pow")]
    pub chain_id: String,

    /// AccountIds of the optional rich accounts
    #[arg(long, value_delimiter = ',', value_parser = parse_account_id, num_args=1..)]
    pub endowed_accounts: Option<Vec<AccountId>>,

    /// The type of the chain. Possible values: "dev", "local", "live" (default)
    #[arg(long, value_name = "TYPE", value_parser = parse_chaintype, default_value = "live")]
    pub chain_type: ChainType,

    #[arg(long, default_value = "4_000_000")]
    pub initial_difficulty: u32,
}

fn parse_chaintype(s: &str) -> Result<ChainType, String> {
    Ok(match s {
        "dev" => ChainType::Development,
        "local" => ChainType::Local,
        "live" => ChainType::Live,
        s => panic!("Wrong chain type {} Possible values: dev local live", s),
    })
}

/// Generate AccountId based on string command line argument.
fn parse_account_id(s: &str) -> Result<AccountId, String> {
    Ok(AccountId::from_string(s).expect("Passed string is not a hex encoding of a public key"))
}

fn parse_sr25519_public_key(i: &str) -> Result<sp_core::sr25519::Public, String> {
    hex::decode(i)
        .map_err(|e| e.to_string())?
        .as_slice()
        .try_into()
        .map_err(|_| "invalid length for SR25519 public key".to_string())
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
