//! Перенос `ClientboundResourcePackPushPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use std::sync::LazyLock;

/// `MAX_HASH_LENGTH`.
pub const MAX_HASH_LENGTH: usize = 40;

static RESOURCE_PACK_PUSH_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("resource_pack_push").unwrap())
});

/// `ClientboundResourcePackPushPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundResourcePackPushPacket {
    pub id: Uuid,
    pub url: String,
    /// `hash` — max 40 символов.
    pub hash: String,
    pub required: bool,
    pub prompt: Option<Component>,
}

impl ClientboundResourcePackPushPacket {
    /// `new ClientboundResourcePackPushPacket(...)` — с проверкой длины hash.
    pub fn new(
        id: Uuid,
        url: impl Into<String>,
        hash: impl Into<String>,
        required: bool,
        prompt: Option<Component>,
    ) -> Result<Self, String> {
        let hash = hash.into();
        if hash.len() > MAX_HASH_LENGTH {
            return Err(format!(
                "Hash is too long (max {}, was {})",
                MAX_HASH_LENGTH,
                hash.len()
            ));
        }
        Ok(Self { id, url: url.into(), hash, required, prompt })
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundResourcePackPushPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundResourcePackPushPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_uuid();
                let url = buf.read_utf().expect("invalid url");
                let hash = buf.read_utf_n(MAX_HASH_LENGTH).expect("invalid hash");
                let required = buf.read_bool();
                let prompt = buf.read_optional(|b| component_json::component_stream_codec().decode(b));
                ClientboundResourcePackPushPacket::new(id, url, hash, required, prompt)
                    .expect("invalid resource pack push")
            },
        )
    }
}

impl Packet for ClientboundResourcePackPushPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        RESOURCE_PACK_PUSH_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_uuid(&self.id);
        buf.write_utf(&self.url).expect("url too long");
        buf.write_utf_n(&self.hash, MAX_HASH_LENGTH).expect("hash too long");
        buf.write_bool(self.required);
        buf.write_optional(&self.prompt, |b, c| {
            component_json::component_stream_codec().encode(b, c);
        });
    }
}
