//! Перенос `net.minecraft.world.level.levelgen.structure.pieces` —
//! `PiecesContainer` и `StructurePieceType`-реестр.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use super::bounding_box::BoundingBox;
use super::structure_piece::StructurePiece;

/// `PiecesContainer` — контейнер частей StructureStart.
#[derive(Default)]
pub struct PiecesContainer {
    /// `pieces`.
    pub pieces: Vec<Box<dyn StructurePiece>>,
}

impl PiecesContainer {
    /// `new(List)`.
    pub fn new(pieces: Vec<Box<dyn StructurePiece>>) -> Self {
        Self { pieces }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// `pieces()`.
    pub fn pieces(&self) -> &[Box<dyn StructurePiece>] {
        &self.pieces
    }

    /// `calculateBoundingBox` — объединяющий бокс всех частей.
    pub fn calculate_bounding_box(&self) -> Option<BoundingBox> {
        if self.pieces.is_empty() {
            return None;
        }
        let mut boxes = Vec::with_capacity(self.pieces.len());
        for p in &self.pieces {
            boxes.push(p.bounding_box());
        }
        BoundingBox::encapsulating_boxes(boxes.iter())
    }

    /// `save(context)` — NBT `Children` list (id+BB+O+GD+
    /// additional data каждой части). Возвращает `Tag::List`.
    pub fn save(&self) -> Tag {
        let mut list = Vec::with_capacity(self.pieces.len());
        for piece in &self.pieces {
            let mut tag = CompoundTag::new();
            tag.put("id", Tag::String(piece.piece_type_id().to_string()));
            let bb = piece.bounding_box();
            let mut bb_tag = CompoundTag::new();
            bb_tag.put("minX", Tag::Int(bb.min_x));
            bb_tag.put("minY", Tag::Int(bb.min_y));
            bb_tag.put("minZ", Tag::Int(bb.min_z));
            bb_tag.put("maxX", Tag::Int(bb.max_x));
            bb_tag.put("maxY", Tag::Int(bb.max_y));
            bb_tag.put("maxZ", Tag::Int(bb.max_z));
            tag.put("BB", Tag::Compound(bb_tag));
            tag.put("O", Tag::Int(piece.orientation().map(|d| d.get_2d_data_value()).unwrap_or(-1)));
            tag.put("GD", Tag::Int(piece.gen_depth()));
            piece.add_additional_save_data(&mut tag);
            list.push(Tag::Compound(tag));
        }
        Tag::List(list)
    }

    /// `load(list, context)` — десериализация через реестр типов
    /// `StructurePieceType` (piece_from_nbt). Принимает `Tag::List`
    /// или `&Vec<Tag>`.
    pub fn load(items: &[Tag]) -> Option<Self> {
        let mut pieces: Vec<Box<dyn StructurePiece>> = Vec::new();
        for item in items {
            let Some(tag) = item.as_compound() else { continue };
            let Some(id_str) = tag.get_string("id") else { continue };
            let Ok(id) = Identifier::parse(id_str) else { continue };
            let Some(piece) = piece_from_nbt(&id, tag) else {
                continue;
            };
            pieces.push(piece);
        }
        Some(Self { pieces })
    }
}

/// `StructurePieceType` — реестр типов частей структуры:
/// id → `fn(CompoundTag) -> Option<Box<dyn StructurePiece>>`.
pub type PieceFactory = Arc<dyn Fn(&CompoundTag) -> Option<Box<dyn StructurePiece>> + Send + Sync>;

fn piece_registry() -> &'static RwLock<HashMap<Identifier, PieceFactory>> {
    static REG: OnceLock<RwLock<HashMap<Identifier, PieceFactory>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Регистрация типа части (`StructurePieceType` static init).
pub fn register_piece_type(id: Identifier, factory: PieceFactory) {
    piece_registry().write().unwrap().insert(id, factory);
}

/// `piece_from_nbt` — создать часть по NBT через реестр.
pub fn piece_from_nbt(id: &Identifier, tag: &CompoundTag) -> Option<Box<dyn StructurePiece>> {
    let factory = piece_registry().read().unwrap().get(id).cloned()?;
    factory(tag)
}

/// Чтение `BB` из тега (compound-форма, см. `save`).
pub fn bounding_box_from_tag(tag: &CompoundTag) -> Option<BoundingBox> {
    let bb = tag.get_compound("BB")?;
    Some(BoundingBox::new(
        bb.get_int("minX")?,
        bb.get_int("minY")?,
        bb.get_int("minZ")?,
        bb.get_int("maxX")?,
        bb.get_int("maxY")?,
        bb.get_int("maxZ")?,
    ))
}

impl PiecesContainer {
    /// `postProcess` по индексу куска — безопасный доступ к
    /// `&mut` элементу через `get_unchecked_mut` не нужен: Vec
    /// даёт `get_mut`.
    pub fn post_process_at(
        &mut self,
        index: usize,
        level: &dyn crate::world::level::level_reader::LevelReader,
        structure_manager: &dyn crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess,
        random: &mut dyn crate::random::RandomSource,
        box_: &BoundingBox,
        chunk_pos: crate::world::level::chunk_pos::ChunkPos,
        pos: crate::core::block_pos::BlockPos,
    ) {
        if let Some(piece) = self.pieces.get_mut(index) {
            piece.post_process(level, structure_manager, random, box_, chunk_pos, pos);
        }
    }
}
