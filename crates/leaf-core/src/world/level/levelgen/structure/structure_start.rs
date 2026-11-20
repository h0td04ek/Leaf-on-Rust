//! Перенос `net.minecraft.world.level.levelgen.structure.StructureStart`
//! + `StructureCheckResult`.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::pieces_container::PiecesContainer;
use crate::world::level::levelgen::structure::structure::Structure;
use crate::world::level::levelgen::structure::structure_piece::StructurePiece;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use std::sync::Arc;

pub use crate::world::level::levelgen::structure::structure_check_result::StructureCheckResult;

/// `StructureStart.INVALID_START_ID`.
pub const INVALID_START_ID: &str = "INVALID";

/// `StructureStart` — старт структуры в чанке.
pub struct StructureStart {
    /// `structure` (None только у INVALID_START).
    pub structure: Option<Arc<Structure>>,
    /// `pieceContainer`.
    pub piece_container: PiecesContainer,
    /// `chunkPos` — чанк, в котором старт сгенерирован.
    pub chunk_pos: ChunkPos,
    /// `references` — счётчик ссылок.
    pub references: i32,
    /// `cachedBoundingBox`.
    cached_bounding_box: Option<BoundingBox>,
}

impl StructureStart {
    /// `new(structure, chunkPos, references, pieceContainer)`.
    pub fn new(
        structure: Option<Arc<Structure>>,
        chunk_pos: ChunkPos,
        references: i32,
        piece_container: PiecesContainer,
    ) -> Self {
        Self {
            structure,
            piece_container,
            chunk_pos,
            references,
            cached_bounding_box: None,
        }
    }

    /// `cachedBoundingBox` — доступ к кэшу (для shallow_clone).
    pub fn cached_bounding_box_field(&self) -> Option<BoundingBox> {
        self.cached_bounding_box
    }

    /// shallow-копия без кусков (для чтения метаданных; pieces
    /// остаются за владельцем — используется менеджером для
    /// списков ссылок).
    pub fn shallow_clone(&self) -> Self {
        Self {
            structure: self.structure.clone(),
            piece_container: PiecesContainer::default(),
            chunk_pos: self.chunk_pos,
            references: self.references,
            cached_bounding_box: self.cached_bounding_box,
        }
    }

    /// INVALID_START — невалидный старт.
    pub fn invalid_start() -> Self {
        Self::new(None, ChunkPos::of(0), 0, PiecesContainer::default())
    }

    /// `isValid`.
    pub fn is_valid(&self) -> bool {
        !self.piece_container.is_empty()
    }

    /// `getChunkPos`.
    pub fn get_chunk_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    /// `getStructure`.
    pub fn get_structure(&self) -> Option<&Arc<Structure>> {
        self.structure.as_ref()
    }

    /// `getPieces`.
    pub fn get_pieces(&self) -> &[Box<dyn StructurePiece>] {
        self.piece_container.pieces()
    }

    /// `getBoundingBox` — `structure.adjustBoundingBox(
    /// pieces.calculateBoundingBox())` с кэшем.
    pub fn get_bounding_box(&mut self) -> BoundingBox {
        if let Some(bb) = self.cached_bounding_box {
            return bb;
        }
        let bb = self
            .piece_container
            .calculate_bounding_box()
            .unwrap_or_else(BoundingBox::infinite);
        let bb = self
            .structure
            .as_ref()
            .map(|s| s.adjust_bounding_box(bb))
            .unwrap_or(bb);
        self.cached_bounding_box = Some(bb);
        bb
    }

    /// read-only вариант `getBoundingBox` (для &self-контекстов).
    pub fn bounding_box(&mut self) -> BoundingBox {
        self.get_bounding_box()
    }

    /// `canBeReferenced` — `references < maxReferences`.
    pub fn can_be_referenced(&self) -> bool {
        self.references < self.get_max_references()
    }

    /// `getMaxReferences` — дефолт 1.
    pub fn get_max_references(&self) -> i32 {
        1
    }

    /// `addReference`.
    pub fn add_reference(&mut self) {
        self.references += 1;
    }

    /// `getReferences`.
    pub fn get_references(&self) -> i32 {
        self.references
    }

    /// `placeInChunk` — пост-обработка кусков в чанке.
    /// `placeInChunk(level, structureManager, generator, random,
    /// box, chunkPos)`.
    pub fn place_in_chunk(
        &mut self,
        level: &dyn LevelReader,
        structure_manager: &dyn StructureManagerAccess,
        generator: Option<&dyn crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        chunk_pos: ChunkPos,
    ) {
        if self.piece_container.is_empty() {
            return;
        }
        let pieces = self.piece_container.pieces();
        if pieces.is_empty() {
            return;
        }
        let first_bb = pieces[0].bounding_box();
        let center = first_bb.get_center();
        let block_pos = BlockPos::new(center.x, first_bb.min_y, center.z);

        // Собираем индексы кусков, пересекающих box (избегаем
        // одновременного &mut self + итерации по полю).
        let mut indices = Vec::new();
        for (idx, piece) in pieces.iter().enumerate() {
            if piece.bounding_box().intersects(box_) {
                indices.push(idx);
            }
        }
        for idx in indices {
            // post_process требует &mut piece — PiecesContainer
            // хранит Box<dyn StructurePiece>; безопасный доступ
            // через split на raw-указатели не нужен: итерируем
            // заново по индексу с unsafe-обёрткой в реальном
            // коде; здесь — через `pieces_mut` метод контейнера.
            self.piece_container.post_process_at(
                idx,
                level,
                structure_manager,
                generator,
                random,
                box_,
                chunk_pos,
                block_pos,
            );
        }
        if let Some(structure) = self.structure.clone() {
            structure.after_place(
                level,
                structure_manager,
                random,
                box_,
                chunk_pos,
                &self.piece_container,
            );
        }
    }

    /// `createTag(context, chunkPos)` — NBT сохранение.
    pub fn create_tag(&self, chunk_pos: ChunkPos) -> CompoundTag {
        let mut tag = CompoundTag::new();
        if self.is_valid() {
            if let Some(structure) = &self.structure {
                tag.put("id", Tag::String(structure.id.to_string()));
            } else {
                tag.put("id", Tag::String(INVALID_START_ID.to_string()));
            }
            tag.put("ChunkX", Tag::Int(chunk_pos.x));
            tag.put("ChunkZ", Tag::Int(chunk_pos.z));
            tag.put("references", Tag::Int(self.references));
            tag.put("Children", self.piece_container.save());
        } else {
            tag.put("id", Tag::String(INVALID_START_ID.to_string()));
        }
        tag
    }

    /// `loadStaticStart(context, tag, seed)` — десериализация.
    pub fn load_static_start(tag: &CompoundTag, _seed: i64) -> Option<Self> {
        let id_str = tag.get_string("id").unwrap_or("");
        if id_str == INVALID_START_ID {
            return Some(Self::invalid_start());
        }
        let id = Identifier::parse(id_str).ok()?;
        let structure = crate::world::level::levelgen::structure::structure::structure_by_identifier(&id)?;
        let chunk_x = tag.get_int("ChunkX").unwrap_or(0);
        let chunk_z = tag.get_int("ChunkZ").unwrap_or(0);
        let references = tag.get_int("references").unwrap_or(0);
        let children = tag.get_list("Children")?;
        let piece_container = PiecesContainer::load(children)?;
        Some(Self::new(
            Some(Arc::new(structure)),
            ChunkPos::new(chunk_x, chunk_z),
            references,
            piece_container,
        ))
    }
}
