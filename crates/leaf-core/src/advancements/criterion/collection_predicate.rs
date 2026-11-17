//! Перенос `net.minecraft.advancements.criterion.CollectionPredicate` /
//! `CollectionContentsPredicate` / `CollectionCountsPredicate` —
//! предикаты коллекций для `BundlePredicate`, `ContainerPredicate`,
//! `FireworksPredicate`, `*BookPredicate`, `AttributeModifiersPredicate`.
//!
//! Java-типы `Single`/`Multiple`/`Zero` — лишь обёртки над списком;
//! NBT-форма: `contains` — список тестов, `count` — список
//! `{test, count}`, `size` — MinMaxBounds.Ints.

use crate::advancements::criterion::min_max_bounds::MinMaxBoundsInts;
use crate::advancements::criterion::nbt_util;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;

/// Тест элемента коллекции: matching + NBT-кодек (CODEC-предиката).
pub trait CollectionTest<T>: Clone {
    /// `test(value)`.
    fn test(&self, value: &T) -> bool;
    /// CODEC → NBT `Tag`.
    fn encode_nbt(&self) -> Tag;
    /// CODEC ← NBT `Tag`.
    fn decode_nbt(tag: &Tag) -> Result<Self, String>
    where
        Self: Sized;
}

/// `CollectionCountsPredicate.Entry` — (test, count bounds).
#[derive(Clone, Debug, PartialEq)]
pub struct CountEntry<T, P: CollectionTest<T>> {
    /// `test`.
    pub test: P,
    /// `count`.
    pub count: MinMaxBoundsInts,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T, P: CollectionTest<T>> CountEntry<T, P> {
    /// `new`.
    pub fn new(test: P, count: MinMaxBoundsInts) -> Self {
        Self { test, count, _phantom: std::marker::PhantomData }
    }

    /// `Entry.CODEC` → NBT: `{test, count}`.
    pub fn encode_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        c.put("test", self.test.encode_nbt());
        c.put("count", self.count.encode());
        Tag::Compound(c)
    }

    /// `Entry.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "count entry")?;
        let test_tag = nbt_util::field(c, "test")?;
        let test = P::decode_nbt(test_tag)?;
        let count_tag = nbt_util::field(c, "count")?;
        let count = MinMaxBoundsInts::decode(count_tag)?;
        Ok(Self::new(test, count))
    }
}

/// `CollectionPredicate<T, P>` — (contains?, count?, size?).
#[derive(Clone, Debug, PartialEq)]
pub struct CollectionPredicate<T, P: CollectionTest<T>> {
    /// `contains` — `CollectionContentsPredicate` (listOf(testCodec)).
    pub contains: Option<Vec<P>>,
    /// `count` — `CollectionCountsPredicate` (listOf(Entry)).
    pub counts: Option<Vec<CountEntry<T, P>>>,
    /// `size` — `MinMaxBounds.Ints`.
    pub size: Option<MinMaxBoundsInts>,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T, P: CollectionTest<T>> CollectionPredicate<T, P> {
    /// `new`.
    pub fn new(
        contains: Option<Vec<P>>,
        counts: Option<Vec<CountEntry<T, P>>>,
        size: Option<MinMaxBoundsInts>,
    ) -> Self {
        Self { contains, counts, size, _phantom: std::marker::PhantomData }
    }

    /// `test(collection)` — все условия.
    pub fn test<I>(&self, collection: I) -> bool
    where
        I: IntoIterator<Item = T>,
        T: Clone,
    {
        let items: Vec<T> = collection.into_iter().collect();
        if let Some(tests) = &self.contains {
            // `contains`: каждый тест должен найти хотя бы один элемент.
            for test in tests {
                if !items.iter().any(|item| test.test(item)) {
                    return false;
                }
            }
        }
        if let Some(entries) = &self.counts {
            for entry in entries {
                let mut n = 0i32;
                for item in &items {
                    if entry.test.test(item) {
                        n += 1;
                    }
                }
                // `Entry.test` — `count.matches(i)` по int-границам.
                if !entry.count.matches(n) {
                    return false;
                }
            }
        }
        if let Some(size) = &self.size {
            if !size.matches(items.len() as i32) {
                return false;
            }
        }
        true
    }

    /// `CollectionPredicate.CODEC` → NBT.
    pub fn encode_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        if let Some(tests) = &self.contains {
            c.put(
                "contains",
                nbt_util::encode_list(tests, |t| t.encode_nbt()),
            );
        }
        if let Some(entries) = &self.counts {
            c.put(
                "count",
                nbt_util::encode_list(entries, |e| e.encode_nbt()),
            );
        }
        if let Some(size) = &self.size {
            c.put("size", size.encode());
        }
        Tag::Compound(c)
    }

    /// `CollectionPredicate.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "collection predicate")?;
        let contains = match c.get("contains") {
            Some(t) => Some(nbt_util::get_list(t, "contains")?.iter()
                .map(P::decode_nbt)
                .collect::<Result<Vec<_>, _>>()?),
            None => None,
        };
        let counts = match c.get("count") {
            Some(t) => Some(nbt_util::get_list(t, "count")?.iter()
                .map(CountEntry::<T, P>::decode_nbt)
                .collect::<Result<Vec<_>, _>>()?),
            None => None,
        };
        let size = match c.get("size") {
            Some(t) => Some(MinMaxBoundsInts::decode(t)?),
            None => None,
        };
        Ok(Self::new(contains, counts, size))
    }
}
