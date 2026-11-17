//! Перенос `net.minecraft.network.chat.LocalChatSession`.

use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::entity::player::profile_key_pair::ProfileKeyPair;
use crate::entity::player::profile_public_key::{ProfilePublicKey, ProfilePublicKeyData};
use crate::network::chat::remote_chat_session::RemoteChatSession;
use crate::util::uuid::Uuid;

/// `LocalChatSession` — локальная сессия чата (ключи игрока).
pub struct LocalChatSession {
    /// `sessionId`.
    pub session_id: Uuid,
    /// `keyPair`.
    pub key_pair: ProfileKeyPair,
}

impl std::fmt::Debug for LocalChatSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalChatSession")
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

impl LocalChatSession {
    /// `create(keyPair)` — случайный sessionId.
    pub fn create(key_pair: ProfileKeyPair) -> Self {
        Self { session_id: Uuid::random(), key_pair }
    }

    /// `new(sessionId, keyPair)`.
    pub fn new(session_id: Uuid, key_pair: ProfileKeyPair) -> Self {
        Self { session_id, key_pair }
    }

    /// `asRemote`.
    pub fn as_remote(&self) -> RemoteChatSession {
        RemoteChatSession::new(
            self.session_id,
            self.key_pair.public_key.clone(),
        )
    }

    /// Удобный конструктор: сгенерировать ключевую пару RSA-2048
    /// и собрать сессию (аналог `ProfileKeyPair` creation в ServerPlayer).
    pub fn generate(
        expires_at: i64,
        key_signature: Vec<u8>,
        refreshed_after: i64,
    ) -> Result<Self, crate::network::encryption::crypt_error::CryptError> {
        let (priv_bytes, pub_bytes) = crate::network::encryption::crypt::generate_key_pair()?;
        let private_key = crate::network::encryption::crypt::byte_to_private_key(&priv_bytes)?;
        let public_key = crate::network::encryption::crypt::byte_to_public_key(&pub_bytes)?;
        let key_pair = ProfileKeyPair::new(
            private_key,
            ProfilePublicKey::new(ProfilePublicKeyData::new(expires_at, public_key, key_signature)),
            refreshed_after,
        );
        Ok(Self::create(key_pair))
    }
}

/// Публичный ключ сессии (для `createMessageEncoder`-валидации).
pub fn public_key_of(session: &LocalChatSession) -> RsaPublicKey {
    RsaPublicKey::from(&session.key_pair.private_key)
}

#[allow(dead_code)]
fn _unused(_: RsaPrivateKey) {}
