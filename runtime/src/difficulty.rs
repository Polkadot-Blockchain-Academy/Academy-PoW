//! A difficulty adjustment algorithm (DAA) to keep the block time close to a particular goal
//! Cribbed from Kulupu https://github.com/kulupu/kulupu/blob/master/runtime/src/difficulty.rs
//!
//! It is possible to implement other DAAs such as that of BTC and BCH. This would be an interesting
//! and worth-while experiment. The DAAs should be abstracted away with a trait.
//! Some ideas: https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3410460

use core::cmp::{max, min};

use frame_support::traits::Time;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::traits::UniqueSaturatedInto;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    /// Pallet's configuration trait.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
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

        /// Now that the pallet is instantiable, we need a way to decide which blocks are
        /// relevant to this instance. This function does just that.
        /// 
        /// The default implementation assumes that all blocks are relevant which is what
        /// you probably want when there is only a single instance.
        fn relevant_to_this_instance() -> bool {
            true
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    type DifficultyList<T, I> =
        [Option<DifficultyAndTimestamp<<<T as Config<I>>::TimeProvider as Time>::Moment>>; 60];

    /// Past difficulties and timestamps, from earliest to latest.
    #[pallet::storage]
    pub type PastDifficultiesAndTimestamps<T: Config<I>, I: 'static = ()> =
        StorageValue<_, DifficultyList<T, I>, ValueQuery, EmptyList<T, I>>;

    pub struct EmptyList<T: Config<I>, I: 'static = ()>(PhantomData<(T, I)>);
    impl<T: Config<I>, I> Get<DifficultyList<T, I>> for EmptyList<T, I> {
        fn get() -> DifficultyList<T, I> {
            [None; DIFFICULTY_ADJUST_WINDOW as usize]
        }
    }

    /// Current difficulty.
    #[pallet::storage]
    #[pallet::getter(fn difficulty)]
    pub type CurrentDifficulty<T: Config<I>, I: 'static = ()> = StorageValue<_, Difficulty, ValueQuery>;

    /// Initial difficulty.
    #[pallet::storage]
    pub type InitialDifficulty<T: Config<I>, I: 'static = ()> = StorageValue<_, Difficulty, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub _ph_data: PhantomData<(T, I)>,
        pub initial_difficulty: Difficulty,
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
        fn build(&self) {
            // Initialize the Current difficulty
            CurrentDifficulty::<T, I>::put(self.initial_difficulty);

            // Store the initial difficulty in storage because we will need it
            // during the first DIFFICULTY_ADJUSTMENT_WINDOW blocks (see todo below).
            InitialDifficulty::<T, I>::put(self.initial_difficulty);
        }
    }

    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            GenesisConfig {
                _ph_data: Default::default(),
                initial_difficulty: 4_000_000.into(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            // First check if this is block is relevant to this instance of the difficulty adjustment algorithm
            if !T::relevant_to_this_instance() {
                return
            }

            let mut data = PastDifficultiesAndTimestamps::<T, I>::get();

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
            //TODO Could we just initialize every array cell to the initial difficulty to not need the
            // separate storage item?
            for item in data.iter().take(DIFFICULTY_ADJUST_WINDOW as usize) {
                let diff = match item.map(|d| d.difficulty) {
                    Some(diff) => diff,
                    None => InitialDifficulty::<T, I>::get(),
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
            let difficulty = min(
                U256::from(T::MaxDifficulty::get()),
                max(
                    U256::from(T::MinDifficulty::get()),
                    diff_sum * U256::from(T::TargetBlockTime::get()) / U256::from(adj_ts),
                ),
            );

            <PastDifficultiesAndTimestamps<T, I>>::put(data);
            <CurrentDifficulty<T, I>>::put(difficulty);
        }
    }
}
