//! Перенос `net.minecraft.util.random.WeightedList<E>`.

use crate::random::RandomSource;
use crate::util::random::weighted::Weighted;
use crate::util::random::weighted_random::WeightedRandom;
use std::fmt;

const FLAT_THRESHOLD: i32 = 64;

/// `WeightedList<E>` — список взвешенных элементов с O(n)/O(log n) выбором.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WeightedList<E> {
    total_weight: i32,
    items: Vec<Weighted<E>>,
    /// Flat — плоский поиск для total < 64; Compact — иначе.
    compact: bool,
}

impl<E: Clone> WeightedList<E> {
    fn new(items: Vec<Weighted<E>>) -> Self {
        let total_weight = WeightedRandom::get_total_weight(&items, |w| w.weight);
        let compact = total_weight >= FLAT_THRESHOLD;
        Self { total_weight, items, compact }
    }

    /// `of()` — пустой список.
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// `of(element)` — один элемент с весом 1.
    pub fn of(element: E) -> Self {
        Self::of_weighted(Weighted::new(element, 1))
    }

    /// `of(Weighted...)`.
    pub fn of_weighted(weighted: Weighted<E>) -> Self {
        Self::new(vec![weighted])
    }

    /// `of(List<Weighted>)`.
    pub fn of_list(items: Vec<Weighted<E>>) -> Self {
        Self::new(items)
    }

    pub fn builder() -> WeightedListBuilder<E> {
        WeightedListBuilder::new()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `map(Function)`
    pub fn map<U: Clone, F: Fn(&E) -> U>(&self, mapper: F) -> WeightedList<U> {
        let items = self
            .items
            .iter()
            .map(|w| Weighted::new(mapper(&w.value), w.weight))
            .collect();
        WeightedList::new(items)
    }

    /// `getRandom(random)` → Option<&E>.
    pub fn get_random<'a>(&'a self, random: &mut dyn RandomSource) -> Option<&'a E> {
        if self.items.is_empty() || self.total_weight == 0 {
            return None;
        }
        let random_int = random.next_int_bound(self.total_weight);
        Some(self.selector_get(random_int))
    }

    /// `getRandomOrThrow(random)` → &E.
    pub fn get_random_or_throw<'a>(&'a self, random: &mut dyn RandomSource) -> &'a E {
        if self.items.is_empty() || self.total_weight == 0 {
            panic!("Weighted list has no elements");
        }
        let random_int = random.next_int_bound(self.total_weight);
        self.selector_get(random_int)
    }

    fn selector_get(&self, random_int: i32) -> &E {
        if self.compact {
            // Compact: плоский проход — как Flat в Java для >=64 тоже корректен
            // семантически (выбор по весам), Compact — оптимизация.
            let mut index = random_int;
            for w in &self.items {
                index -= w.weight;
                if index < 0 {
                    return &w.value;
                }
            }
            unreachable!("index {} exceeded total weight", random_int);
        } else {
            // Flat: кумулятивные границы
            let mut index = random_int;
            for w in &self.items {
                index -= w.weight;
                if index < 0 {
                    return &w.value;
                }
            }
            unreachable!("index {} exceeded total weight", random_int);
        }
    }

    /// `streamCodec` — список `(weight, value)` пар.
    /// Java: `WeightedList.streamCodec(elementCodec)`.
    pub fn stream_codec<E2, C>(
        element_codec: C,
    ) -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf, WeightedList<E2>,
    > + Send + Sync + 'static
    where
        E2: Clone + Send + Sync + 'static,
        C: crate::network::codec::stream_codec::StreamCodec<
            crate::network::friendly_byte_buf::FriendlyByteBuf, E2,
        > + Send + Sync + 'static,
    {
        let codec = std::sync::Arc::new(element_codec);
        let codec2 = codec.clone();
        crate::network::codec::stream_codec::sync_fn(
            move |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, list: &WeightedList<E2>| {
                buf.write_var_int(list.items.len() as i32);
                for w in &list.items {
                    buf.write_var_int(w.weight());
                    codec.encode(buf, w.value());
                }
            },
            move |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                let n = buf.read_var_int();
                let mut items = Vec::with_capacity(n.max(0) as usize);
                for _ in 0..n {
                    let weight = buf.read_var_int();
                    let value = codec2.decode(buf);
                    items.push(Weighted::new(value, weight));
                }
                WeightedList::of_list(items)
            },
        )
    }

    /// `unwrap()` — исходный список Weighted.
    pub fn unwrap(&self) -> &[Weighted<E>] {
        &self.items
    }

    /// `contains(element)` — по equals.
    pub fn contains(&self, element: &E) -> bool
    where
        E: PartialEq,
    {
        self.items.iter().any(|w| &w.value == element)
    }
}

impl<E> fmt::Display for WeightedList<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WeightedList(size={}, totalWeight={})", self.items.len(), self.total_weight)
    }
}

/// `WeightedList.Builder<E>`.
pub struct WeightedListBuilder<E> {
    result: Vec<Weighted<E>>,
}

impl<E> WeightedListBuilder<E> {
    pub fn new() -> Self {
        Self { result: Vec::new() }
    }

    /// `add(element)` — вес 1.
    pub fn add(&mut self, element: E) -> &mut Self {
        self.add_with_weight(element, 1)
    }

    /// `add(element, weight)`.
    pub fn add_with_weight(&mut self, element: E, weight: i32) -> &mut Self {
        self.result.push(Weighted::new(element, weight));
        self
    }

    /// `build()`
    pub fn build(self) -> WeightedList<E>
    where
        E: Clone,
    {
        WeightedList::new(self.result)
    }
}

impl<E> Default for WeightedListBuilder<E> {
    fn default() -> Self {
        Self::new()
    }
}
