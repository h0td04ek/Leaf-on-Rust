//! Перенос `net.minecraft.advancements.criterion.EnchantmentPredicate`.

use crate::advancements::criterion::holder_set_nbt;
use crate::advancements::criterion::min_max_bounds::MinMaxBoundsInts;
use crate::advancements::criterion::nbt_util;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::world::item::enchantment::enchantment::Enchantment;

/// `EnchantmentPredicate` — (enchantments?, levels: Ints).
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentPredicate {
    /// `enchantments` — `HolderSet<Enchantment>`.
    pub enchantments: Option<HolderSet<Enchantment>>,
    /// `level`.
    pub level: MinMaxBoundsInts,
}

impl EnchantmentPredicate {
    /// `new`.
    pub fn new(enchantments: Option<HolderSet<Enchantment>>, level: MinMaxBoundsInts) -> Self {
        Self { enchantments, level }
    }

    /// `EnchantmentPredicate.CODEC` → NBT `{enchantments?, levels?}`.
    pub fn encode_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        if let Some(set) = &self.enchantments {
            if let Ok(t) = holder_set_nbt::holder_set_encode(
                set,
                &Identifier::minecraft("enchantment"),
                |_| None,
            ) {
                c.put("enchantments", t);
            }
        }
        if self.level != MinMaxBoundsInts::ANY {
            c.put("levels", self.level.encode());
        }
        Tag::Compound(c)
    }

    /// `EnchantmentPredicate.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "enchantment predicate")?;
        let enchantments = match c.get("enchantments") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<Enchantment>(
                t,
                Identifier::minecraft("enchantment"),
            )?),
            None => None,
        };
        let level = match c.get("levels") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        Ok(Self::new(enchantments, level))
    }

    /// `containedIn(ItemEnchantments)`:
    /// - `enchantments` задан → true, если ХОТЯ БЫ ОДИН holder из set
    ///   присутствует в карте с level != 0 и в bounds;
    /// - set не задан, level != ANY → true, если ХОТЯ БЫ ОДИН уровень
    ///   в bounds;
    /// - иначе → true, если карта не пуста.
    /// Для Named-тегов (без резолва) holder считается отсутствующим.
    pub fn contained_in(&self, enchantments_map: &[(Holder<Enchantment>, i32)]) -> bool {
        if let Some(set) = &self.enchantments {
            // хотя бы один holder set'а присутствует с level!=0 в bounds
            for h in set_holders(set) {
                let level = enchantments_map
                    .iter()
                    .find(|(eh, _)| holder_eq_any(eh, h))
                    .map(|(_, l)| *l)
                    .unwrap_or(0);
                if level != 0 && (self.level == MinMaxBoundsInts::ANY || self.level.matches(level)) {
                    return true;
                }
            }
            return false;
        } else if self.level != MinMaxBoundsInts::ANY {
            return enchantments_map.iter().any(|(_, l)| self.level.matches(*l));
        } else {
            return !enchantments_map.is_empty();
        }
    }
}

/// Итерация holder'ов Direct-set'а (Named без резолва → пусто).
fn set_holders<T>(set: &HolderSet<T>) -> Vec<&Holder<T>> {
    match set {
        HolderSet::Direct(list) => list.iter().collect(),
        HolderSet::Named(_) => Vec::new(),
    }
}

/// `Holder`-равенство для Key/Reference/Direct.
fn holder_eq_any<T: PartialEq>(a: &Holder<T>, b: &Holder<T>) -> bool {
    match (a, b) {
        (Holder::Key(x), Holder::Key(y)) => x == y,
        (Holder::Reference(x), Holder::Reference(y)) => x == y,
        (Holder::Direct(x), Holder::Direct(y)) => x == y,
        _ => false,
    }
}
