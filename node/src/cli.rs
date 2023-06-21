use academy_pow_runtime::AccountId;
use sc_cli::{
    clap::{self, ArgGroup, Parser},
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

    #[command(flatten)]
    pub pow: AcademyPowCli,

    #[clap(flatten)]
    pub run: RunCmd,
}

#[derive(Debug, Parser, Clone)]
#[clap(group(ArgGroup::new("backup")))]
pub struct AcademyPowCli {
    /// Miner's AccountId (base58 encoding of an SR25519 public key) for the block rewards
    #[clap(long,
           // required_unless(mining_public_key),
           conflicts_with = "mining_public_key",
           value_parser = parse_account_id)]
    pub mining_account_id: Option<AccountId>,

    /// Miner's hex encoding of the SR25519 public key) for the block rewards
    #[clap(
        long,
        // required_unless (mining_account_id),
        conflicts_with = "mining_account_id",
        value_parser = parse_sr25519_public_key
    )]
    pub mining_public_key: Option<sr25519::Public>,

    /// whether to use instant seal
    #[clap(long, default_value = "false")]
    pub instant_seal: bool,
}

impl AcademyPowCli {
    pub fn public_key_bytes(&self) -> [u8; 32] {
        match &self.mining_account_id {
            Some(account_id) => *account_id.as_ref(),
            None => match self.mining_public_key {
                Some(public_key) => public_key.0,
                None => panic!("Specify one of --mining_account_id or --mining_public_key"),
            },
        }
    }
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

/// Parse AccountId from a string argument passed on the command line.
fn parse_account_id(s: &str) -> Result<AccountId, String> {
    Ok(AccountId::from_string(s)
        .expect("Passed string is not a bas58 encoding of a sr25519 public key"))
}

/// Parse sr25519 pubkey from a string argument passed on the command line.
fn parse_sr25519_public_key(s: &str) -> Result<sr25519::Public, String> {
    Ok(sr25519::Public::from_string(s)
        .expect("Passed string is not a hex encoding of a sr25519 public key"))
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
