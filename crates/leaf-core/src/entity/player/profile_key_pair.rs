//! Перенос `net.minecraft.world.entity.player.ProfileKeyPair`.

use rsa::RsaPrivateKey;
use std::time::{SystemTime, UNIX_EPOCH};

use super::profile_public_key::ProfilePublicKey;

/// `ProfileKeyPair` — приватный + публичный ключ игрока и время рефреша.
pub struct ProfileKeyPair {
    /// `privateKey`.
    pub private_key: RsaPrivateKey,
    /// `publicKey`.
    pub public_key: ProfilePublicKey,
    /// `refreshedAfter` — epoch milli.
    pub refreshed_after: i64,
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl std::fmt::Debug for ProfileKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileKeyPair")
            .field("refreshed_after", &self.refreshed_after)
            .finish_non_exhaustive()
    }
}

impl ProfileKeyPair {
    pub fn new(private_key: RsaPrivateKey, public_key: ProfilePublicKey, refreshed_after: i64) -> Self {
        Self { private_key, public_key, refreshed_after }
    }

    /// `dueRefresh`.
    pub fn due_refresh(&self) -> bool {
        self.refreshed_after < now_millis()
    }
}
