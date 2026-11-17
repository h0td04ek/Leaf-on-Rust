//! Перенос `net.minecraft.network.HandlerNames` — имена обработчиков
//! pipeline (сохранены для совместимости логов/дебага).

/// `HandlerNames`.
pub struct HandlerNames;

impl HandlerNames {
    pub const DECOMPRESS: &str = "decompress";
    pub const COMPRESS: &str = "compress";
    pub const DECODER: &str = "decoder";
    pub const ENCODER: &str = "encoder";
    pub const INBOUND_CONFIG: &str = "inbound_config";
    pub const OUTBOUND_CONFIG: &str = "outbound_config";
    pub const SPLITTER: &str = "splitter";
    pub const PREPENDER: &str = "prepender";
    pub const DECRYPT: &str = "decrypt";
    pub const ENCRYPT: &str = "encrypt";
    pub const UNBUNDLER: &str = "unbundler";
    pub const BUNDLER: &str = "bundler";
    pub const PACKET_HANDLER: &str = "packet_handler";
    pub const TIMEOUT: &str = "timeout";
    pub const LEGACY_QUERY: &str = "legacy_query";
    pub const LATENCY: &str = "latency";
}
