use sp_core::sr25519;
use sp_std::vec::Vec;
use sp_runtime::RuntimeString;
use sp_inherents::{InherentIdentifier, IsFatalError};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use parity_scale_codec::{Encode, Decode};


pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use super::*;

	/// The BlockAuthor Inherent pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
	/// The pallet's configuration trait. Nothing to configure.
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::error]
	pub enum Error<T> {
		/// Author already set in block.
		AuthorAlreadySet,
	}

	/// Author of current block.
	#[pallet::storage]
	pub type Author<T: Config> = StorageValue<_, sr25519::Public, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Inherent to set the author of a block
		#[pallet::weight(1_000_000)]
		pub fn set_author(origin: OriginFor<T>, author: sr25519::Public) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(Author::<T>::get().is_none(), Error::<T>::AuthorAlreadySet);

			Author::<T>::put(author);

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
			0
		}
	}

	#[pallet::inherent]
	impl<T:Config> ProvideInherent for Pallet<T> {
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
			let author_raw = data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

			// Decode the Vec<u8> into an actual author
			let author = sr25519::Public::decode(&mut &author_raw[..])
				.expect("Decodes author raw inherent data");

			Some(Call::set_author{author})
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_author{..})
		}
	}
}

//TODO maybe make the trait generic over the "account" type
/// A trait to find the author (miner) of the block.
pub trait BlockAuthor {
	fn block_author() -> Option<sr25519::Public>;
}

impl BlockAuthor for () {
	fn block_author() -> Option<sr25519::Public> {
		None
	}
}

impl<T: Config> BlockAuthor for Pallet<T> {
	fn block_author() -> Option<sr25519::Public> {
		Author::<T>::get()
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
	fn provide_inherent_data(
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
			return None
		}

		// All errors with the author inehrent are fatal
		Some(Err(sp_inherents::Error::Application(Box::from(String::from("Error processing author inherent")))))
	}
}
