//! Перенос `net.minecraft.network` — транспорт, протоколы, пакеты.

pub mod bandwidth_debug_monitor;
pub mod chat;
pub mod codec;
pub mod compression;
pub mod connection;
pub mod connection_protocol;
pub mod disconnection_details;
pub mod encryption;
pub mod friendly_byte_buf;
pub mod lp_vec3;
pub mod packet_processor;
pub mod packet_send_listener;
pub mod protocol;
pub mod syncher;
pub mod skip_packet;
pub mod utf8_string;
pub mod var_int;
pub mod var_long;
