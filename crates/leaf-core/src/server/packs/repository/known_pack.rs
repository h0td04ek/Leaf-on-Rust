//! Перенос `net.minecraft.server.packs.repository.KnownPack`.

use crate::core::shared_constants;

/// `KnownPack` — (namespace, id, version) — идентификатор ресурс-пака.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KnownPack {
    /// `namespace`.
    pub namespace: String,
    /// `id`.
    pub id: String,
    /// `version`.
    pub version: String,
}

/// `VANILLA_NAMESPACE`.
pub const VANILLA_NAMESPACE: &str = "minecraft";

impl KnownPack {
    /// `new KnownPack(namespace, id, version)`.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>, version: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), id: id.into(), version: version.into() }
    }

    /// `KnownPack.vanilla(name)` — ванильный пак с версией игры.
    pub fn vanilla(name: impl Into<String>) -> Self {
        Self::new(VANILLA_NAMESPACE, name, shared_constants::current_version().id)
    }

    /// `isVanilla`.
    pub fn is_vanilla(&self) -> bool {
        self.namespace == VANILLA_NAMESPACE
    }

    /// `toString` — `namespace:id:version`.
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.namespace, self.id, self.version)
    }
}

impl std::fmt::Display for KnownPack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.id, self.version)
    }
}

impl KnownPack {
    /// `STREAM_CODEC` — namespace + id + version (все STRING_UTF8).
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        KnownPack,
    > {
        use crate::network::codec::stream_codec::sync_fn;
        sync_fn(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, p: &KnownPack| {
                buf.write_utf(&p.namespace).expect("namespace too long");
                buf.write_utf(&p.id).expect("id too long");
                buf.write_utf(&p.version).expect("version too long");
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                KnownPack::new(
                    buf.read_utf().expect("invalid namespace"),
                    buf.read_utf().expect("invalid id"),
                    buf.read_utf().expect("invalid version"),
                )
            },
        )
    }
}
