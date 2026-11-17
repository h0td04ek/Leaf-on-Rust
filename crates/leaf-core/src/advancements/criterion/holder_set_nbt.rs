//! Перенос `net.minecraft.core.HolderSet` NBT-формы
//! (`RegistryCodecs.homogeneousList` / `HolderSetCodec`):
//! `either(TagKey.hashedCodec, compactList(holderCodec))`.
//!
//! - `HolderSet::Named(tag)` → `"#namespace:path"`;
//! - `HolderSet::Direct(list)` → compactList: 1 элемент — сам
//!   элемент, иначе список; элемент — строка-id или inline-значение
//!   (для registry-элементов — строка `namespace:path`).
//!
//! `Holder::Key` хранит ResourceKey → строка identifier;
//! `Holder::Reference(id)` без привязки не сериализуется в NBT
//! (в Java reference всегда знает свой key).

use crate::core::holder::{Holder, HolderKind};
use crate::core::holder_set::HolderSet;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `TagKey.hashedCodec` — `"#" + location`.
pub fn tag_key_encode(location: &Identifier) -> Tag {
    Tag::String(format!("#{}", location))
}

/// `TagKey.hashedCodec` decode — `"#loc"` → location.
pub fn tag_key_decode(tag: &Tag) -> Result<Identifier, String> {
    match tag {
        Tag::String(s) if s.starts_with('#') => {
            Identifier::parse(&s[1..]).map_err(|e| format!("invalid tag key: {e}"))
        }
        _ => Err("expected '#tag' string for TagKey".into()),
    }
}

/// Элемент holder-set'а → NBT (registry-элемент = строка-id).
/// `direct_value_to_id` — для `Holder::Direct` (inline-значение
/// реестра, например ItemStack-подобные записи): возвращает id.
pub fn holder_element_encode<T>(
    holder: &Holder<T>,
    direct_value_to_id: impl Fn(&T) -> Option<String>,
) -> Result<Tag, String> {
    match holder {
        Holder::Key(key) => Ok(Tag::String(key.identifier.to_string())),
        Holder::Reference(_) => Err("unbound holder reference cannot be NBT-encoded".into()),
        Holder::Direct(v) => direct_value_to_id(v)
            .map(Tag::String)
            .ok_or_else(|| "direct holder value has no registry id".into()),
    }
}

/// holder-set элемент из строки-id → `Holder::Key`.
/// `#tag` обрабатывается уровнем выше (HolderSet::Named).
pub fn holder_element_decode<T>(
    tag: &Tag,
    registry_name: Identifier,
) -> Result<Holder<T>, String> {
    let id = match tag {
        Tag::String(s) => s.clone(),
        _ => return Err("expected string element for holder set".into()),
    };
    let location = Identifier::parse(&id).map_err(|e| format!("invalid holder id: {e}"))?;
    Ok(Holder::Key(ResourceKey::create_with_registry_name(
        registry_name,
        location,
    )))
}

/// `HolderSet<T>` → NBT.
pub fn holder_set_encode<T>(
    set: &HolderSet<T>,
    registry_name: &Identifier,
    direct_value_to_id: impl Fn(&T) -> Option<String>,
) -> Result<Tag, String> {
    let _ = registry_name;
    match set {
        HolderSet::Named(tag) => Ok(tag_key_encode(tag)),
        HolderSet::Direct(list) => {
            let mut elements = Vec::with_capacity(list.len());
            for h in list {
                elements.push(holder_element_encode(h, &direct_value_to_id)?);
            }
            Ok(crate::advancements::criterion::nbt_util::encode_compact_list(
                &elements,
                |t| t.clone(),
            ))
        }
    }
}

/// `HolderSet<T>` ← NBT.
pub fn holder_set_decode<T>(
    tag: &Tag,
    registry_name: Identifier,
) -> Result<HolderSet<T>, String> {
    match tag {
        Tag::String(s) if s.starts_with('#') => {
            let location = tag_key_decode(tag)?;
            let _ = s;
            Ok(HolderSet::Named(location))
        }
        _ => {
            let elements = crate::advancements::criterion::nbt_util::decode_compact_list(
                tag,
                |t| holder_element_decode::<T>(t, registry_name.clone()),
            )?;
            Ok(HolderSet::Direct(elements))
        }
    }
}

/// `HolderSet::contains(holder)` — для matches-проверок предикатов.
/// Key↔Key по ResourceKey, Reference↔Reference по id.
/// (Java `HolderSet.contains` работает через kind+key/id.)
pub fn holder_set_contains<T: PartialEq>(set: &HolderSet<T>, holder: &Holder<T>) -> bool
where
    T: Send + Sync + 'static,
{
    match set {
        HolderSet::Named(_) => false, // named-теги резолвятся через реестр
        HolderSet::Direct(list) => list.iter().any(|h| holder_eq(h, holder)),
    }
}

fn holder_eq<T: PartialEq>(a: &Holder<T>, b: &Holder<T>) -> bool {
    match (a, b) {
        (Holder::Key(x), Holder::Key(y)) => x == y,
        (Holder::Reference(x), Holder::Reference(y)) => x == y,
        (Holder::Key(x), Holder::Reference(y)) | (Holder::Reference(y), Holder::Key(x)) => {
            // key vs id — без реестра не сравнить; считаем не равными
            let _ = (x, y);
            false
        }
        (Holder::Direct(x), Holder::Direct(y)) => x == y,
        _ => false,
    }
}

/// `Holder::kind` проброс для внешнего кода.
pub fn holder_kind<T>(h: &Holder<T>) -> HolderKind {
    h.kind()
}

/// ResourceKey реестра по имени — helper для вызовов выше.
pub fn registry_name(name: &str) -> Identifier {
    Identifier::minecraft(name)
}
