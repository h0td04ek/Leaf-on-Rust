//! Перенос `net.minecraft.network.chat.MessageSignatureCache`.

use crate::network::chat::message_signature::{MessageSignature, PackedSignature};
use std::collections::HashSet;

/// `NOT_FOUND`.
pub const NOT_FOUND: i32 = -1;
/// `DEFAULT_CAPACITY`.
pub const DEFAULT_CAPACITY: usize = 128;

/// `MessageSignatureCache` — LRU-кеш подписей для `Packed`-ссылок.
pub struct MessageSignatureCache {
    /// `entries` — фиксированный массив; `None` = пустой слот.
    entries: Vec<Option<MessageSignature>>,
}

impl MessageSignatureCache {
    /// `new MessageSignatureCache(size)`.
    pub fn new(size: usize) -> Self {
        Self { entries: vec![None; size] }
    }

    /// `createDefault`.
    pub fn create_default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    /// `pack` — id подписи в кеше или `NOT_FOUND`.
    pub fn pack(&self, signature: &MessageSignature) -> i32 {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.as_ref() == Some(signature) {
                return i as i32;
            }
        }
        NOT_FOUND
    }

    /// `unpack` — подпись по id.
    pub fn unpack(&self, index: i32) -> Option<MessageSignature> {
        if index < 0 || index as usize >= self.entries.len() {
            return None;
        }
        self.entries[index as usize].clone()
    }

    /// `push(signedMessageBody, signature)` — последние seen-подписи
    /// (без `SignedMessageBody` — принимаем список + опциональную подпись).
    pub fn push(&mut self, last_seen: &[MessageSignature], signature: Option<&MessageSignature>) {
        let mut deque: std::collections::VecDeque<MessageSignature> = last_seen.to_vec().into();
        if let Some(sig) = signature {
            deque.push_back(sig.clone());
        }
        self.push_deque(&mut deque);
    }

    /// `push(chatMessages)` — VisibleForTesting-вариант.
    pub fn push_list(&mut self, chat_messages: &[MessageSignature]) {
        let mut deque: std::collections::VecDeque<MessageSignature> =
            chat_messages.iter().cloned().collect();
        self.push_deque(&mut deque);
    }

    fn push_deque(&mut self, deque: &mut std::collections::VecDeque<MessageSignature>) {
        let set: HashSet<MessageSignature> = deque.iter().cloned().collect();
        for slot in self.entries.iter_mut() {
            if deque.is_empty() {
                break;
            }
            let old = slot.take();
            if let Some(new_sig) = deque.pop_back() {
                *slot = Some(new_sig);
            }
            if let Some(old_sig) = old {
                if !set.contains(&old_sig) {
                    deque.push_front(old_sig);
                }
            }
        }
    }

    /// `PackedSignature.unpack` — развернуть packed в полную подпись.
    pub fn unpack_packed(&self, packed: &PackedSignature) -> Option<MessageSignature> {
        packed.full_signature.clone().or_else(|| self.unpack(packed.id))
    }
}
