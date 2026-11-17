//! Перенос `net.minecraft.world.item.component.Weapon`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Weapon` — (itemDamagePerAttack, disableBlockingForSeconds).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weapon {
    /// `itemDamagePerAttack`.
    pub item_damage_per_attack: i32,
    /// `disableBlockingForSeconds`.
    pub disable_blocking_for_seconds: f32,
}

impl Weapon {
    /// `AXE_DISABLES_BLOCKING_FOR_SECONDS`.
    pub const AXE_DISABLES_BLOCKING_FOR_SECONDS: f32 = 5.0;

    /// `new`.
    pub fn new(item_damage_per_attack: i32, disable_blocking_for_seconds: f32) -> Self {
        Self { item_damage_per_attack, disable_blocking_for_seconds }
    }

    /// `STREAM_CODEC` — VAR_INT + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Weapon> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Weapon| {
                buf.write_var_int(v.item_damage_per_attack);
                buf.write_float(v.disable_blocking_for_seconds);
            },
            |buf: &mut FriendlyByteBuf| {
                let item_damage_per_attack = buf.read_var_int();
                let disable_blocking_for_seconds = buf.read_float();
                Weapon::new(item_damage_per_attack, disable_blocking_for_seconds)
            },
        )
    }
}
