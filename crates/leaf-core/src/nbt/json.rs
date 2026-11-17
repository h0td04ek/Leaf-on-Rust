//! JSON ↔ NBT `Tag` конвертер — для кодеков, идущих по сети как NBT
//! (`ByteBufCodecs.fromCodecTrusted` / `tagCodec`).
//!
//! Маппинг (аналог Mojang `JsonOps` поверх `NbtOps`):
//! - object → Compound
//! - array → List
//! - string → String
//! - bool → Byte(0/1)   (NBT не имеет bool; Java-кодеки читают byte)
//! - integer → Int/Long (если не влезает в i32 — Long)
//! - float → Double (JSON не различает int/float; NBT-кодеки обычно читают Double)

use serde_json::Value;

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;

/// `JsonOps`-подобная конвертация JSON → NBT Tag.
pub fn tag_from_json(value: &Value) -> Result<Tag, String> {
    match value {
        Value::Null => Ok(Tag::End),
        Value::Bool(b) => Ok(Tag::Byte(if *b { 1 } else { 0 })),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Ok(Tag::Int(i as i32))
                } else {
                    Ok(Tag::Long(i))
                }
            } else if let Some(u) = n.as_u64() {
                if u <= i32::MAX as u64 {
                    Ok(Tag::Int(u as i32))
                } else if u <= i64::MAX as u64 {
                    Ok(Tag::Long(u as i64))
                } else {
                    Ok(Tag::Double(n.as_f64().unwrap_or(0.0)))
                }
            } else {
                Ok(Tag::Double(n.as_f64().ok_or("bad number")?))
            }
        }
        Value::String(s) => Ok(Tag::String(s.clone())),
        Value::Array(arr) => {
            let mut tags = Vec::with_capacity(arr.len());
            for v in arr {
                tags.push(tag_from_json(v)?);
            }
            Ok(Tag::List(tags))
        }
        Value::Object(obj) => {
            let mut compound = CompoundTag::new();
            for (k, v) in obj {
                let tag = tag_from_json(v)?;
                if !matches!(tag, Tag::End) {
                    compound.put(k.clone(), tag);
                }
            }
            Ok(Tag::Compound(compound))
        }
    }
}

/// NBT Tag → JSON Value (`NbtOps`-подобно).
pub fn json_from_tag(tag: &Tag) -> Result<Value, String> {
    match tag {
        Tag::End => Ok(Value::Null),
        Tag::Byte(b) => Ok(Value::from(*b)),
        Tag::Short(s) => Ok(Value::from(*s)),
        Tag::Int(i) => Ok(Value::from(*i)),
        Tag::Long(l) => Ok(Value::from(*l)),
        Tag::Float(f) => Ok(Value::from(*f)),
        Tag::Double(d) => Ok(Value::from(*d)),
        Tag::ByteArray(arr) => Ok(Value::Array(arr.iter().map(|b| Value::from(*b)).collect())),
        Tag::String(s) => Ok(Value::String(s.clone())),
        Tag::List(list) => {
            let mut out = Vec::with_capacity(list.len());
            for t in list {
                out.push(json_from_tag(t)?);
            }
            Ok(Value::Array(out))
        }
        Tag::Compound(c) => {
            let mut map = serde_json::Map::new();
            for (k, v) in c.iter() {
                map.insert(k.clone(), json_from_tag(v)?);
            }
            Ok(Value::Object(map))
        }
        Tag::IntArray(arr) => Ok(Value::Array(arr.iter().map(|i| Value::from(*i)).collect())),
        Tag::LongArray(arr) => Ok(Value::Array(arr.iter().map(|l| Value::from(*l)).collect())),
    }
}
