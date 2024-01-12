use std::str::FromStr;

use academy_pow_runtime::{
    AccountId, RuntimeGenesisConfig, SS58Prefix, Signature, TOKEN_DECIMALS, TOKEN_SYMBOL,
    WASM_BINARY,
};
use multi_pow::{ForkHeights, ForkingConfig, MaxiPosition};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, ForkingExtensions>;

/// PoW and Forking related chain spec extensions to configure the client side forking behavior.
///
/// The forks here are all related to adding and removing hash algorithms from the PoW.
/// The chain begins supporting only md5. Later is adds sha3 and keccak. Later it removes md5.
/// And finally there is a contentious fork where people become maxis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct ForkingExtensions {
    /// Manual mode is intended for when we you are running a live workshop.
    /// No forking happens automatically. Rather, you have to hard-code the forks.
    ///
    /// If manual mode is enabled, the rest of the parameters are ignored.
    /// This should really be an enum, but I have to work around the broken extension system.
    ///
    /// Aww damn it! I can't even use bool in this broken system? Okay then I guess 0 means automatic mode
    /// and anything else means manual mode.
    pub manual_mode: u32,
    /// The block height to perform the soft fork that adds sha3 and keccak support.
    pub add_sha3_keccak: u32,
    /// The block height to perform the hard fork that removes md5 support.
    pub remove_md5: u32,
    /// The block height to perform the contentious fork where some become sha3- or keccak-maxis.
    pub split_sha3_keccak: u32,
    // Damn extension thing is so fragile, I can't even use an enum here.
    // Let alone that time I tried to use the forked value feature.
    /// The political position that this node will take at the contentious fork.
    pub maxi_position: String,
}

impl From<&ForkingExtensions> for ForkingConfig {
    fn from(e: &ForkingExtensions) -> Self {
        if e.manual_mode > 0 {
            return Self::Manual;
        }

        let fork_heights = ForkHeights {
            add_sha3_keccak: e.add_sha3_keccak,
            remove_md5: e.remove_md5,
            split_sha3_keccak: e.split_sha3_keccak,
        };

        let maxi_position =
            MaxiPosition::from_str(&e.maxi_position).expect("Should have a valid maxi position...");

        Self::Automatic(fork_heights, maxi_position)
    }
}

impl ForkingExtensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        ForkingExtensions {
            manual_mode: 0,
            add_sha3_keccak: 10,
            remove_md5: 20,
            split_sha3_keccak: 30,
            maxi_position: String::from("follow-mining"),
        },
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(genesis(
        // Pre-funded accounts
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        ],
        // Initial Difficulty
        4_000_000,
    ))
    .with_properties(system_properties())
    .build())
}

pub fn testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        ForkingExtensions {
            manual_mode: 1,
            add_sha3_keccak: 0,
            remove_md5: 0,
            split_sha3_keccak: 0,
            maxi_position: String::new(),
        },
    )
    .with_name("Testnet")
    .with_id("testnet")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(genesis(
        // Pre-funded accounts
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
        ],
        4_000_000,
    ))
    .with_properties(system_properties())
    .build())
}

fn genesis(endowed_accounts: Vec<AccountId>, _initial_difficulty: u32) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            // Configure endowed accounts with initial balance of 1 << 50.
            "balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 50)).collect::<Vec<_>>(),
        },
        //TODO Figure out how to convert a u32 into a proper json value here.
        // "difficultyAdjustment": {
        //     "initialDifficulty": serde_json::json!(initial_difficulty),
        // },
    })
}

fn system_properties() -> sc_chain_spec::Properties {
    let mut properties = sc_chain_spec::Properties::new();

    properties.insert("ss58Format".into(), SS58Prefix::get().into());
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());

    properties
}
