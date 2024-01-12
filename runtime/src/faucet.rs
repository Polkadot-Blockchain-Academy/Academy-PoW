//! A simple token faucet that gives the caller 5 tokens per call

use frame_support::traits::Currency;
pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    /// Pallet's configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency type in which the faucet provides token
        type Currency: Currency<Self::AccountId>;

        /// The amount of tokens that should be created for each call into the faucet
        type DripAmount: Get<BalanceOf<Self>>;
    }

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Claim a few tokens from the faucet
        #[pallet::weight(1_000_000)]
        pub fn claim(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let _ = T::Currency::deposit_creating(&caller, T::DripAmount::get());

            Ok(())
        }
    }

    // #[pallet::hooks]
    // impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    // 	fn on_finalize(_n: T::BlockNumber) {}
    // }
}
