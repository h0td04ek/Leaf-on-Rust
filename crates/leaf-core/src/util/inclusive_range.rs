//! Перенос `net.minecraft.util.InclusiveRange<T>` (record).

/// `InclusiveRange(minInclusive, maxInclusive)` — включающий диапазон.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InclusiveRange<T: Ord + Clone> {
    pub min_inclusive: T,
    pub max_inclusive: T,
}

impl<T: Ord + Clone> InclusiveRange<T> {
    pub fn new(min_inclusive: T, max_inclusive: T) -> Result<Self, String> {
        if min_inclusive > max_inclusive {
            Err("min_inclusive must be less than or equal to max_inclusive".to_string())
        } else {
            Ok(Self { min_inclusive, max_inclusive })
        }
    }

    pub fn single(value: T) -> Self {
        Self { min_inclusive: value.clone(), max_inclusive: value }
    }

    /// `create(min, max)` → DataResult-аналог: Result.
    pub fn create(min: T, max: T) -> Result<Self, String> {
        Self::new(min, max)
    }

    /// `map(Function)`
    pub fn map<S: Ord + Clone>(&self, mapper: impl Fn(&T) -> S) -> InclusiveRange<S> {
        InclusiveRange {
            min_inclusive: mapper(&self.min_inclusive),
            max_inclusive: mapper(&self.max_inclusive),
        }
    }

    pub fn is_value_in_range(&self, value: &T) -> bool {
        *value >= self.min_inclusive && *value <= self.max_inclusive
    }

    /// `contains(InclusiveRange)`
    pub fn contains_range(&self, value: &InclusiveRange<T>) -> bool {
        value.min_inclusive >= self.min_inclusive && value.max_inclusive <= self.max_inclusive
    }
}

impl<T: Ord + Clone> std::fmt::Display for InclusiveRange<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min_inclusive, self.max_inclusive)
    }
}
