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
    use frame_support::{DefaultNoBound, pallet_prelude::*};
    use frame_system::{
		pallet_prelude::*,
		RawOrigin,
	};
    use pallet_evm::AddressMapping;
    use sp_core::H160;
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
                <H160ToAccountMapping<T>>::get(target).is_none(),
                Error::<T>::AddressAlreadyMapped,
            );
            ensure!(
                <AccountToH160Mapping<T>>::get(account_id.clone()).is_none(),
                Error::<T>::AddressAlreadyMapped,
            );

            <H160ToAccountMapping<T>>::insert(target, account_id.clone());
            <AccountToH160Mapping<T>>::insert(account_id, target);

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
		/// Retrieve the H160 mapped to the given account_id if there is one, or None otherwise
		pub fn get_mapped_h160(account_id: T::AccountId) -> Option<H160> {
			<AccountToH160Mapping<T>>::get(account_id)
		}
	}

    #[pallet::genesis_config]
    #[derive(DefaultNoBound)]
    pub struct GenesisConfig<T: Config>{
        pub genesis_mappings: Vec<(T::AccountId, H160)>
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (account_id, evm_address) in &self.genesis_mappings {
                AccountToH160Mapping::<T>::insert(account_id, evm_address);
            }
        }
    }

    pub struct EVMAddressMapping<T>(PhantomData<T>);
    impl<T: Config> AddressMapping<T::AccountId> for EVMAddressMapping<T> 
	where
		T::AccountId: IsType<AccountId32>,
	{
        fn into_account_id(address: H160) -> T::AccountId {
            if let Some(account_id) = <H160ToAccountMapping<T>>::get(address) {
				account_id
			} else {
				// we use Acala's approach as a fallback by embedding the H160 address within the 32 bytes of
				// an AccountId32, prefixed with "evm:" to clearly identify these.
				//
				// A quick analysis on security:
				// * it would be easy to find a private key for an AccountId32 which started with b"evm:", so 
				//   this prefix shouldn't be trusted to come only from an EVM address for which there is no
				//   corresponding AccountId32 private key
				// * However, finding a collision such that there is both a private key for the AccountId32 and
				//   the derived b"evm:" based address should be practically impossible
				//
				// In other words: if you see a b"evm:" address, it may have a private key (probably malicously),
				// but it will not also have a mapped AccountId32 with a private key.
				let mut data: [u8; 32] = [0u8; 32];
				data[0..4].copy_from_slice(b"evm:");
				data[4..24].copy_from_slice(&address[..]);
				AccountId32::from(data).into()
			}
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
