//! `ClientboundAwardStatsPacket` — отправка статистики игроку.

use std::collections::HashMap;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::stats::Stat;

/// `ClientboundAwardStatsPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundAwardStatsPacket {
    /// `stats` — `Object2IntMap<Stat<?>>`.
    pub stats: HashMap<Stat, i32>,
}

impl ClientboundAwardStatsPacket {
    /// `new`.
    pub fn new(stats: HashMap<Stat, i32>) -> Self {
        Self { stats }
    }

    /// `STAT_VALUES_STREAM_CODEC` — `ByteBufCodecs.map(Object2IntOpenHashMap::new, Stat.STREAM_CODEC, VAR_INT)`.
    fn stat_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Stat> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, stat: &Stat| {
                buf.write_var_int(stat.stat_type);
                buf.write_var_int(stat.value);
            },
            |buf: &mut FriendlyByteBuf| {
                let stat_type = buf.read_var_int();
                let value = buf.read_var_int();
                Stat::new(stat_type, value, String::new())
            },
        )
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundAwardStatsPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundAwardStatsPacket| {
                buf.write_var_int(pkt.stats.len() as i32);
                for (stat, value) in &pkt.stats {
                    let codec = Self::stat_stream_codec();
                    codec.encode(buf, stat);
                    buf.write_var_int(*value);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let len = buf.read_var_int();
                let mut stats = HashMap::new();
                for _ in 0..len {
                    let codec = Self::stat_stream_codec();
                    let stat = codec.decode(buf);
                    let value = buf.read_var_int();
                    stats.insert(stat, value);
                }
                Self::new(stats)
            },
        )
    }
}

impl Packet for ClientboundAwardStatsPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_award_stats::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.stats.len() as i32);
        for (stat, value) in &self.stats {
            let codec = Self::stat_stream_codec();
            codec.encode(buf, stat);
            buf.write_var_int(*value);
        }
    }
}
