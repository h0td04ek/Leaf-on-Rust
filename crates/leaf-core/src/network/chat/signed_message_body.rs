//! Перенос `net.minecraft.network.chat.SignedMessageBody` (+ `Packed`).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::network::chat::last_seen_messages::{LastSeenMessages, LastSeenMessagesPacked};
use crate::network::chat::message_signature_cache::MessageSignatureCache;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `SignedMessageBody` — content + timestamp + salt + lastSeen.
#[derive(Clone, Debug, PartialEq)]
pub struct SignedMessageBody {
    /// `content`.
    pub content: String,
    /// `timeStamp` — epoch milli.
    pub time_stamp: i64,
    /// `salt`.
    pub salt: i64,
    /// `lastSeen`.
    pub last_seen: LastSeenMessages,
}

impl SignedMessageBody {
    pub fn new(content: String, time_stamp: i64, salt: i64, last_seen: LastSeenMessages) -> Self {
        Self { content, time_stamp, salt, last_seen }
    }

    /// `unsigned(content)`.
    pub fn unsigned(content: String) -> Self {
        Self {
            content,
            time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            salt: 0,
            last_seen: LastSeenMessages::empty(),
        }
    }

    /// `updateSignature` — salt(8) + epochSecond(8) + len(4) + utf8 + lastSeen.
    pub fn update_signature<F>(&self, mut output: F) -> Result<(), crate::util::signature_validator::SignatureException>
    where
        F: FnMut(&[u8]) -> Result<(), crate::util::signature_validator::SignatureException>,
    {
        output(&self.salt.to_be_bytes())?;
        output(&self.epoch_second().to_be_bytes())?;
        let bytes = self.content.as_bytes();
        output(&(bytes.len() as i32).to_be_bytes())?;
        output(bytes)?;
        self.last_seen.update_signature(&mut output)
    }

    /// `timeStamp.getEpochSecond`.
    fn epoch_second(&self) -> i64 {
        self.time_stamp.div_euclid(1000)
    }

    /// `pack`.
    pub fn pack(&self, cache: &mut MessageSignatureCache) -> SignedMessageBodyPacked {
        SignedMessageBodyPacked {
            content: self.content.clone(),
            time_stamp: self.time_stamp,
            salt: self.salt,
            last_seen: self.last_seen.pack(cache),
        }
    }
}

/// `SignedMessageBody.Packed` — wire-форма.
#[derive(Clone, Debug, PartialEq)]
pub struct SignedMessageBodyPacked {
    /// `content`.
    pub content: String,
    /// `timeStamp` — epoch milli.
    pub time_stamp: i64,
    /// `salt`.
    pub salt: i64,
    /// `lastSeen`.
    pub last_seen: LastSeenMessagesPacked,
}

impl SignedMessageBodyPacked {
    /// `new Packed(FriendlyByteBuf)` — readUtf(256) + instant + long + lastSeen.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Result<Self, crate::network::friendly_byte_buf::CodecError> {
        let content = buffer.read_utf_n(256)?;
        let time_stamp = buffer.read_instant();
        let salt = buffer.read_long();
        let last_seen = LastSeenMessagesPacked::read(buffer);
        Ok(Self { content, time_stamp, salt, last_seen })
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_utf_n(&self.content, 256).expect("content too long");
        buffer.write_instant(self.time_stamp);
        buffer.write_long(self.salt);
        self.last_seen.write(buffer);
    }

    /// `unpack`.
    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<SignedMessageBody> {
        Some(SignedMessageBody::new(
            self.content.clone(),
            self.time_stamp,
            self.salt,
            self.last_seen.unpack(cache)?,
        ))
    }
}
