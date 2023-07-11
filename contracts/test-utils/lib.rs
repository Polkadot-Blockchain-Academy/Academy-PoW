use std::{fs, rc::Rc};

use drink::{
    chain_api::ChainApi,
    runtime::MinimalRuntime,
    session::{
        contract_transcode::{ContractMessageTranscoder, Tuple, Value},
        Session, SessionError,
    },
    AccountId32,
};

/// Default drinking actor.
pub const ALICE: AccountId32 = AccountId32::new([1; 32]);
/// Another drinking actor.
pub const BOB: AccountId32 = AccountId32::new([2; 32]);
/// Initial balance for each actor.
pub const INITIAL_BALANCE: u128 = 10_000_000_000_000;

/// Get contract transcoder from its metadata file.
pub fn get_transcoder(contract_name: &str) -> Rc<ContractMessageTranscoder> {
    Rc::new(
        ContractMessageTranscoder::load(format!("target/ink/{contract_name}.json"))
            .expect("Failed to load contract metadata"),
    )
}

/// Read contract's compiled wasm blob.
pub fn get_wasm(contract_name: &str) -> Vec<u8> {
    fs::read(format!("target/ink/{contract_name}.wasm"))
        .expect("Failed to find or read contract file")
}

/// Wraps `v` into `Ok(v)` variant -- at the bottom level, contract messages return result.
pub fn ok(v: Value) -> Value {
    Value::Tuple(Tuple::new(Some("Ok"), vec![v]))
}

/// Get initialized session with contract already deployed and accounts endowed.
pub fn get_initialized_session(
    contract_name: &str,
    constructor: &str,
    constructor_args: &[String],
) -> Result<Session<MinimalRuntime>, SessionError> {
    let mut session = Session::<MinimalRuntime>::new(Some(get_transcoder(contract_name)))?;
    session.deploy(
        get_wasm(contract_name),
        constructor,
        constructor_args,
        vec![],
    )?;

    // Endow BOB with 10_000_000_000_000 tokens. ALICE has already enough funds
    session.chain_api().add_tokens(BOB.clone(), INITIAL_BALANCE);

    Ok(session)
}
