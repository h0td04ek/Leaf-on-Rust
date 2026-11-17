//! `ServerboundDebugSubscriptionRequestPacket` — запрос подписки на debug-данные.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::util::debug::debug_subscription::DebugSubscription;

/// `ServerboundDebugSubscriptionRequestPacket`.
#[derive(Clone, Debug)]
pub struct ServerboundDebugSubscriptionRequestPacket {
    /// `subscriptions`.
    pub subscriptions: Vec<DebugSubscription>,
}

impl ServerboundDebugSubscriptionRequestPacket {
    /// `new`.
    pub fn new(subscriptions: Vec<DebugSubscription>) -> Self {
        Self { subscriptions }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.collection(ReferenceOpenHashSet::new, registry(DEBUG_SUBSCRIPTION), 64)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundDebugSubscriptionRequestPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ServerboundDebugSubscriptionRequestPacket| {
                buf.write_var_int(pkt.subscriptions.len() as i32);
                for sub in &pkt.subscriptions {
                    let codec = DebugSubscription::stream_codec();
                    codec.encode(buf, sub);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let len = buf.read_var_int();
                let mut subs = Vec::new();
                for _ in 0..len {
                    let codec = DebugSubscription::stream_codec();
                    subs.push(codec.decode(buf));
                }
                Self::new(subs)
            },
        )
    }
}

impl Packet for ServerboundDebugSubscriptionRequestPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_debug_subscription_request::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.subscriptions.len() as i32);
        for sub in &self.subscriptions {
            let codec = DebugSubscription::stream_codec();
            codec.encode(buf, sub);
        }
    }
}
