//! Перенос `net.minecraft.advancements.CriterionProgress`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// `CriterionProgress` — прогресс одного критерия.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CriterionProgress {
    /// `obtained` — момент получения (None = не получен).
    pub obtained: Option<i64>, // millis since epoch
}

impl CriterionProgress {
    /// `new()`.
    pub fn new() -> Self {
        Self { obtained: None }
    }

    /// `new(obtained)`.
    pub fn with_obtained(obtained: i64) -> Self {
        Self { obtained: Some(obtained) }
    }

    /// `isDone`.
    pub fn is_done(&self) -> bool {
        self.obtained.is_some()
    }

    /// `grant`.
    pub fn grant(&mut self) {
        self.obtained = Some(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
        );
    }

    /// `revoke`.
    pub fn revoke(&mut self) {
        self.obtained = None;
    }

    /// `serializeToNetwork`.
    pub fn serialize_to_network(&self, buffer: &mut FriendlyByteBuf) {
        match self.obtained {
            Some(ms) => {
                buffer.write_bool(true);
                buffer.write_long(ms);
            }
            None => {
                buffer.write_bool(false);
            }
        }
    }

    /// `fromNetwork`.
    pub fn from_network(buffer: &mut FriendlyByteBuf) -> Self {
        let obtained = if buffer.read_bool() {
            Some(buffer.read_long())
        } else {
            None
        };
        Self { obtained }
    }
}
