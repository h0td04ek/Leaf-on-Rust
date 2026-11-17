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
use crate::core::holder::holder_registry_stream_codec;
use crate::world::damagesource::damage_type::DamageType;
use crate::world::item::component::variants::{
    AxolotlVariant, CatVariant, ChickenVariant, CowVariant, FoxVariant, FrogVariant,
    HorseVariant, LlamaVariant, MooshroomVariant, ParrotVariant, PigVariant,
    PaintingVariant, RabbitVariant, SalmonSize, TropicalFishPattern, VillagerType,
    WolfSoundVariant, WolfVariant, ZombieNautilusVariant,
};
use crate::world::food::food_properties::FoodProperties;
use crate::world::item::component::consumable::Consumable;
use crate::world::item::component::resolvable_profile::ResolvableProfile;
use crate::world::item::component::suspicious_stew_effects::SuspiciousStewEffects;
use crate::world::item::component::{
    attack_range::AttackRange, block_item_state_properties::BlockItemStateProperties,
    blocks_attacks::BlocksAttacks, bundle_contents::BundleContents,
    charged_projectiles::ChargedProjectiles, CustomModelData, custom_data::CustomData,
    damage_resistant::DamageResistant, death_protection::DeathProtection,
    dyed_item_color::DyedItemColor, firework_explosion::FireworkExplosion,
    fireworks::Fireworks, instrument_component::InstrumentComponent, item_lore::ItemLore,
    jukebox_playable::JukeboxPlayable, kinetic_weapon::KineticWeapon,
    lodestone_tracker::LodestoneTracker, map_item_color::MapItemColor,
    map_post_processing::MapPostProcessing, ominous_bottle_amplifier::OminousBottleAmplifier,
    piercing_weapon::PiercingWeapon, potion_contents::PotionContents,
    provides_trim_material::ProvidesTrimMaterial, swing_animation::SwingAnimation,
    TooltipDisplay, tool::Tool, use_cooldown::UseCooldown, use_effects::UseEffects,
    use_remainder::UseRemainder, weapon::Weapon, writable_book_content::WritableBookContent,
    written_book_content::WrittenBookContent,
    adventure_mode_predicate::AdventureModePredicate,
    item_attribute_modifiers::ItemAttributeModifiers,
    item_container_contents::ItemContainerContents,
};
use crate::world::item::component::bees::Bees;
use crate::world::item::component::typed_entity_data::TypedEntityData;
use crate::world::item::enchantment::enchantable::Enchantable;
use crate::world::item::enchantment::item_enchantments::ItemEnchantments;
use crate::world::item::enchantment::repairable::Repairable;
use crate::world::item::equipment::equippable::Equippable;
use crate::world::item::equipment::trim::armor_trim::ArmorTrim;
use crate::world::level::block::entity::banner_pattern_layers::BannerPatternLayers;
use crate::world::level::block::entity::pot_decorations::PotDecorations;
use crate::entity::entity_type::EntityType;
use crate::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::tags::tag_key::TagKey;
use crate::world::item::either_holder::EitherHolder;
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
        nbt_codec: None,
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

fn holder_registry<T>() -> Option<ComponentStreamCodec>
where
    T: Clone + PartialEq + std::fmt::Debug + Send + Sync + 'static,
{
    Some(stream_codec_of::<Holder<T>, _>(holder_registry_stream_codec::<T>()))
}

fn damage_type_either() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<
        EitherHolder<DamageType>,
        _,
    >(EitherHolder::stream_codec(
        Identifier::parse("damage_type").expect("registry id"),
        holder_registry_stream_codec::<DamageType>(),
    )))
}

fn chicken_either() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<
        EitherHolder<ChickenVariant>,
        _,
    >(EitherHolder::stream_codec(
        Identifier::parse("chicken_variant").expect("registry id"),
        holder_registry_stream_codec::<ChickenVariant>(),
    )))
}

fn nautilus_either() -> Option<ComponentStreamCodec> {
    Some(stream_codec_of::<
        EitherHolder<ZombieNautilusVariant>,
        _,
    >(EitherHolder::stream_codec(
        Identifier::parse("zombie_nautilus_variant").expect("registry id"),
        holder_registry_stream_codec::<ZombieNautilusVariant>(),
    )))
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
    // 9. damage_type — EitherHolder<DAMAGE_TYPE>.
    register_data_component_type(mk("damage_type", false, false, damage_type_either()));
    // 10-12.
    register_data_component_type(mk("item_name", false, false, component()));
    register_data_component_type(mk("item_model", false, false, identifier()));
    register_data_component_type(mk("lore", false, false, payload_codec!(ItemLore, ItemLore::stream_codec()))); // ItemLore
    // 13.
    register_data_component_type(mk("rarity", false, false, rarity()));
    // 14-19.
    register_data_component_type(mk("enchantments", false, false, payload_codec!(ItemEnchantments, ItemEnchantments::stream_codec())));
    register_data_component_type(mk("can_place_on", false, false, payload_codec!(AdventureModePredicate, AdventureModePredicate::stream_codec()))); // AdventureModePredicate
    register_data_component_type(mk("can_break", false, false, payload_codec!(AdventureModePredicate, AdventureModePredicate::stream_codec()))); // AdventureModePredicate
    register_data_component_type(mk("attribute_modifiers", false, false, payload_codec!(ItemAttributeModifiers, ItemAttributeModifiers::stream_codec())));
    register_data_component_type(mk("custom_model_data", false, false, payload_codec!(CustomModelData, CustomModelData::stream_codec()))); // CustomModelData
    register_data_component_type(mk("tooltip_display", false, false, payload_codec!(TooltipDisplay, TooltipDisplay::stream_codec()))); // TooltipDisplay
    // 20-23.
    register_data_component_type(mk("repair_cost", false, false, var_int()));
    register_data_component_type(mk("creative_slot_lock", true, false, unit()));
    register_data_component_type(mk("enchantment_glint_override", false, false, bool_codec()));
    register_data_component_type(mk("intangible_projectile", false, false, None)); // net=False
    // 24-35.
    register_data_component_type(mk("food", false, false, payload_codec!(FoodProperties, FoodProperties::stream_codec()))); // FoodProperties
    register_data_component_type(mk("consumable", false, false, payload_codec!(Consumable, Consumable::stream_codec()))); // Consumable
    register_data_component_type(mk("use_remainder", false, false, payload_codec!(UseRemainder, UseRemainder::stream_codec())));
    register_data_component_type(mk("use_cooldown", false, false, payload_codec!(UseCooldown, UseCooldown::stream_codec()))); // UseCooldown
    register_data_component_type(mk("damage_resistant", false, false, payload_codec!(DamageResistant, DamageResistant::stream_codec()))); // DamageResistant
    register_data_component_type(mk("tool", false, false, payload_codec!(Tool, Tool::stream_codec())));
    register_data_component_type(mk("weapon", false, false, payload_codec!(Weapon, Weapon::stream_codec())));
    register_data_component_type(mk("attack_range", false, false, payload_codec!(AttackRange, AttackRange::stream_codec())));
    register_data_component_type(mk("enchantable", false, false, payload_codec!(Enchantable, Enchantable::stream_codec())));
    register_data_component_type(mk("equippable", false, false, payload_codec!(Equippable, Equippable::stream_codec())));
    register_data_component_type(mk("repairable", false, false, payload_codec!(Repairable, Repairable::stream_codec())));
    register_data_component_type(mk("glider", false, false, unit()));
    // 36-45.
    register_data_component_type(mk("tooltip_style", false, false, identifier()));
    register_data_component_type(mk("death_protection", false, false, payload_codec!(DeathProtection, DeathProtection::stream_codec())));
    register_data_component_type(mk("blocks_attacks", false, false, payload_codec!(BlocksAttacks, BlocksAttacks::stream_codec())));
    register_data_component_type(mk("piercing_weapon", false, false, payload_codec!(PiercingWeapon, PiercingWeapon::stream_codec())));
    register_data_component_type(mk("kinetic_weapon", false, false, payload_codec!(KineticWeapon, KineticWeapon::stream_codec())));
    register_data_component_type(mk("swing_animation", false, false, payload_codec!(SwingAnimation, SwingAnimation::stream_codec())));
    register_data_component_type(mk("stored_enchantments", false, false, payload_codec!(ItemEnchantments, ItemEnchantments::stream_codec())));
    register_data_component_type(mk("dyed_color", false, false, payload_codec!(DyedItemColor, DyedItemColor::stream_codec()))); // DyedItemColor
    register_data_component_type(mk("map_color", false, false, payload_codec!(MapItemColor, MapItemColor::stream_codec())));
    register_data_component_type(mk("map_id", false, false, payload_codec!(MapId, MapId::stream_codec()))); // MapId
    // 46-49.
    register_data_component_type(mk("map_decorations", false, false, None)); // net=False
    register_data_component_type(mk("map_post_processing", true, false, payload_codec!(MapPostProcessing, MapPostProcessing::stream_codec())));
    register_data_component_type(mk("charged_projectiles", false, false, payload_codec!(ChargedProjectiles, ChargedProjectiles::stream_codec())));
    register_data_component_type(mk("bundle_contents", false, false, payload_codec!(BundleContents, BundleContents::stream_codec())));
    // 50-55.
    register_data_component_type(mk("potion_contents", false, false, payload_codec!(PotionContents, PotionContents::stream_codec())));
    register_data_component_type(mk("potion_duration_scale", false, false, float()));
    register_data_component_type(mk("suspicious_stew_effects", false, false, payload_codec!(SuspiciousStewEffects, SuspiciousStewEffects::stream_codec()))); // SuspiciousStewEffects
    register_data_component_type(mk("writable_book_content", false, false, payload_codec!(WritableBookContent, WritableBookContent::stream_codec())));
    register_data_component_type(mk("written_book_content", false, false, payload_codec!(WrittenBookContent, WrittenBookContent::stream_codec())));
    register_data_component_type(mk("trim", false, false, payload_codec!(ArmorTrim, ArmorTrim::stream_codec())));
    // 56-65.
    register_data_component_type(mk("debug_stick_state", false, false, None)); // net=False
    register_data_component_type(mk("entity_data", false, false, Some(stream_codec_of::<TypedEntityData<Holder<EntityType>>, _>(TypedEntityData::stream_codec(EntityType::stream_codec())))));
    register_data_component_type(mk("bucket_entity_data", false, false, payload_codec!(CustomData, CustomData::stream_codec())));
    register_data_component_type(mk("block_entity_data", false, false, Some(stream_codec_of::<TypedEntityData<Holder<BlockEntityType>>, _>(TypedEntityData::stream_codec(crate::core::holder::holder_registry_stream_codec::<BlockEntityType>())))));
    register_data_component_type(mk("instrument", false, false, payload_codec!(InstrumentComponent, InstrumentComponent::stream_codec())));
    register_data_component_type(mk("provides_trim_material", false, false, payload_codec!(ProvidesTrimMaterial, ProvidesTrimMaterial::stream_codec())));
    register_data_component_type(mk("ominous_bottle_amplifier", false, false, payload_codec!(OminousBottleAmplifier, OminousBottleAmplifier::stream_codec()))); // OminousBottleAmplifier
    register_data_component_type(mk("jukebox_playable", false, false, payload_codec!(JukeboxPlayable, JukeboxPlayable::stream_codec())));
    register_data_component_type(mk("provides_banner_patterns", false, false, Some(stream_codec_of::<TagKey<crate::world::level::block::entity::banner_pattern::BannerPattern>, _>(TagKey::stream_codec(crate::resources::resource_key::ResourceKey::create_registry_key(Identifier::parse("minecraft:banner_pattern").expect("banner_pattern registry")))))));
    register_data_component_type(mk("recipes", false, false, None)); // net=False
    // 66-79.
    register_data_component_type(mk("lodestone_tracker", false, false, payload_codec!(LodestoneTracker, LodestoneTracker::stream_codec())));
    register_data_component_type(mk("firework_explosion", false, false, payload_codec!(FireworkExplosion, FireworkExplosion::stream_codec())));
    register_data_component_type(mk("fireworks", false, false, payload_codec!(Fireworks, Fireworks::stream_codec())));
    register_data_component_type(mk("profile", false, false, payload_codec!(ResolvableProfile, ResolvableProfile::stream_codec()))); // ResolvableProfile
    register_data_component_type(mk("note_block_sound", false, false, identifier()));
    register_data_component_type(mk("banner_patterns", false, false, payload_codec!(BannerPatternLayers, BannerPatternLayers::stream_codec())));
    register_data_component_type(mk("base_color", false, false, dye_color()));
    register_data_component_type(mk("pot_decorations", false, false, payload_codec!(PotDecorations, PotDecorations::stream_codec())));
    register_data_component_type(mk("container", false, false, payload_codec!(ItemContainerContents, ItemContainerContents::stream_codec())));
    register_data_component_type(mk("block_state", false, false, payload_codec!(BlockItemStateProperties, BlockItemStateProperties::stream_codec())));
    register_data_component_type(mk("bees", false, false, payload_codec!(Bees, Bees::stream_codec())));
    register_data_component_type(mk("lock", false, false, None)); // net=False
    register_data_component_type(mk("container_loot", false, false, None)); // net=False
    register_data_component_type(mk("break_sound", false, false, sound_event_holder()));
    // 80-104. variant-компоненты — Holder<variant>/enum-кодеки.
    register_data_component_type(mk("villager/variant", false, false, holder_registry::<VillagerType>())); // VillagerType
    register_data_component_type(mk("wolf/variant", false, false, holder_registry::<WolfVariant>())); // WolfVariant
    register_data_component_type(mk("wolf/sound_variant", false, false, holder_registry::<WolfSoundVariant>())); // WolfSoundVariant
    register_data_component_type(mk("wolf/collar", false, false, dye_color()));
    register_data_component_type(mk("fox/variant", false, false, payload_codec!(FoxVariant, FoxVariant::stream_codec()))); // Fox.Variant
    register_data_component_type(mk("salmon/size", false, false, payload_codec!(SalmonSize, SalmonSize::stream_codec()))); // Salmon.Variant
    register_data_component_type(mk("parrot/variant", false, false, payload_codec!(ParrotVariant, ParrotVariant::stream_codec()))); // Parrot.Variant
    register_data_component_type(mk("tropical_fish/pattern", false, false, payload_codec!(TropicalFishPattern, TropicalFishPattern::stream_codec()))); // TropicalFish.Pattern
    register_data_component_type(mk("tropical_fish/base_color", false, false, dye_color()));
    register_data_component_type(mk("tropical_fish/pattern_color", false, false, dye_color()));
    register_data_component_type(mk("mooshroom/variant", false, false, payload_codec!(MooshroomVariant, MooshroomVariant::stream_codec()))); // MushroomCow.Variant
    register_data_component_type(mk("rabbit/variant", false, false, payload_codec!(RabbitVariant, RabbitVariant::stream_codec()))); // Rabbit.Variant
    register_data_component_type(mk("pig/variant", false, false, holder_registry::<PigVariant>())); // PigVariant
    register_data_component_type(mk("cow/variant", false, false, holder_registry::<CowVariant>())); // CowVariant
    register_data_component_type(mk("chicken/variant", false, false, chicken_either())); // EitherHolder<ChickenVariant>
    register_data_component_type(mk("zombie_nautilus/variant", false, false, nautilus_either())); // EitherHolder<ZombieNautilusVariant>
    register_data_component_type(mk("frog/variant", false, false, holder_registry::<FrogVariant>())); // FrogVariant
    register_data_component_type(mk("horse/variant", false, false, payload_codec!(HorseVariant, HorseVariant::stream_codec()))); // Variant (horse)
    register_data_component_type(mk("painting/variant", false, false, holder_registry::<PaintingVariant>())); // PaintingVariant
    register_data_component_type(mk("llama/variant", false, false, payload_codec!(LlamaVariant, LlamaVariant::stream_codec()))); // Llama.Variant
    register_data_component_type(mk("axolotl/variant", false, false, payload_codec!(AxolotlVariant, AxolotlVariant::stream_codec()))); // Axolotl.Variant
    register_data_component_type(mk("cat/variant", false, false, holder_registry::<CatVariant>())); // CatVariant
    register_data_component_type(mk("cat/collar", false, false, dye_color()));
    register_data_component_type(mk("sheep/color", false, false, dye_color()));
    register_data_component_type(mk("shulker/color", false, false, dye_color()));
}

static BOOTSTRAP_ONCE: Once = Once::new();

/// Ленивый bootstrap — вызывается перед обращением к реестру.
pub fn ensure_bootstrapped() {
    BOOTSTRAP_ONCE.call_once(bootstrap);
}
