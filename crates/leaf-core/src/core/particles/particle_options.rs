//! Перенос `net.minecraft.core.particles.ParticleOptions` и всех
//! `*ParticleOption(s)` классов — type-erased `ParticleOptionsData`.

use super::particle_type::{ParticleOptionsKind, ParticleType, SharedParticleType};
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::block::state::BlockState;
use crate::world::level::gameevent::position_source::PositionSource;
use crate::world::phys::vec3::Vec3;

/// `ParticleOptions` — опции частицы: тип + payload.
///
/// `type_` — `Arc<ParticleType>` из реестра `particle_types()`.
#[derive(Clone)]
pub struct ParticleOptions {
    /// Тип частицы (`getType`).
    pub type_: SharedParticleType,
    /// Payload опций.
    pub data: ParticleOptionsData,
}

impl ParticleOptions {
    /// `getType`.
    pub fn get_type(&self) -> &ParticleType {
        &self.type_
    }

    /// Создать опции для типа без payload (`SimpleParticleType`).
    pub fn simple(type_: SharedParticleType) -> Self {
        Self { type_, data: ParticleOptionsData::Simple }
    }

    /// `ParticleTypes.STREAM_CODEC` —
    /// `registry(PARTICLE_TYPE).dispatch(getType, type.streamCodec)`:
    /// VarInt registry-id типа + payload опций.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ParticleOptions> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ParticleOptions| {
                buf.write_var_int(v.type_id());
                Self::encode_data(buf, v);
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let type_ = super::particle_types::particle_type_by_id(id)
                    .unwrap_or_else(|| panic!("Unknown particle type id {}", id));
                let data = Self::decode_data(buf, type_.options_kind);
                ParticleOptions { type_, data }
            },
        )
    }

    /// Id типа в реестре (порядок регистрации `ParticleTypes`).
    pub fn type_id(&self) -> i32 {
        super::particle_types::particle_type_index(&self.type_.id)
            .expect("particle type not registered") as i32
    }

    fn encode_data(buf: &mut FriendlyByteBuf, v: &ParticleOptions) {
        match (&v.type_.options_kind, &v.data) {
            (Some(ParticleOptionsKind::Block), ParticleOptionsData::Block(state)) => {
                let id = crate::world::level::block::state::block_state_registry::block_state_id_map()
                    .get_id(state);
                buf.write_var_int(id);
            }
            (Some(ParticleOptionsKind::Color), ParticleOptionsData::Color(color)) => {
                buf.write_int(*color);
            }
            (Some(ParticleOptionsKind::Spell), ParticleOptionsData::Spell { color, power }) => {
                buf.write_int(*color);
                buf.write_float(*power);
            }
            (Some(ParticleOptionsKind::Dust), ParticleOptionsData::Dust { color, scale }) => {
                buf.write_int(*color);
                buf.write_float(*scale);
            }
            (
                Some(ParticleOptionsKind::DustColorTransition),
                ParticleOptionsData::DustColorTransition { from, to, scale },
            ) => {
                buf.write_int(*from);
                buf.write_int(*to);
                buf.write_float(*scale);
            }
            (Some(ParticleOptionsKind::SculkCharge), ParticleOptionsData::SculkCharge { roll }) => {
                buf.write_float(*roll);
            }
            (Some(ParticleOptionsKind::Item), ParticleOptionsData::Item(stack)) => {
                ItemStack::stream_codec().encode(buf, stack);
            }
            (
                Some(ParticleOptionsKind::Vibration),
                ParticleOptionsData::Vibration { destination, arrival_in_ticks },
            ) => {
                PositionSource::stream_codec().encode(buf, destination);
                buf.write_var_int(*arrival_in_ticks);
            }
            (
                Some(ParticleOptionsKind::Trail),
                ParticleOptionsData::Trail { target, color, duration },
            ) => {
                Vec3::stream_codec().encode(buf, target);
                buf.write_int(*color);
                buf.write_var_int(*duration);
            }
            (Some(ParticleOptionsKind::Power), ParticleOptionsData::Power { power }) => {
                buf.write_float(*power);
            }
            (Some(ParticleOptionsKind::Shriek), ParticleOptionsData::Shriek { delay }) => {
                buf.write_var_int(*delay);
            }
            (None, ParticleOptionsData::Simple) => {}
            _ => panic!("ParticleOptions data does not match type"),
        }
    }

    fn decode_data(buf: &mut FriendlyByteBuf, kind: Option<ParticleOptionsKind>) -> ParticleOptionsData {
        match kind {
            None | Some(ParticleOptionsKind::Simple) => ParticleOptionsData::Simple,
            Some(ParticleOptionsKind::Block) => {
                let id = buf.read_var_int();
                let state = crate::world::level::block::state::block_state_registry::block_state_by_id(id)
                    .unwrap_or_else(|| panic!("Unknown block state id {}", id));
                ParticleOptionsData::Block(state)
            }
            Some(ParticleOptionsKind::Color) => ParticleOptionsData::Color(buf.read_int()),
            Some(ParticleOptionsKind::Spell) => ParticleOptionsData::Spell {
                color: buf.read_int(),
                power: buf.read_float(),
            },
            Some(ParticleOptionsKind::Dust) => ParticleOptionsData::Dust {
                color: buf.read_int(),
                scale: buf.read_float(),
            },
            Some(ParticleOptionsKind::DustColorTransition) => ParticleOptionsData::DustColorTransition {
                from: buf.read_int(),
                to: buf.read_int(),
                scale: buf.read_float(),
            },
            Some(ParticleOptionsKind::SculkCharge) => {
                ParticleOptionsData::SculkCharge { roll: buf.read_float() }
            }
            Some(ParticleOptionsKind::Item) => {
                ParticleOptionsData::Item(ItemStack::stream_codec().decode(buf))
            }
            Some(ParticleOptionsKind::Vibration) => ParticleOptionsData::Vibration {
                destination: PositionSource::stream_codec().decode(buf),
                arrival_in_ticks: buf.read_var_int(),
            },
            Some(ParticleOptionsKind::Trail) => ParticleOptionsData::Trail {
                target: Vec3::stream_codec().decode(buf),
                color: buf.read_int(),
                duration: buf.read_var_int(),
            },
            Some(ParticleOptionsKind::Power) => {
                ParticleOptionsData::Power { power: buf.read_float() }
            }
            Some(ParticleOptionsKind::Shriek) => {
                ParticleOptionsData::Shriek { delay: buf.read_var_int() }
            }
        }
    }
}

/// `ParticleOptions` payload — аналог `ParticleOptions` дженерик-иерархии.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticleOptionsData {
    /// `SimpleParticleType` — пустой payload.
    Simple,
    /// `BlockParticleOption`.
    Block(BlockState),
    /// `ColorParticleOption`.
    Color(i32),
    /// `SpellParticleOption`.
    Spell { color: i32, power: f32 },
    /// `DustParticleOptions`.
    Dust { color: i32, scale: f32 },
    /// `DustColorTransitionOptions`.
    DustColorTransition { from: i32, to: i32, scale: f32 },
    /// `SculkChargeParticleOptions`.
    SculkCharge { roll: f32 },
    /// `ItemParticleOption`.
    Item(ItemStack),
    /// `VibrationParticleOption`.
    Vibration { destination: PositionSource, arrival_in_ticks: i32 },
    /// `TrailParticleOption`.
    Trail { target: Vec3, color: i32, duration: i32 },
    /// `PowerParticleOption`.
    Power { power: f32 },
    /// `ShriekParticleOption`.
    Shriek { delay: i32 },
}

impl PartialEq for ParticleOptions {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.type_, &other.type_) && self.data == other.data
    }
}
impl Eq for ParticleOptions {}
impl std::fmt::Debug for ParticleOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParticleOptions").field("data", &self.data).finish()
    }
}
