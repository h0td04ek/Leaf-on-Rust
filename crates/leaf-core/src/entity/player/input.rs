//! Перенос `net.minecraft.world.entity.player.Input` — флаги ввода игрока.

/// `Input` — record из 7 boolean (упаковывается в 1 байт).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    /// `forward`.
    pub forward: bool,
    /// `backward`.
    pub backward: bool,
    /// `left`.
    pub left: bool,
    /// `right`.
    pub right: bool,
    /// `jump`.
    pub jump: bool,
    /// `shift`.
    pub shift: bool,
    /// `sprint`.
    pub sprint: bool,
}

const FLAG_FORWARD: i32 = 1;
const FLAG_BACKWARD: i32 = 2;
const FLAG_LEFT: i32 = 4;
const FLAG_RIGHT: i32 = 8;
const FLAG_JUMP: i32 = 16;
const FLAG_SHIFT: i32 = 32;
const FLAG_SPRINT: i32 = 64;

/// `Input.EMPTY`.
pub const EMPTY: Input = Input {
    forward: false,
    backward: false,
    left: false,
    right: false,
    jump: false,
    shift: false,
    sprint: false,
};

impl Input {
    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        Input,
    > {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, i: &Input| {
                let mut b: i32 = 0;
                if i.forward { b |= FLAG_FORWARD; }
                if i.backward { b |= FLAG_BACKWARD; }
                if i.left { b |= FLAG_LEFT; }
                if i.right { b |= FLAG_RIGHT; }
                if i.jump { b |= FLAG_JUMP; }
                if i.shift { b |= FLAG_SHIFT; }
                if i.sprint { b |= FLAG_SPRINT; }
                buf.write_byte(b);
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                let b = buf.read_byte() as i32;
                Input {
                    forward: b & FLAG_FORWARD != 0,
                    backward: b & FLAG_BACKWARD != 0,
                    left: b & FLAG_LEFT != 0,
                    right: b & FLAG_RIGHT != 0,
                    jump: b & FLAG_JUMP != 0,
                    shift: b & FLAG_SHIFT != 0,
                    sprint: b & FLAG_SPRINT != 0,
                }
            },
        )
    }
}
