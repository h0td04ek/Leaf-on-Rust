//! JSON-сериализация `Component` — аналог `ComponentSerialization.CODEC`
//! (JSON-форма Minecraft: строка | массив | объект).
//!
//! Формат объекта: `{ "<content-key>": ..., "extra": [...], <style-поля> }`.
//! Стилевые ключи плоские: color, bold, italic, underlined, strikethrough,
//! obfuscated, font, click_event, hover_event, insertion, shadow_color.

use crate::network::chat::click_event::{ClickEvent, ClickEventAction};
use crate::network::chat::component::Component;
use crate::network::chat::component_contents::{
    BlockDataSource, ComponentContents, DataSource, EntityDataSource, ObjectInfo,
    SelectorPattern, StorageDataSource, TranslatableArg,
};
use crate::network::chat::font_description::FontDescription;
use crate::network::chat::hover_event::{EntityTooltipInfo, HoverEvent, HoverEventAction};
use crate::network::chat::style::Style;
use crate::network::chat::text_color::TextColor;
use crate::nbt::{snbt, snbt_parse};
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use serde_json::{json, Map, Value};

/// `ComponentSerialization.CODEC` → JSON.
pub fn component_to_json(component: &Component) -> Value {
    // Коллапс в строку — только чистый literal без стилей и siblings
    // (аналог `tryCollapseToString` + encode-маппинг).
    if component.siblings.is_empty()
        && component.style.is_empty()
        && matches!(&component.contents, ComponentContents::PlainText(_))
    {
        let ComponentContents::PlainText(text) = &component.contents else { unreachable!() };
        return Value::String(text.clone());
    }

    let mut obj = Map::new();
    write_contents_json(&component.contents, &mut obj);
    if !component.siblings.is_empty() {
        obj.insert(
            "extra".to_string(),
            Value::Array(component.siblings.iter().map(component_to_json).collect()),
        );
    }
    write_style_json(&component.style, &mut obj);
    Value::Object(obj)
}

fn write_contents_json(contents: &ComponentContents, obj: &mut Map<String, Value>) {
    match contents {
        ComponentContents::PlainText(text) => {
            obj.insert("text".to_string(), Value::String(text.clone()));
        }
        ComponentContents::Translatable(t) => {
            obj.insert("translate".to_string(), Value::String(t.key.clone()));
            if let Some(fallback) = &t.fallback {
                obj.insert("fallback".to_string(), Value::String(fallback.clone()));
            }
            if !t.args.is_empty() {
                obj.insert(
                    "with".to_string(),
                    Value::Array(
                        t.args
                            .iter()
                            .map(|a| match a {
                                TranslatableArg::Component(c) => component_to_json(c),
                                TranslatableArg::Text(s) => Value::String(s.clone()),
                            })
                            .collect(),
                    ),
                );
            }
        }
        ComponentContents::Keybind(key) => {
            obj.insert("keybind".to_string(), Value::String(key.clone()));
        }
        ComponentContents::Score { name, objective } => {
            let name_json = match name {
                crate::util::either::Either::Left(sel) => Value::String(sel.pattern.clone()),
                crate::util::either::Either::Right(s) => Value::String(s.clone()),
            };
            obj.insert(
                "score".to_string(),
                json!({ "name": name_json, "objective": objective }),
            );
        }
        ComponentContents::Selector { selector, separator } => {
            let mut sel = Map::new();
            sel.insert("selector".to_string(), Value::String(selector.pattern.clone()));
            if let Some(sep) = separator {
                sel.insert("separator".to_string(), component_to_json(sep));
            }
            obj.insert("selector".to_string(), Value::Object(sel));
        }
        ComponentContents::Nbt {
            nbt_path_pattern,
            interpreting,
            separator,
            data_source,
        } => {
            obj.insert("nbt".to_string(), Value::String(nbt_path_pattern.clone()));
            if *interpreting {
                obj.insert("interpret".to_string(), Value::Bool(true));
            }
            if let Some(sep) = separator {
                obj.insert("separator".to_string(), component_to_json(sep));
            }
            write_data_source_json(data_source, obj);
        }
        ComponentContents::Object(info) => {
            obj.insert("object".to_string(), object_info_to_json(info));
        }
    }
}

fn write_data_source_json(source: &DataSource, obj: &mut Map<String, Value>) {
    match source {
        DataSource::Block(b) => {
            obj.insert("block".to_string(), Value::String(b.pos_pattern.clone()));
        }
        DataSource::Entity(e) => {
            obj.insert("entity".to_string(), Value::String(e.selector_pattern.clone()));
        }
        DataSource::Storage(s) => {
            obj.insert("storage".to_string(), Value::String(s.id.to_string()));
        }
    }
}

fn object_info_to_json(info: &ObjectInfo) -> Value {
    match info {
        ObjectInfo::AtlasSprite { atlas, sprite } => json!({
            "atlas": { "sprite": sprite.to_string(), "atlas": atlas.to_string() },
        }),
        ObjectInfo::PlayerSprite { profile_name, profile_id, hat } => json!({
            "player": {
                "name": profile_name,
                "id": profile_id.as_ref().map(|u| u.to_string()),
                "hat": hat,
            },
        }),
    }
}

fn write_style_json(style: &Style, obj: &mut Map<String, Value>) {
    if let Some(color) = &style.color {
        obj.insert("color".to_string(), Value::String(color.serialize()));
    }
    if let Some(shadow) = style.shadow_color {
        obj.insert("shadow_color".to_string(), json!(shadow));
    }
    for (key, value) in [
        ("bold", style.bold),
        ("italic", style.italic),
        ("underlined", style.underlined),
        ("strikethrough", style.strikethrough),
        ("obfuscated", style.obfuscated),
    ] {
        if let Some(v) = value {
            obj.insert(key.to_string(), Value::Bool(v));
        }
    }
    if let Some(font) = &style.font {
        obj.insert("font".to_string(), font_to_json(font));
    }
    if let Some(click) = &style.click_event {
        obj.insert("click_event".to_string(), click_event_to_json(click));
    }
    if let Some(hover) = &style.hover_event {
        obj.insert("hover_event".to_string(), hover_event_to_json(hover));
    }
    if let Some(insertion) = &style.insertion {
        obj.insert("insertion".to_string(), Value::String(insertion.clone()));
    }
}

fn font_to_json(font: &FontDescription) -> Value {
    match font {
        FontDescription::Resource(id) => Value::String(id.to_string()),
        FontDescription::AtlasSprite { atlas_id, sprite_id } => json!({
            "type": "atlas",
            "atlas": atlas_id.to_string(),
            "sprite": sprite_id.to_string(),
        }),
        FontDescription::PlayerSprite { profile_name, profile_id, hat } => json!({
            "type": "player",
            "name": profile_name,
            "id": profile_id.as_ref().map(|u| u.to_string()),
            "hat": hat,
        }),
    }
}

fn click_event_to_json(event: &ClickEvent) -> Value {
    let mut obj = Map::new();
    obj.insert("action".to_string(), Value::String(event.action().get_serialized_name().to_string()));
    match event {
        ClickEvent::OpenUrl(url) => {
            obj.insert("url".to_string(), Value::String(url.clone()));
        }
        ClickEvent::OpenFile(path) => {
            obj.insert("file".to_string(), Value::String(path.clone()));
        }
        ClickEvent::RunCommand(cmd) => {
            obj.insert("command".to_string(), Value::String(cmd.clone()));
        }
        ClickEvent::SuggestCommand(cmd) => {
            obj.insert("suggest_command".to_string(), Value::String(cmd.clone()));
        }
        ClickEvent::ShowDialog(id) => {
            obj.insert("dialog".to_string(), Value::String(id.clone()));
        }
        ClickEvent::ChangePage(page) => {
            obj.insert("change_page".to_string(), json!(page));
        }
        ClickEvent::CopyToClipboard(value) => {
            obj.insert("copy_to_clipboard".to_string(), Value::String(value.clone()));
        }
        ClickEvent::Custom { id, payload } => {
            obj.insert("id".to_string(), Value::String(id.to_string()));
            if let Some(tag) = payload {
                obj.insert("payload".to_string(), Value::String(snbt::to_string(tag)));
            }
        }
    }
    Value::Object(obj)
}

fn hover_event_to_json(event: &HoverEvent) -> Value {
    let mut obj = Map::new();
    obj.insert("action".to_string(), Value::String(event.action().get_serialized_name().to_string()));
    match event {
        HoverEvent::ShowText(component) => {
            obj.insert("value".to_string(), component_to_json(component));
        }
        HoverEvent::ShowItem(stack) => {
            obj.insert("item".to_string(), item_stack_to_json(stack));
        }
        HoverEvent::ShowEntity(info) => {
            obj.insert(
                "contents".to_string(),
                json!({
                    "type": "entity",
                    "id": info.entity_type.to_string(),
                    "uuid": info.uuid.to_string(),
                    "name": info.name.as_ref().map(|c| component_to_json(c)),
                }),
            );
        }
    }
    Value::Object(obj)
}

/// `ItemStack.MAP_CODEC` — JSON-форма (`id`, `count`, `components`).
/// `components` — snapshot NBT → SNBT-строка (DataComponentPatch — позже).
fn item_stack_to_json(stack: &crate::world::item::ItemStack) -> Value {
    let mut obj = Map::new();
    if let Some(id) = stack.item_identifier() {
        obj.insert("id".to_string(), Value::String(id.to_string()));
    }
    if stack.count != 1 {
        obj.insert("count".to_string(), json!(stack.count));
    }
    // components: SNBT-строка patch'а (persistent-кодеки
    // DataComponentType портируются отдельно — до них пишем
    // отладочное представление записей patch).
    let patch = stack.components.as_patch();
    if !patch.is_empty() {
        let mut tag = crate::nbt::CompoundTag::new();
        for (type_id, value) in patch.entries() {
            let v = match value {
                crate::core::component::ComponentPatchValue::Set(v) => v.value_debug(),
                crate::core::component::ComponentPatchValue::Removed => "<removed>".to_string(),
            };
            tag.put(type_id.to_string(), crate::nbt::Tag::String(v));
        }
        obj.insert("components".to_string(), Value::String(snbt::compound_to_string(&tag)));
    }
    Value::Object(obj)
}

// ------------------------------------------------------------ decode

/// `ComponentSerialization.CODEC.decode` из JSON.
pub fn component_from_json(value: &Value) -> Result<Component, String> {
    match value {
        Value::String(s) => Ok(Component::literal(s.clone())),
        Value::Array(items) => {
            let mut iter = items.iter();
            let first = iter
                .next()
                .ok_or_else(|| "empty component array".to_string())?;
            let mut component = component_from_json(first)?;
            for item in iter {
                component.append(component_from_json(item)?);
            }
            Ok(component)
        }
        Value::Object(obj) => component_from_object(obj),
        _ => Err(format!("expected string, array or object, got {}", value)),
    }
}

fn component_from_object(obj: &Map<String, Value>) -> Result<Component, String> {
    let contents = read_contents_json(obj)?;
    let mut component = Component::new(contents);

    if let Some(extra) = obj.get("extra") {
        let arr = extra
            .as_array()
            .ok_or_else(|| "\"extra\" must be an array".to_string())?;
        for item in arr {
            component.append(component_from_json(item)?);
        }
    }

    component.style = read_style_json(obj)?;
    Ok(component)
}

fn read_contents_json(obj: &Map<String, Value>) -> Result<ComponentContents, String> {
    if let Some(v) = obj.get("text") {
        return Ok(ComponentContents::PlainText(
            v.as_str().ok_or("\"text\" must be a string")?.to_string(),
        ));
    }
    if let Some(v) = obj.get("translate") {
        let key = v.as_str().ok_or("\"translate\" must be a string")?.to_string();
        let fallback = obj
            .get("fallback")
            .and_then(|f| f.as_str())
            .map(|s| s.to_string());
        let args = match obj.get("with") {
            Some(Value::Array(arr)) => arr
                .iter()
                .map(|a| match a {
                    Value::String(s) => Ok(TranslatableArg::Text(s.clone())),
                    other => component_from_json(other).map(|c| TranslatableArg::Component(Box::new(c))),
                })
                .collect::<Result<Vec<_>, String>>()?,
            Some(_) => return Err("\"with\" must be an array".to_string()),
            None => Vec::new(),
        };
        return Ok(ComponentContents::Translatable(
            crate::network::chat::component_contents::TranslatableContents::new(key, fallback, args),
        ));
    }
    if let Some(v) = obj.get("keybind") {
        return Ok(ComponentContents::Keybind(
            v.as_str().ok_or("\"keybind\" must be a string")?.to_string(),
        ));
    }
    if let Some(v) = obj.get("score") {
        let score = v.as_object().ok_or("\"score\" must be an object")?;
        let name_v = score.get("name").ok_or("score.name required")?;
        let name = match name_v {
            Value::String(s) if s.starts_with('@') => {
                crate::util::either::Either::Left(SelectorPattern::new(s.clone()))
            }
            Value::String(s) => crate::util::either::Either::Right(s.clone()),
            _ => return Err("score.name must be a string".to_string()),
        };
        let objective = score
            .get("objective")
            .and_then(|o| o.as_str())
            .ok_or("score.objective required")?
            .to_string();
        return Ok(ComponentContents::Score { name, objective });
    }
    if let Some(v) = obj.get("selector") {
        let sel = v.as_object().ok_or("\"selector\" must be an object")?;
        let pattern = sel
            .get("selector")
            .and_then(|s| s.as_str())
            .ok_or("selector.selector required")?;
        let separator = sel
            .get("separator")
            .map(component_from_json)
            .transpose()?;
        return Ok(ComponentContents::Selector {
            selector: SelectorPattern::new(pattern),
            separator: separator.map(Box::new),
        });
    }
    if let Some(v) = obj.get("nbt") {
        let path = v.as_str().ok_or("\"nbt\" must be a string")?.to_string();
        let interpreting = obj
            .get("interpret")
            .and_then(|i| i.as_bool())
            .unwrap_or(false);
        let separator = obj
            .get("separator")
            .map(component_from_json)
            .transpose()?;
        let data_source = read_data_source_json(obj)?;
        return Ok(ComponentContents::Nbt {
            nbt_path_pattern: path,
            interpreting,
            separator: separator.map(Box::new),
            data_source,
        });
    }
    if obj.contains_key("object") {
        return Ok(ComponentContents::Object(read_object_info_json(obj)?));
    }
    Err("component object has no content key".to_string())
}

fn read_data_source_json(obj: &Map<String, Value>) -> Result<DataSource, String> {
    if let Some(v) = obj.get("block") {
        return Ok(DataSource::Block(BlockDataSource::new(
            v.as_str().ok_or("\"block\" must be a string")?,
        )));
    }
    if let Some(v) = obj.get("entity") {
        return Ok(DataSource::Entity(EntityDataSource {
            selector_pattern: v.as_str().ok_or("\"entity\" must be a string")?.to_string(),
        }));
    }
    if let Some(v) = obj.get("storage") {
        let id = Identifier::parse(v.as_str().ok_or("\"storage\" must be a string")?)
            .map_err(|e| e.to_string())?;
        return Ok(DataSource::Storage(StorageDataSource { id }));
    }
    Err("nbt component requires block/entity/storage data source".to_string())
}

fn read_object_info_json(obj: &Map<String, Value>) -> Result<ObjectInfo, String> {
    let v = obj.get("object").ok_or("object required")?;
    // Формы: {"object": {"atlas": {...}}}, {"object": {"player": {...}}}
    // или плоская {"object": "atlas", "sprite": ...}.
    if let Value::Object(inner) = v {
        if let Some(atlas) = inner.get("atlas") {
            let a = atlas.as_object().ok_or("atlas must be object")?;
            let sprite = a
                .get("sprite")
                .and_then(|s| s.as_str())
                .ok_or("atlas.sprite required")?;
            let atlas_id = a
                .get("atlas")
                .and_then(|s| s.as_str())
                .map(Identifier::parse)
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| {
                    Identifier::with_default_namespace("blocks").unwrap()
                });
            return Ok(ObjectInfo::AtlasSprite {
                atlas: atlas_id,
                sprite: Identifier::parse(sprite).map_err(|e| e.to_string())?,
            });
        }
        if let Some(player) = inner.get("player") {
            let p = player.as_object().ok_or("player must be object")?;
            let name = p
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or("player.name required")?
                .to_string();
            let id = p
                .get("id")
                .and_then(|i| i.as_str())
                .and_then(Uuid::from_string);
            let hat = p.get("hat").and_then(|h| h.as_bool()).unwrap_or(true);
            return Ok(ObjectInfo::PlayerSprite {
                profile_name: name,
                profile_id: id,
                hat,
            });
        }
    }
    Err("unsupported object info".to_string())
}

fn read_style_json(obj: &Map<String, Value>) -> Result<Style, String> {
    let mut style = Style::empty();
    if let Some(v) = obj.get("color") {
        let s = v.as_str().ok_or("color must be a string")?;
        style.color = Some(TextColor::parse_color(s).map_err(|e| e)?);
    }
    if let Some(v) = obj.get("shadow_color") {
        style.shadow_color = Some(
            v.as_u64()
                .ok_or("shadow_color must be a number")? as u32,
        );
    }
    for (key, field) in [
        ("bold", &mut style.bold),
        ("italic", &mut style.italic),
        ("underlined", &mut style.underlined),
        ("strikethrough", &mut style.strikethrough),
        ("obfuscated", &mut style.obfuscated),
    ] {
        if let Some(v) = obj.get(key) {
            // NBT round-trip: bool приходит как byte 0/1
            *field = Some(match v {
                Value::Bool(b) => *b,
                Value::Number(n) => n.as_i64().map_or(false, |x| x != 0),
                _ => return Err(format!("{} must be bool", key)),
            });
        }
    }
    if let Some(v) = obj.get("font") {
        style.font = Some(read_font_json(v)?);
    }
    if let Some(v) = obj.get("click_event") {
        style.click_event = Some(read_click_event_json(v)?);
    }
    if let Some(v) = obj.get("hover_event") {
        style.hover_event = Some(read_hover_event_json(v)?);
    }
    if let Some(v) = obj.get("insertion") {
        style.insertion = Some(
            v.as_str()
                .ok_or("insertion must be a string")?
                .to_string(),
        );
    }
    Ok(style)
}

fn read_font_json(v: &Value) -> Result<FontDescription, String> {
    match v {
        Value::String(s) => Identifier::parse(s)
            .map(FontDescription::Resource)
            .map_err(|e| e.to_string()),
        Value::Object(obj) => {
            let kind = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match kind {
                "atlas" => Ok(FontDescription::AtlasSprite {
                    atlas_id: Identifier::parse(
                        obj.get("atlas").and_then(|a| a.as_str()).unwrap_or("minecraft:blocks"),
                    )
                    .map_err(|e| e.to_string())?,
                    sprite_id: Identifier::parse(
                        obj.get("sprite")
                            .and_then(|s| s.as_str())
                            .ok_or("font.sprite required")?,
                    )
                    .map_err(|e| e.to_string())?,
                }),
                "player" => Ok(FontDescription::PlayerSprite {
                    profile_name: obj
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or("font.name required")?
                        .to_string(),
                    profile_id: obj
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(Uuid::from_string),
                    hat: obj.get("hat").and_then(|h| h.as_bool()).unwrap_or(true),
                }),
                _ => Err(format!("unknown font type: {}", kind)),
            }
        }
        _ => Err("font must be string or object".to_string()),
    }
}

fn click_action_name(action: ClickEventAction) -> &'static str {
    action.get_serialized_name()
}

fn read_click_event_json(v: &Value) -> Result<ClickEvent, String> {
    let obj = v.as_object().ok_or("click_event must be object")?;
    let action = obj
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or("click_event.action required")?;
    let get_str = |key: &str| -> Result<String, String> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("click_event.{} required", key))
    };
    match action {
        "open_url" => Ok(ClickEvent::OpenUrl(get_str("url")?)),
        "open_file" => Ok(ClickEvent::OpenFile(get_str("file")?)),
        "run_command" => Ok(ClickEvent::RunCommand(get_str("command")?)),
        "suggest_command" => Ok(ClickEvent::SuggestCommand(get_str("suggest_command")?)),
        "show_dialog" => Ok(ClickEvent::ShowDialog(get_str("dialog")?)),
        "change_page" => Ok(ClickEvent::ChangePage(
            obj.get("change_page")
                .and_then(|p| p.as_i64())
                .ok_or("click_event.change_page required")? as i32,
        )),
        "copy_to_clipboard" => Ok(ClickEvent::CopyToClipboard(get_str("copy_to_clipboard")?)),
        "custom" => {
            let id = Identifier::parse(&get_str("id")?).map_err(|e| e.to_string())?;
            let payload = obj
                .get("payload")
                .and_then(|p| p.as_str())
                .and_then(|s| snbt_parse::parse(s).ok());
            Ok(ClickEvent::Custom { id, payload })
        }
        other => Err(format!("unknown click_event action: {}", other)),
    }
    .map(|e| {
        debug_assert_eq!(click_action_name(e.action()), action);
        e
    })
}

fn read_hover_event_json(v: &Value) -> Result<HoverEvent, String> {
    let obj = v.as_object().ok_or("hover_event must be object")?;
    let action = obj
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or("hover_event.action required")?;
    match action {
        "show_text" => Ok(HoverEvent::ShowText(Box::new(
            component_from_json(obj.get("value").ok_or("hover_event.value required")?)?,
        ))),
        "show_item" => {
            let item = obj.get("item").ok_or("hover_event.item required")?;
            let iobj = item.as_object().ok_or("item must be object")?;
            let id = Identifier::parse(
                iobj.get("id").and_then(|i| i.as_str()).ok_or("item.id required")?,
            )
            .map_err(|e| e.to_string())?;
            let count = iobj.get("count").and_then(|c| c.as_i64()).unwrap_or(1) as i32;
            // components: SNBT-строка (см. item_stack_to_json)
            let components = iobj
                .get("components")
                .and_then(|c| c.as_str())
                .and_then(|s| snbt_parse::parse(s).ok())
                .and_then(|t| t.as_compound().cloned());
            // components из SNBT пока не разворачиваются в typed patch
            // (persistent-кодеки — отдельный этап); храним только item+count.
            let _ = components;
            Ok(HoverEvent::ShowItem(crate::world::item::ItemStack::new(
                crate::core::holder::Holder::Direct(crate::world::item::item::Item::new(id)),
                count,
            )))
        }
        "show_entity" => {
            let contents = obj
                .get("contents")
                .and_then(|c| c.as_object())
                .ok_or("hover_event.contents required")?;
            if contents.get("type").and_then(|t| t.as_str()) != Some("entity") {
                return Err("unsupported hover contents type".to_string());
            }
            let entity_type = Identifier::parse(
                contents
                    .get("id")
                    .and_then(|i| i.as_str())
                    .ok_or("contents.id required")?,
            )
            .map_err(|e| e.to_string())?;
            let uuid = Uuid::from_string(
                contents
                    .get("uuid")
                    .and_then(|u| u.as_str())
                    .ok_or("contents.uuid required")?,
            )
            .ok_or("invalid uuid")?;
            let name = contents
                .get("name")
                .map(component_from_json)
                .transpose()?;
            Ok(HoverEvent::ShowEntity(EntityTooltipInfo::new(
                entity_type,
                uuid,
                name,
            )))
        }
        other => Err(format!("unknown hover_event action: {}", other)),
    }
}

// ------------------------------------------------------------ stream codec

use crate::network::codec::stream_codec::StreamCodec;

/// `ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC`
/// (`ByteBufCodecs.fromCodecTrusted(CODEC)`) — компонент как NBT tag
/// (JSON-представление кодека, конвертированное через NbtOps).
pub fn component_stream_codec() -> impl StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, Component> {
    crate::network::codec::stream_codec::sync_fn(
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, c: &Component| {
            let json = component_to_json(c);
            let tag = crate::nbt::json::tag_from_json(&json).expect("component nbt encode");
            buf.write_nbt(Some(&tag)).expect("component nbt write");
        },
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
            let tag = buf.read_nbt().expect("invalid component nbt").unwrap_or(crate::nbt::tag::Tag::End);
            let value = crate::nbt::json::json_from_tag(&tag).expect("component nbt decode");
            component_from_json(&value).expect("invalid component")
        },
    )
}

/// `OPTIONAL_STREAM_CODEC` — Option<Component>.
pub fn optional_component_stream_codec() -> impl StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, Option<Component>> {
    crate::network::codec::stream_codec::sync_fn(
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, v: &Option<Component>| {
            buf.write_bool(v.is_some());
            if let Some(c) = v {
                component_stream_codec().encode(buf, c);
            }
        },
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
            if !buf.read_bool() {
                return None;
            }
            Some(component_stream_codec().decode(buf))
        },
    )
}

// ------------------------------------------------------------ style codec JSON

/// `Style.Serializer.CODEC` — JSON-представление (для NBT stream codec
/// и `ChatTypeDecoration.STREAM_CODEC`).
pub fn style_to_json(style: &crate::network::chat::style::Style) -> Value {
    let mut obj = Map::new();
    write_style_json(style, &mut obj);
    Value::Object(obj)
}

/// Обратная конвертация `style_to_json`.
pub fn style_from_json(value: &Value) -> Result<crate::network::chat::style::Style, String> {
    match value {
        Value::Object(obj) => read_style_json(obj),
        _ => Err("style must be an object".to_string()),
    }
}
