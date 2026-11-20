//! Перенос `net.minecraft.world.entity.npc.villager.VillagerData` —
//! record (type, profession, level) + XP-пороги.

use crate::core::holder::Holder;
use crate::resources::identifier::Identifier;
use crate::world::entity::npc::villager::villager_profession::VillagerProfession;
use crate::world::entity::npc::villager::villager_type::VillagerType;

/// `VillagerData(type, profession, level)`.
#[derive(Clone, Debug)]
pub struct VillagerData {
    /// `type` — Holder<VillagerType>.
    pub ty: Holder<VillagerType>,
    /// `profession` — Holder<VillagerProfession>.
    pub profession: Holder<VillagerProfession>,
    /// `level` — 1..=5.
    pub level: i32,
}

impl VillagerData {
    /// `MIN_VILLAGER_LEVEL`.
    pub const MIN_VILLAGER_LEVEL: i32 = 1;
    /// `MAX_VILLAGER_LEVEL`.
    pub const MAX_VILLAGER_LEVEL: i32 = 5;
    /// `NEXT_LEVEL_XP_THRESHOLDS`.
    const NEXT_LEVEL_XP_THRESHOLDS: [i32; 5] = [0, 10, 70, 150, 250];

    /// `new(type, profession, level)` — level клампится к >=1.
    pub fn new(
        ty: Holder<VillagerType>,
        profession: Holder<VillagerProfession>,
        level: i32,
    ) -> Self {
        Self {
            ty,
            profession,
            level: level.max(Self::MIN_VILLAGER_LEVEL),
        }
    }

    /// `withType`.
    pub fn with_type(&self, ty: Holder<VillagerType>) -> Self {
        Self::new(ty, self.profession.clone(), self.level)
    }

    /// `withProfession`.
    pub fn with_profession(&self, profession: Holder<VillagerProfession>) -> Self {
        Self::new(self.ty.clone(), profession, self.level)
    }

    /// `withLevel`.
    pub fn with_level(&self, level: i32) -> Self {
        Self::new(self.ty.clone(), self.profession.clone(), level)
    }

    /// `getMinXpPerLevel(level)`.
    pub fn get_min_xp_per_level(level: i32) -> i32 {
        if Self::can_level_up(level) {
            Self::NEXT_LEVEL_XP_THRESHOLDS[(level - 1) as usize]
        } else {
            0
        }
    }

    /// `getMaxXpPerLevel(level)`.
    pub fn get_max_xp_per_level(level: i32) -> i32 {
        if Self::can_level_up(level) {
            Self::NEXT_LEVEL_XP_THRESHOLDS[level as usize]
        } else {
            0
        }
    }

    /// `canLevelUp(level)` — 1 <= level < 5.
    pub fn can_level_up(level: i32) -> bool {
        level >= 1 && level < 5
    }
}

/// Дефолтные данные жителя (plains + none + level 1).
pub fn default_villager_data() -> VillagerData {
    VillagerData::new(
        Holder::Key(crate::resources::resource_key::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::villager_type(),
            Identifier::minecraft("plains"),
        )),
        Holder::Key(crate::resources::resource_key::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::villager_profession(),
            Identifier::minecraft("none"),
        )),
        1,
    )
}

/// `profession == key` — Holder-compare по registry-ключу/id
/// (аналог `profession().is(key)` / `equals`).
pub fn profession_is(
    holder: &Holder<VillagerProfession>,
    key: &crate::resources::resource_key::ResourceKey<VillagerProfession>,
) -> bool {
    match holder {
        Holder::Key(k) => *k == *key,
        Holder::Direct(p) => p.id == key.identifier,
        Holder::Reference(_) => false,
    }
}

/// `profession().is(VillagerProfession.NONE)`-проверка через
/// ключ holder'а.
pub fn is_profession_none(holder: &Holder<VillagerProfession>) -> bool {
    match holder {
        Holder::Key(k) => k.identifier.get_path() == "none",
        Holder::Direct(p) => p.id.get_path() == "none",
        Holder::Reference(_) => false,
    }
}

/// `profession().is(NITWIT)`.
pub fn is_profession_nitwit(holder: &Holder<VillagerProfession>) -> bool {
    match holder {
        Holder::Key(k) => k.identifier.get_path() == "nitwit",
        _ => false,
    }
}

/// `withProfession(registries, key)` — holder по ключу.
pub fn holder_for_profession(key: &crate::resources::resource_key::ResourceKey<VillagerProfession>) -> Holder<VillagerProfession> {
    Holder::Key(key.clone())
}

/// `withProfession(registries, key)` для типа.
pub fn holder_for_type(key: &crate::resources::resource_key::ResourceKey<VillagerType>) -> Holder<VillagerType> {
    Holder::Key(key.clone())
}
