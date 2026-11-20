//! Перенос `pools/alias/*` — pool alias binding'и
//! (Direct/Random/RandomGroup) + `PoolAliasLookup.create`.

use super::jigsaw_placement::PoolAliasLookup;
use super::structure_template_pool::StructureTemplatePool;
use crate::resources::resource_key::ResourceKey;
use crate::random::RandomSource;
use crate::random::xoroshiro::XoroshiroRandomSource;
use crate::util::random::weighted::Weighted;
use crate::util::random::weighted_list::{WeightedList, WeightedListBuilder};
use std::collections::HashMap;
use std::rc::Rc;

type PoolKey = ResourceKey<Rc<StructureTemplatePool>>;

/// `PoolAliasBinding` — трейт связывания alias→target.
pub trait PoolAliasBinding {
    /// `forEachResolved(random, consumer)` — выдаёт пары
    /// (alias, target) после разрешения random-выбора.
    fn for_each_resolved(
        &self,
        random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(PoolKey, PoolKey),
    );

    /// `allTargets` — все возможные target-ключи.
    fn all_targets(&self) -> Vec<PoolKey>;
}

/// `DirectPoolAlias` — фиксированный alias→target.
pub struct DirectPoolAlias {
    /// `alias`.
    pub alias: PoolKey,
    /// `target`.
    pub target: PoolKey,
}

impl DirectPoolAlias {
    /// `direct(alias, target)` (id-строки → ключи).
    pub fn direct(alias: &str, target: &str) -> Option<Self> {
        Some(Self {
            alias: super::jigsaw_placement::pool_key(alias)?,
            target: super::jigsaw_placement::pool_key(target)?,
        })
    }
}

impl PoolAliasBinding for DirectPoolAlias {
    fn for_each_resolved(
        &self,
        _random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(PoolKey, PoolKey),
    ) {
        consumer(self.alias.clone(), self.target.clone());
    }

    fn all_targets(&self) -> Vec<PoolKey> {
        vec![self.target.clone()]
    }
}

/// `RandomPoolAlias` — alias → weighted-выбор target.
pub struct RandomPoolAlias {
    /// `alias`.
    pub alias: PoolKey,
    /// `targets`.
    pub targets: WeightedList<PoolKey>,
}

impl RandomPoolAlias {
    /// `random(alias, targets)`.
    pub fn random(alias: &str, targets: Vec<Weighted<PoolKey>>) -> Option<Self> {
        Some(Self {
            alias: super::jigsaw_placement::pool_key(alias)?,
            targets: WeightedList::of_list(targets),
        })
    }
}

impl PoolAliasBinding for RandomPoolAlias {
    fn for_each_resolved(
        &self,
        random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(PoolKey, PoolKey),
    ) {
        if let Some(target) = self.targets.get_random(random) {
            consumer(self.alias.clone(), target.clone());
        }
    }

    fn all_targets(&self) -> Vec<PoolKey> {
        self.targets
            .unwrap()
            .iter()
            .map(|w| w.value().clone())
            .collect()
    }
}

/// `RandomGroupPoolAlias` — выбор группы binding'ов.
pub struct RandomGroupPoolAlias {
    /// `groups`.
    pub groups: WeightedList<Vec<Rc<dyn PoolAliasBinding>>>,
}

impl PoolAliasBinding for RandomGroupPoolAlias {
    fn for_each_resolved(
        &self,
        random: &mut dyn RandomSource,
        consumer: &mut dyn FnMut(PoolKey, PoolKey),
    ) {
        if let Some(group) = self.groups.get_random(random) {
            for binding in group.iter() {
                binding.for_each_resolved(random, consumer);
            }
        }
    }

    fn all_targets(&self) -> Vec<PoolKey> {
        let mut out = Vec::new();
        for group in self.groups.unwrap() {
            for binding in group.value() {
                out.extend(binding.all_targets());
            }
        }
        out
    }
}

/// `PoolAliasLookup.create(aliases, pos, seed)` — строит map
/// alias→target через positional random.
pub fn create_alias_lookup(
    aliases: &[Rc<dyn PoolAliasBinding>],
    pos: crate::core::block_pos::BlockPos,
    seed: i64,
) -> Box<dyn PoolAliasLookup> {
    if aliases.is_empty() {
        return Box::new(super::jigsaw_placement::EmptyPoolAliasLookup);
    }
    let mut base = XoroshiroRandomSource::new(seed);
    let mut positional = base.fork_positional();
    let mut random = positional.at(pos.x, pos.y, pos.z);
    let mut map: HashMap<PoolKey, PoolKey> = HashMap::new();
    for binding in aliases {
        binding.for_each_resolved(random.as_mut(), &mut |alias, target| {
            map.insert(alias, target);
        });
    }
    Box::new(MapAliasLookup { map })
}

/// `PoolAliasLookup` поверх HashMap.
struct MapAliasLookup {
    map: HashMap<PoolKey, PoolKey>,
}

impl PoolAliasLookup for MapAliasLookup {
    fn lookup(&self, key: PoolKey) -> PoolKey {
        self.map.get(&key).cloned().unwrap_or(key)
    }
}

/// `PoolAliasBindings.registerTargetsAsPools` — id-пути target-пулов
/// для последующей регистрации single-element пулов при bootstrap.
pub fn alias_target_paths(bindings: &[Rc<dyn PoolAliasBinding>]) -> Vec<String> {
    let mut out = Vec::new();
    for binding in bindings {
        for target in binding.all_targets() {
            let path = target.identifier.path.clone();
            if !out.contains(&path) {
                out.push(path);
            }
        }
    }
    out
}
