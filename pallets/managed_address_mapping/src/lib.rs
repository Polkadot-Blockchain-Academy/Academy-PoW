//! TODO: license...
//!
//! "Managed" account mapping pallet. This exists to map Ethereum-style H160 addresses
//! to the H256 addresses used elsewhere.
//!
//! It allows users to claim a H160 address for ownership by the signed extrinsic origin.
//!
//! WARNING! THIS PALLET IS INSECURE, DO NOT USE IT IN PRODUCTION!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::{
		pallet_prelude::*,
		RawOrigin,
	};
    use pallet_evm::AddressMapping;
    use sp_core::{H160, H256};
    use sp_runtime::AccountId32;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::error]
    pub enum Error<T> {
        /// Address is already mapped
        AddressAlreadyMapped,
    }

    /// Mapping of H160 addresses to T::AccountId
    #[pallet::storage]
    pub type H160ToAccountMapping<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H160,
        T::AccountId,
        OptionQuery,
    >;

    /// Mapping of T::AccountId addresses to H160 addresses
    #[pallet::storage]
    pub type AccountToH160Mapping<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        H160,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn map_address(origin: OriginFor<T>, target: H160) -> DispatchResultWithPostInfo {
            let account_id = frame_system::ensure_signed(origin)?;

			// ensure that our account mapping is 1:1 by ensuring entries in both are empty
            ensure!(
                <H160ToAccountMapping<T>>::get(target) == None,
                Error::<T>::AddressAlreadyMapped,
            );
            ensure!(
                <AccountToH160Mapping<T>>::get(account_id.clone()) == None,
                Error::<T>::AddressAlreadyMapped,
            );

            <H160ToAccountMapping<T>>::insert(target, account_id.clone());
            <AccountToH160Mapping<T>>::insert(account_id, target);

            Ok(().into())
        }
    }

    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }

    pub struct EVMAddressMapping<T>(PhantomData<T>);
    impl<T: Config> AddressMapping<T::AccountId> for EVMAddressMapping<T> 
	{
        fn into_account_id(address: H160) -> T::AccountId {
			// TODO:
			// AccountId32 doesn't implement Default (although in at least some versions of Substrate it did/does)
			// The AddressMapping trait doesn't return an Option/Result
			// ...so what else can we do here?
            <H160ToAccountMapping<T>>::get(address).expect("fixme")
        }
    }

	pub struct EnsureAddressMapped<T>(PhantomData<T>);
	impl<OuterOrigin, T: Config> pallet_evm::EnsureAddressOrigin<OuterOrigin> for EnsureAddressMapped<T>
	where
		OuterOrigin: Into<Result<RawOrigin<T::AccountId>, OuterOrigin>> + From<RawOrigin<T::AccountId>> + Clone,
	{

		type Success = T::AccountId;

		fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<T::AccountId, OuterOrigin> {
			let account_id = ensure_signed(origin.clone()).map_err(|_| origin.clone())?;

			match <AccountToH160Mapping<T>>::get(account_id.clone()) {
				Some(mapped_address) if mapped_address == *address => Ok(account_id),
				_ => Err(origin)
			}
		}
	}
}
