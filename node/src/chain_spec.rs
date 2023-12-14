use academy_pow_runtime::{
    AccountId, RuntimeGenesisConfig, Signature,
    WASM_BINARY,
};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sc_service::ChainType;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

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
		None,
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
	.build())
}

pub fn testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
    .with_name("Testnet")
    .with_id("testnet")
    .with_chain_type(ChainType::Local)
	.with_genesis_config_patch(genesis(
		// Pre-funded accounts
        vec![
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        ],
        4_000_000,
    ))
    .build())
}

fn genesis(
    endowed_accounts: Vec<AccountId>,
    initial_difficulty: u32,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
			// Configure endowed accounts with initial balance of 1 << 60.
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		//TODO Figure out how to convert a u32 into a proper json value here.
        // "difficultyAdjustment": {
        //     "initialDifficulty": serde_json::json!(initial_difficulty),
        // },
    })
}
