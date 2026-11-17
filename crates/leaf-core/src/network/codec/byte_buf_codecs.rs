//! Перенос `net.minecraft.network.codec.ByteBufCodecs` — стандартные кодеки.
//! Реестрозависимые кодеки (registry, holder, holderSet) — после порта
//! RegistryFriendlyByteBuf/Holder.

use crate::core::block_pos::BlockPos;
use crate::core::vec3i::Vec3i;
use crate::nbt::accounter::NbtAccounter;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::{CodecError, FriendlyByteBuf};
use crate::network::{utf8_string, var_int, var_long};
use crate::resources::Identifier;
use crate::util::argb;
use crate::util::mth;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::phys::Vec3;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Ошибка декодирования → panic (аналог DecoderException).
fn dec_err(msg: impl Into<String>) -> ! {
    panic!("DecoderException: {}", msg.into())
}

fn enc_err(msg: impl Into<String>) -> ! {
    panic!("EncoderException: {}", msg.into())
}

// ---------------------------------------------------------- примитивы

#[derive(Clone, Copy)]
pub struct BoolCodec;
impl StreamCodec<FriendlyByteBuf, bool> for BoolCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &bool) {
        buf.write_bool(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> bool {
        buf.read_bool()
    }
}
pub const BOOL: BoolCodec = BoolCodec;

#[derive(Clone, Copy)]
pub struct ByteCodec;
impl StreamCodec<FriendlyByteBuf, i8> for ByteCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i8) {
        buf.write_byte(*value as i32);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i8 {
        buf.read_byte()
    }
}
pub const BYTE: ByteCodec = ByteCodec;

/// `ROTATION_BYTE` — byte → float degrees через Mth.unpackDegrees.
#[derive(Clone, Copy)]
pub struct RotationByteCodec;
impl StreamCodec<FriendlyByteBuf, f32> for RotationByteCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &f32) {
        buf.write_byte(mth::pack_degrees(*value) as i32);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> f32 {
        mth::unpack_degrees(buf.read_byte())
    }
}
pub const ROTATION_BYTE: RotationByteCodec = RotationByteCodec;

#[derive(Clone, Copy)]
pub struct ShortCodec;
impl StreamCodec<FriendlyByteBuf, i16> for ShortCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i16) {
        buf.write_short(*value as i32);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i16 {
        buf.read_short()
    }
}
pub const SHORT: ShortCodec = ShortCodec;

#[derive(Clone, Copy)]
pub struct UnsignedShortCodec;
impl StreamCodec<FriendlyByteBuf, i32> for UnsignedShortCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i32) {
        buf.write_short(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i32 {
        buf.read_unsigned_short()
    }
}
pub const UNSIGNED_SHORT: UnsignedShortCodec = UnsignedShortCodec;

#[derive(Clone, Copy)]
pub struct IntCodec;
impl StreamCodec<FriendlyByteBuf, i32> for IntCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i32) {
        buf.write_int(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i32 {
        buf.read_int()
    }
}
pub const INT: IntCodec = IntCodec;

/// `OPTIONAL_VAR_INT` — -1 → None.
#[derive(Clone, Copy)]
pub struct OptionalVarIntCodec;
impl StreamCodec<FriendlyByteBuf, Option<i32>> for OptionalVarIntCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Option<i32>) {
        buf.write_var_int(value.unwrap_or(-1));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Option<i32> {
        let i = buf.read_var_int();
        if i == -1 { None } else { Some(i) }
    }
}
pub const OPTIONAL_VAR_INT: OptionalVarIntCodec = OptionalVarIntCodec;

#[derive(Clone, Copy)]
pub struct VarIntCodec;
impl StreamCodec<FriendlyByteBuf, i32> for VarIntCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i32) {
        var_int::write(buf, *value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i32 {
        var_int::read(buf)
    }
}
pub const VAR_INT: VarIntCodec = VarIntCodec;

#[derive(Clone, Copy)]
pub struct LongCodec;
impl StreamCodec<FriendlyByteBuf, i64> for LongCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i64) {
        buf.write_long(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i64 {
        buf.read_long()
    }
}
pub const LONG: LongCodec = LongCodec;

#[derive(Clone, Copy)]
pub struct VarLongCodec;
impl StreamCodec<FriendlyByteBuf, i64> for VarLongCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i64) {
        var_long::write(buf, *value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i64 {
        var_long::read(buf)
    }
}
pub const VAR_LONG: VarLongCodec = VarLongCodec;

#[derive(Clone, Copy)]
pub struct FloatCodec;
impl StreamCodec<FriendlyByteBuf, f32> for FloatCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &f32) {
        buf.write_float(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> f32 {
        buf.read_float()
    }
}
pub const FLOAT: FloatCodec = FloatCodec;

#[derive(Clone, Copy)]
pub struct DoubleCodec;
impl StreamCodec<FriendlyByteBuf, f64> for DoubleCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &f64) {
        buf.write_double(*value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> f64 {
        buf.read_double()
    }
}
pub const DOUBLE: DoubleCodec = DoubleCodec;

// ---------------------------------------------------------- составные

/// `byteArray(maxSize)`
pub fn byte_array(max_size: usize) -> ByteArrayCodec {
    ByteArrayCodec { max_size }
}

#[derive(Clone, Copy)]
pub struct ByteArrayCodec {
    pub max_size: usize,
}

impl StreamCodec<FriendlyByteBuf, Vec<u8>> for ByteArrayCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Vec<u8>) {
        if value.len() > self.max_size {
            enc_err(format!("ByteArray with size {} is bigger than allowed {}", value.len(), self.max_size));
        }
        buf.write_byte_array(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Vec<u8> {
        buf.read_byte_array_n(self.max_size).unwrap_or_else(|e| dec_err(e.0))
    }
}

pub const BYTE_ARRAY: ByteArrayCodec = ByteArrayCodec { max_size: 1 << 20 }; // MAX_STRING_LENGTH*? — в Java BYTE_ARRAY = byteArray(1048576)

/// `longArray`
pub fn long_array() -> LongArrayCodec {
    LongArrayCodec
}

#[derive(Clone, Copy)]
pub struct LongArrayCodec;
impl StreamCodec<FriendlyByteBuf, Vec<i64>> for LongArrayCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Vec<i64>) {
        buf.write_long_array(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Vec<i64> {
        buf.read_long_array()
    }
}
pub const LONG_ARRAY: LongArrayCodec = LongArrayCodec;

/// `stringUtf8(maxLength)`
pub fn string_utf8(max_length: usize) -> StringUtf8Codec {
    StringUtf8Codec { max_length }
}

#[derive(Clone, Copy)]
pub struct StringUtf8Codec {
    pub max_length: usize,
}

impl StreamCodec<FriendlyByteBuf, String> for StringUtf8Codec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &String) {
        buf.write_utf_n(value, self.max_length).unwrap_or_else(|e| enc_err(e.0));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> String {
        buf.read_utf_n(self.max_length).unwrap_or_else(|e| dec_err(e.0))
    }
}

pub const STRING_UTF8: StringUtf8Codec = StringUtf8Codec { max_length: 32767 };
pub const PLAYER_NAME: StringUtf8Codec = StringUtf8Codec { max_length: 16 };

/// `tagCodec(accounter)`
pub fn tag_codec(unlimited: bool) -> TagCodec {
    TagCodec { unlimited }
}

#[derive(Clone, Copy)]
pub struct TagCodec {
    pub unlimited: bool,
}

impl StreamCodec<FriendlyByteBuf, Option<Tag>> for TagCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Option<Tag>) {
        buf.write_nbt(value.as_ref()).unwrap_or_else(|e| enc_err(e.0));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Option<Tag> {
        buf.read_nbt().unwrap_or_else(|e| dec_err(e.0))
    }
}

pub const TAG: TagCodec = TagCodec { unlimited: false };
pub const TRUSTED_TAG: TagCodec = TagCodec { unlimited: true };

/// `compoundTagCodec(accounter)`
pub fn compound_tag_codec(unlimited: bool) -> CompoundTagCodec {
    CompoundTagCodec { unlimited }
}

#[derive(Clone, Copy)]
pub struct CompoundTagCodec {
    pub unlimited: bool,
}

impl StreamCodec<FriendlyByteBuf, Option<CompoundTag>> for CompoundTagCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Option<CompoundTag>) {
        buf.write_nbt_compound(value.as_ref()).unwrap_or_else(|e| enc_err(e.0));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Option<CompoundTag> {
        buf.read_nbt_compound().unwrap_or_else(|e| dec_err(e.0))
    }
}

pub const COMPOUND_TAG: CompoundTagCodec = CompoundTagCodec { unlimited: false };
pub const TRUSTED_COMPOUND_TAG: CompoundTagCodec = CompoundTagCodec { unlimited: true };

/// `OPTIONAL_COMPOUND_TAG`
pub const OPTIONAL_COMPOUND_TAG: OptionalCompoundTagCodec = OptionalCompoundTagCodec;

#[derive(Clone, Copy)]
pub struct OptionalCompoundTagCodec;
impl StreamCodec<FriendlyByteBuf, Option<CompoundTag>> for OptionalCompoundTagCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Option<CompoundTag>) {
        // Java: Optional.of(tag) пишет compound; empty пишет EndTag? —
        // точнее: пишет bool? Проверим: OPTIONAL_COMPOUND_TAG пишет
        // tag==null ? EndTag : compound — это tagCodec с Optional-обёрткой
        // на уровне NBT (End = absent). Используем то же.
        buf.write_nbt_compound(value.as_ref()).unwrap_or_else(|e| enc_err(e.0));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Option<CompoundTag> {
        buf.read_nbt_compound().unwrap_or_else(|e| dec_err(e.0))
    }
}

/// `VECTOR3F` / `QUATERNIONF`
#[derive(Clone, Copy)]
pub struct Vector3fCodec;
impl StreamCodec<FriendlyByteBuf, crate::joml::Vector3f> for Vector3fCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &crate::joml::Vector3f) {
        buf.write_vector3f(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> crate::joml::Vector3f {
        buf.read_vector3f()
    }
}
pub const VECTOR3F: Vector3fCodec = Vector3fCodec;

#[derive(Clone, Copy)]
pub struct QuaternionfCodec;
impl StreamCodec<FriendlyByteBuf, crate::joml::Quaternionf> for QuaternionfCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &crate::joml::Quaternionf) {
        buf.write_quaternion(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> crate::joml::Quaternionf {
        buf.read_quaternion()
    }
}
pub const QUATERNIONF: QuaternionfCodec = QuaternionfCodec;

/// `CONTAINER_ID` — VarInt.
pub const CONTAINER_ID: VarIntCodec = VarIntCodec;

/// `RGB_COLOR` — VarInt ARGB (порядок: writeInt? Java: writeVarInt? проверим).
#[derive(Clone, Copy)]
pub struct RgbColorCodec;
impl StreamCodec<FriendlyByteBuf, i32> for RgbColorCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &i32) {
        buf.write_byte(argb::red(*value));
        buf.write_byte(argb::green(*value));
        buf.write_byte(argb::blue(*value));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> i32 {
        argb::color3(buf.read_unsigned_byte(), buf.read_unsigned_byte(), buf.read_unsigned_byte())
    }
}
pub const RGB_COLOR: RgbColorCodec = RgbColorCodec;

// ---------------------------------------------------------- позиции/ресурсы

/// BlockPos — packed long.
#[derive(Clone, Copy)]
pub struct BlockPosCodec;
impl StreamCodec<FriendlyByteBuf, BlockPos> for BlockPosCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &BlockPos) {
        buf.write_block_pos(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> BlockPos {
        buf.read_block_pos()
    }
}
pub const BLOCK_POS: BlockPosCodec = BlockPosCodec;

#[derive(Clone, Copy)]
pub struct ChunkPosCodec;
impl StreamCodec<FriendlyByteBuf, ChunkPos> for ChunkPosCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &ChunkPos) {
        buf.write_chunk_pos(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> ChunkPos {
        buf.read_chunk_pos()
    }
}
pub const CHUNK_POS: ChunkPosCodec = ChunkPosCodec;

/// `Identifier.STREAM_CODEC` — UTF8 string.
#[derive(Clone, Copy)]
pub struct IdentifierCodec;
impl StreamCodec<FriendlyByteBuf, Identifier> for IdentifierCodec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Identifier) {
        buf.write_identifier(value).unwrap_or_else(|e| enc_err(e.0));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Identifier {
        buf.read_identifier().unwrap_or_else(|e| dec_err(e.0))
    }
}
pub const IDENTIFIER: IdentifierCodec = IdentifierCodec;

/// `Vec3` — 3 double.
#[derive(Clone, Copy)]
pub struct Vec3Codec;
impl StreamCodec<FriendlyByteBuf, Vec3> for Vec3Codec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Vec3) {
        buf.write_vec3(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Vec3 {
        buf.read_vec3()
    }
}
pub const VEC3: Vec3Codec = Vec3Codec;

/// `LpVec3` — компактный вектор (movement packets).
#[derive(Clone, Copy)]
pub struct LpVec3Codec;
impl StreamCodec<FriendlyByteBuf, Vec3> for LpVec3Codec {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Vec3) {
        buf.write_lp_vec3(value);
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Vec3 {
        buf.read_lp_vec3()
    }
}
pub const LP_VEC3: LpVec3Codec = LpVec3Codec;

// ---------------------------------------------------------- collection helpers

/// `collection(factory, codec)`
pub fn collection<T, C, F>(
    factory: F,
    codec: impl StreamCodec<FriendlyByteBuf, T> + 'static,
) -> CollectionCodec<T, C, F>
where
    F: Fn(usize) -> C + 'static,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    C: Extend<T>,
{
    CollectionCodec { factory, codec: Box::new(codec), _t: std::marker::PhantomData }
}

pub struct CollectionCodec<T, C, F> {
    factory: F,
    codec: Box<dyn StreamCodec<FriendlyByteBuf, T>>,
    _t: std::marker::PhantomData<fn() -> (T, C)>,
}

impl<T, C, F> StreamCodec<FriendlyByteBuf, C> for CollectionCodec<T, C, F>
where
    F: Fn(usize) -> C,
    for<'a> &'a C: IntoIterator<Item = &'a T>,
    C: Extend<T>,
{
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &C) {
        let items: Vec<&T> = value.into_iter().collect();
        buf.write_var_int(items.len() as i32);
        for item in items {
            self.codec.encode(buf, item);
        }
    }

    fn decode(&self, buf: &mut FriendlyByteBuf) -> C {
        let i = buf.read_var_int();
        let mut out = (self.factory)(i.max(0) as usize);
        for _ in 0..i {
            out.extend(std::iter::once(self.codec.decode(buf)));
        }
        out
    }
}

/// `list(codec)` / `list(maxSize, codec)`
pub fn list<T: 'static>(codec: impl StreamCodec<FriendlyByteBuf, T> + 'static) -> CollectionCodec<T, Vec<T>, fn(usize) -> Vec<T>> {
    collection(Vec::with_capacity, codec)
}

/// `optional(codec)`
pub fn optional<T>(codec: impl StreamCodec<FriendlyByteBuf, T> + 'static) -> OptionalCodec<T> {
    OptionalCodec { codec: Box::new(codec) }
}

pub struct OptionalCodec<T> {
    codec: Box<dyn StreamCodec<FriendlyByteBuf, T>>,
}

impl<T: Clone> StreamCodec<FriendlyByteBuf, Option<T>> for OptionalCodec<T> {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Option<T>) {
        match value {
            Some(v) => {
                buf.write_bool(true);
                self.codec.encode(buf, v);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> Option<T> {
        if buf.read_bool() {
            Some(self.codec.decode(buf))
        } else {
            None
        }
    }
}

/// `either(left, right)`
pub fn either<L, R>(
    left: impl StreamCodec<FriendlyByteBuf, L> + 'static,
    right: impl StreamCodec<FriendlyByteBuf, R> + 'static,
) -> EitherCodec<L, R> {
    EitherCodec { left: Box::new(left), right: Box::new(right) }
}

pub struct EitherCodec<L, R> {
    left: Box<dyn StreamCodec<FriendlyByteBuf, L>>,
    right: Box<dyn StreamCodec<FriendlyByteBuf, R>>,
}

impl<L, R> StreamCodec<FriendlyByteBuf, crate::util::either::Either<L, R>> for EitherCodec<L, R> {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &crate::util::either::Either<L, R>) {
        match value {
            crate::util::either::Either::Left(l) => {
                buf.write_bool(true);
                self.left.encode(buf, l);
            }
            crate::util::either::Either::Right(r) => {
                buf.write_bool(false);
                self.right.encode(buf, r);
            }
        }
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> crate::util::either::Either<L, R> {
        if buf.read_bool() {
            crate::util::either::Either::Left(self.left.decode(buf))
        } else {
            crate::util::either::Either::Right(self.right.decode(buf))
        }
    }
}

/// `idMapper(idLookup, idGetter)`
pub fn id_mapper<T: 'static>(
    id_lookup: impl Fn(i32) -> Option<T> + 'static,
    id_getter: impl Fn(&T) -> i32 + 'static,
) -> IdMapperCodec<T> {
    IdMapperCodec { id_lookup: Box::new(id_lookup), id_getter: Box::new(id_getter) }
}

pub struct IdMapperCodec<T> {
    id_lookup: Box<dyn Fn(i32) -> Option<T>>,
    id_getter: Box<dyn Fn(&T) -> i32>,
}

impl<T> StreamCodec<FriendlyByteBuf, T> for IdMapperCodec<T> {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &T) {
        buf.write_var_int((self.id_getter)(value));
    }
    fn decode(&self, buf: &mut FriendlyByteBuf) -> T {
        let id = buf.read_var_int();
        (self.id_lookup)(id).unwrap_or_else(|| dec_err(format!("Unknown id {}", id)))
    }
}

// ---------------------------------------------------------- lengthPrefixed

// ---------------------------------------------------------- trackDepth/increaseDepth

/// `trackDepth(codec)` — подсчёт глубины NBT-like структур.
/// Упрощённо: пропускает (глубина контролируется NbtAccounter внутри NBT).
pub fn track_depth<T>(codec: impl StreamCodec<FriendlyByteBuf, T> + 'static) -> impl StreamCodec<FriendlyByteBuf, T> {
    codec
}

/// `increaseDepth` — аналогично.
pub fn increase_depth<T>(codec: impl StreamCodec<FriendlyByteBuf, T> + 'static) -> impl StreamCodec<FriendlyByteBuf, T> {
    codec
}

// ---------------------------------------------------------- game profile

/// `ByteBufCodecs.GAME_PROFILE_PROPERTIES` — PropertyMap.
pub fn game_profile_properties() -> impl StreamCodec<FriendlyByteBuf, crate::authlib::property_map::PropertyMap> {
    FnCodec::new(
        |buf: &mut FriendlyByteBuf, map: &crate::authlib::property_map::PropertyMap| {
            buf.write_var_int(map.size() as i32);
            for p in map.values() {
                buf.write_utf_n(&p.name, 64).expect("property name too long");
                buf.write_utf_n(&p.value, 32767).expect("property value too long");
                match &p.signature {
                    Some(sig) => {
                        buf.write_bool(true);
                        buf.write_utf_n(sig, 1024).expect("signature too long");
                    }
                    None => {
                        buf.write_bool(false);
                    }
                }
            }
        },
        |buf: &mut FriendlyByteBuf| {
            let count = buf.read_var_int();
            let mut map = crate::authlib::property_map::PropertyMap::new();
            for _ in 0..count {
                let name = buf.read_utf_n(64).expect("invalid property name");
                let value = buf.read_utf_n(32767).expect("invalid property value");
                let signature = if buf.read_bool() {
                    Some(buf.read_utf_n(1024).expect("invalid signature"))
                } else {
                    None
                };
                map.put(crate::authlib::property::Property::new(name, value, signature));
            }
            map
        },
    )
}

/// `ByteBufCodecs.GAME_PROFILE` — uuid + name(16) + properties.
pub fn game_profile() -> impl StreamCodec<FriendlyByteBuf, crate::authlib::game_profile::GameProfile> {
    let props = game_profile_properties();
    FnCodec::new(
        move |buf: &mut FriendlyByteBuf, p: &crate::authlib::game_profile::GameProfile| {
            buf.write_uuid(&p.id);
            buf.write_utf_n(&p.name, 16).expect("profile name too long");
            props.encode(buf, &p.properties);
        },
        {
            let props = game_profile_properties();
            move |buf: &mut FriendlyByteBuf| {
                let id = buf.read_uuid();
                let name = buf.read_utf_n(16).expect("invalid profile name");
                let properties = props.decode(buf);
                crate::authlib::game_profile::GameProfile::with_properties(id, name, properties)
            }
        },
    )
}
