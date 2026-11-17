//! `EntityDataSerializers` — реестр сериализаторов entity data.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::syncher::entity_data_accessor::{EntityDataSerializer, ForValueTypeSerializer};
use crate::world::item::item_stack::ItemStack;

/// `EntityDataSerializers` — константы сериализаторов.
pub struct EntityDataSerializers;

impl EntityDataSerializers {
    /// `BYTE`.
    pub fn byte() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::byte(), 0)
    }

    /// `INT`.
    pub fn int() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::var_int(), 1)
    }

    /// `LONG`.
    pub fn long() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::var_long(), 2)
    }

    /// `FLOAT`.
    pub fn float() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::float(), 3)
    }

    /// `STRING`.
    pub fn string() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::string_utf8(32767), 4)
    }

    /// `COMPONENT`.
    pub fn component() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::chat::component_json::component_stream_codec(), 5)
    }

    /// `OPTIONAL_COMPONENT`.
    pub fn optional_component() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::chat::component_json::optional_component_stream_codec(), 6)
    }

    /// `ITEM_STACK` — `OVERSIZED_ITEM_CODEC` (Paper: no obfuscation).
    pub fn item_stack() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(ItemStack::optional_stream_codec(), 7)
    }

    /// `BOOLEAN`.
    pub fn boolean() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::bool_(), 8)
    }

    /// `ROTATIONS`.
    pub fn rotations() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::core::rotations::Rotations::stream_codec(), 9)
    }

    /// `BLOCK_POS`.
    pub fn block_pos() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::core::block_pos::BlockPos::stream_codec(), 10)
    }

    /// `OPTIONAL_BLOCK_POS`.
    pub fn optional_block_pos() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::byte_buf_codecs::optional(crate::core::block_pos::BlockPos::stream_codec()), 11)
    }

    /// `DIRECTION`.
    pub fn direction() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::core::direction::Direction::stream_codec(), 12)
    }

    /// `OPTIONAL_LIVING_ENTITY_REFERENCE` — `Optional<EntityReference>` как VarInt.
    pub fn optional_living_entity_reference() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::stream_codec::sync_fn(
                |buf: &mut FriendlyByteBuf, v: &Option<i32>| {
                    match v { Some(id) => { buf.write_var_int(*id + 1); }, None => { buf.write_var_int(0); } }
                },
                |buf: &mut FriendlyByteBuf| {
                    let i = buf.read_var_int();
                    if i == 0 { None } else { Some(i - 1) }
                },
            ), 13)
    }

    /// `BLOCK_STATE` — `ByteBufCodecs.idMapper(Block.BLOCK_STATE_REGISTRY)`.
    pub fn block_state() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::var_int(), 14)
    }

    /// `OPTIONAL_BLOCK_STATE` — `Optional<BlockState>` как VarInt (0 = empty).
    pub fn optional_block_state() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::stream_codec::sync_fn(
                |buf: &mut FriendlyByteBuf, v: &Option<i32>| {
                    buf.write_var_int(v.unwrap_or(0));
                },
                |buf: &mut FriendlyByteBuf| {
                    let i = buf.read_var_int();
                    if i == 0 { None } else { Some(i) }
                },
            ), 15)
    }

    /// `PARTICLE`.
    pub fn particle() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::core::particles::particle_options::ParticleOptions::stream_codec(), 16)
    }

    /// `PARTICLES` — `List<ParticleOptions>`.
    pub fn particles() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::byte_buf_codecs::list(
                crate::core::particles::particle_options::ParticleOptions::stream_codec()), 17)
    }

    /// `VILLAGER_DATA`.
    pub fn villager_data() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::var_int(), 18)
    }

    /// `OPTIONAL_UNSIGNED_INT` — `OptionalInt` как VarInt (0 = empty, else i-1).
    pub fn optional_unsigned_int() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::stream_codec::sync_fn(
                |buf: &mut FriendlyByteBuf, v: &Option<u32>| {
                    buf.write_var_int(v.map(|x| x as i32 + 1).unwrap_or(0));
                },
                |buf: &mut FriendlyByteBuf| {
                    let i = buf.read_var_int();
                    if i == 0 { None } else { Some((i - 1) as u32) }
                },
            ), 19)
    }

    /// `POSE`.
    pub fn pose() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::entity::pose::Pose::stream_codec(), 20)
    }

    // Variant serializers — placeholder until types are ported
    fn cat_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(21) }
    fn cow_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(22) }
    fn wolf_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(23) }
    fn wolf_sound_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(24) }
    fn frog_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(25) }
    fn pig_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(26) }
    fn chicken_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(27) }
    fn zombie_nautilus_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(28) }

    /// `OPTIONAL_GLOBAL_POS` — `Optional<GlobalPos>` как VarInt.
    pub fn optional_global_pos() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::stream_codec::sync_fn(
                |buf: &mut FriendlyByteBuf, v: &Option<i32>| {
                    buf.write_var_int(v.map(|x| x + 1).unwrap_or(0));
                },
                |buf: &mut FriendlyByteBuf| {
                    let i = buf.read_var_int();
                    if i == 0 { None } else { Some(i - 1) }
                },
            ), 29)
    }

    fn painting_variant() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(30) }
    fn sniffer_state() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(31) }
    fn armadillo_state() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(32) }
    fn copper_golem_state() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(33) }
    fn weathering_copper_state() -> Arc<dyn EntityDataSerializer> { Self::var_int_serializer(34) }

    /// `VECTOR3` — `Vector3fc`.
    pub fn vector3() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::byte_buf_codecs::vector3f(), 35)
    }

    /// `QUATERNION` — `Quaternionfc`.
    pub fn quaternion() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::network::codec::byte_buf_codecs::quaternionf(), 36)
    }

    /// `RESOLVABLE_PROFILE`.
    pub fn resolvable_profile() -> Arc<dyn EntityDataSerializer> {
        Self::var_int_serializer(37)
    }

    /// `HUMANOID_ARM`.
    pub fn humanoid_arm() -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(
            crate::entity::humanoid_arm::HumanoidArm::stream_codec(), 38)
    }

    fn var_int_serializer(id: i32) -> Arc<dyn EntityDataSerializer> {
        ForValueTypeSerializer::new(crate::network::codec::byte_buf_codecs::var_int(), id)
    }
}

/// `EntityDataSerializers` — реестр сериализаторов по id.
static SERIALIZERS: std::sync::OnceLock<Vec<Arc<dyn EntityDataSerializer>>> = std::sync::OnceLock::new();

fn serializers() -> &'static Vec<Arc<dyn EntityDataSerializer>> {
    SERIALIZERS.get_or_init(|| {
        vec![
            EntityDataSerializers::byte(),       // 0
            EntityDataSerializers::int(),        // 1
            EntityDataSerializers::long(),       // 2
            EntityDataSerializers::float(),      // 3
            EntityDataSerializers::string(),     // 4
            EntityDataSerializers::component(),  // 5
            EntityDataSerializers::optional_component(), // 6
            EntityDataSerializers::item_stack(), // 7
            EntityDataSerializers::boolean(),    // 8
            EntityDataSerializers::rotations(),  // 9
            EntityDataSerializers::block_pos(),  // 10
            EntityDataSerializers::optional_block_pos(), // 11
            EntityDataSerializers::direction(),  // 12
            EntityDataSerializers::optional_living_entity_reference(), // 13
            EntityDataSerializers::block_state(), // 14
            EntityDataSerializers::optional_block_state(), // 15
            EntityDataSerializers::particle(),   // 16
            EntityDataSerializers::particles(),  // 17
            EntityDataSerializers::villager_data(), // 18
            EntityDataSerializers::optional_unsigned_int(), // 19
            EntityDataSerializers::pose(),       // 20
            EntityDataSerializers::cat_variant(), // 21
            EntityDataSerializers::cow_variant(), // 22
            EntityDataSerializers::wolf_variant(), // 23
            EntityDataSerializers::wolf_sound_variant(), // 24
            EntityDataSerializers::frog_variant(), // 25
            EntityDataSerializers::pig_variant(), // 26
            EntityDataSerializers::chicken_variant(), // 27
            EntityDataSerializers::zombie_nautilus_variant(), // 28
            EntityDataSerializers::optional_global_pos(), // 29
            EntityDataSerializers::painting_variant(), // 30
            EntityDataSerializers::sniffer_state(), // 31
            EntityDataSerializers::armadillo_state(), // 32
            EntityDataSerializers::copper_golem_state(), // 33
            EntityDataSerializers::weathering_copper_state(), // 34
            EntityDataSerializers::vector3(),     // 35
            EntityDataSerializers::quaternion(),  // 36
            EntityDataSerializers::resolvable_profile(), // 37
            EntityDataSerializers::humanoid_arm(), // 38
        ]
    })
}

/// `getSerializer(id)`.
pub fn get_serializer(id: i32) -> Option<Arc<dyn EntityDataSerializer>> {
    serializers().get(id as usize).cloned()
}

/// `getSerializedId(serializer)`.
pub fn get_serialized_id(serializer: &Arc<dyn EntityDataSerializer>) -> i32 {
    serializer.serialized_id()
}
