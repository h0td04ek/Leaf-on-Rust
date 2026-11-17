//! Перенос `net.minecraft.core.particles.ParticleTypes` — реестр
//! `Registries.PARTICLE_TYPE` (порядок = порядок регистрации).

use super::particle_type::{ParticleOptionsKind, ParticleType, SharedParticleType};
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::sync::{Arc, OnceLock, RwLock};

/// Запись реестра: (registry-key path, overrideLimiter, optionsKind).
struct Entry {
    name: &'static str,
    override_limiter: bool,
    options_kind: Option<ParticleOptionsKind>,
}

use ParticleOptionsKind as K;

/// Все типы частиц в порядке регистрации vanilla 1.21.11.
const ENTRIES: &[Entry] = &[
    e("angry_villager", false, None),
    e("block", false, Some(K::Block)),
    e("block_marker", true, Some(K::Block)),
    e("bubble", false, None),
    e("cloud", false, None),
    e("copper_fire_flame", false, None),
    e("crit", false, None),
    e("damage_indicator", true, None),
    e("dragon_breath", false, Some(K::Power)),
    e("dripping_lava", false, None),
    e("falling_lava", false, None),
    e("landing_lava", false, None),
    e("dripping_water", false, None),
    e("falling_water", false, None),
    e("dust", false, Some(K::Dust)),
    e("dust_color_transition", false, Some(K::DustColorTransition)),
    e("effect", false, Some(K::Spell)),
    e("elder_guardian", true, None),
    e("enchanted_hit", false, None),
    e("enchant", false, None),
    e("end_rod", false, None),
    e("entity_effect", false, Some(K::Color)),
    e("explosion_emitter", true, None),
    e("explosion", true, None),
    e("gust", true, None),
    e("small_gust", false, None),
    e("gust_emitter_large", true, None),
    e("gust_emitter_small", true, None),
    e("sonic_boom", true, None),
    e("falling_dust", false, Some(K::Block)),
    e("firework", false, None),
    e("fishing", false, None),
    e("flame", false, None),
    e("infested", false, None),
    e("cherry_leaves", false, None),
    e("pale_oak_leaves", false, None),
    e("tinted_leaves", false, Some(K::Color)),
    e("sculk_soul", false, None),
    e("sculk_charge", true, Some(K::SculkCharge)),
    e("sculk_charge_pop", true, None),
    e("soul_fire_flame", false, None),
    e("soul", false, None),
    e("flash", false, Some(K::Color)),
    e("happy_villager", false, None),
    e("composter", false, None),
    e("heart", false, None),
    e("instant_effect", false, Some(K::Spell)),
    e("item", false, Some(K::Item)),
    e("vibration", true, Some(K::Vibration)),
    e("trail", false, Some(K::Trail)),
    e("item_slime", false, None),
    e("item_cobweb", false, None),
    e("item_snowball", false, None),
    e("large_smoke", false, None),
    e("lava", false, None),
    e("mycelium", false, None),
    e("note", false, None),
    e("poof", true, None),
    e("portal", false, None),
    e("rain", false, None),
    e("smoke", false, None),
    e("white_smoke", false, None),
    e("sneeze", false, None),
    e("spit", true, None),
    e("squid_ink", true, None),
    e("sweep_attack", true, None),
    e("totem_of_undying", false, None),
    e("underwater", false, None),
    e("splash", false, None),
    e("witch", false, None),
    e("bubble_pop", false, None),
    e("current_down", false, None),
    e("bubble_column_up", false, None),
    e("nautilus", false, None),
    e("dolphin", false, None),
    e("campfire_cosy_smoke", true, None),
    e("campfire_signal_smoke", true, None),
    e("dripping_honey", false, None),
    e("falling_honey", false, None),
    e("landing_honey", false, None),
    e("falling_nectar", false, None),
    e("falling_spore_blossom", false, None),
    e("ash", false, None),
    e("crimson_spore", false, None),
    e("warped_spore", false, None),
    e("spore_blossom_air", false, None),
    e("dripping_obsidian_tear", false, None),
    e("falling_obsidian_tear", false, None),
    e("landing_obsidian_tear", false, None),
    e("reverse_portal", false, None),
    e("white_ash", false, None),
    e("small_flame", false, None),
    e("snowflake", false, None),
    e("dripping_dripstone_lava", false, None),
    e("falling_dripstone_lava", false, None),
    e("dripping_dripstone_water", false, None),
    e("falling_dripstone_water", false, None),
    e("glow_squid_ink", true, None),
    e("glow", true, None),
    e("wax_on", true, None),
    e("wax_off", true, None),
    e("electric_spark", true, None),
    e("scrape", true, None),
    e("shriek", false, Some(K::Shriek)),
    e("egg_crack", false, None),
    e("dust_plume", false, None),
    e("trial_spawner_detection", true, None),
    e("trial_spawner_detection_ominous", true, None),
    e("vault_connection", true, None),
    e("dust_pillar", false, Some(K::Block)),
    e("ominous_spawning", true, None),
    e("raid_omen", false, None),
    e("trial_omen", false, None),
    e("block_crumble", false, Some(K::Block)),
    e("firefly", false, None),
];

const fn e(name: &'static str, override_limiter: bool, options_kind: Option<ParticleOptionsKind>) -> Entry {
    Entry { name, override_limiter, options_kind }
}

fn build_registry() -> Vec<SharedParticleType> {
    ENTRIES
        .iter()
        .map(|entry| {
            Arc::new(ParticleType::new(
                Identifier::minecraft(entry.name),
                entry.override_limiter,
                entry.options_kind,
            )) as SharedParticleType
        })
        .collect()
}

fn registry() -> &'static RwLock<Vec<SharedParticleType>> {
    static REG: OnceLock<RwLock<Vec<SharedParticleType>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(build_registry()))
}

/// `BuiltInRegistries.PARTICLE_TYPE` — все типы по registry-id.
pub fn particle_types() -> Vec<SharedParticleType> {
    registry().read().unwrap().clone()
}

/// Число типов.
pub fn particle_type_count() -> usize {
    registry().read().unwrap().len()
}

/// Тип по registry-id.
pub fn particle_type_by_id(id: i32) -> Option<SharedParticleType> {
    registry().read().unwrap().get(id as usize).cloned()
}

/// Индекс типа по registry-key (`minecraft:name` или `name`).
pub fn particle_type_index(id: &Identifier) -> Option<usize> {
    registry()
        .read()
        .unwrap()
        .iter()
        .position(|t| &t.id == id)
}

/// Тип по `ResourceLocation`.
pub fn particle_type_by_identifier(id: &Identifier) -> Option<SharedParticleType> {
    particle_type_index(id).and_then(|i| particle_type_by_id(i as i32))
}

/// `ByteBufCodecs.registry(PARTICLE_TYPE)` — VarInt id типа.
pub fn particle_type_stream_codec() -> impl StreamCodec<FriendlyByteBuf, SharedParticleType> + Send + Sync + 'static {
    crate::network::codec::stream_codec::sync_fn(
        |buf: &mut FriendlyByteBuf, v: &SharedParticleType| {
            let idx = particle_type_index(&v.id).expect("unregistered particle type");
            buf.write_var_int(idx as i32);
        },
        |buf: &mut FriendlyByteBuf| {
            let id = buf.read_var_int();
            particle_type_by_id(id).expect("unknown particle type id")
        },
    )
}
