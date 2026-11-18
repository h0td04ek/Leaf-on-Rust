//! Перенос `net.minecraft.world.level.storage.loot.providers.number.NumberProvider`.

use crate::world::level::storage::loot::loot_context::LootContext;
use crate::world::level::storage::loot::providers::number::providers::*;

/// `NumberProvider` — провайдер числа из loot-контекста.
pub trait NumberProvider {
    /// `getFloat(lootContext)`.
    fn get_float(&self, loot_context: &mut LootContext) -> f32;

    /// `getInt(lootContext)` — `Math.round(getFloat)`.
    fn get_int(&self, loot_context: &mut LootContext) -> i32 {
        self.get_float(loot_context).round() as i32
    }

    /// `getReferencedContextParams()` — имена используемых параметров.
    fn get_referenced_context_params(&self) -> Vec<crate::resources::identifier::Identifier> {
        Vec::new()
    }

    /// `asConstant` — для `IntRange.unpackExact`.
    fn as_constant(&self) -> Option<f32> {
        None
    }
}

impl NumberProvider for ConstantValue {
    fn get_float(&self, _ctx: &mut LootContext) -> f32 { self.0 }
    fn as_constant(&self) -> Option<f32> { Some(self.0) }
}

impl NumberProvider for UniformGenerator {
    fn get_float(&self, ctx: &mut LootContext) -> f32 {
        { let min = self.min.get_float(ctx); let max = self.max.get_float(ctx); crate::util::mth::next_float(ctx.random(), min, max) }
    }
    fn get_int(&self, ctx: &mut LootContext) -> i32 {
        { let min = self.min.get_int(ctx); let max = self.max.get_int(ctx); crate::util::mth::next_int(ctx.random(), min, max) }
    }
    fn get_referenced_context_params(&self) -> Vec<crate::resources::identifier::Identifier> {
        let mut v = self.min.get_referenced_context_params();
        for p in self.max.get_referenced_context_params() {
            if !v.contains(&p) { v.push(p); }
        }
        v
    }
}

impl NumberProvider for BinomialDistributionGenerator {
    fn get_int(&self, ctx: &mut LootContext) -> i32 {
        let n = self.n.get_int(ctx);
        let p = self.p.get_float(ctx);
        let mut count = 0;
        for _ in 0..n {
            if ctx.random().next_float() < p {
                count += 1;
            }
        }
        count
    }
    fn get_float(&self, ctx: &mut LootContext) -> f32 {
        self.get_int(ctx) as f32
    }
    fn get_referenced_context_params(&self) -> Vec<crate::resources::identifier::Identifier> {
        let mut v = self.n.get_referenced_context_params();
        for p in self.p.get_referenced_context_params() {
            if !v.contains(&p) { v.push(p); }
        }
        v
    }
}

impl NumberProvider for EnchantmentLevelProvider {
    fn get_float(&self, ctx: &mut LootContext) -> f32 {
        let level = *ctx.get_parameter(crate::world::level::storage::loot::parameters::loot_context_params::enchantment_level());
        self.amount.calculate(level)
    }
}

impl NumberProvider for ScoreboardValue {
    fn get_float(&self, ctx: &mut LootContext) -> f32 {
        let Some(holder) = self.target.get_score_holder(ctx) else { return 0.0 };
        let Some(level) = ctx.level().cloned() else { return 0.0 };
        let Some(scoreboard) = level.scoreboard() else { return 0.0 };
        let name = holder.scoreboard_name();
        let Some(objective) = scoreboard.get_objective(&self.score) else { return 0.0 };
        scoreboard.get_player_score_info(&name, &objective.name)
            .map(|info| info.value as f32 * self.scale)
            .unwrap_or(0.0)
    }
    fn get_referenced_context_params(&self) -> Vec<crate::resources::identifier::Identifier> {
        self.target.get_referenced_context_params()
    }
}

impl NumberProvider for StorageValue {
    fn get_float(&self, ctx: &mut LootContext) -> f32 {
        self.get_numeric_tag(ctx).map(|n| n as f64 as f32).unwrap_or(0.0)
    }
    fn get_int(&self, ctx: &mut LootContext) -> i32 {
        self.get_numeric_tag(ctx).unwrap_or(0) as i32
    }
}

impl StorageValue {
    fn get_numeric_tag(&self, _ctx: &LootContext) -> Option<i64> {
        let tag = command_storage_get(&self.storage)?;
        let mut cur = tag;
        for seg in self.path.segments() {
            cur = match seg {
                crate::nbt::path::PathSegment::Key(k) => cur.as_compound()?.get(k)?.clone(),
                crate::nbt::path::PathSegment::Index(i) => cur.as_list()?.get(*i as usize)?.clone(),
                crate::nbt::path::PathSegment::All
                | crate::nbt::path::PathSegment::Match(_) => return None,
            };
        }
        match &cur {
            crate::nbt::tag::Tag::Int(v) => Some(*v as i64),
            crate::nbt::tag::Tag::Long(v) => Some(*v),
            crate::nbt::tag::Tag::Float(v) => Some(*v as i64),
            crate::nbt::tag::Tag::Double(v) => Some(*v as i64),
            crate::nbt::tag::Tag::Byte(v) => Some(*v as i64),
            crate::nbt::tag::Tag::Short(v) => Some(*v as i64),
            _ => None,
        }
    }
}

/// `Server.getCommandStorage()` — глобальное хранилище командных данных.
fn command_storage_get(name: &crate::resources::identifier::Identifier) -> Option<crate::nbt::tag::Tag> {
    command_storage().lock().unwrap().get(name).cloned()
}

fn command_storage() -> &'static std::sync::Mutex<std::collections::HashMap<crate::resources::identifier::Identifier, crate::nbt::tag::Tag>> {
    static STORAGE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<crate::resources::identifier::Identifier, crate::nbt::tag::Tag>>> = std::sync::OnceLock::new();
    STORAGE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// `CommandStorage.set(name, tag)` — запись значения.
pub fn command_storage_set(name: crate::resources::identifier::Identifier, tag: crate::nbt::tag::Tag) {
    command_storage().lock().unwrap().insert(name, tag);
}
