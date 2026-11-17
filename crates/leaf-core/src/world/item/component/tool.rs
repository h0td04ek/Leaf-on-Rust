//! Перенос `net.minecraft.world.item.component.Tool`.

use crate::core::holder_set::{holder_set_stream_codec, HolderSet};
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::block::Block;

/// `Tool.Rule` — (blocks, speed?, correctForDrops?).
#[derive(Clone, Debug, PartialEq)]
pub struct ToolRule {
    /// `blocks`.
    pub blocks: HolderSet<Block>,
    /// `speed`.
    pub speed: Option<f32>,
    /// `correctForDrops`.
    pub correct_for_drops: Option<bool>,
}

impl ToolRule {
    /// `new`.
    pub fn new(blocks: HolderSet<Block>, speed: Option<f32>, correct_for_drops: Option<bool>) -> Self {
        Self { blocks, speed, correct_for_drops }
    }

    /// `minesAndDrops`.
    pub fn mines_and_drops(blocks: HolderSet<Block>, speed: f32) -> Self {
        Self::new(blocks, Some(speed), Some(true))
    }

    /// `deniesDrops`.
    pub fn denies_drops(blocks: HolderSet<Block>) -> Self {
        Self::new(blocks, None, Some(false))
    }

    /// `overrideSpeed`.
    pub fn override_speed(blocks: HolderSet<Block>, speed: f32) -> Self {
        Self::new(blocks, Some(speed), None)
    }

    /// `Rule.STREAM_CODEC` — holderSet(BLOCK) + optional(FLOAT) +
    /// optional(BOOL).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let speed_enc = byte_buf_codecs::optional(byte_buf_codecs::FLOAT);
        let speed_dec = byte_buf_codecs::optional(byte_buf_codecs::FLOAT);
        let drops_enc = byte_buf_codecs::optional(byte_buf_codecs::BOOL);
        let drops_dec = byte_buf_codecs::optional(byte_buf_codecs::BOOL);
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                holder_set_stream_codec::<Block>().encode(buf, &v.blocks);
                speed_enc.encode(buf, &v.speed);
                drops_enc.encode(buf, &v.correct_for_drops);
            },
            move |buf: &mut FriendlyByteBuf| {
                let blocks = holder_set_stream_codec::<Block>().decode(buf);
                let speed = speed_dec.decode(buf);
                let correct_for_drops = drops_dec.decode(buf);
                ToolRule::new(blocks, speed, correct_for_drops)
            },
        )
    }
}

/// `Tool` — (rules, defaultMiningSpeed, damagePerBlock,
/// canDestroyBlocksInCreative).
#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    /// `rules`.
    pub rules: Vec<ToolRule>,
    /// `defaultMiningSpeed`.
    pub default_mining_speed: f32,
    /// `damagePerBlock`.
    pub damage_per_block: i32,
    /// `canDestroyBlocksInCreative`.
    pub can_destroy_blocks_in_creative: bool,
}

impl Tool {
    /// `new`.
    pub fn new(
        rules: Vec<ToolRule>,
        default_mining_speed: f32,
        damage_per_block: i32,
        can_destroy_blocks_in_creative: bool,
    ) -> Self {
        Self { rules, default_mining_speed, damage_per_block, can_destroy_blocks_in_creative }
    }

    /// `getMiningSpeed(state)` — первое правило со speed и
    /// совпадающим blocks; иначе defaultMiningSpeed.
    /// (`state.is(holderSet)` — семантика block-state проверки
    /// добавится с BlockState.)
    pub fn get_mining_speed_for_tag(&self, tag_matches: impl Fn(&HolderSet<Block>) -> bool) -> f32 {
        for rule in &self.rules {
            if rule.speed.is_some() && tag_matches(&rule.blocks) {
                return rule.speed.unwrap();
            }
        }
        self.default_mining_speed
    }

    /// `STREAM_CODEC` — list(Rule) + FLOAT + VAR_INT + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Tool> + Send + Sync + 'static {
        let rules_enc = byte_buf_codecs::list(ToolRule::stream_codec());
        let rules_dec = byte_buf_codecs::list(ToolRule::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Tool| {
                rules_enc.encode(buf, &v.rules);
                buf.write_float(v.default_mining_speed);
                buf.write_var_int(v.damage_per_block);
                buf.write_bool(v.can_destroy_blocks_in_creative);
            },
            move |buf: &mut FriendlyByteBuf| {
                let rules = rules_dec.decode(buf);
                let default_mining_speed = buf.read_float();
                let damage_per_block = buf.read_var_int();
                let can_destroy_blocks_in_creative = buf.read_bool();
                Tool::new(rules, default_mining_speed, damage_per_block, can_destroy_blocks_in_creative)
            },
        )
    }
}
