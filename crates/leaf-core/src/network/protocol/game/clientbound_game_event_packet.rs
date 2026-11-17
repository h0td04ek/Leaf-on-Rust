//! Перенос `ClientboundGameEventPacket` (+ `Type`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundGameEventPacket.Type` — game event id → тип.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameEventType {
    /// `id`.
    pub id: u8,
}

/// `ClientboundGameEventPacket.Type` константы (id 0..=13).
pub const NO_RESPAWN_BLOCK_AVAILABLE: GameEventType = GameEventType { id: 0 };
pub const START_RAINING: GameEventType = GameEventType { id: 1 };
pub const STOP_RAINING: GameEventType = GameEventType { id: 2 };
pub const CHANGE_GAME_MODE: GameEventType = GameEventType { id: 3 };
pub const WIN_GAME: GameEventType = GameEventType { id: 4 };
pub const DEMO_EVENT: GameEventType = GameEventType { id: 5 };
pub const PLAY_ARROW_HIT_SOUND: GameEventType = GameEventType { id: 6 };
pub const RAIN_LEVEL_CHANGE: GameEventType = GameEventType { id: 7 };
pub const THUNDER_LEVEL_CHANGE: GameEventType = GameEventType { id: 8 };
pub const PUFFER_FISH_STING: GameEventType = GameEventType { id: 9 };
pub const GUARDIAN_ELDER_EFFECT: GameEventType = GameEventType { id: 10 };
pub const IMMEDIATE_RESPAWN: GameEventType = GameEventType { id: 11 };
pub const LIMITED_CRAFTING: GameEventType = GameEventType { id: 12 };
pub const LEVEL_CHUNKS_LOAD_START: GameEventType = GameEventType { id: 13 };

/// `ClientboundGameEventPacket.Type.TYPES` — все известные типы.
pub const GAME_EVENT_TYPES: [GameEventType; 14] = [
    NO_RESPAWN_BLOCK_AVAILABLE,
    START_RAINING,
    STOP_RAINING,
    CHANGE_GAME_MODE,
    WIN_GAME,
    DEMO_EVENT,
    PLAY_ARROW_HIT_SOUND,
    RAIN_LEVEL_CHANGE,
    THUNDER_LEVEL_CHANGE,
    PUFFER_FISH_STING,
    GUARDIAN_ELDER_EFFECT,
    IMMEDIATE_RESPAWN,
    LIMITED_CRAFTING,
    LEVEL_CHUNKS_LOAD_START,
];

/// `DEMO_PARAM_*` константы.
pub const DEMO_PARAM_INTRO: i32 = 0;
pub const DEMO_PARAM_HINT_1: i32 = 101;
pub const DEMO_PARAM_HINT_2: i32 = 102;
pub const DEMO_PARAM_HINT_3: i32 = 103;
pub const DEMO_PARAM_HINT_4: i32 = 104;

impl GameEventType {
    /// `Type.TYPES.get(id)` — неизвестный id → panic как в Java
    /// (`Int2ObjectMap.get` вернул бы null → NPE при использовании).
    pub fn by_id(id: u8) -> GameEventType {
        GAME_EVENT_TYPES
            .iter()
            .copied()
            .find(|t| t.id == id)
            .expect("unknown game event type")
    }
}

/// `ClientboundGameEventPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundGameEventPacket {
    /// `event`.
    pub event: GameEventType,
    /// `param`.
    pub param: f32,
}

impl ClientboundGameEventPacket {
    pub fn new(event: GameEventType, param: f32) -> Self {
        Self { event, param }
    }

    /// `getEvent`.
    pub fn get_event(&self) -> GameEventType {
        self.event
    }

    /// `getParam`.
    pub fn get_param(&self) -> f32 {
        self.param
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundGameEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundGameEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let event = GameEventType::by_id(buf.read_unsigned_byte() as u8);
                let param = buf.read_float();

                ClientboundGameEventPacket::new(event, param)
            },
        )
    }
}

impl Packet for ClientboundGameEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_game_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(self.event.id as i32);
        buf.write_float(self.param);
    }
}
