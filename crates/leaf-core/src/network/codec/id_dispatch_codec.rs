//! Перенос `net.minecraft.network.codec.IdDispatchCodec`.
//!
//! `B` зафиксирован как `FriendlyByteBuf` — единственный буфер порта
//! (Java-обобщение `ByteBuf → B` через `mapStream` не требуется).
//!
//! Ошибки — panic (аналог unchecked DecoderException/EncoderException):
//! `panic_any(DecoderException/EncoderException)`, `SkipPacket*` —
//! для пропуска остатка буфера.

use crate::network::codec::exceptions::{DecoderException, EncoderException};
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::skip_packet::payload_is_skip_packet;
use crate::network::var_int;
use std::any::Any;
use std::collections::HashMap;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

/// `IdDispatchCodec.DontDecorateException` — маркер «не оборачивать ошибку».
pub trait DontDecorateException {}

/// Сериализатор пакета (`StreamCodec<ByteBuf, V>`).
pub type PacketSerializer<V> = Arc<dyn StreamCodec<FriendlyByteBuf, V>>;

struct Entry<V, T> {
    serializer: PacketSerializer<V>,
    type_: T,
}

/// `IdDispatchCodec<ByteBuf, V, T>`.
pub struct IdDispatchCodec<V, T> {
    type_getter: Arc<dyn Fn(&V) -> T>,
    by_id: Vec<Entry<V, T>>,
    to_id: HashMap<T, usize>,
}

impl<V, T> IdDispatchCodec<V, T>
where
    V: 'static,
    T: Clone + Eq + std::hash::Hash + std::fmt::Display + 'static,
{
    /// `IdDispatchCodec.builder(typeGetter)`.
    pub fn builder(
        type_getter: impl Fn(&V) -> T + 'static,
    ) -> IdDispatchCodecBuilder<V, T> {
        IdDispatchCodecBuilder {
            type_getter: Arc::new(type_getter),
            entries: Vec::new(),
        }
    }

    fn decode_impl(&self, buffer: &mut FriendlyByteBuf) -> Result<V, Box<dyn Any + Send>> {
        let id = var_int::read(buffer);
        if id >= 0 && (id as usize) < self.by_id.len() {
            let entry = &self.by_id[id as usize];
            match catch_unwind(AssertUnwindSafe(|| entry.serializer.decode(buffer))) {
                Ok(v) => Ok(v),
                Err(payload) => {
                    // DontDecorateException — пробрасываем как есть.
                    if payload_is_skip_packet(payload.as_ref()) {
                        Err(payload)
                    } else {
                        Err(Box::new(DecoderException::new(format!(
                            "Failed to decode packet '{}'",
                            entry.type_.to_string()
                        ))) as Box<dyn Any + Send>)
                    }
                }
            }
        } else {
            Err(Box::new(DecoderException::new(format!(
                "Received unknown packet id {}",
                id
            ))) as Box<dyn Any + Send>)
        }
    }

    fn encode_impl(&self, buffer: &mut FriendlyByteBuf, value: &V) -> Result<(), Box<dyn Any + Send>> {
        let type_ = (self.type_getter)(value);
        let Some(&id) = self.to_id.get(&type_) else {
            return Err(Box::new(EncoderException::new(format!(
                "Sending unknown packet '{}'",
                type_.to_string()
            ))) as Box<dyn Any + Send>);
        };
        var_int::write(buffer, id as i32);
        let entry = &self.by_id[id];
        match catch_unwind(AssertUnwindSafe(|| entry.serializer.encode(buffer, value))) {
            Ok(()) => Ok(()),
            Err(payload) => {
                if payload_is_skip_packet(payload.as_ref()) {
                    Err(payload)
                } else {
                    Err(Box::new(EncoderException::new(format!(
                        "Failed to encode packet '{}'",
                        type_.to_string()
                    ))) as Box<dyn Any + Send>)
                }
            }
        }
    }
}

impl<V, T> StreamCodec<FriendlyByteBuf, V> for IdDispatchCodec<V, T>
where
    V: 'static,
    T: Clone + Eq + std::hash::Hash + std::fmt::Display + 'static,
{
    fn decode(&self, buffer: &mut FriendlyByteBuf) -> V {
        match self.decode_impl(buffer) {
            Ok(v) => v,
            Err(payload) => resume_unwind(payload),
        }
    }

    fn encode(&self, buffer: &mut FriendlyByteBuf, value: &V) {
        match self.encode_impl(buffer, value) {
            Ok(()) => {}
            Err(payload) => resume_unwind(payload),
        }
    }
}

/// `IdDispatchCodec.Builder`.
pub struct IdDispatchCodecBuilder<V, T> {
    type_getter: Arc<dyn Fn(&V) -> T>,
    entries: Vec<(T, PacketSerializer<V>)>,
}

impl<V, T> IdDispatchCodecBuilder<V, T>
where
    V: 'static,
    T: Clone + Eq + std::hash::Hash + std::fmt::Display + 'static,
{
    /// `add(type, serializer)`.
    pub fn add(
        &mut self,
        type_: T,
        serializer: impl StreamCodec<FriendlyByteBuf, V> + 'static,
    ) -> &mut Self {
        self.entries.push((type_, Arc::new(serializer)));
        self
    }

    /// `build()` — дубликаты типов → panic (IllegalStateException).
    pub fn build(self) -> IdDispatchCodec<V, T> {
        let mut to_id: HashMap<T, usize> = HashMap::new();
        let mut by_id = Vec::with_capacity(self.entries.len());
        for (i, (type_, serializer)) in self.entries.into_iter().enumerate() {
            if to_id.insert(type_.clone(), i).is_some() {
                panic!("Duplicate registration for type {}", type_.to_string());
            }
            by_id.push(Entry { serializer, type_ });
        }
        IdDispatchCodec { type_getter: self.type_getter, by_id, to_id }
    }
}
