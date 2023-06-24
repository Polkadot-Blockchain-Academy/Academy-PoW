use academy_pow_runtime::AccountId;
use sc_cli::{
    clap::{ArgGroup, Parser},
    RunCmd,
};
use sc_service::ChainType;
use sp_core::crypto::Ss58Codec;
use sp_core::sr25519;

#[derive(Debug, Parser)]
#[clap(subcommand_negates_reqs(true), version(env!("SUBSTRATE_CLI_IMPL_VERSION")))]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// whether to use instant seal
    #[clap(long, default_value = "false")]
    pub instant_seal: bool,

    /// Miner's AccountId (base58 encoding of an SR25519 public key) for the block rewards
    #[clap(long, value_parser = parse_account_id, default_value = "0000000000000000000000000000000000000000")]
    pub mining_account_id: AccountId,

    #[clap(flatten)]
    pub run: RunCmd,
}

#[derive(Debug, Parser)]
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

    #[arg(long, default_value = "4000000")]
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

/// Parse AccountId from a string argument passed on the command line.
fn parse_account_id(s: &str) -> Result<AccountId, String> {
    // Handle the optional 0x prefix
    let s = if s.starts_with("0x") {
        &s[2..]
    } else {
        &s[..]
    };

    // Decode the hex.
    let v = hex::decode(s).map_err(|_| "Could not decode account id as hex")?;
    if v.len() != 20 {
        Err("Account id bytes were the wrong length. Expected 20.")?;
    }

    // Isn't there a method to cast to a fixed length array?
    let mut bytes = [0u8; 20];
    for i in 0..20 {
        bytes[i] = v[i];
    }

    Ok(AccountId::from(bytes))
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
