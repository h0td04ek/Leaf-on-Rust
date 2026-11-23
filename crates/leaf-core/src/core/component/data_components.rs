//! Перенос `net.minecraft.core.component.DataComponents` —
//! `bootstrap()` регистрирует все 104 `DataComponentType` в точном
//! порядке vanilla (wire-id = индекс реестра).
//!
//! Для компонентов без явного stream-кодека Java передаёт результат
//! persistent-кодека как NBT. Записи, требующие ещё не портированных
//! игровых типов, сохраняются в виде NBT-compound.

use crate::core::component::data_component_type::{
    register_builtin_data_component_type as register_data_component_type, stream_codec_of,
    ComponentNbtCodec, ComponentStreamCodec, DataComponentType,
};
use crate::core::holder::Holder;
use crate::nbt::{CompoundTag, Tag};
use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::sync_fn;
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
use crate::world::item::item_stack::ItemStack;
use crate::world::level::saveddata::maps::MapId;
use std::sync::Once;

mod item_crafting_nbt;

/// Size of the Java 1.21.11 registry used by crafting component codecs.
pub fn crafting_registry_len(name: &str) -> usize {
    item_crafting_nbt::registry_len(name)
}
pub fn enchantment_id(key: &Identifier) -> Option<i32> {
    item_crafting_nbt::enchantment_id(key)
}
mod item_properties_nbt;

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

fn persistent<T>(
    id: &str,
    encode: fn(&T) -> Result<Tag, String>,
    decode: fn(&Tag) -> Result<T, String>,
) -> DataComponentType
where
    T: Clone + Send + Sync + PartialEq + std::fmt::Debug + 'static,
{
    let wire = sync_fn(
        move |buf: &mut FriendlyByteBuf, value: &T| {
            let tag = encode(value).expect("component NBT encode");
            buf.write_nbt(Some(&tag)).expect("write component NBT");
        },
        move |buf: &mut FriendlyByteBuf| {
            let tag = buf.read_nbt().expect("read component NBT").expect("missing component NBT");
            decode(&tag).expect("component NBT decode")
        },
    );
    let mut ty = mk(id, false, false, Some(stream_codec_of::<T, _>(wire)));
    ty.nbt_codec = Some(ComponentNbtCodec::new(encode, decode));
    ty
}

fn read_compound_payload(tag: &Tag) -> Result<CompoundTag, String> {
    match tag {
        Tag::Compound(tag) => Ok(tag.clone()),
        _ => Err("expected an NBT compound for component payload".to_owned()),
    }
}

fn persistent_compound(id: &str) -> DataComponentType {
    persistent(id, |tag: &CompoundTag| Ok(Tag::Compound(tag.clone())), read_compound_payload)
}

fn read_recipes(tag: &Tag) -> Result<Vec<Identifier>, String> {
    let Tag::List(recipes) = tag else {
        return Err("expected an NBT list of recipe identifiers".to_owned());
    };
    recipes.iter().map(|recipe| {
        let Tag::String(id) = recipe else {
            return Err("expected a recipe identifier".to_owned());
        };
        Identifier::parse(id).map_err(|error| error.to_string())
    }).collect()
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

fn with_nbt<T>(
    mut ty: DataComponentType,
    encode: impl Fn(&T) -> Result<Tag, String> + Send + Sync + 'static,
    decode: impl Fn(&Tag) -> Result<T, String> + Send + Sync + 'static,
) -> DataComponentType
where
    T: Clone + Send + Sync + PartialEq + std::fmt::Debug + 'static,
{
    ty.nbt_codec = Some(ComponentNbtCodec::new(encode, decode));
    ty
}

fn int_component(id: &str, min: i32, max: i32, ignore_swap: bool) -> DataComponentType {
    int_nbt(mk(id, false, ignore_swap, var_int()), min, max, |value| *value, |value| value)
}

fn int_nbt<T>(
    ty: DataComponentType,
    min: i32,
    max: i32,
    unwrap: fn(&T) -> i32,
    wrap: fn(i32) -> T,
) -> DataComponentType
where
    T: Clone + Send + Sync + PartialEq + std::fmt::Debug + 'static,
{
    let check = move |value| {
        if (min..=max).contains(&value) {
            Ok(value)
        } else {
            Err(format!("expected integer in {min}..={max}, got {value}"))
        }
    };
    with_nbt(
        ty,
        move |value| check(unwrap(value)).map(Tag::Int),
        move |tag| {
            let value = tag.as_long().and_then(|value| i32::try_from(value).ok())
                .ok_or("expected an integer")?;
            check(value).map(wrap)
        },
    )
}

fn float_component(id: &str, min: f32, max: f32) -> DataComponentType {
    let check = move |value| {
        if (min..=max).contains(&value) {
            Ok(value)
        } else {
            Err(format!("expected float in {min}..={max}, got {value}"))
        }
    };
    with_nbt(
        mk(id, false, false, float()),
        move |value: &f32| check(*value).map(Tag::Float),
        move |tag| check(tag.as_float().ok_or("expected a number")?),
    )
}

fn unit_component(id: &str) -> DataComponentType {
    with_nbt(mk(id, false, false, unit()),
        |_: &Unit| Ok(Tag::Compound(CompoundTag::new())),
        |tag| tag.as_compound().map(|_| Unit).ok_or_else(|| "expected a compound".to_owned()),
    )
}

fn identifier_component(id: &str) -> DataComponentType {
    with_nbt(mk(id, false, false, identifier()),
        |value: &Identifier| Ok(Tag::String(value.to_string())),
        |tag| Identifier::parse(tag.as_string().ok_or("expected an identifier")?)
            .map_err(|error| error.to_string()),
    )
}

fn encode_chat(value: &Component) -> Result<Tag, String> {
    crate::nbt::json::tag_from_json(&component_json::component_to_json(value))
}

fn decode_chat(tag: &Tag) -> Result<Component, String> {
    component_json::component_from_json(&crate::nbt::json::json_from_tag(tag)?)
}

fn lore_component() -> DataComponentType {
    with_nbt(mk("lore", false, false, payload_codec!(ItemLore, ItemLore::stream_codec())),
        |lore: &ItemLore| {
            if lore.lines.len() > 256 {
                return Err("lore must contain at most 256 lines".into());
            }
            lore.lines.iter().map(encode_chat).collect::<Result<Vec<_>, _>>().map(Tag::List)
        },
        |tag| {
            let Tag::List(lines) = tag else { return Err("expected a lore list".into()); };
            if lines.len() > 256 {
                return Err("lore must contain at most 256 lines".into());
            }
            let lines = lines.iter().map(decode_chat).collect::<Result<Vec<_>, _>>()?;
            Ok(ItemLore::new(lines))
        },
    )
}

fn dye_component(id: &str) -> DataComponentType {
    with_nbt(mk(id, false, false, dye_color()),
        |value: &DyeColor| Ok(Tag::String(value.get_serialized_name().to_owned())),
        |tag| DyeColor::by_name(tag.as_string().ok_or("expected a dye name")?)
            .ok_or_else(|| "unknown dye color".into()),
    )
}

fn encode_stack(stack: &ItemStack) -> Result<Tag, String> {
    if stack.is_empty() {
        return Err("expected a nonempty item stack".into());
    }
    stack.save(CompoundTag::new())
}

fn decode_stack(tag: &Tag) -> Result<ItemStack, String> {
    let stack = ItemStack::from_nbt(tag.as_compound().ok_or("expected an item stack compound")?)?;
    if stack.is_empty() {
        return Err("expected a nonempty item stack".into());
    }
    Ok(stack)
}

fn check_item_list_size(len: usize, max: usize) -> Result<(), String> {
    if len > max {
        return Err(format!("expected at most {max} item entries"));
    }
    Ok(())
}

fn encode_item_list(items: &[ItemStack], max: usize) -> Result<Tag, String> {
    check_item_list_size(items.len(), max)?;
    items.iter().map(encode_stack).collect::<Result<Vec<_>, _>>().map(Tag::List)
}

fn decode_item_list(tag: &Tag, max: usize) -> Result<Vec<ItemStack>, String> {
    let items = tag.as_list().ok_or("expected a list of item stacks")?;
    check_item_list_size(items.len(), max)?;
    items.iter().map(decode_stack).collect()
}

fn container_component() -> DataComponentType {
    let ty = mk("container", false, false, payload_codec!(ItemContainerContents, ItemContainerContents::stream_codec()));
    with_nbt(
        ty,
        |container: &ItemContainerContents| {
            check_item_list_size(container.items.len(), ItemContainerContents::MAX_SIZE)?;
            let mut slots = Vec::new();
            for (index, item) in container.items.iter().enumerate() {
                if item.is_empty() {
                    continue;
                }
                let mut slot = CompoundTag::new();
                slot.put_int("slot", index as i32);
                slot.put("item", encode_stack(item)?);
                slots.push(Tag::Compound(slot));
            }
            Ok(Tag::List(slots))
        },
        |tag| {
            let slots = tag.as_list().ok_or("expected a list of container slots")?;
            check_item_list_size(slots.len(), ItemContainerContents::MAX_SIZE)?;
            let mut items = Vec::new();
            for slot in slots {
                let slot = slot.as_compound().ok_or("expected a container slot compound")?;
                let index = slot.get("slot").and_then(Tag::as_long)
                    .and_then(|value| usize::try_from(value).ok())
                    .filter(|&value| value < ItemContainerContents::MAX_SIZE)
                    .ok_or("expected container slot in 0..=255")?;
                let stack = decode_stack(slot.get("item").ok_or("missing container slot item")?)?;
                if index >= items.len() {
                    items.resize_with(index + 1, ItemStack::empty);
                }
                items[index] = stack;
            }
            Ok(ItemContainerContents::new(items))
        },
    )
}

fn register_builtin_types() {
    register_data_component_type(persistent(
        "custom_data",
        |data: &CustomData| Ok(Tag::Compound(data.tag.clone())),
        |tag| {
            let compound = match tag {
                Tag::String(snbt) => crate::nbt::snbt_parse::parse_compound(snbt)
                    .map_err(|error| error.to_string())?,
                _ => read_compound_payload(tag)?,
            };
            Ok(CustomData::new(compound))
        },
    ));
    // 2-5. скалярные.
    register_data_component_type(int_component("max_stack_size", 1, 99, false));
    register_data_component_type(int_component("max_damage", 1, i32::MAX, false));
    register_data_component_type(int_component("damage", 0, i32::MAX, true));
    register_data_component_type(unit_component("unbreakable"));
    // 6. use_effects — UseEffects.STREAM_CODEC (payload портируется).
    register_data_component_type(item_properties_nbt::use_effects(mk("use_effects", false, false, payload_codec!(UseEffects, UseEffects::stream_codec()))));
    // 7-8.
    register_data_component_type(with_nbt(mk("custom_name", false, false, component()), encode_chat, decode_chat));
    register_data_component_type(float_component("minimum_attack_charge", 0.0, 1.0));
    // 9. damage_type — EitherHolder<DAMAGE_TYPE>.
    register_data_component_type(mk("damage_type", false, false, damage_type_either()));
    // 10-12.
    register_data_component_type(with_nbt(mk("item_name", false, false, component()), encode_chat, decode_chat));
    register_data_component_type(identifier_component("item_model"));
    register_data_component_type(lore_component());
    // 13.
    register_data_component_type(with_nbt(mk("rarity", false, false, rarity()), |value: &Rarity| Ok(Tag::String(value.get_serialized_name().to_owned())), |tag| Rarity::by_name(tag.as_string().ok_or("expected a rarity name")?).ok_or_else(|| "unknown rarity".into())));
    // 14-19.
    register_data_component_type(item_crafting_nbt::enchantments(mk("enchantments", false, false, payload_codec!(ItemEnchantments, ItemEnchantments::stream_codec()))));
    register_data_component_type(mk("can_place_on", false, false, payload_codec!(AdventureModePredicate, AdventureModePredicate::stream_codec()))); // AdventureModePredicate
    register_data_component_type(mk("can_break", false, false, payload_codec!(AdventureModePredicate, AdventureModePredicate::stream_codec()))); // AdventureModePredicate
    register_data_component_type(mk("attribute_modifiers", false, false, payload_codec!(ItemAttributeModifiers, ItemAttributeModifiers::stream_codec())));
    register_data_component_type(mk("custom_model_data", false, false, payload_codec!(CustomModelData, CustomModelData::stream_codec()))); // CustomModelData
    register_data_component_type(mk("tooltip_display", false, false, payload_codec!(TooltipDisplay, TooltipDisplay::stream_codec()))); // TooltipDisplay
    // 20-23.
    register_data_component_type(int_component("repair_cost", 0, i32::MAX, false));
    register_data_component_type(mk("creative_slot_lock", true, false, unit()));
    register_data_component_type(with_nbt(mk("enchantment_glint_override", false, false, bool_codec()), |value: &bool| Ok(Tag::Byte(i8::from(*value))), |tag| tag.as_byte().map(|value| value != 0).ok_or_else(|| "expected a boolean byte".into())));
    register_data_component_type(persistent(
        "intangible_projectile",
        |_: &Unit| Ok(Tag::Compound(CompoundTag::new())),
        |tag| match tag {
            Tag::Compound(_) => Ok(Unit),
            _ => Err("expected a compound for intangible_projectile".to_owned()),
        },
    ));
    // 24-35.
    register_data_component_type(item_properties_nbt::food(mk("food", false, false, payload_codec!(FoodProperties, FoodProperties::stream_codec())))); // FoodProperties
    register_data_component_type(mk("consumable", false, false, payload_codec!(Consumable, Consumable::stream_codec()))); // Consumable
    register_data_component_type(with_nbt(
        mk("use_remainder", false, false, payload_codec!(UseRemainder, UseRemainder::stream_codec())),
        |value: &UseRemainder| encode_stack(&value.convert_into),
        |tag| decode_stack(tag).map(UseRemainder::new),
    ));
    register_data_component_type(item_properties_nbt::use_cooldown(mk("use_cooldown", false, false, payload_codec!(UseCooldown, UseCooldown::stream_codec())))); // UseCooldown
    register_data_component_type(mk("damage_resistant", false, false, payload_codec!(DamageResistant, DamageResistant::stream_codec()))); // DamageResistant
    register_data_component_type(mk("tool", false, false, payload_codec!(Tool, Tool::stream_codec())));
    register_data_component_type(item_properties_nbt::weapon(mk("weapon", false, false, payload_codec!(Weapon, Weapon::stream_codec()))));
    register_data_component_type(item_properties_nbt::attack_range(mk("attack_range", false, false, payload_codec!(AttackRange, AttackRange::stream_codec()))));
    register_data_component_type(int_nbt(mk("enchantable", false, false, payload_codec!(Enchantable, Enchantable::stream_codec())), 1, i32::MAX, |value: &Enchantable| value.value, Enchantable::new));
    register_data_component_type(mk("equippable", false, false, payload_codec!(Equippable, Equippable::stream_codec())));
    register_data_component_type(mk("repairable", false, false, payload_codec!(Repairable, Repairable::stream_codec())));
    register_data_component_type(unit_component("glider"));
    // 36-45.
    register_data_component_type(identifier_component("tooltip_style"));
    register_data_component_type(mk("death_protection", false, false, payload_codec!(DeathProtection, DeathProtection::stream_codec())));
    register_data_component_type(mk("blocks_attacks", false, false, payload_codec!(BlocksAttacks, BlocksAttacks::stream_codec())));
    register_data_component_type(mk("piercing_weapon", false, false, payload_codec!(PiercingWeapon, PiercingWeapon::stream_codec())));
    register_data_component_type(mk("kinetic_weapon", false, false, payload_codec!(KineticWeapon, KineticWeapon::stream_codec())));
    register_data_component_type(item_properties_nbt::swing_animation(mk("swing_animation", false, false, payload_codec!(SwingAnimation, SwingAnimation::stream_codec()))));
    register_data_component_type(mk("stored_enchantments", false, false, payload_codec!(ItemEnchantments, ItemEnchantments::stream_codec())));
    register_data_component_type(with_nbt(
        mk("dyed_color", false, false, payload_codec!(DyedItemColor, DyedItemColor::stream_codec())),
        |color: &DyedItemColor| Ok(Tag::Int(color.rgb)),
        |tag| {
            if let Some(rgb) = tag.as_int() {
                return Ok(DyedItemColor { rgb });
            }
            let Some([red, green, blue]) = tag.as_list() else {
                return Err("expected an RGB integer or three color channels".into());
            };
            let rgb = crate::util::argb::color_from_float(
                1.0,
                red.as_float().ok_or("expected a red color channel")?,
                green.as_float().ok_or("expected a green color channel")?,
                blue.as_float().ok_or("expected a blue color channel")?,
            );
            Ok(DyedItemColor { rgb })
        },
    )); // DyedItemColor
    register_data_component_type(int_nbt(mk("map_color", false, false, payload_codec!(MapItemColor, MapItemColor::stream_codec())), i32::MIN, i32::MAX, |value: &MapItemColor| value.rgb, MapItemColor::new));
    register_data_component_type(int_nbt(mk("map_id", false, false, payload_codec!(MapId, MapId::stream_codec())), i32::MIN, i32::MAX, |value: &MapId| value.0, MapId)); // MapId
    // 46-49.
    register_data_component_type(persistent_compound("map_decorations"));
    register_data_component_type(mk("map_post_processing", true, false, payload_codec!(MapPostProcessing, MapPostProcessing::stream_codec())));
    register_data_component_type(with_nbt(
        mk("charged_projectiles", false, false, payload_codec!(ChargedProjectiles, ChargedProjectiles::stream_codec())),
        |value: &ChargedProjectiles| encode_item_list(&value.items, 64),
        |tag| decode_item_list(tag, 64).map(ChargedProjectiles::of_list),
    ));
    register_data_component_type(with_nbt(
        mk("bundle_contents", false, false, payload_codec!(BundleContents, BundleContents::stream_codec())),
        |value: &BundleContents| encode_item_list(&value.items, 256),
        |tag| BundleContents::from_persistent_items(decode_item_list(tag, 256)?),
    ));
    // 50-55.
    register_data_component_type(item_crafting_nbt::potion(mk("potion_contents", false, false, payload_codec!(PotionContents, PotionContents::stream_codec()))));
    register_data_component_type(float_component("potion_duration_scale", 0.0, f32::MAX));
    register_data_component_type(item_crafting_nbt::stew(mk("suspicious_stew_effects", false, false, payload_codec!(SuspiciousStewEffects, SuspiciousStewEffects::stream_codec())))); // SuspiciousStewEffects
    register_data_component_type(item_crafting_nbt::writable_book(mk("writable_book_content", false, false, payload_codec!(WritableBookContent, WritableBookContent::stream_codec()))));
    register_data_component_type(item_crafting_nbt::written_book(mk("written_book_content", false, false, payload_codec!(WrittenBookContent, WrittenBookContent::stream_codec()))));
    register_data_component_type(mk("trim", false, false, payload_codec!(ArmorTrim, ArmorTrim::stream_codec())));
    // 56-65.
    register_data_component_type(persistent_compound("debug_stick_state"));
    register_data_component_type(mk("entity_data", false, false, Some(stream_codec_of::<TypedEntityData<Holder<EntityType>>, _>(TypedEntityData::stream_codec(EntityType::stream_codec())))));
    register_data_component_type(mk("bucket_entity_data", false, false, payload_codec!(CustomData, CustomData::stream_codec())));
    register_data_component_type(mk("block_entity_data", false, false, Some(stream_codec_of::<TypedEntityData<Holder<BlockEntityType>>, _>(TypedEntityData::stream_codec(crate::core::holder::holder_registry_stream_codec::<BlockEntityType>())))));
    register_data_component_type(mk("instrument", false, false, payload_codec!(InstrumentComponent, InstrumentComponent::stream_codec())));
    register_data_component_type(mk("provides_trim_material", false, false, payload_codec!(ProvidesTrimMaterial, ProvidesTrimMaterial::stream_codec())));
    register_data_component_type(int_nbt(mk("ominous_bottle_amplifier", false, false, payload_codec!(OminousBottleAmplifier, OminousBottleAmplifier::stream_codec())), 0, 4, |value: &OminousBottleAmplifier| value.value, |value| OminousBottleAmplifier { value })); // OminousBottleAmplifier
    register_data_component_type(mk("jukebox_playable", false, false, payload_codec!(JukeboxPlayable, JukeboxPlayable::stream_codec())));
    register_data_component_type(mk("provides_banner_patterns", false, false, Some(stream_codec_of::<TagKey<crate::world::level::block::entity::banner_pattern::BannerPattern>, _>(TagKey::stream_codec(crate::resources::resource_key::ResourceKey::create_registry_key(Identifier::parse("minecraft:banner_pattern").expect("banner_pattern registry")))))));
    register_data_component_type(persistent(
        "recipes",
        |recipes: &Vec<Identifier>| Ok(Tag::List(recipes.iter().map(|recipe| Tag::String(recipe.to_string())).collect())),
        read_recipes,
    ));
    // 66-79.
    register_data_component_type(mk("lodestone_tracker", false, false, payload_codec!(LodestoneTracker, LodestoneTracker::stream_codec())));
    register_data_component_type(with_nbt(
        mk("firework_explosion", false, false, payload_codec!(FireworkExplosion, FireworkExplosion::stream_codec())),
        |value: &FireworkExplosion| Ok(value.to_nbt()),
        FireworkExplosion::from_nbt,
    ));
    register_data_component_type(with_nbt(
        mk("fireworks", false, false, payload_codec!(Fireworks, Fireworks::stream_codec())),
        Fireworks::to_nbt,
        Fireworks::from_nbt,
    ));
    register_data_component_type(mk("profile", false, false, payload_codec!(ResolvableProfile, ResolvableProfile::stream_codec()))); // ResolvableProfile
    register_data_component_type(identifier_component("note_block_sound"));
    register_data_component_type(item_crafting_nbt::banner(mk("banner_patterns", false, false, payload_codec!(BannerPatternLayers, BannerPatternLayers::stream_codec()))));
    register_data_component_type(dye_component("base_color"));
    register_data_component_type(item_crafting_nbt::pot(mk("pot_decorations", false, false, payload_codec!(PotDecorations, PotDecorations::stream_codec()))));
    register_data_component_type(container_component());
    register_data_component_type(mk("block_state", false, false, payload_codec!(BlockItemStateProperties, BlockItemStateProperties::stream_codec())));
    register_data_component_type(mk("bees", false, false, payload_codec!(Bees, Bees::stream_codec())));
    register_data_component_type(persistent_compound("lock"));
    register_data_component_type(persistent_compound("container_loot"));
    register_data_component_type(mk("break_sound", false, false, sound_event_holder()));
    // 80-104. variant-компоненты — Holder<variant>/enum-кодеки.
    register_data_component_type(mk("villager/variant", false, false, holder_registry::<VillagerType>())); // VillagerType
    register_data_component_type(mk("wolf/variant", false, false, holder_registry::<WolfVariant>())); // WolfVariant
    register_data_component_type(mk("wolf/sound_variant", false, false, holder_registry::<WolfSoundVariant>())); // WolfSoundVariant
    register_data_component_type(dye_component("wolf/collar"));
    register_data_component_type(mk("fox/variant", false, false, payload_codec!(FoxVariant, FoxVariant::stream_codec()))); // Fox.Variant
    register_data_component_type(mk("salmon/size", false, false, payload_codec!(SalmonSize, SalmonSize::stream_codec()))); // Salmon.Variant
    register_data_component_type(mk("parrot/variant", false, false, payload_codec!(ParrotVariant, ParrotVariant::stream_codec()))); // Parrot.Variant
    register_data_component_type(mk("tropical_fish/pattern", false, false, payload_codec!(TropicalFishPattern, TropicalFishPattern::stream_codec()))); // TropicalFish.Pattern
    register_data_component_type(dye_component("tropical_fish/base_color"));
    register_data_component_type(dye_component("tropical_fish/pattern_color"));
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

/// `DataComponents.bootstrap` — однократная регистрация всех встроенных типов.
pub fn bootstrap() {
    BOOTSTRAP_ONCE.call_once(register_builtin_types);
}

/// Ленивый bootstrap — вызывается перед обращением к реестру.
pub fn ensure_bootstrapped() {
    bootstrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::data_component_patch::DataComponentPatch;
    use crate::core::component::data_component_type::{
        all_data_component_types, data_component_type_by_identifier,
    };
    use crate::network::codec::stream_codec::StreamCodec;

    #[test]
    fn bootstraps_once_with_a_wire_codec_for_each_builtin() {
        bootstrap();
        bootstrap();
        let types = all_data_component_types();
        assert_eq!(types.len(), 104);
        assert!(types.iter().all(|ty| ty.stream_codec.is_some()));
        assert_eq!(types[0].id.to_string(), "minecraft:custom_data");
        assert_eq!(types[103].id.to_string(), "minecraft:shulker/color");
    }

    #[test]
    fn custom_data_wire_id_zero_keeps_its_nbt_payload() {
        let id = Identifier::parse("custom_data").unwrap();
        let ty = data_component_type_by_identifier(&id).unwrap();
        let mut tag = CompoundTag::new();
        tag.put_int("x", 7);
        let data = CustomData::new(tag);
        let mut builder = DataComponentPatch::builder();
        builder.set(&ty, data.clone());
        let patch = builder.build();

        let mut buf = FriendlyByteBuf::new();
        let codec = DataComponentPatch::stream_codec();
        codec.encode(&mut buf, &patch);
        assert_eq!(buf.readable_bytes_vec(), [1, 0, 0, 10, 3, 0, 1, b'x', 0, 0, 0, 7, 0]);
        let decoded = codec.decode(&mut buf);
        assert_eq!(decoded.get::<CustomData>(&ty), Some(&data));
        assert_eq!(buf.readable_bytes(), 0);
    }
}
