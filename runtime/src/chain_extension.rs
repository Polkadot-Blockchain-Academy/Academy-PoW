use crate::{Randomness, RandomnessCollectiveFlip, Runtime};
use frame_support::log::{debug, error};
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_core::Encode;
use sp_runtime::DispatchError;

#[derive(Default)]
pub struct AcademyPowChainExtension;

// Function identifiers
pub const RANDOM_FUNCTION_ID: u16 = 10;

// Return codes
pub const RANDOM_FUNCTION_OK: u32 = 10_000;

// TODO : make it generic over Runtime as Config and move to the extension crate
// this would allow us to e.g. have one definition of the return and error codes
impl ChainExtension<Runtime> for AcademyPowChainExtension {
    fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        let func_id = env.func_id();

        match func_id {
            RANDOM_FUNCTION_ID => Self::random(env),
            _ => {
                error!("Called an unregistered func_id: {}", func_id);
                Err(DispatchError::Other("Unimplemented func_id"))
            }
        }
    }
}

impl AcademyPowChainExtension {
    fn random<E: Ext>(env: Environment<E, InitState>) -> Result<RetVal, DispatchError> {
        debug!(
            target: "runtime",
            "[ChainExtension]|call|func_id:{:}",
            RANDOM_FUNCTION_OK
        );

        let mut env = env.buf_in_buf_out();

        let arg: [u8; 32] = env.read_as()?;
        let random_seed = RandomnessCollectiveFlip::random(&arg).0;
        let random_slice = random_seed.encode();

        env.write(&random_slice, false, None)?;

        Ok(RetVal::Converging(RANDOM_FUNCTION_OK))
    }
}
