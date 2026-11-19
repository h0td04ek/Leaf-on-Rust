//! Перенос `net.minecraft.world.level.levelgen.feature.stateproviders`
//! — провайдеры `BlockState` для генерации фич (7 типов + rule-based).

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::random::RandomSource;
use crate::util::mth;
use crate::util::random::weighted_list::WeightedList;
use crate::util::valueproviders::int_provider::IntProvider;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::levelgen::synth::normal_noise::{NoiseParameters, NormalNoise};
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;

/// `BlockStateProvider` — tagged union провайдеров состояний.
#[derive(Clone)]
pub enum StateProvider {
    /// `SimpleStateProvider`.
    Simple(BlockState),
    /// `WeightedStateProvider` — `WeightedList<BlockState>`.
    Weighted(WeightedList<BlockState>),
    /// `RotatedBlockProvider` — случайная `axis`-ось.
    RotatedBlock(Arc<crate::world::level::block::Block>),
    /// `RandomizedIntStateProvider`.
    RandomizedInt {
        source: Box<StateProvider>,
        property_name: String,
        values: Arc<dyn IntProvider>,
    },
    /// `NoiseProvider`.
    Noise {
        noise: NormalNoise,
        scale: f32,
        states: Vec<BlockState>,
    },
    /// `NoiseThresholdProvider`.
    NoiseThreshold {
        noise: NormalNoise,
        scale: f32,
        threshold: f32,
        high_chance: f32,
        default_state: BlockState,
        low_states: Vec<BlockState>,
        high_states: Vec<BlockState>,
    },
    /// `DualNoiseProvider`.
    DualNoise {
        variety_min: i32,
        variety_max: i32,
        slow_noise: NormalNoise,
        slow_scale: f32,
        noise: NormalNoise,
        scale: f32,
        states: Vec<BlockState>,
    },
    /// `RuleBasedBlockStateProvider`.
    RuleBased {
        fallback: Box<StateProvider>,
        rules: Vec<Rule>,
    },
}

/// `RuleBasedBlockStateProvider.Rule` — `ifTrue`-предикат + провайдер.
#[derive(Clone)]
pub struct Rule {
    /// `ifTrue` — `BlockPredicate` levelgen (см. `blockpredicates`).
    pub if_true: crate::world::level::levelgen::blockpredicates::BlockPredicate,
    /// `then`.
    pub then: StateProvider,
}

impl StateProvider {
    /// `SimpleStateProvider`.
    pub fn simple(state: BlockState) -> Self {
        Self::Simple(state)
    }

    /// `WeightedStateProvider`.
    pub fn weighted(weighted_list: WeightedList<BlockState>) -> Self {
        Self::Weighted(weighted_list)
    }

    /// `RotatedBlockProvider`.
    pub fn rotated_block(block: Arc<crate::world::level::block::Block>) -> Self {
        Self::RotatedBlock(block)
    }

    /// `RandomizedIntStateProvider`.
    pub fn randomized_int(
        source: StateProvider,
        property_name: impl Into<String>,
        values: Arc<dyn IntProvider>,
    ) -> Self {
        Self::RandomizedInt {
            source: Box::new(source),
            property_name: property_name.into(),
            values,
        }
    }

    /// Общий конструктор noise-провайдеров: `NormalNoise.create`
    /// с `WorldgenRandom(seed)` (vanilla использует `LegacyRandomSource`).
    fn noise_from_seed(
        seed: i64,
        parameters: NoiseParameters,
    ) -> NormalNoise {
        // Java: `new WorldgenRandom(new LegacyRandomSource(seed))`.
        let mut rng = WorldgenRandom::new(Box::new(
            crate::random::legacy::LegacyRandomSource::new(seed),
        ));
        NormalNoise::create_with_params(&mut rng, parameters)
    }

    /// `NoiseProvider` — `seed`, `noise` params, `scale`, `states`.
    pub fn noise(
        seed: i64,
        parameters: NoiseParameters,
        scale: f32,
        states: Vec<BlockState>,
    ) -> Self {
        Self::Noise {
            noise: Self::noise_from_seed(seed, parameters),
            scale,
            states,
        }
    }

    /// `NoiseThresholdProvider`.
    #[allow(clippy::too_many_arguments)]
    pub fn noise_threshold(
        seed: i64,
        parameters: NoiseParameters,
        scale: f32,
        threshold: f32,
        high_chance: f32,
        default_state: BlockState,
        low_states: Vec<BlockState>,
        high_states: Vec<BlockState>,
    ) -> Self {
        Self::NoiseThreshold {
            noise: Self::noise_from_seed(seed, parameters),
            scale,
            threshold,
            high_chance,
            default_state,
            low_states,
            high_states,
        }
    }

    /// `DualNoiseProvider`.
    #[allow(clippy::too_many_arguments)]
    pub fn dual_noise(
        variety_min: i32,
        variety_max: i32,
        slow_parameters: NoiseParameters,
        slow_scale: f32,
        seed: i64,
        parameters: NoiseParameters,
        scale: f32,
        states: Vec<BlockState>,
    ) -> Self {
        Self::DualNoise {
            variety_min,
            variety_max,
            slow_noise: Self::noise_from_seed(seed, slow_parameters),
            slow_scale,
            noise: Self::noise_from_seed(seed, parameters),
            scale,
            states,
        }
    }

    /// `RuleBasedBlockStateProvider`.
    pub fn rule_based(
        fallback: StateProvider,
        rules: Vec<Rule>,
    ) -> Self {
        Self::RuleBased {
            fallback: Box::new(fallback),
            rules,
        }
    }

    /// `getState(random, pos)`.
    pub fn get_state(&self, random: &mut dyn RandomSource, pos: BlockPos) -> BlockState {
        match self {
            StateProvider::Simple(state) => state.clone(),
            StateProvider::Weighted(list) => list
                .get_random_or_throw(random)
                .clone(),
            StateProvider::RotatedBlock(block) => {
                // `defaultBlockState().trySetValue(AXIS, randomAxis)`.
                let mut state = block
                    .default_block_state()
                    .unwrap_or_else(crate::world::level::block::state::block_state::BlockState::air);
                let axis = crate::core::direction::Axis::VALUES
                    [random.next_int_bound(3) as usize];
                state
                    .values
                    .set_erased("axis", crate::world::level::block::state::property::StateValue::Str(
                        axis.name().to_string(),
                    ));
                state
            }
            StateProvider::RandomizedInt {
                source,
                property_name,
                values,
            } => {
                let mut state = source.get_state(random, pos);
                let value = values.sample(random);
                // Java кэширует IntegerProperty после первого
                // разрешения; мы ищем по имени каждый раз
                // (эквивалентно по результату).
                state
                    .values
                    .set_erased(property_name, crate::world::level::block::state::property::StateValue::Int(value));
                state
            }
            StateProvider::Noise { noise, scale, states } => {
                let noise_value =
                    noise.get_value(pos.x as f64 * *scale as f64, pos.y as f64 * *scale as f64, pos.z as f64 * *scale as f64);
                Self::random_state_from_noise(states, noise_value).clone()
            }
            StateProvider::NoiseThreshold {
                noise,
                scale,
                threshold,
                high_chance,
                default_state,
                low_states,
                high_states,
            } => {
                let noise_value =
                    noise.get_value(pos.x as f64 * *scale as f64, pos.y as f64 * *scale as f64, pos.z as f64 * *scale as f64);
                if noise_value < *threshold as f64 {
                    low_states
                        .get(random.next_int_bound(low_states.len() as i32) as usize)
                        .cloned()
                        .unwrap_or_else(|| default_state.clone())
                } else if random.next_float() < *high_chance {
                    high_states
                        .get(random.next_int_bound(high_states.len() as i32) as usize)
                        .cloned()
                        .unwrap_or_else(|| default_state.clone())
                } else {
                    default_state.clone()
                }
            }
            StateProvider::DualNoise {
                variety_min,
                variety_max,
                slow_noise,
                slow_scale,
                noise,
                scale,
                states,
            } => {
                let slow = slow_noise.get_value(
                    pos.x as f64 * *slow_scale as f64,
                    pos.y as f64 * *slow_scale as f64,
                    pos.z as f64 * *slow_scale as f64,
                );
                // `clampedMap(slow, -1, 1, min, max+1)`.
                let i = mth::clamped_map_f64(
                    slow,
                    -1.0,
                    1.0,
                    *variety_min as f64,
                    (*variety_max + 1) as f64,
                ) as i32;
                // Собираем i случайных состояний по оффсетам
                // `pos.offset(i1*54545, 0, i1*34234)`.
                let mut list: Vec<BlockState> = Vec::with_capacity(i.max(0) as usize);
                for i1 in 0..i {
                    let offset_pos = pos.offset(i1 * 54545, 0, i1 * 34234);
                    let slow_v = slow_noise.get_value(
                        offset_pos.x as f64 * *slow_scale as f64,
                        offset_pos.y as f64 * *slow_scale as f64,
                        offset_pos.z as f64 * *slow_scale as f64,
                    );
                    list.push(
                        Self::random_state_from_noise(states, slow_v).clone(),
                    );
                }
                // `getRandomState(list, pos, scale)` — шум по pos
                // со списком list (может быть пустым → states).
                let noise_value = noise.get_value(
                    pos.x as f64 * *scale as f64,
                    pos.y as f64 * *scale as f64,
                    pos.z as f64 * *scale as f64,
                );
                let source_list: &[BlockState] = if list.is_empty() { states } else { &list };
                Self::random_state_from_noise(source_list, noise_value).clone()
            }
            StateProvider::RuleBased { fallback, rules } => {
                // RuleBased требует WorldGenLevel для предикатов —
                // без уровня применяем fallback (как `getState`
                // вызывается вне контекста уровня в Java через
                // `getState(random, pos)` → rules.test(level,pos)
                // недоступен). Вызывающий код должен использовать
                // `get_state_with_level` при наличии уровня.
                fallback.get_state(random, pos)
            }
        }
    }

    /// `getState` с уровнем — для `RuleBasedBlockStateProvider`
    /// (правила проверяются `BlockPredicate.test(level,pos)`).
    pub fn get_state_with_level(
        &self,
        random: &mut dyn RandomSource,
        pos: BlockPos,
        level: &dyn LevelReader,
    ) -> BlockState {
        match self {
            StateProvider::RuleBased { fallback, rules } => {
                for rule in rules {
                    if rule.if_true.test(level, pos) {
                        return rule.then.get_state_with_level(random, pos, level);
                    }
                }
                fallback.get_state_with_level(random, pos, level)
            }
            other => other.get_state(random, pos),
        }
    }

    /// `getRandomState(states, noiseValue)` —
    /// `clamp((1+delta)/2, 0, 0.9999) * size`.
    fn random_state_from_noise(states: &[BlockState], noise_value: f64) -> &BlockState {
        let d = ((1.0 + noise_value) / 2.0).clamp(0.0, 0.9999);
        let idx = (d * states.len() as f64) as usize;
        &states[idx.min(states.len() - 1)]
    }
}

use crate::world::level::level_reader::LevelReader;
