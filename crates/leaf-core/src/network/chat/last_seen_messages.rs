//! Перенос `net.minecraft.network.chat.LastSeenMessages` (+ `Packed`, `Update`).

use std::collections::HashSet;

use crate::network::chat::message_signature::{MessageSignature, PackedSignature};
use crate::network::chat::message_signature_cache::MessageSignatureCache;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `LAST_SEEN_MESSAGES_MAX_LENGTH`.
pub const LAST_SEEN_MESSAGES_MAX_LENGTH: usize = 20;
/// `Update.IGNORE_CHECKSUM`.
pub const IGNORE_CHECKSUM: u8 = 0;

/// `LastSeenMessages` — список подписей последних сообщений.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LastSeenMessages {
    /// `entries`.
    pub entries: Vec<MessageSignature>,
}

impl LastSeenMessages {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }

    /// `updateSignature` — обновляет `SignatureUpdater.Output`.
    /// Порядок: Ints.toByteArray(size) затем bytes каждой подписи.
    pub fn update_signature<F>(&self, mut output: F) -> Result<(), crate::util::signature_validator::SignatureException>
    where
        F: FnMut(&[u8]) -> Result<(), crate::util::signature_validator::SignatureException>,
    {
        output(&(self.entries.len() as i32).to_be_bytes())?;
        for sig in &self.entries {
            output(&sig.bytes)?;
        }
        Ok(())
    }

    /// `pack` — упаковать в `Packed` через кеш.
    pub fn pack(&self, cache: &mut MessageSignatureCache) -> LastSeenMessagesPacked {
        LastSeenMessagesPacked {
            entries: self
                .entries
                .iter()
                .map(|sig| {
                    let id = cache.pack(sig);
                    if id != -1 {
                        PackedSignature::new_id(id)
                    } else {
                        PackedSignature::new_full(sig.clone())
                    }
                })
                .collect(),
        }
    }

    /// `computeChecksum` — Java `Arrays.hashCode`-подобный rolling.
    pub fn compute_checksum(&self) -> u8 {
        let mut i: i32 = 1;
        for sig in &self.entries {
            // Arrays.hashCode(byte[]) — 31*h + element
            let mut h: i32 = 1;
            for b in sig.bytes.iter() {
                h = 31 * h + (*b as i8 as i32);
            }
            i = 31 * i + h;
        }
        let b = i as u8;
        if b == 0 { 1 } else { b }
    }
}

/// `LastSeenMessages.Packed`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LastSeenMessagesPacked {
    /// `entries`.
    pub entries: Vec<PackedSignature>,
}

impl LastSeenMessagesPacked {
    /// `new Packed(FriendlyByteBuf)` — collection max 20.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let entries = buffer.read_collection(
            |_| Vec::new(),
            |buf| PackedSignature::read(buf),
        );
        let entries = entries.into_iter().take(LAST_SEEN_MESSAGES_MAX_LENGTH).collect();
        Self { entries }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_collection(&self.entries, PackedSignature::write);
    }

    /// `unpack` — развернуть через кеш; `None` если подпись не найдена.
    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<LastSeenMessages> {
        let mut list = Vec::with_capacity(self.entries.len());
        for packed in &self.entries {
            list.push(cache.unpack_packed(packed)?);
        }
        Some(LastSeenMessages { entries: list })
    }
}

/// `LastSeenMessages.Update` — offset + acknowledged bitset(20) + checksum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    /// `offset`.
    pub offset: i32,
    /// `acknowledged` — bitset из 20 бит (слова u64, little-endian биты).
    pub acknowledged: Vec<u64>,
    /// `checksum`.
    pub checksum: u8,
}

impl LastSeenMessagesUpdate {
    /// `new Update(FriendlyByteBuf)`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let offset = buffer.read_var_int();
        let acknowledged = buffer.read_fixed_bit_set(LAST_SEEN_MESSAGES_MAX_LENGTH).into_iter().map(|w| w as u64).collect();
        let checksum = buffer.read_byte() as u8;
        Self { offset, acknowledged, checksum }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_var_int(self.offset);
        let _ = buffer.write_fixed_bit_set(&self.acknowledged, LAST_SEEN_MESSAGES_MAX_LENGTH);
        buffer.write_byte(self.checksum as i32);
    }

    /// `bit(i)` — установлен ли i-й бит acknowledged.
    pub fn acknowledged_bit(&self, i: usize) -> bool {
        let word = i / 64;
        let bit = i % 64;
        self.acknowledged.get(word).map_or(false, |w| (w >> bit) & 1 == 1)
    }

    /// `verifyChecksum`.
    pub fn verify_checksum(&self, messages: &LastSeenMessages) -> bool {
        self.checksum == IGNORE_CHECKSUM || self.checksum == messages.compute_checksum()
    }

    /// Число различных acknowledged-индексов (для валидации ≤20).
    pub fn acknowledged_count(&self) -> usize {
        let mut count = 0;
        for i in 0..LAST_SEEN_MESSAGES_MAX_LENGTH {
            if self.acknowledged_bit(i) {
                count += 1;
            }
        }
        count
    }
}

/// Хелпер: применить `Update` к `Vec<LastSeenTrackedEntry>`-подобному списку —
/// здесь как свободная функция над сигнатурами (последние N).
pub fn apply_update(
    update: &LastSeenMessagesUpdate,
    previous: &[MessageSignature],
) -> Vec<MessageSignature> {
    let mut result: Vec<MessageSignature> = Vec::new();
    let mut seen = HashSet::new();
    for (i, sig) in previous.iter().enumerate() {
        if update.acknowledged_bit(i) && seen.insert(sig) {
            result.push(sig.clone());
        }
    }
    result
}
