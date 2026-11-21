//! Перенос `net.minecraft.world.level.levelgen.structure.pieces` —
//! `PiecesContainer` и `StructurePieceType`-реестр.

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
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

    /// `isInsidePiece(pos)` — позиция внутри bounding box хотя бы
    /// одного куска.
    pub fn is_inside_piece(&self, pos: BlockPos) -> bool {
        self.pieces
            .iter()
            .any(|p| p.bounding_box().is_inside_pos(pos))
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
            tag.put_int_array(
                "BB",
                &[bb.min_x, bb.min_y, bb.min_z, bb.max_x, bb.max_y, bb.max_z],
            );
            tag.put(
                "O",
                Tag::Int(
                    piece
                        .orientation()
                        .map(|d| d.get_2d_data_value())
                        .unwrap_or(-1),
                ),
            );
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
            let Some(tag) = item.as_compound() else {
                continue;
            };
            let Some(id_str) = tag.get_string("id") else {
                continue;
            };
            let Ok(id) = Identifier::parse(&id_str.to_ascii_lowercase()) else {
                continue;
            };
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
    let factory = piece_registry().read().unwrap().get(id).cloned();
    if let Some(factory) = factory {
        return factory(tag);
    }
    if id.get_namespace() != "minecraft" {
        return None;
    }

    use super::structures::{
        buried_treasure::BuriedTreasurePiece,
        desert_pyramid::DesertPyramidPiece,
        jungle_temple::JungleTemplePiece,
        mineshaft::{
            MineShaftCorridorPiece, MineShaftCrossingPiece, MineShaftRoomPiece,
            MineShaftStairsPiece,
        },
        swamp_hut::SwampHutPiece,
    };
    match id.get_path() {
        "btp" | "buried_treasure" => Some(Box::new(BuriedTreasurePiece::from_nbt(tag)?)),
        "tedp" | "desert_pyramid" => Some(Box::new(DesertPyramidPiece::from_nbt(tag)?)),
        "tejp" | "jungle_pyramid" => Some(Box::new(JungleTemplePiece::from_nbt(tag)?)),
        "tesh" | "swamp_hut" => Some(Box::new(SwampHutPiece::from_nbt(tag)?)),
        "mscorridor" => Some(Box::new(MineShaftCorridorPiece::from_nbt(tag)?)),
        "mscrossing" => Some(Box::new(MineShaftCrossingPiece::from_nbt(tag)?)),
        "msroom" => Some(Box::new(MineShaftRoomPiece::from_nbt(tag)?)),
        "msstairs" => Some(Box::new(MineShaftStairsPiece::from_nbt(tag)?)),
        _ => None,
    }
}

pub(crate) struct PieceHeader {
    pub bounds: BoundingBox,
    pub orientation: Option<Direction>,
    pub depth: i32,
}

impl PieceHeader {
    pub(crate) fn read(tag: &CompoundTag) -> Option<Self> {
        let orientation = if let Some(name) = tag.get_string("O") {
            Direction::by_name(name)
        } else {
            let direction = tag.get_int_or("O", 0);
            (direction != -1).then(|| Direction::from_2d_data_value(direction))
        };
        Some(Self {
            bounds: bounding_box_from_tag(tag)?,
            orientation,
            depth: tag.get_int_or("GD", 0),
        })
    }
}

/// Сохранения Leaf используют массив из шести координат. Ранний Rust-код
/// записывал compound с именованными координатами, его тоже читаем.
pub fn bounding_box_from_tag(tag: &CompoundTag) -> Option<BoundingBox> {
    if let Some(values) = tag.get_int_array("BB") {
        let &[x0, y0, z0, x1, y1, z1] = values else {
            return None;
        };
        return Some(BoundingBox::new(x0, y0, z0, x1, y1, z1));
    }
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
        generator: Option<&dyn crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess>,
        random: &mut dyn crate::random::RandomSource,
        box_: &BoundingBox,
        chunk_pos: crate::world::level::chunk_pos::ChunkPos,
        pos: crate::core::block_pos::BlockPos,
    ) {
        if let Some(piece) = self.pieces.get_mut(index) {
            piece.post_process(
                level,
                structure_manager,
                generator,
                random,
                box_,
                chunk_pos,
                pos,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_leaf_piece_ids_headers_and_type_specific_fields() {
        let coords = [-42, -12, 73, -22, 4, 99];
        let cases = [
            ("BTP", "minecraft:btp", None, -1),
            (
                "minecraft:TeDP",
                "minecraft:tedp",
                Some("hasPlacedChest1"),
                1,
            ),
            ("minecraft:tejp", "minecraft:tejp", Some("placedTrap2"), 2),
            ("minecraft:tesh", "minecraft:tesh", Some("Cat"), 3),
        ];
        for (id, canonical, flag, orientation) in cases {
            let mut input = CompoundTag::new();
            input.put_string("id", id);
            input.put_int_array("BB", &coords);
            input.put_int("O", orientation);
            input.put_int("GD", 7);
            if let Some(flag) = flag {
                input.put_bool(flag, true);
            }
            let container = PiecesContainer::load(&[Tag::Compound(input)]).unwrap();
            let piece = &container.pieces()[0];
            assert_eq!(
                piece.bounding_box(),
                BoundingBox::new(-42, -12, 73, -22, 4, 99)
            );
            assert_eq!(piece.gen_depth(), 7);
            assert_eq!(
                piece.orientation().map(|d| d.get_2d_data_value()),
                (orientation >= 0).then_some(orientation)
            );
            let Tag::List(pieces) = container.save() else {
                panic!("expected piece list")
            };
            let saved = pieces[0].as_compound().unwrap();
            assert_eq!(saved.get_string("id"), Some(canonical));
            assert_eq!(saved.get_int_array("BB"), Some(coords.as_slice()));
            assert_eq!(saved.get_int("O"), Some(orientation));
            if let Some(flag) = flag {
                assert_eq!(saved.get_bool(flag), Some(true));
            }
        }
    }

    #[test]
    fn reads_previous_rust_piece_encoding_and_rejects_malformed_boxes() {
        let mut piece = CompoundTag::new();
        piece.put_string("id", "minecraft:desert_pyramid");
        piece.put_string("O", "west");
        let mut bounds = CompoundTag::new();
        for (key, value) in ["minX", "minY", "minZ", "maxX", "maxY", "maxZ"]
            .into_iter()
            .zip([32, 63, 45, 52, 79, 65])
        {
            bounds.put_int(key, value);
        }
        piece.put("BB", Tag::Compound(bounds));
        let input = PiecesContainer::load(&[Tag::Compound(piece.clone())]).unwrap();
        assert_eq!(input.pieces()[0].orientation(), Some(Direction::WEST));
        assert_eq!(
            input.pieces()[0].bounding_box(),
            BoundingBox::new(32, 63, 45, 52, 79, 65)
        );

        piece.put_int_array("BB", &[1, 2]);
        let input = PiecesContainer::load(&[Tag::Compound(piece)]).unwrap();
        assert!(input.is_empty());
    }

    #[test]
    fn loads_and_serializes_all_mineshaft_pieces_with_saved_state() {
        let coords = [-41, 28, 12, -30, 34, 24];
        let entrance = vec![-31, 28, 12, -30, 30, 14];
        let entries = [
            ("MSCorridor", "minecraft:mscorridor", 2),
            ("MSCrossing", "minecraft:mscrossing", -1),
            ("MSRoom", "minecraft:msroom", -1),
            ("MSStairs", "minecraft:msstairs", 1),
        ];
        let pieces = entries
            .iter()
            .map(|(id, _, orientation)| {
                let mut tag = CompoundTag::new();
                tag.put_string("id", id);
                tag.put_int("O", *orientation);
                tag.put_int("GD", 6);
                tag.put_int_array("BB", &coords);
                tag.put_int("MST", 1);
                match *id {
                    "MSCorridor" => {
                        tag.put_bool("hr", false);
                        tag.put_bool("sc", true);
                        tag.put_bool("hps", true);
                        tag.put_int("Num", 2);
                    }
                    "MSCrossing" => {
                        tag.put_bool("tf", true);
                        tag.put_int("D", 3);
                    }
                    "MSRoom" => {
                        tag.put(
                            "Entrances",
                            Tag::List(vec![Tag::IntArray(entrance.clone())]),
                        );
                    }
                    _ => {}
                }
                Tag::Compound(tag)
            })
            .collect::<Vec<_>>();
        let mut container = PiecesContainer::load(&pieces).unwrap();
        assert_eq!(container.pieces().len(), pieces.len());
        container.pieces[2].load_additional_save_data(pieces[2].as_compound().unwrap());
        let Tag::List(saved) = container.save() else {
            panic!("expected piece list")
        };
        for (index, (_, id, orientation)) in entries.into_iter().enumerate() {
            let data = saved[index].as_compound().unwrap();
            assert_eq!(data.get_string("id"), Some(id));
            assert_eq!(data.get_int("O"), Some(orientation));
            assert_eq!(data.get_int_array("BB"), Some(coords.as_slice()));
            assert_eq!(data.get_int("GD"), Some(6));
            assert_eq!(data.get_int("MST"), Some(1));
            let original = pieces[index].as_compound().unwrap();
            for key in ["hr", "sc", "hps", "Num", "tf", "D", "Entrances"] {
                assert_eq!(data.get(key), original.get(key), "{id} field {key}");
            }
        }
    }
}
