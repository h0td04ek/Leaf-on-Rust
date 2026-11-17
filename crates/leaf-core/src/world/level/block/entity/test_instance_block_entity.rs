//! Перенос `net.minecraft.world.level.block.entity.TestInstanceBlockEntity`
//! — `Data`/`Status`/`ErrorMarker` для wire-формата.

use crate::core::vec3i::Vec3i;
use crate::network::chat::component_json::optional_component_stream_codec;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::resource_key::ResourceKey;
use crate::world::gametest::game_test_instance::GameTestInstance;
use crate::world::level::block::rotation::Rotation;

/// `TestInstanceBlockEntity.Data`.
#[derive(Clone, Debug, PartialEq)]
pub struct TestInstanceBlockData {
    /// `test` — `ResourceKey<GameTestInstance>` (optional).
    pub test: Option<ResourceKey<GameTestInstance>>,
    /// `size`.
    pub size: Vec3i,
    /// `rotation`.
    pub rotation: Rotation,
    /// `ignoreEntities`.
    pub ignore_entities: bool,
    /// `status`.
    pub status: TestInstanceBlockStatus,
    /// `errorMessage`.
    pub error_message: Option<crate::network::chat::component::Component>,
}

impl TestInstanceBlockData {
    /// `withSize`.
    pub fn with_size(&self, size: Vec3i) -> Self {
        Self { size, ..self.clone() }
    }

    /// `withStatus` — errorMessage сбрасывается.
    pub fn with_status(&self, status: TestInstanceBlockStatus) -> Self {
        Self { status, error_message: None, ..self.clone() }
    }

    /// `withError`.
    pub fn with_error(&self, error: crate::network::chat::component::Component) -> Self {
        Self {
            status: TestInstanceBlockStatus::FINISHED,
            error_message: Some(error),
            ..self.clone()
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TestInstanceBlockData> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, d: &TestInstanceBlockData| {
                buf.write_optional(&d.test, |b, k| {
                    b.write_resource_key(k);
                });
                write_vec3i(buf, &d.size);
                Rotation::stream_codec().encode(buf, &d.rotation);
                buf.write_bool(d.ignore_entities);
                TestInstanceBlockStatus::stream_codec().encode(buf, &d.status);
                optional_component_stream_codec().encode(buf, &d.error_message);
            },
            |buf: &mut FriendlyByteBuf| {
                let test = buf.read_optional(|b| {
                    let registry = crate::resources::Identifier::from_namespace_and_path(
                        "minecraft", "test_instance",
                    )
                    .expect("valid id");
                    b.read_resource_key_for::<GameTestInstance>(registry)
                });
                let size = read_vec3i(buf);
                let rotation = Rotation::stream_codec().decode(buf);
                let ignore_entities = buf.read_bool();
                let status = TestInstanceBlockStatus::stream_codec().decode(buf);
                let error_message = optional_component_stream_codec().decode(buf);
                TestInstanceBlockData {
                    test,
                    size,
                    rotation,
                    ignore_entities,
                    status,
                    error_message,
                }
            },
        )
    }
}

/// `Vec3i.STREAM_CODEC` — 3×VAR_INT.
pub fn write_vec3i(buf: &mut FriendlyByteBuf, v: &Vec3i) {
    buf.write_var_int(v.x);
    buf.write_var_int(v.y);
    buf.write_var_int(v.z);
}

/// `Vec3i.STREAM_CODEC` decode.
pub fn read_vec3i(buf: &mut FriendlyByteBuf) -> Vec3i {
    Vec3i::new(buf.read_var_int(), buf.read_var_int(), buf.read_var_int())
}

/// `TestInstanceBlockEntity.Status`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestInstanceBlockStatus {
    /// `CLEARED`.
    CLEARED,
    /// `RUNNING`.
    RUNNING,
    /// `FINISHED`.
    FINISHED,
}

impl TestInstanceBlockStatus {
    /// `values()`.
    pub const VALUES: [TestInstanceBlockStatus; 3] = [
        TestInstanceBlockStatus::CLEARED,
        TestInstanceBlockStatus::RUNNING,
        TestInstanceBlockStatus::FINISHED,
    ];

    /// `index`.
    pub fn index(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., ZERO)` — clamp.
    pub fn by_index(index: i32) -> TestInstanceBlockStatus {
        Self::VALUES[index.clamp(0, 2) as usize]
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            TestInstanceBlockStatus::CLEARED => "cleared",
            TestInstanceBlockStatus::RUNNING => "running",
            TestInstanceBlockStatus::FINISHED => "finished",
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper` (VarInt index).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TestInstanceBlockStatus> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &TestInstanceBlockStatus| {
                buf.write_var_int(v.index());
            },
            |buf: &mut FriendlyByteBuf| TestInstanceBlockStatus::by_index(buf.read_var_int()),
        )
    }
}

/// `TestInstanceBlockEntity.ErrorMarker`.
#[derive(Clone, Debug, PartialEq)]
pub struct TestInstanceErrorMarker {
    /// `pos`.
    pub pos: crate::core::block_pos::BlockPos,
    /// `text`.
    pub text: crate::network::chat::component::Component,
}

/// Маркер `TestInstanceBlockEntity`.
#[derive(Debug)]
pub struct TestInstanceBlockEntity;
