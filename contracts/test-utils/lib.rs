use std::{fs, rc::Rc};

use drink::{
    session::contract_transcode::{ContractMessageTranscoder, Tuple, Value},
    AccountId32,
};

/// Default drinking actor.
pub const ALICE: AccountId32 = AccountId32::new([1; 32]);
/// Another drinking actor.
pub const BOB: AccountId32 = AccountId32::new([2; 32]);

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

pub fn ok(v: Value) -> Value {
    Value::Tuple(Tuple::new(Some("Ok"), vec![v]))
}
