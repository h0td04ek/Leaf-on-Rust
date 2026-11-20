//! Перенос `net.minecraft.world.level.levelgen.WorldGenerationContext`
//! + `world.level.levelgen.heightproviders.*` — провайдеры высоты
//! (constant/uniform/biased_to_bottom/very_biased_to_bottom/
//! trapezoid/weighted_list).

use crate::random::RandomSource;
use crate::util::mth;
use crate::util::random::weighted_list::WeightedList;
use super::vertical_anchor::VerticalAnchor;

/// `WorldGenerationContext` — minY/height генерации (пересечение
/// level и chunkGenerator диапазонов).
#[derive(Clone, Copy, Debug)]
pub struct WorldGenerationContext {
    min_y: i32,
    height: i32,
}

impl WorldGenerationContext {
    /// `new(minY, height)` — уже вычисленные границы.
    pub fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    /// `new(generator, level)` — `max(level.minY, generator.minY)`,
    /// `min(level.height, generator.genDepth)`. У нас generator
    /// доступен через `PoolGeneratorAccess`-совместимые значения —
    /// вызывающий код передаёт уже разрешённые границы.
    pub fn from_bounds(level_min_y: i32, level_height: i32, generator_min_y: i32, generator_gen_depth: i32) -> Self {
        Self {
            min_y: level_min_y.max(generator_min_y),
            height: level_height.min(generator_gen_depth),
        }
    }

    /// `getMinGenY`.
    pub fn get_min_gen_y(&self) -> i32 {
        self.min_y
    }

    /// `getGenDepth`.
    pub fn get_gen_depth(&self) -> i32 {
        self.height
    }
}

/// `HeightProvider` — провайдер высоты (enum-варианты вместо
/// sealed-иерархии Java).
#[derive(Clone)]
pub enum HeightProvider {
    /// `ConstantHeight`.
    Constant(VerticalAnchor),
    /// `UniformHeight`.
    Uniform {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
    },
    /// `BiasedToBottomHeight`.
    BiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    /// `VeryBiasedToBottomHeight`.
    VeryBiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    /// `TrapezoidHeight`.
    Trapezoid {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        plateau: i32,
    },
    /// `WeightedListHeight`.
    WeightedList(WeightedList<HeightProvider>),
}

impl HeightProvider {
    /// `ConstantHeight.of`.
    pub fn constant(value: VerticalAnchor) -> Self {
        Self::Constant(value)
    }

    /// `UniformHeight.of`.
    pub fn uniform(min_inclusive: VerticalAnchor, max_inclusive: VerticalAnchor) -> Self {
        Self::Uniform { min_inclusive, max_inclusive }
    }

    /// `BiasedToBottomHeight.of`.
    pub fn biased_to_bottom(
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    ) -> Self {
        Self::BiasedToBottom { min_inclusive, max_inclusive, inner }
    }

    /// `VeryBiasedToBottomHeight.of`.
    pub fn very_biased_to_bottom(
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    ) -> Self {
        Self::VeryBiasedToBottom { min_inclusive, max_inclusive, inner }
    }

    /// `TrapezoidHeight.of`.
    pub fn trapezoid(
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        plateau: i32,
    ) -> Self {
        Self::Trapezoid { min_inclusive, max_inclusive, plateau }
    }

    /// `WeightedListHeight`.
    pub fn weighted_list(distribution: WeightedList<HeightProvider>) -> Self {
        Self::WeightedList(distribution)
    }

    /// `ConstantHeight.ZERO`.
    pub fn zero() -> Self {
        Self::Constant(VerticalAnchor::absolute(0))
    }

    /// `sample(random, context)`.
    pub fn sample(&self, random: &mut dyn RandomSource, context: &WorldGenerationContext) -> i32 {
        match self {
            Self::Constant(value) => value.resolve_y(context),
            Self::Uniform { min_inclusive, max_inclusive } => {
                let i = min_inclusive.resolve_y(context);
                let i1 = max_inclusive.resolve_y(context);
                if i > i1 {
                    i
                } else {
                    mth::random_between_inclusive(random, i, i1)
                }
            }
            Self::BiasedToBottom { min_inclusive, max_inclusive, inner } => {
                let i = min_inclusive.resolve_y(context);
                let i1 = max_inclusive.resolve_y(context);
                if i1 - i - inner + 1 <= 0 {
                    i
                } else {
                    let random_int = random.next_int_bound(i1 - i - inner + 1);
                    random.next_int_bound(random_int + inner) + i
                }
            }
            Self::VeryBiasedToBottom { min_inclusive, max_inclusive, inner } => {
                let i = min_inclusive.resolve_y(context);
                let i1 = max_inclusive.resolve_y(context);
                if i1 - i - inner + 1 <= 0 {
                    i
                } else {
                    let random_int = mth::next_int(random, i + inner, i1);
                    let random_int1 = mth::next_int(random, i, random_int - 1);
                    mth::next_int(random, i, random_int1 - 1 + inner)
                }
            }
            Self::Trapezoid { min_inclusive, max_inclusive, plateau } => {
                let i = min_inclusive.resolve_y(context);
                let i1 = max_inclusive.resolve_y(context);
                if i > i1 {
                    i
                } else {
                    let i2 = i1 - i;
                    if *plateau >= i2 {
                        mth::random_between_inclusive(random, i, i1)
                    } else {
                        let i3 = (i2 - plateau) / 2;
                        let i4 = i2 - i3;
                        i + mth::random_between_inclusive(random, 0, i4)
                            + mth::random_between_inclusive(random, 0, i3)
                    }
                }
            }
            Self::WeightedList(distribution) => {
                distribution.get_random_or_throw(random).sample(random, context)
            }
        }
    }
}

impl std::fmt::Debug for HeightProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(v) => write!(f, "{} absolute", match v { VerticalAnchor::Absolute(y) => *y, _ => 0 }),
            Self::Uniform { .. } => write!(f, "[uniform]"),
            Self::BiasedToBottom { inner, .. } => write!(f, "biased[inner: {}]", inner),
            Self::VeryBiasedToBottom { inner, .. } => write!(f, "biased[inner: {}]", inner),
            Self::Trapezoid { plateau, .. } => {
                if *plateau == 0 {
                    write!(f, "triangle")
                } else {
                    write!(f, "trapezoid({})", plateau)
                }
            }
            Self::WeightedList(_) => write!(f, "weighted_list"),
        }
    }
}
