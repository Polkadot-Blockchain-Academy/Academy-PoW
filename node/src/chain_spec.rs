use academy_pow_runtime::{
    AccountId, BalancesConfig, DifficultyAdjustmentConfig, EVMChainIdConfig, GenesisConfig,
    SystemConfig, WASM_BINARY,
};
use hex_literal::hex;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
                    AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
                    AccountId::from(hex!("0Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
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
                    AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
                    AccountId::from(hex!("0Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
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

// pub fn custom_config(
//     chain_name: &str,
//     chain_id: &str,
//     endowed_accounts: Vec<AccountId>,
//     initial_difficulty: u32,
// ) -> Result<ChainSpec, String> {
//     let wasm_binary = WASM_BINARY.ok_or_else(|| "runtime WASM binary not available".to_string())?;

//     Ok(ChainSpec::from_genesis(
//         chain_name,
//         chain_id,
//         sc_service::ChainType::Live,
//         move || genesis(wasm_binary, endowed_accounts.clone(), initial_difficulty),
//         vec![],
//         None,
//         None,
//         None,
//         None,
//         None,
//     ))
// }

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

        evm: Default::default(),
        evm_chain_id: EVMChainIdConfig { chain_id: 4242 },
        ethereum: Default::default(),
        base_fee: Default::default(),
        transaction_payment: Default::default(),
    }
}
