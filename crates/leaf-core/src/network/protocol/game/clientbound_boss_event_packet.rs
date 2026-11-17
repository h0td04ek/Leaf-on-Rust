//! Перенос `ClientboundBossEventPacket` — босс-бар операции.

use crate::network::chat::component::Component;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::uuid::Uuid;
use crate::world::boss_event::{BossBarColor, BossBarOverlay};

const FLAG_DARKEN: u8 = 1;
const FLAG_MUSIC: u8 = 2;
const FLAG_FOG: u8 = 4;

/// `ClientboundBossEventPacket.OperationType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossEventOperationType {
    /// `ADD`.
    Add,
    /// `REMOVE`.
    Remove,
    /// `UPDATE_PROGRESS`.
    UpdateProgress,
    /// `UPDATE_NAME`.
    UpdateName,
    /// `UPDATE_STYLE`.
    UpdateStyle,
    /// `UPDATE_PROPERTIES`.
    UpdateProperties,
}

impl BossEventOperationType {
    /// Все значения в порядке объявления.
    pub const VALUES: [BossEventOperationType; 6] = [
        BossEventOperationType::Add,
        BossEventOperationType::Remove,
        BossEventOperationType::UpdateProgress,
        BossEventOperationType::UpdateName,
        BossEventOperationType::UpdateStyle,
        BossEventOperationType::UpdateProperties,
    ];

    /// `readEnum`.
    pub fn by_ordinal(id: i32) -> BossEventOperationType {
        Self::VALUES[id.rem_euclid(6) as usize]
    }

    /// `writeEnum` — VarInt ordinal.
    pub fn ordinal(&self) -> i32 {
        Self::VALUES.iter().position(|v| v == self).unwrap() as i32
    }
}

/// `ClientboundBossEventPacket.Operation` — вариант операции.
#[derive(Clone, Debug, PartialEq)]
pub enum BossEventOperation {
    /// `AddOperation`.
    Add {
        /// `name`.
        name: Component,
        /// `progress`.
        progress: f32,
        /// `color`.
        color: BossBarColor,
        /// `overlay`.
        overlay: BossBarOverlay,
        /// `darkenScreen`.
        darken_screen: bool,
        /// `playMusic`.
        play_music: bool,
        /// `createWorldFog`.
        create_world_fog: bool,
    },
    /// `REMOVE_OPERATION`.
    Remove,
    /// `UpdateProgressOperation`.
    UpdateProgress {
        /// `progress`.
        progress: f32,
    },
    /// `UpdateNameOperation`.
    UpdateName {
        /// `name`.
        name: Component,
    },
    /// `UpdateStyleOperation`.
    UpdateStyle {
        /// `color`.
        color: BossBarColor,
        /// `overlay`.
        overlay: BossBarOverlay,
    },
    /// `UpdatePropertiesOperation`.
    UpdateProperties {
        /// `darkenScreen`.
        darken_screen: bool,
        /// `playMusic`.
        play_music: bool,
        /// `createWorldFog`.
        create_world_fog: bool,
    },
}

fn encode_properties(darken: bool, music: bool, fog: bool) -> u8 {
    (if darken { FLAG_DARKEN } else { 0 })
        | (if music { FLAG_MUSIC } else { 0 })
        | (if fog { FLAG_FOG } else { 0 })
}

fn decode_properties(flags: u8) -> (bool, bool, bool) {
    (flags & FLAG_DARKEN != 0, flags & FLAG_MUSIC != 0, flags & FLAG_FOG != 0)
}

fn color_by_ordinal(id: i32) -> BossBarColor {
    BossBarColor::values()[id.rem_euclid(7) as usize]
}

fn color_ordinal(c: &BossBarColor) -> i32 {
    BossBarColor::values().iter().position(|v| v == c).unwrap() as i32
}

fn overlay_by_ordinal(id: i32) -> BossBarOverlay {
    BossBarOverlay::values()[id.rem_euclid(5) as usize]
}

fn overlay_ordinal(o: &BossBarOverlay) -> i32 {
    BossBarOverlay::values().iter().position(|v| v == o).unwrap() as i32
}

fn write_operation(buf: &mut FriendlyByteBuf, op: &BossEventOperation) {
    match op {
        BossEventOperation::Add {
            name,
            progress,
            color,
            overlay,
            darken_screen,
            play_music,
            create_world_fog,
        } => {
            component_stream_codec().encode(buf, name);
            buf.write_float(*progress);
            buf.write_enum(color, color_ordinal(color));
            buf.write_enum(overlay, overlay_ordinal(overlay));
            buf.write_byte(encode_properties(*darken_screen, *play_music, *create_world_fog) as i32);
        }
        BossEventOperation::Remove => {}
        BossEventOperation::UpdateProgress { progress } => {
            buf.write_float(*progress);
        }
        BossEventOperation::UpdateName { name } => {
            component_stream_codec().encode(buf, name);
        }
        BossEventOperation::UpdateStyle { color, overlay } => {
            buf.write_enum(color, color_ordinal(color));
            buf.write_enum(overlay, overlay_ordinal(overlay));
        }
        BossEventOperation::UpdateProperties {
            darken_screen,
            play_music,
            create_world_fog,
        } => {
            buf.write_byte(encode_properties(*darken_screen, *play_music, *create_world_fog) as i32);
        }
    }
}

fn read_operation(buf: &mut FriendlyByteBuf, op_type: BossEventOperationType) -> BossEventOperation {
    match op_type {
        BossEventOperationType::Add => {
            let name = component_stream_codec().decode(buf);
            let progress = buf.read_float();
            let color = color_by_ordinal(buf.read_var_int());
            let overlay = overlay_by_ordinal(buf.read_var_int());
            let flags = buf.read_byte() as u8;
            let (d, m, f) = decode_properties(flags);
            BossEventOperation::Add {
                name,
                progress,
                color,
                overlay,
                darken_screen: d,
                play_music: m,
                create_world_fog: f,
            }
        }
        BossEventOperationType::Remove => BossEventOperation::Remove,
        BossEventOperationType::UpdateProgress => BossEventOperation::UpdateProgress {
            progress: buf.read_float(),
        },
        BossEventOperationType::UpdateName => BossEventOperation::UpdateName {
            name: component_stream_codec().decode(buf),
        },
        BossEventOperationType::UpdateStyle => BossEventOperation::UpdateStyle {
            color: color_by_ordinal(buf.read_var_int()),
            overlay: overlay_by_ordinal(buf.read_var_int()),
        },
        BossEventOperationType::UpdateProperties => {
            let flags = buf.read_byte() as u8;
            let (d, m, f) = decode_properties(flags);
            BossEventOperation::UpdateProperties {
                darken_screen: d,
                play_music: m,
                create_world_fog: f,
            }
        }
    }
}

fn operation_type(op: &BossEventOperation) -> BossEventOperationType {
    match op {
        BossEventOperation::Add { .. } => BossEventOperationType::Add,
        BossEventOperation::Remove => BossEventOperationType::Remove,
        BossEventOperation::UpdateProgress { .. } => BossEventOperationType::UpdateProgress,
        BossEventOperation::UpdateName { .. } => BossEventOperationType::UpdateName,
        BossEventOperation::UpdateStyle { .. } => BossEventOperationType::UpdateStyle,
        BossEventOperation::UpdateProperties { .. } => BossEventOperationType::UpdateProperties,
    }
}

/// `ClientboundBossEventPacket.Handler` — колбэки обработки.
/// В порте — trait для будущего `ClientPacketListener`.
pub trait BossEventPacketHandler {
    /// `add`.
    fn boss_add(
        &mut self,
        _id: Uuid,
        _name: &Component,
        _progress: f32,
        _color: BossBarColor,
        _overlay: BossBarOverlay,
        _darken_screen: bool,
        _play_music: bool,
        _create_world_fog: bool,
    ) {
    }
    /// `remove`.
    fn boss_remove(&mut self, _id: Uuid) {}
    /// `updateProgress`.
    fn boss_update_progress(&mut self, _id: Uuid, _progress: f32) {}
    /// `updateName`.
    fn boss_update_name(&mut self, _id: Uuid, _name: &Component) {}
    /// `updateStyle`.
    fn boss_update_style(&mut self, _id: Uuid, _color: BossBarColor, _overlay: BossBarOverlay) {}
    /// `updateProperties`.
    fn boss_update_properties(
        &mut self,
        _id: Uuid,
        _darken_screen: bool,
        _play_music: bool,
        _create_world_fog: bool,
    ) {
    }
}

/// `ClientboundBossEventPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundBossEventPacket {
    /// `id`.
    pub id: Uuid,
    /// `operation`.
    pub operation: BossEventOperation,
}

impl ClientboundBossEventPacket {
    /// `ClientboundBossEventPacket::new`.
    pub fn new(id: Uuid, operation: BossEventOperation) -> Self {
        Self { id, operation }
    }

    /// `createAddPacket`.
    pub fn create_add_packet(event: &crate::world::boss_event::BossEvent) -> Self {
        Self::new(
            event.id,
            BossEventOperation::Add {
                name: event.name.copy(),
                progress: event.progress,
                color: event.color,
                overlay: event.overlay,
                darken_screen: event.darken_screen,
                play_music: event.play_music,
                create_world_fog: event.create_world_fog,
            },
        )
    }

    /// `createRemovePacket`.
    pub fn create_remove_packet(id: Uuid) -> Self {
        Self::new(id, BossEventOperation::Remove)
    }

    /// `createUpdateProgressPacket`.
    pub fn create_update_progress_packet(id: Uuid, progress: f32) -> Self {
        Self::new(id, BossEventOperation::UpdateProgress { progress })
    }

    /// `createUpdateNamePacket`.
    pub fn create_update_name_packet(id: Uuid, name: Component) -> Self {
        Self::new(id, BossEventOperation::UpdateName { name })
    }

    /// `createUpdateStylePacket`.
    pub fn create_update_style_packet(id: Uuid, color: BossBarColor, overlay: BossBarOverlay) -> Self {
        Self::new(id, BossEventOperation::UpdateStyle { color, overlay })
    }

    /// `createUpdatePropertiesPacket`.
    pub fn create_update_properties_packet(
        id: Uuid,
        darken_screen: bool,
        play_music: bool,
        create_world_fog: bool,
    ) -> Self {
        Self::new(
            id,
            BossEventOperation::UpdateProperties {
                darken_screen,
                play_music,
                create_world_fog,
            },
        )
    }

    /// `id`.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// `operation`.
    pub fn operation(&self) -> &BossEventOperation {
        &self.operation
    }

    /// `dispatch` — вызвать соответствующий метод handler'а.
    pub fn dispatch(&self, handler: &mut dyn BossEventPacketHandler) {
        match &self.operation {
            BossEventOperation::Add {
                name,
                progress,
                color,
                overlay,
                darken_screen,
                play_music,
                create_world_fog,
            } => handler.boss_add(
                self.id,
                name,
                *progress,
                *color,
                *overlay,
                *darken_screen,
                *play_music,
                *create_world_fog,
            ),
            BossEventOperation::Remove => handler.boss_remove(self.id),
            BossEventOperation::UpdateProgress { progress } => {
                handler.boss_update_progress(self.id, *progress)
            }
            BossEventOperation::UpdateName { name } => handler.boss_update_name(self.id, name),
            BossEventOperation::UpdateStyle { color, overlay } => {
                handler.boss_update_style(self.id, *color, *overlay)
            }
            BossEventOperation::UpdateProperties {
                darken_screen,
                play_music,
                create_world_fog,
            } => handler.boss_update_properties(self.id, *darken_screen, *play_music, *create_world_fog),
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBossEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBossEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_uuid();
                let op_type = BossEventOperationType::by_ordinal(buf.read_var_int());
                let operation = read_operation(buf, op_type);

                ClientboundBossEventPacket::new(id, operation)
            },
        )
    }
}

impl Packet for ClientboundBossEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_boss_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_uuid(&self.id);
        buf.write_var_int(operation_type(&self.operation).ordinal());
        write_operation(buf, &self.operation);
    }
}
