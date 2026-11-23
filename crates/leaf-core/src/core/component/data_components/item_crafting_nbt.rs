use super::{decode_chat, encode_chat, with_nbt};
use crate::core::component::DataComponentType;
use crate::core::holder::Holder;
use crate::nbt::{CompoundTag, Tag};
use crate::resources::Identifier;
use crate::server::network::filterable::Filterable;
use crate::world::effect::mob_effect_instance::{MobEffectDetails, MobEffectInstance};
use crate::world::item::component::{
    potion_contents::PotionContents,
    suspicious_stew_effects::{SuspiciousStewEffects, SuspiciousStewEntry},
    writable_book_content::WritableBookContent,
    written_book_content::WrittenBookContent,
};
use crate::world::item::{dye_color::DyeColor, item};
use crate::world::level::block::entity::{
    banner_pattern_layers::{BannerPatternLayer, BannerPatternLayers},
    pot_decorations::PotDecorations,
};

fn registry(registry: &str) -> impl Iterator<Item = &'static str> + '_ {
    include_str!("crafting_registries_774.txt")
        .lines()
        .filter_map(move |line| {
            let (group, key) = line.split_once(' ')?;
            (group == registry).then_some(key)
        })
}

pub(super) fn registry_len(name: &str) -> usize {
    registry(name).count()
}

fn key_for<T>(name: &str, holder: &Holder<T>) -> Result<String, String> {
    match holder {
        Holder::Reference(id) => registry(name).nth(*id as usize).map(str::to_owned),
        Holder::Key(key) => Some(key.id().to_string()),
        Holder::Direct(_) => None,
    }
    .ok_or_else(|| format!("unregistered {name}"))
}

fn holder_for<T>(name: &str, key: &str) -> Result<Holder<T>, String> {
    let key = Identifier::parse(key)
        .map_err(|error| error.to_string())?
        .to_string();
    registry(name)
        .position(|entry| entry == key)
        .map(|id| Holder::Reference(id as i32))
        .ok_or_else(|| format!("unknown {name} {key}"))
}

pub(super) fn stew(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |value: &SuspiciousStewEffects| {
            value
                .effects
                .iter()
                .map(|effect| {
                    let mut tag = CompoundTag::new();
                    tag.put_string("id", &key_for("mob_effect", &effect.effect)?);
                    tag.put_int("duration", effect.duration);
                    Ok(Tag::Compound(tag))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Tag::List)
        },
        |tag| {
            let effects = list(tag, 256)?
                .iter()
                .map(|value| {
                    let tag = compound(value)?;
                    Ok(SuspiciousStewEntry::new(
                        holder_for(
                            "mob_effect",
                            tag.get_string("id").ok_or("missing effect id")?,
                        )?,
                        tag.get_int("duration").unwrap_or(160),
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(SuspiciousStewEffects::new(effects))
        },
    )
}

pub(super) fn pot(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |value: &PotDecorations| {
            value
                .ordered()
                .into_iter()
                .map(|holder| {
                    match holder {
                        Holder::Reference(id) => {
                            item::item_by_id(id).map(|item| item.id.to_string())
                        }
                        Holder::Direct(item) => Some(item.id.to_string()),
                        Holder::Key(key) => Some(key.id().to_string()),
                    }
                    .map(Tag::String)
                    .ok_or_else(|| "unregistered pottery sherd".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Tag::List)
        },
        |tag| {
            let sides = list(tag, 4)?;
            if sides.len() != 4 {
                return Err("expected four pot decorations".into());
            }
            let item = |path: &str| {
                let id = Identifier::parse(path).map_err(|err| err.to_string())?;
                let item = item::item_by_identifier(&id).ok_or("unknown pottery sherd")?;
                Ok::<_, String>(Holder::Reference(item::item_registry_id(&item).unwrap()))
            };
            let side =
                |i: usize| item(sides[i].as_string().ok_or("invalid pot decoration")?).map(Some);
            Ok(PotDecorations {
                back: side(0)?,
                left: side(1)?,
                right: side(2)?,
                front: side(3)?,
                brick_holder: Some(item("minecraft:brick")?),
            })
        },
    )
}

pub(super) fn banner(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |value: &BannerPatternLayers| {
            value
                .layers
                .iter()
                .map(|layer| {
                    let mut tag = CompoundTag::new();
                    tag.put_string("pattern", &key_for("banner_pattern", &layer.pattern)?);
                    tag.put_string("color", layer.color.get_serialized_name());
                    Ok(Tag::Compound(tag))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Tag::List)
        },
        |tag| {
            let layers = list(tag, 256)?
                .iter()
                .map(|value| {
                    let tag = compound(value)?;
                    let pattern = holder_for(
                        "banner_pattern",
                        tag.get_string("pattern").ok_or("missing banner pattern")?,
                    )?;
                    let color =
                        DyeColor::by_name(tag.get_string("color").ok_or("missing banner color")?)
                            .ok_or("invalid banner color")?;
                    Ok(BannerPatternLayer::new(pattern, color))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(BannerPatternLayers::new(layers))
        },
    )
}

pub(super) fn potion(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |value: &PotionContents| {
            let mut tag = CompoundTag::new();
            if let Some(potion) = &value.potion {
                tag.put_string("potion", &key_for("potion", potion)?);
            }
            if let Some(color) = value.custom_color {
                tag.put_int("custom_color", color);
            }
            if let Some(name) = &value.custom_name {
                tag.put_string("custom_name", name);
            }
            if !value.custom_effects.is_empty() {
                let effects = value
                    .custom_effects
                    .iter()
                    .map(|effect| {
                        let mut detail = encode_effect(&effect.details);
                        detail.put_string("id", &key_for("mob_effect", &effect.effect)?);
                        Ok(Tag::Compound(detail))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                tag.put("custom_effects", Tag::List(effects));
            }
            Ok(Tag::Compound(tag))
        },
        |tag| {
            let tag = compound(tag)?;
            let potion = tag
                .get_string("potion")
                .map(|name| holder_for("potion", name))
                .transpose()?;
            let effects = optional_list(tag, "custom_effects", 256)?
                .iter()
                .map(|effect| {
                    let effect = compound(effect)?;
                    let id = holder_for(
                        "mob_effect",
                        effect.get_string("id").ok_or("missing potion effect id")?,
                    )?;
                    Ok(MobEffectInstance::new(id, decode_effect(effect, 0)?))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(PotionContents::new(
                potion,
                tag.get_int("custom_color"),
                effects,
                tag.get_string("custom_name").map(str::to_owned),
            ))
        },
    )
}

fn encode_effect(effect: &MobEffectDetails) -> CompoundTag {
    let mut tag = CompoundTag::new();
    tag.put_int("amplifier", effect.amplifier);
    tag.put_int("duration", effect.duration);
    tag.put_bool("ambient", effect.ambient);
    tag.put_bool("show_particles", effect.show_particles);
    tag.put_bool("show_icon", effect.show_icon);
    if let Some(hidden) = &effect.hidden_effect {
        tag.put("hidden_effect", Tag::Compound(encode_effect(hidden)));
    }
    tag
}

fn decode_effect(tag: &CompoundTag, depth: usize) -> Result<MobEffectDetails, String> {
    if depth >= 16 {
        return Err("too many hidden effects".into());
    }
    let particles = tag.get_bool("show_particles").unwrap_or(true);
    Ok(MobEffectDetails::new(
        tag.get_int("amplifier").unwrap_or(0),
        tag.get_int("duration").unwrap_or(0),
        tag.get_bool("ambient").unwrap_or(false),
        particles,
        tag.get_bool("show_icon").unwrap_or(particles),
        tag.get_compound("hidden_effect")
            .map(|hidden| decode_effect(hidden, depth + 1).map(Box::new))
            .transpose()?,
    ))
}

pub(super) fn writable_book(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |book: &WritableBookContent| {
            let pages = book
                .pages
                .iter()
                .map(|page| encode_filterable(page, encode_string))
                .collect::<Result<Vec<_>, _>>()?;
            let mut tag = CompoundTag::new();
            tag.put("pages", Tag::List(pages));
            Ok(Tag::Compound(tag))
        },
        |tag| {
            let pages = optional_list(compound(tag)?, "pages", 100)?
                .iter()
                .map(|page| decode_filterable(page, decode_string))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(WritableBookContent::new(pages))
        },
    )
}

pub(super) fn written_book(ty: DataComponentType) -> DataComponentType {
    with_nbt(
        ty,
        |book: &WrittenBookContent| {
            let mut tag = CompoundTag::new();
            tag.put(
                "title",
                encode_filterable(
                    &book
                        .title
                        .clone()
                        .unwrap_or_else(|| Filterable::pass_through(String::new())),
                    encode_string,
                )?,
            );
            tag.put_string("author", &book.author);
            tag.put_int("generation", book.generation);
            tag.put_bool("resolved", book.resolved);
            let pages = book
                .pages
                .iter()
                .map(|page| encode_filterable(page, encode_chat))
                .collect::<Result<Vec<_>, _>>()?;
            tag.put("pages", Tag::List(pages));
            Ok(Tag::Compound(tag))
        },
        |tag| {
            let tag = compound(tag)?;
            let title =
                decode_filterable(tag.get("title").ok_or("missing book title")?, decode_string)?;
            let pages = optional_list(tag, "pages", 100)?
                .iter()
                .map(|page| decode_filterable(page, decode_chat))
                .collect::<Result<Vec<_>, _>>()?;
            let generation = tag.get_int("generation").unwrap_or(0);
            if !(0..=3).contains(&generation) {
                return Err("invalid book generation".into());
            }
            Ok(WrittenBookContent::new(
                Some(title),
                tag.get_string("author")
                    .ok_or("missing book author")?
                    .to_owned(),
                generation,
                pages,
                tag.get_bool("resolved").unwrap_or(false),
            ))
        },
    )
}

fn encode_string(value: &String) -> Result<Tag, String> {
    Ok(Tag::String(value.clone()))
}

fn decode_string(value: &Tag) -> Result<String, String> {
    value
        .as_string()
        .map(str::to_owned)
        .ok_or_else(|| "expected text".to_owned())
}

fn encode_filterable<T>(
    value: &Filterable<T>,
    encode: fn(&T) -> Result<Tag, String>,
) -> Result<Tag, String> {
    let raw = encode(&value.raw)?;
    if let Some(filtered) = &value.filtered {
        let mut tag = CompoundTag::new();
        tag.put("raw", raw);
        tag.put("filtered", encode(filtered)?);
        Ok(Tag::Compound(tag))
    } else {
        Ok(raw)
    }
}

fn decode_filterable<T>(
    value: &Tag,
    decode: fn(&Tag) -> Result<T, String>,
) -> Result<Filterable<T>, String> {
    if let Some(tag) = value.as_compound() {
        if let Some(raw) = tag.get("raw") {
            return Ok(Filterable {
                raw: decode(raw)?,
                filtered: tag.get("filtered").map(decode).transpose()?,
            });
        }
    }
    Ok(Filterable {
        raw: decode(value)?,
        filtered: None,
    })
}

fn compound(tag: &Tag) -> Result<&CompoundTag, String> {
    tag.as_compound()
        .ok_or_else(|| "expected NBT compound".into())
}

fn list(tag: &Tag, limit: usize) -> Result<&[Tag], String> {
    tag.as_list()
        .filter(|entries| entries.len() <= limit)
        .ok_or_else(|| format!("expected NBT list with at most {limit} entries"))
}

fn optional_list<'a>(tag: &'a CompoundTag, name: &str, limit: usize) -> Result<&'a [Tag], String> {
    tag.get(name).map_or(Ok(&[]), |value| list(value, limit))
}
