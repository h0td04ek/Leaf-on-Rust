//! Перенос `net.minecraft.util.SingleKeyCache<K, V>`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// `SingleKeyCache` — кэш одного ключа: пересчёт только при смене ключа.
/// В Java сравнение через equals; здесь K: Eq + Hash.
pub struct SingleKeyCache<K, V> {
    compute_value: Box<dyn Fn(&K) -> V>,
    cache_key: Option<K>,
    cached_value: Option<V>,
}

impl<K: Eq + Hash, V> SingleKeyCache<K, V> {
    pub fn new(compute_value: impl Fn(&K) -> V + 'static) -> Self {
        Self { compute_value: Box::new(compute_value), cache_key: None, cached_value: None }
    }

    /// `getValue(cacheKey)`.
    pub fn get_value(&mut self, cache_key: K) -> &V {
        let stale = match &self.cache_key {
            Some(k) => *k != cache_key,
            None => true,
        };
        if self.cached_value.is_none() || stale {
            self.cached_value = Some((self.compute_value)(&cache_key));
            self.cache_key = Some(cache_key);
        }
        self.cached_value.as_ref().unwrap()
    }

    /// Версия без Eq/Hash — по хэшу (как identityHashCode в Java).
    pub fn get_value_by_hash(&mut self, cache_key: K, key_hash: u64) -> &V {
        let stale = match &self.cache_key {
            Some(k) => {
                let mut h = DefaultHasher::new();
                k.hash(&mut h);
                h.finish() != key_hash
            }
            None => true,
        };
        if self.cached_value.is_none() || stale {
            self.cached_value = Some((self.compute_value)(&cache_key));
            self.cache_key = Some(cache_key);
        }
        self.cached_value.as_ref().unwrap()
    }
}
