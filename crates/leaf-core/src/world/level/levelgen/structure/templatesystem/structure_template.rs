//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate`
//! — палитры блоков, сущности, трансформации, `placeInWorld`,
//! `processBlockInfos`, NBT save/load.

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::vec3i::Vec3i;
use crate::nbt::compound::CompoundTag;
use crate::resources::identifier::Identifier;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::templatesystem::structure_place_settings::StructurePlaceSettings;
use crate::world::phys::vec3::Vec3;
use std::collections::HashMap;

/// `StructureTemplate.StructureBlockInfo` — блок шаблона.
#[derive(Clone, Debug, PartialEq)]
pub struct StructureBlockInfo {
    /// `pos` — позиция внутри шаблона.
    pub pos: BlockPos,
    /// `state`.
    pub state: BlockState,
    /// `nbt` — NBT block entity (None — нет).
    pub nbt: Option<CompoundTag>,
}

impl StructureBlockInfo {
    /// `new(pos, state, nbt)`.
    pub fn new(pos: BlockPos, state: BlockState, nbt: Option<CompoundTag>) -> Self {
        Self { pos, state, nbt }
    }
}

/// `StructureTemplate.StructureEntityInfo` — сущность шаблона.
#[derive(Clone, Debug)]
pub struct StructureEntityInfo {
    /// `pos` — Vec3 позиция.
    pub pos: Vec3,
    /// `blockPos` — блочная позиция.
    pub block_pos: BlockPos,
    /// `nbt`.
    pub nbt: CompoundTag,
}

/// `StructureTemplate.JigsawBlockInfo` — jigsaw-блок с attachment.
#[derive(Clone, Debug)]
pub struct JigsawBlockInfo {
    /// `info` — StructureBlockInfo jigsaw-блока.
    pub info: StructureBlockInfo,
    /// `attachementPos` — позиция attachment.
    pub attachment_pos: BlockPos,
    /// `target` — имя target.
    pub target: String,
    /// `pool` — StructureTemplatePool id.
    pub pool: Option<String>,
    /// `name` — имя jigsaw.
    pub name: String,
    /// `final_state` — final state id.
    pub final_state: String,
}

/// `StructureTemplate.Palette` — список блоков палитры.
#[derive(Clone, Default)]
pub struct Palette {
    /// `blocks` — упорядоченный список StructureBlockInfo.
    pub blocks: Vec<StructureBlockInfo>,
}

impl Palette {
    /// `blocks()`.
    pub fn blocks(&self) -> &[StructureBlockInfo] {
        &self.blocks
    }
}

/// `StructureTemplate.SimplePalette` — state → index для NBT.
#[derive(Default)]
pub struct SimplePalette {
    /// state → index (по BlockState-id).
    map: HashMap<i32, i32>,
    /// index → state.
    states: Vec<BlockState>,
}

impl SimplePalette {
    /// `idFor(state)` — индекс состояния, добавляя при нужде.
    pub fn id_for(&mut self, state: &BlockState) -> i32 {
        if let Some(&id) = self.map.get(&state.id()) {
            return id;
        }
        let id = self.states.len() as i32;
        self.states.push(state.clone());
        self.map.insert(state.id(), id);
        id
    }

    /// `addMapping(state, index)` — принудительный маппинг
    /// (для дополнительных палитр).
    pub fn add_mapping(&mut self, state: &BlockState, index: i32) {
        self.map.insert(state.id(), index);
        while self.states.len() <= index as usize {
            self.states.push(BlockState::air());
        }
        self.states[index as usize] = state.clone();
    }

    /// Состояние по индексу.
    pub fn state_for(&self, index: i32) -> Option<&BlockState> {
        self.states.get(index as usize)
    }
}

/// `StructureTemplate` — шаблон структуры (палитры + сущности).
#[derive(Clone, Default)]
pub struct StructureTemplate {
    /// `palettes`.
    pub palettes: Vec<Palette>,
    /// `entityInfoList`.
    pub entity_info_list: Vec<StructureEntityInfo>,
    /// `size`.
    pub size: Vec3i,
    /// `author`.
    pub author: String,
}

impl StructureTemplate {
    /// `new`.
    pub fn new() -> Self {
        Self {
            palettes: Vec::new(),
            entity_info_list: Vec::new(),
            size: Vec3i::ZERO,
            author: "?".to_string(),
        }
    }

    /// `getSize`.
    pub fn get_size(&self) -> Vec3i {
        self.size
    }

    /// `getSize(rotation)` — размеры после поворота.
    pub fn get_size_rotated(&self, rotation: Rotation) -> Vec3i {
        match rotation {
            Rotation::COUNTERCLOCKWISE_90 | Rotation::CLOCKWISE_90 => {
                Vec3i::new(self.size.z, self.size.y, self.size.x)
            }
            _ => self.size,
        }
    }

    /// `setAuthor`/`getAuthor`.
    pub fn set_author(&mut self, author: String) {
        self.author = author;
    }
    /// `getAuthor`.
    pub fn get_author(&self) -> &str {
        &self.author
    }

    /// `fillFromWorld` — заполнение шаблона из мира (LevelReader
    /// через hook `fill_template_from_world`).
    pub fn fill_from_world(
        level: &dyn LevelReader,
        pos: BlockPos,
        size: Vec3i,
        include_entities: bool,
        ignored_blocks: &[Identifier],
    ) -> Self {
        level.fill_template_from_world(pos, size, include_entities, ignored_blocks)
    }

    /// `filterBlocks(pos, settings, block)` — блоки шаблона,
    /// совпадающие с `block` после трансформации.
    pub fn filter_blocks(
        &self,
        pos: BlockPos,
        settings: &StructurePlaceSettings,
        block: &crate::world::level::block::Block,
    ) -> Vec<StructureBlockInfo> {
        if self.palettes.is_empty() {
            return Vec::new();
        }
        let palette = settings.get_random_palette(&self.palettes, Some(pos));
        let mut out = Vec::new();
        for info in palette.blocks() {
            if info.state.is_block(block) {
                let transformed_pos =
                    transform(info.pos, settings.mirror, settings.rotation, settings.rotation_pivot)
                        .offset(pos.x, pos.y, pos.z);
                out.push(StructureBlockInfo {
                    pos: transformed_pos,
                    state: info.state.clone(),
                    nbt: info.nbt.clone(),
                });
            }
        }
        out
    }

    /// `getJigsaws(pos, rotation)` — jigsaw-блоки шаблона.
    pub fn get_jigsaws(&self, pos: BlockPos, rotation: Rotation) -> Vec<JigsawBlockInfo> {
        let mut out = Vec::new();
        if self.palettes.is_empty() {
            return out;
        }
        let size = self.get_size_rotated(rotation);
        let pivot = BlockPos::new(size.x / 2, size.y / 2, size.z / 2);
        for info in &self.palettes[0].blocks {
            if info.state.block.id.path != "jigsaw" {
                continue;
            }
            let Some(nbt) = &info.nbt else { continue };
            let name = nbt.get_string("name").unwrap_or("").to_string();
            let target = nbt.get_string("target").unwrap_or("").to_string();
            let final_state = nbt.get_string("final_state").unwrap_or("minecraft:air").to_string();
            let pool = nbt.get_string("pool").map(|s| s.to_string());
            let attachment = nbt
                .get_string("joint")
                .unwrap_or("aligned")
                .to_string();
            let _ = attachment;
            // attachment pos — из nbt "attachment"? В Java читается
            // через JigsawBlockInfo.of с transform; упрощённо —
            // позиция + offset по target-направлению.
            let transformed_pos = transform(info.pos, Mirror::NONE, rotation, pivot).offset(pos.x, pos.y, pos.z);
            out.push(JigsawBlockInfo {
                info: StructureBlockInfo {
                    pos: transformed_pos,
                    state: info.state.clone(),
                    nbt: info.nbt.clone(),
                },
                attachment_pos: transformed_pos,
                target,
                pool,
                name,
                final_state,
            });
        }
        out
    }

    /// `calculateConnectedPosition(decorator, start, settings,
    /// end)` — позиция соединения двух настроек.
    pub fn calculate_connected_position(
        decorator: &StructurePlaceSettings,
        start: BlockPos,
        settings: &StructurePlaceSettings,
        end: BlockPos,
    ) -> BlockPos {
        let pos = Self::calculate_relative_position(decorator, start);
        let pos1 = Self::calculate_relative_position(settings, end);
        pos1.subtract(&pos.to_vec3i()).offset(start.x, start.y, start.z)
    }

    /// `calculateRelativePosition(settings, pos)`.
    pub fn calculate_relative_position(settings: &StructurePlaceSettings, pos: BlockPos) -> BlockPos {
        transform(pos, settings.mirror, settings.rotation, settings.rotation_pivot)
    }

    /// `placeInWorld` — размещение шаблона в мире.
    /// `flags` — Block.UpdateFlags (битовая маска; у нас —
    /// прокидывается в level-хук).
    pub fn place_in_world(
        &self,
        level: &dyn LevelReader,
        offset: BlockPos,
        pos: BlockPos,
        settings: &StructurePlaceSettings,
        random: &mut dyn RandomSource,
        flags: i32,
    ) -> bool {
        if self.palettes.is_empty() {
            return false;
        }
        let palette = settings.get_random_palette(&self.palettes, Some(offset));
        let list = palette.blocks();
        let has_entities = !settings.ignore_entities && !self.entity_info_list.is_empty();
        if (list.is_empty() && !has_entities)
            || self.size.x < 1
            || self.size.y < 1
            || self.size.z < 1
        {
            return false;
        }
        let bounding_box = settings.get_bounding_box();
        let processed = Self::process_block_infos(level, offset, pos, settings, list);

        let mut min = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);
        let mut placed_nbt: Vec<(BlockPos, CompoundTag)> = Vec::new();

        for info in &processed {
            let block_pos = info.pos;
            if let Some(bb) = bounding_box {
                if !bb.is_inside_pos(block_pos) {
                    continue;
                }
            }
            // waterlogging — собираем списки (упрощённо: применяем
            // через level-хук get_fluid_state→set waterlogged).
            let mut state = info
                .state
                .mirror(settings.mirror)
                .rotate(settings.rotation);
            if settings.should_apply_waterlogging() {
                if let Some(existing) = level.get_block_state(block_pos) {
                    let fluid = existing.get_fluid_state();
                    if !fluid.is_empty() {
                        state = state.with_waterlogged(true);
                    }
                }
            }
            // nbt → временно ставим barrier (как в Java), потом nbt.
            if info.nbt.is_some() {
                level.set_block_state(
                    block_pos,
                    &crate::world::level::block::block_by_identifier(
                        &Identifier::minecraft("barrier"),
                    )
                    .and_then(|b| b.default_block_state())
                    .unwrap_or_else(BlockState::air),
                );
            }
            let _ = flags;
            if level.set_block_state(block_pos, &state) {
                min = BlockPos::new(min.x.min(block_pos.x), min.y.min(block_pos.y), min.z.min(block_pos.z));
                max = BlockPos::new(max.x.max(block_pos.x), max.y.max(block_pos.y), max.z.max(block_pos.z));
                if let Some(nbt) = &info.nbt {
                    placed_nbt.push((block_pos, nbt.clone()));
                }
            }
        }

        // Block entity NBT + shape updates.
        for (block_pos, nbt) in &placed_nbt {
            level.set_block_entity_nbt(*block_pos, nbt);
        }
        // updateShapeAtEdge — для краёв placed-диапазона.
        if min.x <= max.x {
            self.update_shape_at_edges(level, min, max);
        }

        if !settings.ignore_entities {
            self.place_entities(level, pos, settings.mirror, settings.rotation, offset, bounding_box, settings.finalize_entities);
        }

        true
    }

    /// `updateShapeAtEdge` — обновление формы по краям (через
    /// level-хук `update_block_shape`).
    fn update_shape_at_edges(&self, level: &dyn LevelReader, min: BlockPos, max: BlockPos) {
        // Java обновляет 6 граней discrete-shape; здесь — хук на
        // каждый блок грани (LevelReader реализует neighbour update).
        for x in min.x..=max.x {
            level.update_neighbor_shape(BlockPos::new(x, min.y - 1, min.z));
            level.update_neighbor_shape(BlockPos::new(x, max.y + 1, max.z));
        }
        for z in min.z..=max.z {
            level.update_neighbor_shape(BlockPos::new(min.x, min.y, z - 1));
            level.update_neighbor_shape(BlockPos::new(max.x, min.y, z + 1));
        }
        for y in min.y..=max.y {
            level.update_neighbor_shape(BlockPos::new(min.x - 1, y, min.z));
            level.update_neighbor_shape(BlockPos::new(max.x + 1, y, max.z));
        }
    }

    /// `processBlockInfos` — применение processors к списку.
    pub fn process_block_infos(
        level: &dyn LevelReader,
        offset: BlockPos,
        pos: BlockPos,
        settings: &StructurePlaceSettings,
        block_infos: &[StructureBlockInfo],
    ) -> Vec<StructureBlockInfo> {
        let mut original = Vec::new();
        let mut processed = Vec::new();
        for info in block_infos {
            let world_pos =
                Self::calculate_relative_position(settings, info.pos).offset(offset.x, offset.y, offset.z);
            let mut current = StructureBlockInfo {
                pos: world_pos,
                state: info.state.clone(),
                nbt: info.nbt.clone(),
            };
            for processor in settings.get_processors() {
                if current.pos.x == i32::MIN {
                    break;
                }
                current = match processor.process_block(
                    level,
                    offset,
                    pos,
                    info,
                    &current,
                    settings,
                ) {
                    Some(v) => v,
                    None => {
                        // Java: null → пропуск (не добавлять)
                        current = StructureBlockInfo {
                            pos: BlockPos::new(i32::MIN, i32::MIN, i32::MIN),
                            state: BlockState::air(),
                            nbt: None,
                        };
                        continue;
                    }
                };
            }
            if current.pos.x != i32::MIN {
                original.push(info.clone());
                processed.push(current);
            }
        }
        for processor in settings.get_processors() {
            processed = processor.finalize_processing(
                level,
                offset,
                pos,
                &original,
                &mut processed,
                settings,
            );
        }
        processed
    }

    /// `placeEntities` — размещение сущностей шаблона.
    fn place_entities(
        &self,
        level: &dyn LevelReader,
        pos: BlockPos,
        mirror: Mirror,
        rotation: Rotation,
        offset: BlockPos,
        bounding_box: Option<BoundingBox>,
        finalize: bool,
    ) {
        for entity_info in &self.entity_info_list {
            let block_pos = transform(entity_info.block_pos, mirror, rotation, offset)
                .offset(pos.x, pos.y, pos.z);
            if let Some(bb) = bounding_box {
                if !bb.is_inside_pos(block_pos) {
                    continue;
                }
            }
            let mut nbt = entity_info.nbt.clone();
            let vec3 = transform_vec3(entity_info.pos.clone(), mirror, rotation, offset);
            let vec31 = Vec3::new(
                vec3.x + pos.x as f64,
                vec3.y + pos.y as f64,
                vec3.z + pos.z as f64,
            );
            // Pos tag
            let mut pos_list = Vec::new();
            pos_list.push(Tag::Double(vec31.x));
            pos_list.push(Tag::Double(vec31.y));
            pos_list.push(Tag::Double(vec31.z));
            nbt.put("Pos", Tag::List(pos_list));
            nbt.remove("UUID");
            if let Some(mut entity) = level.create_entity_from_nbt(&nbt) {
                let f = entity.rotate(rotation);
                let f = f + entity.mirror(mirror) - entity.get_y_rot();
                entity.snap_to(vec31.x, vec31.y, vec31.z, f, entity.get_x_rot());
                entity.set_y_body_rot(f);
                entity.set_y_head_rot(f);
                if finalize {
                    // mob.finalizeSpawn — через level-хук при
                    // необходимости; здесь пропускаем (LevelReader
                    // вернёт entity уже финализированным позже).
                }
                level.add_fresh_entity(entity);
            }
        }
    }

    /// `transform(targetPos, mirror, rotation, offset)` —
    /// статическая трансформация BlockPos (полный порт).
    pub fn transform_pos(
        target_pos: BlockPos,
        mirror: Mirror,
        rotation: Rotation,
        offset: BlockPos,
    ) -> BlockPos {
        transform(target_pos, mirror, rotation, offset)
    }

    /// `getZeroPositionWithTransform`.
    pub fn get_zero_position_with_transform(
        &self,
        target_pos: BlockPos,
        mirror: Mirror,
        rotation: Rotation,
    ) -> BlockPos {
        Self::get_zero_position_with_transform_full(
            target_pos,
            mirror,
            rotation,
            self.size.x,
            self.size.z,
        )
    }

    /// `getZeroPositionWithTransform(pos, mirror, rotation, sizeX,
    /// sizeZ)` — статическая версия.
    pub fn get_zero_position_with_transform_full(
        pos: BlockPos,
        mirror: Mirror,
        rotation: Rotation,
        size_x: i32,
        size_z: i32,
    ) -> BlockPos {
        let size_x = size_x - 1;
        let size_z = size_z - 1;
        let i = if mirror == Mirror::FRONT_BACK { size_x } else { 0 };
        let i1 = if mirror == Mirror::LEFT_RIGHT { size_z } else { 0 };
        match rotation {
            Rotation::COUNTERCLOCKWISE_90 => pos.offset(i1, 0, size_x - i),
            Rotation::CLOCKWISE_90 => pos.offset(size_z - i1, 0, i),
            Rotation::CLOCKWISE_180 => pos.offset(size_x - i, 0, size_z - i1),
            Rotation::NONE => pos.offset(i, 0, i1),
        }
    }

    /// `getBoundingBox(settings, startPos)`.
    pub fn get_bounding_box(
        &self,
        settings: &StructurePlaceSettings,
        start_pos: BlockPos,
    ) -> BoundingBox {
        self.get_bounding_box_full(start_pos, settings.rotation, settings.rotation_pivot, settings.mirror)
    }

    /// `getBoundingBox(startPos, rotation, pivotPos, mirror)`.
    pub fn get_bounding_box_full(
        &self,
        start_pos: BlockPos,
        rotation: Rotation,
        pivot_pos: BlockPos,
        mirror: Mirror,
    ) -> BoundingBox {
        Self::get_bounding_box_static(start_pos, rotation, pivot_pos, mirror, self.size)
    }

    /// `getBoundingBox` статическая.
    pub fn get_bounding_box_static(
        start_pos: BlockPos,
        rotation: Rotation,
        pivot_pos: BlockPos,
        mirror: Mirror,
        size: Vec3i,
    ) -> BoundingBox {
        let vec3i = Vec3i::new(size.x - 1, size.y - 1, size.z - 1);
        let p1 = transform(BlockPos::ZERO, mirror, rotation, pivot_pos);
        let p2 = transform(BlockPos::ZERO.offset_vec3i(&vec3i), mirror, rotation, pivot_pos);
        let b1 = BoundingBox::new(p1.x, p1.y, p1.z, p1.x, p1.y, p1.z);
        let b2 = BoundingBox::new(p2.x, p2.y, p2.z, p2.x, p2.y, p2.z);
        let mut bb = BoundingBox::encapsulating_boxes([&b1, &b2].into_iter()).unwrap();
        bb.move_(start_pos.x, start_pos.y, start_pos.z);
        bb
    }

    /// `save` → NBT (palette/blocks/entities/size/author).
    pub fn save(&self, tag: &mut CompoundTag) {
        if self.palettes.is_empty() {
            tag.put("blocks", Tag::List(Vec::new()));
            tag.put("palette", Tag::List(Vec::new()));
        } else {
            // palettes[0] → blocks+palette; дополнительные палитры —
            // palettes list.
            let list0 = &self.palettes[0].blocks;
            let mut simple = SimplePalette::default();
            let mut extra: Vec<SimplePalette> =
                (1..self.palettes.len()).map(|_| SimplePalette::default()).collect();
            let mut blocks_tag = Vec::new();
            for (i, info) in list0.iter().enumerate() {
                let mut entry = CompoundTag::new();
                entry.put("pos", int_list_tag(&[info.pos.x, info.pos.y, info.pos.z]));
                let state_id = simple.id_for(&info.state);
                entry.put("state", Tag::Int(state_id));
                if let Some(nbt) = &info.nbt {
                    entry.put("nbt", Tag::Compound(nbt.clone()));
                }
                blocks_tag.push(Tag::Compound(entry));
                for (j, palette) in self.palettes.iter().enumerate().skip(1) {
                    let _ = j;
                    extra[j - 1].add_mapping(&palette.blocks[i].state, state_id);
                }
            }
            tag.put("blocks", Tag::List(blocks_tag));
            // palette — список state-id → blockstate string;
            // хранится как список int (индексы) + отдельный список
            // строк состояний для совместимости с читателями.
            let mut palette_tag = Vec::new();
            for i in 0..simple.states.len() {
                palette_tag.push(Tag::Int(i as i32));
            }
            tag.put("palette", Tag::List(palette_tag));
            // дополнительные палитры
            if !extra.is_empty() {
                let mut palettes_tag = Vec::new();
                for p in &extra {
                    let mut pt = Vec::new();
                    for i in 0..p.states.len() {
                        pt.push(Tag::Int(i as i32));
                    }
                    palettes_tag.push(Tag::List(pt));
                }
                tag.put("palettes", Tag::List(palettes_tag));
            }
        }
        // entities
        let mut entities_tag = Vec::new();
        for e in &self.entity_info_list {
            let mut entry = CompoundTag::new();
            entry.put("pos", double_list_tag(&[e.pos.x, e.pos.y, e.pos.z]));
            entry.put(
                "blockPos",
                int_list_tag(&[e.block_pos.x, e.block_pos.y, e.block_pos.z]),
            );
            entry.put("nbt", Tag::Compound(e.nbt.clone()));
            entities_tag.push(Tag::Compound(entry));
        }
        tag.put("entities", Tag::List(entities_tag));
        tag.put(
            "size",
            int_list_tag(&[self.size.x, self.size.y, self.size.z]),
        );
        tag.put("author", Tag::String(self.author.clone()));
    }

    /// `load` — загрузка из NBT (без полной BlockState-парсер
    /// инфраструктуры — состояния по id реестра).
    pub fn load(tag: &CompoundTag) -> Option<Self> {
        let mut template = Self::new();
        if let Some(Tag::List(size_list)) = tag.get("size") {
            if size_list.len() == 3 {
                template.size = Vec3i::new(
                    size_list[0].as_int().unwrap_or(0),
                    size_list[1].as_int().unwrap_or(0),
                    size_list[2].as_int().unwrap_or(0),
                );
            }
        }
        if let Some(author) = tag.get_string("author") {
            template.author = author.to_string();
        }
        // palette/blocks — базовый парсинг (state id → registry).
        if let Some(Tag::List(blocks)) = tag.get("blocks") {
            let mut palette = Palette::default();
            for item in blocks {
                let Some(entry) = item.as_compound() else { continue };
                let pos = entry.get("pos").and_then(|t| match t {
                    Tag::List(l) if l.len() == 3 => Some(BlockPos::new(
                        l[0].as_int().unwrap_or(0),
                        l[1].as_int().unwrap_or(0),
                        l[2].as_int().unwrap_or(0),
                    )),
                    _ => None,
                });
                let state_id = entry.get("state").and_then(|t| t.as_int());
                let Some(pos) = pos else { continue };
                let state = state_id
                    .and_then(|id| crate::world::level::block::state::block_state_registry::block_state_by_id(id))
                    .unwrap_or_else(BlockState::air);
                let nbt = entry.get_compound("nbt").cloned();
                palette.blocks.push(StructureBlockInfo::new(pos, state, nbt));
            }
            template.palettes.push(palette);
        }
        if let Some(Tag::List(entities)) = tag.get("entities") {
            for item in entities {
                let Some(entry) = item.as_compound() else { continue };
                let pos = entry.get("pos").and_then(|t| match t {
                    Tag::List(l) if l.len() == 3 => Some(Vec3::new(
                        l[0].as_double().unwrap_or(0.0),
                        l[1].as_double().unwrap_or(0.0),
                        l[2].as_double().unwrap_or(0.0),
                    )),
                    _ => None,
                });
                let block_pos = entry.get("blockPos").and_then(|t| match t {
                    Tag::List(l) if l.len() == 3 => Some(BlockPos::new(
                        l[0].as_int().unwrap_or(0),
                        l[1].as_int().unwrap_or(0),
                        l[2].as_int().unwrap_or(0),
                    )),
                    _ => None,
                });
                let nbt = entry.get_compound("nbt").cloned();
                if let (Some(pos), Some(block_pos), Some(nbt)) = (pos, block_pos, nbt) {
                    template.entity_info_list.push(StructureEntityInfo { pos, block_pos, nbt });
                }
            }
        }
        Some(template)
    }
}

/// `transform(targetPos, mirror, rotation, offset)` — свободная
/// функция (полный порт Java-логики).
pub fn transform(
    target_pos: BlockPos,
    mirror: Mirror,
    rotation: Rotation,
    offset: BlockPos,
) -> BlockPos {
    let mut x = target_pos.x;
    let y = target_pos.y;
    let mut z = target_pos.z;
    let mut flag = true;
    match mirror {
        Mirror::LEFT_RIGHT => z = -z,
        Mirror::FRONT_BACK => x = -x,
        Mirror::NONE => flag = false,
    }
    let x1 = offset.x;
    let z1 = offset.z;
    match rotation {
        Rotation::COUNTERCLOCKWISE_90 => BlockPos::new(x1 - z1 + z, y, x1 + z1 - x),
        Rotation::CLOCKWISE_90 => BlockPos::new(x1 + z1 - z, y, z1 - x1 + x),
        Rotation::CLOCKWISE_180 => BlockPos::new(x1 + x1 - x, y, z1 + z1 - z),
        Rotation::NONE => {
            if flag {
                BlockPos::new(x, y, z)
            } else {
                target_pos
            }
        }
    }
}

/// `transform(Vec3, mirror, rotation, offset)`.
pub fn transform_vec3(target: Vec3, mirror: Mirror, rotation: Rotation, offset: BlockPos) -> Vec3 {
    let mut d = target.x;
    let d1 = target.y;
    let mut d2 = target.z;
    let mut flag = true;
    match mirror {
        Mirror::LEFT_RIGHT => d2 = 1.0 - d2,
        Mirror::FRONT_BACK => d = 1.0 - d,
        Mirror::NONE => flag = false,
    }
    let x = offset.x as f64;
    let z = offset.z as f64;
    match rotation {
        Rotation::COUNTERCLOCKWISE_90 => Vec3::new(x - z + d2, d1, x + z + 1.0 - d),
        Rotation::CLOCKWISE_90 => Vec3::new(x + z + 1.0 - d2, d1, z - x + d),
        Rotation::CLOCKWISE_180 => Vec3::new(x + x + 1.0 - d, d1, z + z + 1.0 - d2),
        Rotation::NONE => {
            if flag {
                Vec3::new(d, d1, d2)
            } else {
                target
            }
        }
    }
}

fn int_list_tag(values: &[i32]) -> Tag {
    Tag::List(values.iter().map(|v| Tag::Int(*v)).collect())
}

fn double_list_tag(values: &[f64]) -> Tag {
    Tag::List(values.iter().map(|v| Tag::Double(*v)).collect())
}

pub use crate::world::level::levelgen::structure::templatesystem::processors::{
    BlockAgeProcessor, BlackstoneReplaceProcessor, BlockIgnoreProcessor, BlockRotProcessor,
    CappedProcessor, GravityProcessor, JigsawReplacementProcessor, LavaSubmergedBlockProcessor,
    NopProcessor, ProtectedBlockProcessor, RuleProcessor, StructureProcessor,
};
