//! Перенос `net.minecraft.core.component.DataComponents` —
//! `bootstrap()` регистрирует все 104 `DataComponentType` в точном
//! порядке vanilla (wire-id = индекс реестра).
//!
//! `stream_codec: None` — у типа нет network-кодека: для transient-only
//! компонентов (`custom_data`, `intangible_projectile`, `map_decorations`,
//! `debug_stick_state`, `recipes`, `lock`, `container_loot`) это точное
//! соответствие Java (`streamCodec == null`); для типов, чьи payload-классы
//! ещё портируются (`world::item::component::*` и связанные реестры),
//! codec появится при переносе соответствующего класса.

use crate::core::component::data_component_type::{
    register_data_component_type, stream_codec_of, ComponentStreamCodec, DataComponentType,
};
use crate::core::holder::Holder;
use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::byte_buf_codecs;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::sounds::sound_event::SoundEvent;
use crate::util::unit::Unit;
use crate::world::item::component::{
    CustomModelData, DamageResistant, DyedItemColor, ItemLore, OminousBottleAmplifier,
    TooltipDisplay, UseCooldown, UseEffects,
};
use crate::world::item::dye_color::DyeColor;
use crate::world::item::rarity::Rarity;
use crate::world::level::saveddata::maps::MapId;
use std::sync::Once;

fn mk(
    id: &str,
    transient: bool,
    ignore_swap_animation: bool,
    stream_codec: Option<ComponentStreamCodec>,
) -> DataComponentType {
    DataComponentType {
        id: Identifier::parse(id).expect("component id"),
        transient,
        ignore_swap_animation,
        stream_codec,
    }
}

fn var_int() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<i32, _>(byte_buf_codecs::VAR_INT))
}

fn float() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<f32, _>(byte_buf_codecs::FLOAT))
}

fn bool_codec() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<bool, _>(byte_buf_codecs::BOOL))
}

fn unit() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<Unit, _>(crate::util::unit::stream_codec()))
}

fn identifier() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<Identifier, _>(byte_buf_codecs::IDENTIFIER))
}

fn component() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<Component, _>(component_json::component_stream_codec()))
}

fn rarity() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<Rarity, _>(Rarity::stream_codec()))
}

fn dye_color() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<DyeColor, _>(DyeColor::stream_codec()))
}

fn sound_event_holder() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<Holder<SoundEvent>, _>(SoundEvent::stream_codec()))
}

macro_rules! payload_codec {
    ($ty:ty, $codec:expr) => {
        Some(stream_codec_of::<$ty, _>($codec))
    };
}

/// `DataComponents.bootstrap` — регистрация всех типов по порядку.
pub fn bootstrap() {
    // 1. custom_data — persistent, БЕЗ network-кодека.
    register_data_component_type(mk("custom_data", false, false, None));
    // 2-5. скалярные.
    register_data_component_type(mk("max_stack_size", false, false, var_int()));
    register_data_component_type(mk("max_damage", false, false, var_int()));
    register_data_component_type(mk("damage", false, true, var_int()));
    register_data_component_type(mk("unbreakable", false, false, unit()));
    // 6. use_effects — UseEffects.STREAM_CODEC (payload портируется).
    register_data_component_type(mk("use_effects", false, false, payload_codec!(UseEffects, UseEffects::stream_codec())));
    // 7-8.
    register_data_component_type(mk("custom_name", false, false, component()));
    register_data_component_type(mk("minimum_attack_charge", false, false, float()));
    // 9. damage_type — EitherHolder<DAMAGE_TYPE> (портируется).
    register_data_component_type(mk("damage_type", false, false, None));
    // 10-12.
    register_data_component_type(mk("item_name", false, false, component()));
    register_data_component_type(mk("item_model", false, false, identifier()));
    register_data_component_type(mk("lore", false, false, payload_codec!(ItemLore, ItemLore::stream_codec()))); // ItemLore
    // 13.
    register_data_component_type(mk("rarity", false, false, rarity()));
    // 14-19.
    register_data_component_type(mk("enchantments", false, false, None)); // ItemEnchantments
    register_data_component_type(mk("can_place_on", false, false, None)); // AdventureModePredicate
    register_data_component_type(mk("can_break", false, false, None)); // AdventureModePredicate
    register_data_component_type(mk("attribute_modifiers", false, false, None)); // ItemAttributeModifiers
    register_data_component_type(mk("custom_model_data", false, false, payload_codec!(CustomModelData, CustomModelData::stream_codec()))); // CustomModelData
    register_data_component_type(mk("tooltip_display", false, false, payload_codec!(TooltipDisplay, TooltipDisplay::stream_codec()))); // TooltipDisplay
    // 20-23.
    register_data_component_type(mk("repair_cost", false, false, var_int()));
    register_data_component_type(mk("creative_slot_lock", true, false, unit()));
    register_data_component_type(mk("enchantment_glint_override", false, false, bool_codec()));
    register_data_component_type(mk("intangible_projectile", false, false, None)); // net=False
    // 24-35.
    register_data_component_type(mk("food", false, false, None)); // FoodProperties
    register_data_component_type(mk("consumable", false, false, None)); // Consumable
    register_data_component_type(mk("use_remainder", false, false, None)); // UseRemainder (ItemStack)
    register_data_component_type(mk("use_cooldown", false, false, payload_codec!(UseCooldown, UseCooldown::stream_codec()))); // UseCooldown
    register_data_component_type(mk("damage_resistant", false, false, payload_codec!(DamageResistant, DamageResistant::stream_codec()))); // DamageResistant
    register_data_component_type(mk("tool", false, false, None)); // Tool
    register_data_component_type(mk("weapon", false, false, None)); // Weapon
    register_data_component_type(mk("attack_range", false, false, None)); // AttackRange
    register_data_component_type(mk("enchantable", false, false, None)); // Enchantable
    register_data_component_type(mk("equippable", false, false, None)); // Equippable
    register_data_component_type(mk("repairable", false, false, None)); // Repairable
    register_data_component_type(mk("glider", false, false, unit()));
    // 36-45.
    register_data_component_type(mk("tooltip_style", false, false, identifier()));
    register_data_component_type(mk("death_protection", false, false, None)); // DeathProtection
    register_data_component_type(mk("blocks_attacks", false, false, None)); // BlocksAttacks
    register_data_component_type(mk("piercing_weapon", false, false, None)); // PiercingWeapon
    register_data_component_type(mk("kinetic_weapon", false, false, None)); // KineticWeapon
    register_data_component_type(mk("swing_animation", false, false, None)); // SwingAnimation
    register_data_component_type(mk("stored_enchantments", false, false, None)); // ItemEnchantments
    register_data_component_type(mk("dyed_color", false, false, payload_codec!(DyedItemColor, DyedItemColor::stream_codec()))); // DyedItemColor
    register_data_component_type(mk("map_color", false, false, None)); // MapItemColor
    register_data_component_type(mk("map_id", false, false, payload_codec!(MapId, MapId::stream_codec()))); // MapId
    // 46-49.
    register_data_component_type(mk("map_decorations", false, false, None)); // net=False
    register_data_component_type(mk("map_post_processing", true, false, None)); // MapPostProcessing
    register_data_component_type(mk("charged_projectiles", false, false, None)); // ChargedProjectiles
    register_data_component_type(mk("bundle_contents", false, false, None)); // BundleContents
    // 50-55.
    register_data_component_type(mk("potion_contents", false, false, None)); // PotionContents
    register_data_component_type(mk("potion_duration_scale", false, false, float()));
    register_data_component_type(mk("suspicious_stew_effects", false, false, None)); // SuspiciousStewEffects
    register_data_component_type(mk("writable_book_content", false, false, None)); // WritableBookContent
    register_data_component_type(mk("written_book_content", false, false, None)); // WrittenBookContent
    register_data_component_type(mk("trim", false, false, None)); // ArmorTrim
    // 56-65.
    register_data_component_type(mk("debug_stick_state", false, false, None)); // net=False
    register_data_component_type(mk("entity_data", false, false, None)); // TypedEntityData<EntityType>
    register_data_component_type(mk("bucket_entity_data", false, false, None)); // CustomData
    register_data_component_type(mk("block_entity_data", false, false, None)); // TypedEntityData<BlockEntityType>
    register_data_component_type(mk("instrument", false, false, None)); // InstrumentComponent
    register_data_component_type(mk("provides_trim_material", false, false, None)); // ProvidesTrimMaterial
    register_data_component_type(mk("ominous_bottle_amplifier", false, false, payload_codec!(OminousBottleAmplifier, OminousBottleAmplifier::stream_codec()))); // OminousBottleAmplifier
    register_data_component_type(mk("jukebox_playable", false, false, None)); // JukeboxPlayable
    register_data_component_type(mk("provides_banner_patterns", false, false, None)); // TagKey<BANNER_PATTERN>
    register_data_component_type(mk("recipes", false, false, None)); // net=False
    // 66-79.
    register_data_component_type(mk("lodestone_tracker", false, false, None)); // LodestoneTracker
    register_data_component_type(mk("firework_explosion", false, false, None)); // FireworkExplosion
    register_data_component_type(mk("fireworks", false, false, None)); // Fireworks
    register_data_component_type(mk("profile", false, false, None)); // ResolvableProfile
    register_data_component_type(mk("note_block_sound", false, false, identifier()));
    register_data_component_type(mk("banner_patterns", false, false, None)); // BannerPatternLayers
    register_data_component_type(mk("base_color", false, false, dye_color()));
    register_data_component_type(mk("pot_decorations", false, false, None)); // PotDecorations
    register_data_component_type(mk("container", false, false, None)); // ItemContainerContents
    register_data_component_type(mk("block_state", false, false, None)); // BlockItemStateProperties
    register_data_component_type(mk("bees", false, false, None)); // Bees
    register_data_component_type(mk("lock", false, false, None)); // net=False
    register_data_component_type(mk("container_loot", false, false, None)); // net=False
    register_data_component_type(mk("break_sound", false, false, sound_event_holder()));
    // 80-104. variant-компоненты — Holder<variant>/enum-кодеки.
    register_data_component_type(mk("villager/variant", false, false, None)); // VillagerType
    register_data_component_type(mk("wolf/variant", false, false, None)); // WolfVariant
    register_data_component_type(mk("wolf/sound_variant", false, false, None)); // WolfSoundVariant
    register_data_component_type(mk("wolf/collar", false, false, dye_color()));
    register_data_component_type(mk("fox/variant", false, false, None)); // Fox.Variant
    register_data_component_type(mk("salmon/size", false, false, None)); // Salmon.Variant
    register_data_component_type(mk("parrot/variant", false, false, None)); // Parrot.Variant
    register_data_component_type(mk("tropical_fish/pattern", false, false, None)); // TropicalFish.Pattern
    register_data_component_type(mk("tropical_fish/base_color", false, false, dye_color()));
    register_data_component_type(mk("tropical_fish/pattern_color", false, false, dye_color()));
    register_data_component_type(mk("mooshroom/variant", false, false, None)); // MushroomCow.Variant
    register_data_component_type(mk("rabbit/variant", false, false, None)); // Rabbit.Variant
    register_data_component_type(mk("pig/variant", false, false, None)); // PigVariant
    register_data_component_type(mk("cow/variant", false, false, None)); // CowVariant
    register_data_component_type(mk("chicken/variant", false, false, None)); // EitherHolder<ChickenVariant>
    register_data_component_type(mk("zombie_nautilus/variant", false, false, None)); // EitherHolder<ZombieNautilusVariant>
    register_data_component_type(mk("frog/variant", false, false, None)); // FrogVariant
    register_data_component_type(mk("horse/variant", false, false, None)); // Variant (horse)
    register_data_component_type(mk("painting/variant", false, false, None)); // PaintingVariant
    register_data_component_type(mk("llama/variant", false, false, None)); // Llama.Variant
    register_data_component_type(mk("axolotl/variant", false, false, None)); // Axolotl.Variant
    register_data_component_type(mk("cat/variant", false, false, None)); // CatVariant
    register_data_component_type(mk("cat/collar", false, false, dye_color()));
    register_data_component_type(mk("sheep/color", false, false, dye_color()));
    register_data_component_type(mk("shulker/color", false, false, dye_color()));
}

static BOOTSTRAP_ONCE: Once = Once::new();

/// Ленивый bootstrap — вызывается перед обращением к реестру.
pub fn ensure_bootstrapped() {
    BOOTSTRAP_ONCE.call_once(bootstrap);
}
