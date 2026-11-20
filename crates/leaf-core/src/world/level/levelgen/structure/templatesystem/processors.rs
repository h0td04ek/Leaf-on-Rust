//! Перенос `StructureProcessor` + всех наследников:
//! `RuleProcessor`, `NopProcessor`, `CappedProcessor`,
//! `GravityProcessor`, `BlockRotProcessor`, `BlockIgnoreProcessor`,
//! `BlockAgeProcessor`, `BlackstoneReplaceProcessor`,
//! `LavaSubmergedBlockProcessor`, `JigsawReplacementProcessor`,
//! `ProtectedBlockProcessor`.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::random::create as random_create;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::tags::{block_tags::BlockTags, is_block_in_named_tag};
use crate::util::mth;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::property::StateValue;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::templatesystem::processor_rule::ProcessorRule;
use crate::world::level::levelgen::structure::templatesystem::structure_place_settings::StructurePlaceSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::StructureBlockInfo;
use crate::util::valueproviders::int_provider::IntProvider;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// `StructureProcessor` — процессор блоков шаблона.
pub trait StructureProcessor {
    /// `processBlock(level, offset, pos, blockInfo, relativeBlockInfo,
    /// settings)` → Option<StructureBlockInfo> (None = удалить).
    fn process_block(
        &self,
        level: &dyn LevelReader,
        offset: BlockPos,
        pos: BlockPos,
        block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let _ = (level, offset, pos, block_info, settings);
        Some(relative_block_info.clone())
    }

    /// `finalizeProcessing` — пост-обработка всего списка.
    fn finalize_processing(
        &self,
        level: &dyn LevelReader,
        offset: BlockPos,
        pos: BlockPos,
        original_block_infos: &[StructureBlockInfo],
        processed_block_infos: &mut Vec<StructureBlockInfo>,
        settings: &StructurePlaceSettings,
    ) -> Vec<StructureBlockInfo> {
        let _ = (level, offset, pos, original_block_infos, settings);
        std::mem::take(processed_block_infos)
    }
}

/// `NopProcessor` — процессор-заглушка.
pub struct NopProcessor;

impl StructureProcessor for NopProcessor {}

/// `RuleProcessor` — правила замены блоков.
pub struct RuleProcessor {
    /// `rules`.
    pub rules: Vec<ProcessorRule>,
}

impl RuleProcessor {
    /// `new(rules)`.
    pub fn new(rules: Vec<ProcessorRule>) -> Self {
        Self { rules }
    }
}

impl StructureProcessor for RuleProcessor {
    fn process_block(
        &self,
        level: &dyn LevelReader,
        _offset: BlockPos,
        pos: BlockPos,
        block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let mut random = random_create(mth::get_seed(
            relative_block_info.pos.x,
            relative_block_info.pos.y,
            relative_block_info.pos.z,
        ));
        let block_state = level
            .get_block_state(relative_block_info.pos)
            .unwrap_or_else(BlockState::air);
        for rule in &self.rules {
            if rule.test(
                &relative_block_info.state,
                &block_state,
                block_info.pos,
                relative_block_info.pos,
                pos,
                &mut *random,
            ) {
                let output_tag = rule.get_output_tag(&mut *random, relative_block_info.nbt.as_ref());
                return Some(StructureBlockInfo {
                    pos: relative_block_info.pos,
                    state: rule.output_state.clone(),
                    nbt: output_tag,
                });
            }
        }
        Some(relative_block_info.clone())
    }
}

/// `CappedProcessor` — ограничивает число замен delegate'а.
pub struct CappedProcessor {
    /// `delegate`.
    pub delegate: Rc<dyn StructureProcessor>,
    /// `limit`.
    pub limit: Box<dyn IntProvider>,
}

impl CappedProcessor {
    /// `new(delegate, limit)`.
    pub fn new(delegate: Rc<dyn StructureProcessor>, limit: Box<dyn IntProvider>) -> Self {
        Self { delegate, limit }
    }
}

impl StructureProcessor for CappedProcessor {
    fn finalize_processing(
        &self,
        level: &dyn LevelReader,
        offset: BlockPos,
        pos: BlockPos,
        original_block_infos: &[StructureBlockInfo],
        processed_block_infos: &mut Vec<StructureBlockInfo>,
        settings: &StructurePlaceSettings,
    ) -> Vec<StructureBlockInfo> {
        if self.limit.max_value() == 0 || processed_block_infos.is_empty() {
            return std::mem::take(processed_block_infos);
        }
        if original_block_infos.len() != processed_block_infos.len() {
            return std::mem::take(processed_block_infos);
        }
        let mut random = random_create(level.world_seed());
        let min = self
            .limit
            .sample(&mut *random)
            .min(processed_block_infos.len() as i32) as usize;
        if min < 1 {
            return std::mem::take(processed_block_infos);
        }
        // toShuffledList(IntStream.range(0, size), random)
        let mut indices: Vec<i32> = (0..processed_block_infos.len() as i32).collect();
        // Fisher-Yates как в Util.toShuffledList
        for i in (1..indices.len()).rev() {
            let j = random.next_int_bound(i as i32 + 1) as usize;
            indices.swap(i, j);
        }
        let mut count = 0;
        let mut result = std::mem::take(processed_block_infos);
        for &idx in indices.iter().take(min) {
            let i1 = idx as usize;
            let original = original_block_infos[i1].clone();
            let processed = result[i1].clone();
            let new_info = self.delegate.process_block(
                level,
                offset,
                pos,
                &original,
                &processed,
                settings,
            );
            if let Some(new_info) = new_info {
                if new_info != processed {
                    count += 1;
                    result[i1] = new_info;
                }
            }
        }
        let _ = count;
        result
    }
}

/// `GravityProcessor` — опускает блоки до heightmap + offset.
pub struct GravityProcessor {
    /// `heightmap`.
    pub heightmap: HeightmapType,
    /// `offset`.
    pub offset: i32,
}

impl GravityProcessor {
    /// `new(heightmap, offset)`.
    pub fn new(heightmap: HeightmapType, offset: i32) -> Self {
        Self { heightmap, offset }
    }
}

impl StructureProcessor for GravityProcessor {
    fn process_block(
        &self,
        level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let block_pos = relative_block_info.pos;
        // ServerLevel → WORLD_SURFACE_WG→WORLD_SURFACE и
        // OCEAN_FLOOR_WG→OCEAN_FLOOR; для LevelReader-хука
        // используем переданный heightmap напрямую.
        let target_height =
            level.get_height(self.heightmap, block_pos.x, block_pos.z) + self.offset;
        let y = block_info.pos.y;
        if target_height <= y {
            Some(StructureBlockInfo {
                pos: BlockPos::new(block_pos.x, target_height, block_pos.z),
                state: relative_block_info.state.clone(),
                nbt: relative_block_info.nbt.clone(),
            })
        } else {
            Some(StructureBlockInfo {
                pos: block_pos,
                state: BlockState::air(),
                nbt: None,
            })
        }
    }
}

/// `BlockRotProcessor` — поворот mirror/rotation блоков.
pub struct BlockRotProcessor;

impl StructureProcessor for BlockRotProcessor {
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let mut state = relative_block_info.state.clone();
        // mirror/rotation применение — через property-aware
        // трансформации (state_mirror/state_rotate из structure_piece).
        if settings.mirror != Mirror::NONE {
            state = crate::world::level::levelgen::structure::structure_piece::state_mirror(
                state,
                settings.mirror,
            );
        }
        if settings.rotation != Rotation::NONE {
            state = crate::world::level::levelgen::structure::structure_piece::state_rotate(
                state,
                settings.rotation,
            );
        }
        Some(StructureBlockInfo {
            pos: relative_block_info.pos,
            state,
            nbt: relative_block_info.nbt.clone(),
        })
    }
}

/// `BlockIgnoreProcessor` — удаляет блоки по предикату `toIgnore`
/// (tag/список блоков, которые пропускаются → null).
pub struct BlockIgnoreProcessor {
    /// `toIgnore` — список block id (упрощённо; Java хранит
    /// HolderSet<Block>).
    pub to_ignore: Vec<Identifier>,
}

impl StructureProcessor for BlockIgnoreProcessor {
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        if self.to_ignore.contains(&relative_block_info.state.block.id) {
            None
        } else {
            Some(relative_block_info.clone())
        }
    }
}

/// `BlockAgeProcessor` — старение блоков (mossy/cracked/crying).
pub struct BlockAgeProcessor {
    /// `mossiness`.
    pub mossiness: f32,
}

impl BlockAgeProcessor {
    /// `new(mossiness)`.
    pub fn new(mossiness: f32) -> Self {
        Self { mossiness }
    }

    fn block_state_by_path(path: &str) -> BlockState {
        block_state_by_path(path)
    }

    fn is_block(state: &BlockState, path: &str) -> bool {
        state.block.id.path == path
    }

    fn random_facing_stairs(random: &mut dyn RandomSource, block: &str) -> BlockState {
        let mut state = Self::block_state_by_path(block);
        state
            .values
            .set_erased("facing", StateValue::Str(crate::world::level::levelgen::structure::structure_piece::get_random_horizontal_direction(random).get_name().to_string()));
        state
            .values
            .set_erased("half", StateValue::Str(if random.next_boolean() { "top".to_string() } else { "bottom".to_string() }));
        state
    }

    fn maybe_replace_full_stone_block(&self, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() >= 0.5 {
            return None;
        }
        let normal = [
            Self::block_state_by_path("cracked_stone_bricks"),
            Self::random_facing_stairs(random, "stone_brick_stairs"),
        ];
        let mossy = [
            Self::block_state_by_path("mossy_stone_bricks"),
            Self::random_facing_stairs(random, "mossy_stone_brick_stairs"),
        ];
        Some(Self::get_random_block(random, &normal, &mossy, self.mossiness))
    }

    fn maybe_replace_stairs(&self, state: &BlockState, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() >= 0.5 {
            return None;
        }
        let normal = [
            Self::with_properties_of(Self::block_state_by_path("stone_brick_stairs"), state),
            Self::block_state_by_path("stone_brick_slab"),
        ];
        let mossy = [
            Self::with_properties_of(Self::block_state_by_path("mossy_stone_brick_stairs"), state),
            Self::block_state_by_path("mossy_stone_brick_slab"),
        ];
        Some(Self::get_random_block(random, &normal, &mossy, self.mossiness))
    }

    fn maybe_replace_slab(&self, state: &BlockState, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() < self.mossiness {
            Some(Self::with_properties_of(
                Self::block_state_by_path("mossy_stone_brick_slab"),
                state,
            ))
        } else {
            None
        }
    }

    fn maybe_replace_wall(&self, state: &BlockState, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() < self.mossiness {
            Some(Self::with_properties_of(
                Self::block_state_by_path("mossy_stone_brick_wall"),
                state,
            ))
        } else {
            None
        }
    }

    fn maybe_replace_obsidian(&self, random: &mut dyn RandomSource) -> Option<BlockState> {
        if random.next_float() < 0.15 {
            Some(Self::block_state_by_path("crying_obsidian"))
        } else {
            None
        }
    }

    /// `withPropertiesOf` — копирует общие свойства (facing/half/
    /// type/waterlogged) из `from` в `to`.
    fn with_properties_of(mut to: BlockState, from: &BlockState) -> BlockState {
        for name in ["facing", "half", "type", "waterlogged", "shape"] {
            if let Some(value) = from.values.get_by_name(name) {
                if to.values.get_by_name(name).is_some() {
                    to.values.set_erased(name, value);
                }
            }
        }
        to
    }

    fn get_random_block(
        random: &mut dyn RandomSource,
        normal: &[BlockState],
        mossy: &[BlockState],
        mossiness: f32,
    ) -> BlockState {
        let states = if random.next_float() < mossiness { mossy } else { normal };
        states[random.next_int_bound(states.len() as i32) as usize].clone()
    }
}

impl StructureProcessor for BlockAgeProcessor {
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let mut random = settings.get_random(Some(relative_block_info.pos));
        let block_state = &relative_block_info.state;
        let block_pos = relative_block_info.pos;
        let new_state = if Self::is_block(block_state, "stone_bricks")
            || Self::is_block(block_state, "stone")
            || Self::is_block(block_state, "chiseled_stone_bricks")
        {
            self.maybe_replace_full_stone_block(&mut *random)
        } else if is_block_in_named_tag(&block_state.block.id, &BlockTags::stairs()) {
            self.maybe_replace_stairs(block_state, &mut *random)
        } else if is_block_in_named_tag(&block_state.block.id, &BlockTags::slabs()) {
            self.maybe_replace_slab(block_state, &mut *random)
        } else if is_block_in_named_tag(&block_state.block.id, &BlockTags::walls()) {
            self.maybe_replace_wall(block_state, &mut *random)
        } else if Self::is_block(block_state, "obsidian") {
            self.maybe_replace_obsidian(&mut *random)
        } else {
            None
        };
        match new_state {
            Some(state) => Some(StructureBlockInfo {
                pos: block_pos,
                state,
                nbt: relative_block_info.nbt.clone(),
            }),
            None => Some(relative_block_info.clone()),
        }
    }
}

/// `BlackstoneReplaceProcessor` — замена на blackstone-варианты
/// (карта замен как в Java).
pub struct BlackstoneReplaceProcessor;


/// `blockStateByPath` — BlockState по id блока (default state).
fn block_state_by_path(path: &str) -> BlockState {
    crate::world::level::block::block_by_identifier(&Identifier::minecraft(path))
        .and_then(|b| b.default_block_state())
        .unwrap_or_else(BlockState::air)
}

impl BlackstoneReplaceProcessor {
    /// `replacements` — block id → replacement id.
    fn replacements() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("cobblestone", "blackstone"),
            ("mossy_cobblestone", "blackstone"),
            ("stone", "polished_blackstone"),
            ("stone_bricks", "polished_blackstone_bricks"),
            ("mossy_stone_bricks", "polished_blackstone_bricks"),
            ("cobblestone_stairs", "blackstone_stairs"),
            ("mossy_cobblestone_stairs", "blackstone_stairs"),
            ("stone_stairs", "polished_blackstone_stairs"),
            ("stone_brick_stairs", "polished_blackstone_brick_stairs"),
            ("mossy_stone_brick_stairs", "polished_blackstone_brick_stairs"),
            ("cobblestone_slab", "blackstone_slab"),
            ("mossy_cobblestone_slab", "blackstone_slab"),
            ("smooth_stone_slab", "polished_blackstone_slab"),
            ("stone_slab", "polished_blackstone_slab"),
            ("stone_brick_slab", "polished_blackstone_brick_slab"),
            ("mossy_stone_brick_slab", "polished_blackstone_brick_slab"),
            ("stone_brick_wall", "polished_blackstone_brick_wall"),
            ("mossy_stone_brick_wall", "polished_blackstone_brick_wall"),
            ("cobblestone_wall", "blackstone_wall"),
            ("mossy_cobblestone_wall", "blackstone_wall"),
            ("chiseled_stone_bricks", "chiseled_polished_blackstone"),
            ("cracked_stone_bricks", "cracked_polished_blackstone_bricks"),
            ("iron_bars", "iron_chain"),
        ])
    }
}

impl StructureProcessor for BlackstoneReplaceProcessor {
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let replacements = Self::replacements();
        let Some(&replacement) = replacements.get(relative_block_info.state.block.id.path.as_str())
        else {
            return Some(relative_block_info.clone());
        };
        let mut new_state = block_state_by_path(replacement);
        let old = &relative_block_info.state;
        // StairBlock.FACING/HALF, SlabBlock.TYPE — копируем общие
        // свойства при наличии.
        for name in ["facing", "half", "type"] {
            if let Some(value) = old.values.get_by_name(name) {
                if new_state.values.get_by_name(name).is_some() {
                    new_state.values.set_erased(name, value);
                }
            }
        }
        Some(StructureBlockInfo {
            pos: relative_block_info.pos,
            state: new_state,
            nbt: relative_block_info.nbt.clone(),
        })
    }
}

/// `LavaSubmergedBlockProcessor` — лава заменяет не-solid блоки.
pub struct LavaSubmergedBlockProcessor;

impl StructureProcessor for LavaSubmergedBlockProcessor {
    fn process_block(
        &self,
        level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let block_pos = relative_block_info.pos;
        let is_lava = level
            .get_block_state(block_pos)
            .map(|s| s.block.id.path == "lava")
            .unwrap_or(false);
        if is_lava && !relative_block_info.state.is_solid_render(level, block_pos) {
            Some(StructureBlockInfo {
                pos: block_pos,
                state: block_state_by_path("lava"),
                nbt: relative_block_info.nbt.clone(),
            })
        } else {
            Some(relative_block_info.clone())
        }
    }
}

/// `ProtectedBlockProcessor` — блоки в `cannotReplace` теге не
/// заменяются (protected).
pub struct ProtectedBlockProcessor {
    /// `cannotReplace`.
    pub cannot_replace: Identifier,
}

impl StructureProcessor for ProtectedBlockProcessor {
    fn process_block(
        &self,
        _level: &dyn LevelReader,
        _offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        // Feature.isReplaceable(tag).test(state) — protected если
        // блок НЕ в cannotReplace-теге? В Java: test = isReplaceable
        // → вернуть relativeBlockInfo (разрешить замену); иначе null.
        // isReplaceable(tag) — блок НЕ в списке защищённых.
        if !is_block_in_named_tag(&relative_block_info.state.block.id, &self.cannot_replace) {
            Some(relative_block_info.clone())
        } else {
            None
        }
    }
}

/// `JigsawReplacementProcessor` — jigsaw → final_state или
/// structure_void удаление.
pub struct JigsawReplacementProcessor;

impl StructureProcessor for JigsawReplacementProcessor {
    fn process_block(
        &self,
        level: &dyn LevelReader,
        offset: BlockPos,
        _pos: BlockPos,
        _block_info: &StructureBlockInfo,
        relative_block_info: &StructureBlockInfo,
        _settings: &StructurePlaceSettings,
    ) -> Option<StructureBlockInfo> {
        let block_state = &relative_block_info.state;
        if block_state.block.id.path != "jigsaw" {
            return Some(relative_block_info.clone());
        }
        let Some(nbt) = &relative_block_info.nbt else {
            return Some(relative_block_info.clone());
        };
        let final_state_str = nbt
            .get_string("final_state")
            .unwrap_or("minecraft:air")
            .to_string();
        // BlockStateParser.parseForBlock — упрощённо через
        // block_state_by_id по identifier.
        let state = if let Ok(id) = Identifier::parse(&final_state_str) {
            crate::world::level::block::block_by_identifier(&id)
                .and_then(|b| b.default_block_state())
                .unwrap_or_else(BlockState::air)
        } else {
            BlockState::air()
        };
        let _ = (level, offset);
        if state.block.id.path == "structure_void" {
            None
        } else {
            Some(StructureBlockInfo {
                pos: relative_block_info.pos,
                state,
                nbt: None,
            })
        }
    }
}
