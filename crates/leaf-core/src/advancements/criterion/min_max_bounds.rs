//! Перенос `net.minecraft.advancements.criterion.MinMaxBounds` —
//! границы min/max для int/double/float в NBT-кодеках предикатов.
//!
//! NBT-форма `Bounds.CODEC` (`either(bounds{min,max}, value)`):
//! - точка (min==max) → одно число;
//! - иначе → compound `{min?, max?}` (поля опускаются при None).

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;

/// `MinMaxBounds.Bounds<T>` — (min?, max?).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds<T> {
    /// `min`.
    pub min: Option<T>,
    /// `max`.
    pub max: Option<T>,
}

impl<T: Eq> Eq for Bounds<T> {}

impl<T: PartialOrd + Copy> Bounds<T> {
    /// `any()`.
    pub fn any() -> Self {
        Self { min: None, max: None }
    }

    /// `exactly(value)`.
    pub fn exactly(value: T) -> Self {
        Self { min: Some(value), max: Some(value) }
    }

    /// `between(min, max)`.
    pub fn between(min: T, max: T) -> Self {
        Self { min: Some(min), max: Some(max) }
    }

    /// `atLeast(min)`.
    pub fn at_least(min: T) -> Self {
        Self { min: Some(min), max: None }
    }

    /// `atMost(max)`.
    pub fn at_most(max: T) -> Self {
        Self { min: None, max: Some(max) }
    }

    /// `isAny`.
    pub fn is_any(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    /// `asPoint` — Some если min==max.
    pub fn as_point(&self) -> Option<T> {
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => Some(a),
            _ => None,
        }
    }
}

/// Числовое значение из `Tag` (NBTOps принимает любые числовые теги
/// для int/double кодеков — проверяем влезание в диапазон).
fn tag_as_i64(tag: &Tag) -> Option<i64> {
    match tag {
        Tag::Byte(v) => Some(*v as i64),
        Tag::Short(v) => Some(*v as i64),
        Tag::Int(v) => Some(*v as i64),
        Tag::Long(v) => Some(*v),
        _ => None,
    }
}

fn tag_as_f64(tag: &Tag) -> Option<f64> {
    match tag {
        Tag::Byte(v) => Some(*v as f64),
        Tag::Short(v) => Some(*v as f64),
        Tag::Int(v) => Some(*v as f64),
        Tag::Long(v) => Some(*v as f64),
        Tag::Float(v) => Some(*v as f64),
        Tag::Double(v) => Some(*v),
        _ => None,
    }
}

/// NBT-кодек `MinMaxBounds.Ints` (`Bounds<Integer>` + boundsSqr).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinMaxBoundsInts {
    /// `bounds`.
    pub bounds: Bounds<i32>,
    /// `boundsSqr` — `bounds.map(Mth::square)` (i64).
    pub bounds_sqr: Bounds<i64>,
}

impl MinMaxBoundsInts {
    /// `ANY`.
    pub const ANY: MinMaxBoundsInts = MinMaxBoundsInts {
        bounds: Bounds { min: None, max: None },
        bounds_sqr: Bounds { min: None, max: None },
    };

    /// `new(bounds)` — вычисляет boundsSqr.
    pub fn new(bounds: Bounds<i32>) -> Self {
        let sqr = |v: Option<i32>| v.map(|x| (x as i64) * (x as i64));
        Self {
            bounds,
            bounds_sqr: Bounds { min: sqr(bounds.min), max: sqr(bounds.max) },
        }
    }

    /// `exactly`.
    pub fn exactly(value: i32) -> Self {
        Self::new(Bounds::exactly(value))
    }

    /// `between`.
    pub fn between(min: i32, max: i32) -> Self {
        Self::new(Bounds::between(min, max))
    }

    /// `atLeast`.
    pub fn at_least(min: i32) -> Self {
        Self::new(Bounds::at_least(min))
    }

    /// `atMost`.
    pub fn at_most(max: i32) -> Self {
        Self::new(Bounds::at_most(max))
    }

    /// `matches(value)` — `min <= v <= max` по `bounds`.
    pub fn matches(&self, value: i32) -> bool {
        self.bounds.min.map_or(true, |m| m <= value)
            && self.bounds.max.map_or(true, |m| m >= value)
    }

    /// `matchesSqr(value)` — по `boundsSqr` (value² как long).
    pub fn matches_sqr(&self, value: i64) -> bool {
        let sqr = value * value;
        self.bounds_sqr.min.map_or(true, |m| m <= sqr)
            && self.bounds_sqr.max.map_or(true, |m| m >= sqr)
    }

    /// NBT → `MinMaxBounds.Ints` (`Ints.CODEC` — either(int, bounds)).
    pub fn decode(tag: &Tag) -> Result<Self, String> {
        if let Some(v) = tag_as_i64(tag) {
            let v = i32::try_from(v).map_err(|_| "int bounds out of range".to_string())?;
            return Ok(Self::exactly(v));
        }
        let compound = match tag {
            Tag::Compound(c) => c,
            _ => return Err("expected int or compound for MinMaxBounds.Ints".into()),
        };
        let mut bounds = Bounds::any();
        if let Some(t) = compound.get("min") {
            let v = tag_as_i64(t).ok_or("min is not a number")?;
            bounds.min = Some(i32::try_from(v).map_err(|_| "min out of range")?);
        }
        if let Some(t) = compound.get("max") {
            let v = tag_as_i64(t).ok_or("max is not a number")?;
            bounds.max = Some(i32::try_from(v).map_err(|_| "max out of range")?);
        }
        Ok(Self::new(bounds))
    }

    /// `MinMaxBounds.Ints` → NBT.
    pub fn encode(&self) -> Tag {
        if let Some(p) = self.bounds.as_point() {
            return Tag::Int(p);
        }
        let mut c = CompoundTag::new();
        if let Some(min) = self.bounds.min {
            c.put("min", Tag::Int(min));
        }
        if let Some(max) = self.bounds.max {
            c.put("max", Tag::Int(max));
        }
        Tag::Compound(c)
    }
}

/// NBT-кодек `MinMaxBounds.Doubles` (`Bounds<Double>` + boundsSqr).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinMaxBoundsDoubles {
    /// `bounds`.
    pub bounds: Bounds<f64>,
    /// `boundsSqr`.
    pub bounds_sqr: Bounds<f64>,
}

impl MinMaxBoundsDoubles {
    /// `ANY`.
    pub const ANY: MinMaxBoundsDoubles = MinMaxBoundsDoubles {
        bounds: Bounds { min: None, max: None },
        bounds_sqr: Bounds { min: None, max: None },
    };

    /// `new(bounds)`.
    pub fn new(bounds: Bounds<f64>) -> Self {
        let sqr = |v: Option<f64>| v.map(|x| x * x);
        Self {
            bounds,
            bounds_sqr: Bounds { min: sqr(bounds.min), max: sqr(bounds.max) },
        }
    }

    /// `exactly`.
    pub fn exactly(value: f64) -> Self {
        Self::new(Bounds::exactly(value))
    }

    /// `matches(value)`.
    pub fn matches(&self, value: f64) -> bool {
        self.bounds.min.map_or(true, |m| m <= value)
            && self.bounds.max.map_or(true, |m| m >= value)
    }

    /// `Doubles.CODEC` → NBT.
    pub fn decode(tag: &Tag) -> Result<Self, String> {
        if let Some(v) = tag_as_f64(tag) {
            return Ok(Self::exactly(v));
        }
        let compound = match tag {
            Tag::Compound(c) => c,
            _ => return Err("expected double or compound for MinMaxBounds.Doubles".into()),
        };
        let mut bounds = Bounds::any();
        if let Some(t) = compound.get("min") {
            bounds.min = Some(tag_as_f64(t).ok_or("min is not a number")?);
        }
        if let Some(t) = compound.get("max") {
            bounds.max = Some(tag_as_f64(t).ok_or("max is not a number")?);
        }
        Ok(Self::new(bounds))
    }

    /// `Doubles` → NBT.
    pub fn encode(&self) -> Tag {
        if let Some(p) = self.bounds.as_point() {
            return Tag::Double(p);
        }
        let mut c = CompoundTag::new();
        if let Some(min) = self.bounds.min {
            c.put("min", Tag::Double(min));
        }
        if let Some(max) = self.bounds.max {
            c.put("max", Tag::Double(max));
        }
        Tag::Compound(c)
    }
}
