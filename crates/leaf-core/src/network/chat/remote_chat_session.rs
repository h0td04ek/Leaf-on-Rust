//! Перенос `net.minecraft.network.chat.RemoteChatSession` (+ `Data`).

use crate::entity::player::profile_public_key::{ProfilePublicKey, ProfilePublicKeyData};
use crate::network::friendly_byte_buf::{CodecError, FriendlyByteBuf};
use crate::util::uuid::Uuid;

/// `RemoteChatSession` — сессия чата удалённого игрока.
pub struct RemoteChatSession {
    /// `sessionId`.
    pub session_id: Uuid,
    /// `profilePublicKey`.
    pub profile_public_key: ProfilePublicKey,
}

impl std::fmt::Debug for RemoteChatSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteChatSession")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl Clone for RemoteChatSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id,
            profile_public_key: self.profile_public_key.clone(),
        }
    }
}

impl RemoteChatSession {
    pub fn new(session_id: Uuid, profile_public_key: ProfilePublicKey) -> Self {
        Self { session_id, profile_public_key }
    }

    /// `asData`.
    pub fn as_data(&self) -> RemoteChatSessionData {
        RemoteChatSessionData {
            session_id: self.session_id,
            profile_public_key: self.profile_public_key.data.clone(),
        }
    }

    /// `hasExpired`.
    pub fn has_expired(&self) -> bool {
        self.profile_public_key.data.has_expired()
    }
}

/// `RemoteChatSession.Data` — wire-форма (UUID + ProfilePublicKey.Data).
#[derive(Clone)]
pub struct RemoteChatSessionData {
    /// `sessionId`.
    pub session_id: Uuid,
    /// `profilePublicKey`.
    pub profile_public_key: ProfilePublicKeyData,
}

impl std::fmt::Debug for RemoteChatSessionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteChatSession.Data")
            .field("session_id", &self.session_id)
            .field("profile_public_key", &self.profile_public_key)
            .finish()
    }
}
impl PartialEq for RemoteChatSessionData {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id && self.profile_public_key == other.profile_public_key
    }
}
impl Eq for RemoteChatSessionData {}

impl RemoteChatSessionData {
    pub fn new(session_id: Uuid, profile_public_key: ProfilePublicKeyData) -> Self {
        Self { session_id, profile_public_key }
    }

    /// `RemoteChatSession.Data.read`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Result<Self, CodecError> {
        let session_id = buffer.read_uuid();
        let profile_public_key = ProfilePublicKeyData::read(buffer)?;
        Ok(Self { session_id, profile_public_key })
    }

    /// `RemoteChatSession.Data.write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) -> Result<(), CodecError> {
        buffer.write_uuid(&self.session_id);
        self.profile_public_key.write(buffer)
    }

    /// `validate` — проверить подпись ключа профиля и собрать сессию.
    /// `profile_id` — UUID игрока, `signature_validator` — публичный ключ Mojang-подписи.
    pub fn validate(
        &self,
        profile_id: &Uuid,
        signature_validator: &crate::util::signature_validator::SignatureValidator,
    ) -> Result<RemoteChatSession, String> {
        if !self.profile_public_key.validate_signature(signature_validator, profile_id) {
            return Err("invalid public key signature".to_string());
        }
        Ok(RemoteChatSession::new(
            self.session_id,
            ProfilePublicKey::new(self.profile_public_key.clone()),
        ))
    }
}
