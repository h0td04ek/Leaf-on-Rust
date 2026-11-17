//! Перенос `net.minecraft.world.InteractionHand`.

/// `InteractionHand` — MAIN_HAND/OFF_HAND.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InteractionHand {
    /// `MAIN_HAND`.
    MainHand = 0,
    /// `OFF_HAND`.
    OffHand = 1,
}

/// Все значения (`values()`).
pub const INTERACTION_HANDS: [InteractionHand; 2] = [
    InteractionHand::MainHand,
    InteractionHand::OffHand,
];

impl InteractionHand {
    /// `values()`.
    pub fn values() -> &'static [InteractionHand] {
        &INTERACTION_HANDS
    }

    /// `byId` — ordinal (`readEnum`).
    pub fn by_id(id: i32) -> InteractionHand {
        INTERACTION_HANDS[id.rem_euclid(2) as usize]
    }

    /// `ordinal` (`writeEnum` = VarInt ordinal).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// `asEquipmentSlot` — EquipmentSlot ещё не портирован;
    /// возвращаем строковый эквивалент для совместимости.
    pub fn as_equipment_slot_name(&self) -> &'static str {
        match self {
            InteractionHand::MainHand => "mainhand",
            InteractionHand::OffHand => "offhand",
        }
    }

    /// `STREAM_CODEC` (`ByteBufCodecs.idMapper`-эквивалент enum codec).
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        InteractionHand,
    > {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, h: &InteractionHand| {
                buf.write_var_int(h.ordinal());
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                InteractionHand::by_id(buf.read_var_int())
            },
        )
    }
}
