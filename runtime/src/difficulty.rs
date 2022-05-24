//! A difficuty adjustment algorithm (DAA) to keep the block time close to a particular goal
//! Cribbed from Kulupu https://github.com/kulupu/kulupu/blob/master/runtime/src/difficulty.rs
//!
//! It is possible to implement other DAAs such as that of BTC and BCH. This would be an interesting
//! and worth-while experiment. The DAAs should be abstracted away with a trait.
//! Some ideas: https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3410460

use core::cmp::{min, max};
use parity_scale_codec::{Encode, Decode};
use sp_core::U256;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Debug)]
pub struct DifficultyAndTimestamp<M> {
	pub difficulty: Difficulty,
	pub timestamp: M,
}

/// Move value linearly toward a goal
pub fn damp(actual: u128, goal: u128, damp_factor: u128) -> u128 {
	(actual + (damp_factor - 1) * goal) / damp_factor
}

/// Limit value to be within some factor from a goal
pub fn clamp(actual: u128, goal: u128, clamp_factor: u128) -> u128 {
	max(goal / clamp_factor, min(actual, goal * clamp_factor))
}

const DIFFICULTY_ADJUST_WINDOW: u128 = 60;
type Difficulty = U256;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Pallet's configuration trait.
	/// Tightly coupled to the timestamp trait because we need it's timestamp information
	pub trait Config: frame_system::Trait {
		/// A Source for timestamp data
		type TimeProvider: Time;
		/// The block time that the DAA will attempt to maintain
		type TargetBlockTime: Get<u128>;
		/// Dampening factor to use for difficulty adjustment
		type DampFactor: Get<u128>;
		/// Clamp factor to use for difficulty adjustment
		/// Limit value to within this factor of goal. Recommended value: 2
		type ClampFactor: Get<u128>;
		/// The maximum difficulty allowed. Recommended to use u128::max_value()
		type MaxDifficulty: Get<u128>;
		/// Minimum difficulty, enforced in difficulty retargetting
		/// avoids getting stuck when trying to increase difficulty subject to dampening
		/// Recommended to use same value as DampFactor
		type MinDifficulty: Get<u128>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Past difficulties and timestamps, from earliest to latest.
	#[pallet::storage]
	pub type PastDifficultiesAndTimestamps<T>: StorageValue<
		_,
		[Option<DifficultyAndTimestamp<<<T as Trait>::TimeProvider as Time>::Moment>>; 60],
		ValueQuery
	>;
	
	
	/// Current difficulty.
	#[pallet::storage]
	#[pallet::getter(fn difficulty)]
	pub type CurrentDifficulty = StorageValue<_, Difficulty, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_difficulty: Difficlty,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Initialize the past data to all NONE
			PastDifficultiesAndTimestamps::put([None; DIFFICULTY_ADJUST_WINDOW as usize]);

			// Store the intial difficulty
			CurrentDifficulty::put(self.initial_difficulty);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: T::BlockNumber) {
			let mut data = PastDifficultiesAndTimestamps::<T>::get();

			for i in 1..data.len() {
				data[i - 1] = data[i];
			}

			data[data.len() - 1] = Some(DifficultyAndTimestamp {
				timestamp: T::TimeProvider::now(),
				difficulty: Self::difficulty(),
			});

			let mut ts_delta = 0;
			for i in 1..(DIFFICULTY_ADJUST_WINDOW as usize) {
				let prev: Option<u128> = data[i - 1].map(|d| d.timestamp.unique_saturated_into());
				let cur: Option<u128> = data[i].map(|d| d.timestamp.unique_saturated_into());

				let delta = match (prev, cur) {
					(Some(prev), Some(cur)) => cur.saturating_sub(prev),
					_ => T::TargetBlockTime::get(),
				};
				ts_delta += delta;
			}

			if ts_delta == 0 {
				ts_delta = 1;
			}

			let mut diff_sum = U256::zero();
			for i in 0..(DIFFICULTY_ADJUST_WINDOW as usize) {
				let diff = match data[i].map(|d| d.difficulty) {
					Some(diff) => diff,
					None => InitialDifficulty::get(),
				};
				diff_sum += diff;
			}

			if diff_sum < U256::from(T::MinDifficulty::get()) {
				diff_sum = U256::from(T::MinDifficulty::get());
			}

			// Calculate the average length of the adjustment window
			let adjustment_window = DIFFICULTY_ADJUST_WINDOW * T::TargetBlockTime::get();

			// adjust time delta toward goal subject to dampening and clamping
			let adj_ts = clamp(
				damp(ts_delta, adjustment_window, T::DampFactor::get()),
				adjustment_window,
				T::ClampFactor::get(),
			);

			// minimum difficulty avoids getting stuck due to dampening
			let difficulty = min(U256::from(T::MaxDifficulty::get()),
								 max(U256::from(T::MinDifficulty::get()),
									 diff_sum * U256::from(T::TargetBlockTime::get()) / U256::from(adj_ts)));

			<PastDifficultiesAndTimestamps<T>>::put(data);
			<CurrentDifficulty>::put(difficulty);
		}
	}
}
