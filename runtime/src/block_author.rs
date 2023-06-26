//! This pallet allows block authors to self-identify by providing an account id
//!
//! The included trait allows other pallets to fetch the author's account.

pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError};
use sp_runtime::RuntimeString;
use sp_std::vec::Vec;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    /// The BlockAuthor Inherent pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);
    /// The pallet's configuration trait. Nothing to configure.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        fn on_author_set(_author_account: Self::AccountId) {}
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Author already set in block.
        AuthorAlreadySet,
    }

    /// Author of current block.
    #[pallet::storage]
    pub type Author<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Inherent to set the author of a block
        #[pallet::weight(1_000_000)]
        pub fn set_author(origin: OriginFor<T>, author: T::AccountId) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(Author::<T>::get().is_none(), Error::<T>::AuthorAlreadySet);

            // Store the author in case other pallets want to fetch it and to let
            // offchain tools inspect it
            Author::<T>::put(&author);

            // Call the hook
            T::on_author_set(author);

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            // Reset the author to None at the beginning of the block
            Author::<T>::kill();

            // Return zero weight because we are not using weight-based
            // transaction fees.
            Weight::zero()
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T> {
        type Call = Call<T>;
        type Error = InherentError;
        const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

        fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
            // Return Ok(Some(_)) unconditionally because this inherent is required in every block
            // If it is not found, throw an AuthorInherentRequired error.
            Ok(Some(InherentError::Other(
                sp_runtime::RuntimeString::Borrowed("BlockAuthorInherentRequired"),
            )))
        }

        fn create_inherent(data: &InherentData) -> Option<Self::Call> {
            // Grab the Vec<u8> labelled with "author_" from the map of all inherent data
            let author_raw = data
                .get_data::<InherentType>(&INHERENT_IDENTIFIER)
                .expect("Gets and decodes authorship inherent data")?;

            // Decode the Vec<u8> into an actual author
            let author = T::AccountId::decode(&mut &author_raw[..])
                .expect("Decodes author raw inherent data");

            Some(Call::set_author { author })
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::set_author { .. })
        }
    }
}

/// A trait to find the author (miner) of the block.
pub trait BlockAuthor<AccountId> {
    fn block_author() -> Option<AccountId>;
}

impl<AccountId> BlockAuthor<AccountId> for () {
    fn block_author() -> Option<AccountId> {
        None
    }
}

impl<T: Config, U> BlockAuthor<U> for Pallet<T>
where
    T::AccountId: Into<U>,
{
    fn block_author() -> Option<U> {
        Author::<T>::get().map(|x| x.into())
    }
}

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
    Other(RuntimeString),
}

impl IsFatalError for InherentError {
    fn is_fatal_error(&self) -> bool {
        match *self {
            InherentError::Other(_) => true,
        }
    }
}

impl InherentError {
    /// Try to create an instance ouf of the given identifier and data.
    #[cfg(feature = "std")]
    pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
        if id == &INHERENT_IDENTIFIER {
            <InherentError as parity_scale_codec::Decode>::decode(&mut &data[..]).ok()
        } else {
            None
        }
    }
}

/// The type of data that the inherent will contain.
/// Just a byte array. It will be decoded to an actual pubkey later
pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider(pub InherentType);

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut InherentData,
    ) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
    }

    async fn try_handle_error(
        &self,
        identifier: &InherentIdentifier,
        _error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        // Dont' process modules from other inherents
        if *identifier != INHERENT_IDENTIFIER {
            return None;
        }

        // All errors with the author inehrent are fatal
        Some(Err(sp_inherents::Error::Application(Box::from(
            String::from("Error processing author inherent"),
        ))))
    }
}
