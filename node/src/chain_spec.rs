use academy_pow_runtime::{
    AccountId, BalancesConfig, DifficultyAdjustmentConfig, GenesisConfig, Signature, SystemConfig,
    WASM_BINARY,
};
use sp_core::{ecdsa, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use hex_literal::hex;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        sc_service::ChainType::Development,
        || {
            genesis(
                wasm_binary,
                vec![
                    get_account_id_from_seed::<ecdsa::Public>("Alice"),
                    AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
                    get_account_id_from_seed::<ecdsa::Public>("Bob"),
                    get_account_id_from_seed::<ecdsa::Public>("Alice//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Bob//stash"),
                ],
                4_000_000,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
        None,
    ))
}

pub fn testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        "Testnet",
        "testnet",
        sc_service::ChainType::Local,
        || {
            genesis(
                wasm_binary,
                vec![
                    get_account_id_from_seed::<ecdsa::Public>("Alice"),
                    get_account_id_from_seed::<ecdsa::Public>("Bob"),
                    get_account_id_from_seed::<ecdsa::Public>("Charlie"),
                    get_account_id_from_seed::<ecdsa::Public>("Dave"),
                    get_account_id_from_seed::<ecdsa::Public>("Eve"),
                    get_account_id_from_seed::<ecdsa::Public>("Ferdie"),
                    get_account_id_from_seed::<ecdsa::Public>("Alice//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Bob//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Charlie//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Dave//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Eve//stash"),
                    get_account_id_from_seed::<ecdsa::Public>("Ferdie//stash"),
                ],
                4_000_000,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
        None,
    ))
}

pub fn custom_config(
    chain_name: &str,
    chain_id: &str,
    endowed_accounts: Vec<AccountId>,
    initial_difficulty: u32,
) -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "runtime WASM binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        chain_name,
        chain_id,
        sc_service::ChainType::Live,
        move || genesis(wasm_binary, endowed_accounts.clone(), initial_difficulty),
        vec![],
        None,
        None,
        None,
        None,
        None,
    ))
}

fn genesis(
    wasm_binary: &[u8],
    endowed_accounts: Vec<AccountId>,
    initial_difficulty: u32,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        // sudo: SudoConfig {
        //     key: Some(root_key),
        // },
        difficulty_adjustment: DifficultyAdjustmentConfig {
            initial_difficulty: initial_difficulty.into(),
        },
        transaction_payment: Default::default(),
    }
}
