//! Перенос `net.minecraft.world.entity.HumanoidArm`.

use crate::network::chat::component::Component;

/// `HumanoidArm` — основная рука.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HumanoidArm {
    Left,
    Right,
}

impl HumanoidArm {
    /// Порядок в `values()` (id = ordinal в Java).
    pub const VALUES: [HumanoidArm; 2] = [HumanoidArm::Left, HumanoidArm::Right];

    /// `getOpposite()`.
    pub fn get_opposite(&self) -> HumanoidArm {
        match self {
            HumanoidArm::Left => HumanoidArm::Right,
            HumanoidArm::Right => HumanoidArm::Left,
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            HumanoidArm::Left => "left",
            HumanoidArm::Right => "right",
        }
    }

    /// `caption()`.
    pub fn caption(&self) -> Component {
        Component::translatable(
            match self {
                HumanoidArm::Left => "options.mainHand.left",
                HumanoidArm::Right => "options.mainHand.right",
            },
            Vec::new(),
        )
    }

    /// `id()` — ordinal в `VALUES`.
    pub fn id(&self) -> i32 {
        match self {
            HumanoidArm::Left => 0,
            HumanoidArm::Right => 1,
        }
    }

    /// `BY_ID` (OutOfBoundsStrategy.ZERO — непрерывный маппинг 0..1).
    pub fn by_id(id: i32) -> HumanoidArm {
        HumanoidArm::VALUES[id.rem_euclid(2) as usize]
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(HumanoidArm::byId)`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, HumanoidArm> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, arm: &HumanoidArm| {
                buf.write_var_int(arm.id());
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                HumanoidArm::by_id(buf.read_var_int())
            },
        )
    }
}
