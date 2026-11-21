//! Разрешение палитр Anvil в состояния блоков и ID биомов измерения.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::core::holder::Holder;
use crate::core::registry::{registries, Registry};
use crate::core::registry_access::RegistryAccess;
use crate::nbt::CompoundTag;
use crate::resources::{Identifier, ResourceKey};
use crate::world::level::biome::biome::Biome;
use crate::world::level::block::state::BlockState;
use crate::world::level::chunk::palette::{IdMap, SharedIdMap, Strategy, StrategyKind};
use crate::world::level::chunk::{LevelChunkSection, PalettedContainer};

use super::serializable_chunk_data::ChunkPaletteLookup;

pub struct RegistryChunkPalettes {
    default_states: HashMap<Identifier, BlockState>,
    block_ids: SharedIdMap<BlockState>,
    biome_ids: Arc<BiomeIdMap>,
    default_biome: Holder<Biome>,
    air: BlockState,
}

impl RegistryChunkPalettes {
    /// Использует карту состояний блоков сетевого протокола и реестр биомов измерения.
    pub fn new(
        registries: &RegistryAccess,
        block_ids: SharedIdMap<BlockState>,
        default_biome: &Identifier,
    ) -> io::Result<Self> {
        let biome_registry = registries
            .lookup(&registries::biome())
            .ok_or_else(|| invalid_data("world biome registry is missing"))?;
        let biome_ids = Arc::new(BiomeIdMap(biome_registry));
        let default_biome = biome_ids
            .get_by_name(default_biome)
            .ok_or_else(|| invalid_data("default biome is unregistered"))?;

        let mut default_states = HashMap::new();
        for id in 0..block_ids.size() as i32 {
            if let Some(state) = block_ids.by_id(id) {
                default_states
                    .entry(state.block.id.clone())
                    .or_insert_with(|| state.block.default_block_state().unwrap_or(state));
            }
        }
        let air = default_states
            .get(&Identifier::minecraft("air"))
            .and_then(|state| block_ids.by_id(block_ids.get_id(state)))
            .ok_or_else(|| invalid_data("air block state is unregistered"))?;

        Ok(Self {
            default_states,
            block_ids,
            biome_ids,
            default_biome,
            air,
        })
    }
}

impl ChunkPaletteLookup for RegistryChunkPalettes {
    fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState> {
        let id = Identifier::parse(nbt.get_string("Name")?).ok()?;
        let mut state = self.default_states.get(&id)?.clone();
        if let Some(properties) = nbt.get("Properties") {
            for (name, value) in properties.as_compound()?.iter() {
                let property = state.definition.get_property(name)?;
                state
                    .values
                    .set_erased(name, property.parse(value.as_string()?)?);
            }
        }
        self.block_ids.by_id(self.block_ids.get_id(&state))
    }

    fn biome(&self, id: &Identifier) -> Option<Holder<Biome>> {
        self.biome_ids.get_by_name(id)
    }

    fn default_biome(&self) -> Holder<Biome> {
        self.default_biome.clone()
    }

    fn create_section(&self) -> LevelChunkSection {
        LevelChunkSection::new(
            PalettedContainer::new(
                self.air.clone(),
                Strategy::new(StrategyKind::BlockStates, self.block_ids.size()),
                Some(self.block_ids.clone()),
            ),
            PalettedContainer::new(
                self.default_biome(),
                Strategy::new(StrategyKind::Biomes, self.biome_ids.size()),
                Some(self.biome_ids.clone()),
            ),
        )
    }
}

struct BiomeIdMap(Arc<Registry<Biome>>);

impl BiomeIdMap {
    fn get_by_name(&self, id: &Identifier) -> Option<Holder<Biome>> {
        self.0.get_holder(&ResourceKey::create_with_registry_name(
            registries::biome(),
            id.clone(),
        ))
    }
}

impl IdMap<Holder<Biome>> for BiomeIdMap {
    fn by_id(&self, id: i32) -> Option<Holder<Biome>> {
        self.0.get_holder_by_id(id)
    }

    fn get_id(&self, biome: &Holder<Biome>) -> i32 {
        match biome {
            Holder::Reference(id) => self.0.get_holder_by_id(*id).map(|_| *id),
            Holder::Key(key) => self.0.get_id(key),
            Holder::Direct(value) => self.0.get_id_of_value(value),
        }
        .unwrap_or(-1)
    }

    fn size(&self) -> usize {
        self.0.len()
    }
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::block_pos::BlockPos;
    use crate::nbt::{utils, Tag};
    use crate::util::bit_storage::{BitStorage, SimpleBitStorage};
    use crate::world::level::biome::biome::BiomeBuilder;
    use crate::world::level::block::state::property::{Property, StateValue};
    use crate::world::level::block::state::state_definition::{enumerate_states, StateDefinition};
    use crate::world::level::block::Block;
    use crate::world::level::chunk::storage::serializable_chunk_data::{
        ChunkLoadingContext, SerializableChunkData,
    };
    use crate::world::level::chunk_pos::ChunkPos;
    use crate::world::level::level_height_accessor::SimpleLevelHeightAccessor;

    struct BlockStates(Vec<BlockState>);

    impl IdMap<BlockState> for BlockStates {
        fn by_id(&self, id: i32) -> Option<BlockState> {
            self.0.get(id as usize).cloned()
        }

        fn get_id(&self, state: &BlockState) -> i32 {
            self.0
                .iter()
                .position(|other| other == state)
                .map_or(-1, |id| id as i32)
        }

        fn size(&self) -> usize {
            self.0.len()
        }
    }

    #[test]
    fn loads_block_variants_and_reencodes_dimension_biomes_from_global_ids() {
        let lamp = Identifier::minecraft("redstone_lamp");
        let properties: Vec<crate::world::level::block::state::property::PropertyRef> =
            vec![Arc::new(Property::new("lit", vec![false, true]))];
        let states = enumerate_states(&properties);
        let definition = Arc::new(StateDefinition::new(
            lamp.clone(),
            properties,
            states.clone(),
        ));
        let block = Arc::new(Block::new(lamp.clone()).with_state_definition(definition));
        let mut block_states = vec![BlockState::air()];
        block_states.extend(
            states
                .into_iter()
                .map(|state| BlockState::new(block.clone(), state)),
        );

        let mut biomes = Registry::new();
        let names: Vec<_> = (0..9)
            .map(|id| Identifier::parse(&format!("fixture:biome_{id}")).unwrap())
            .collect();
        for name in &names {
            biomes.register(
                ResourceKey::create_with_registry_name(registries::biome(), name.clone()),
                BiomeBuilder::new().build(),
            );
        }
        let registries = RegistryAccess::new();
        registries.register(registries::biome(), Arc::new(biomes));
        let palettes =
            RegistryChunkPalettes::new(&registries, Arc::new(BlockStates(block_states)), &names[0])
                .unwrap();

        let mut block_state = CompoundTag::new();
        block_state.put_string("Name", &lamp.to_string());
        let mut properties = CompoundTag::new();
        properties.put_string("lit", "true");
        block_state.put("Properties", Tag::Compound(properties));
        let mut blocks_nbt = CompoundTag::new();
        blocks_nbt.put("palette", Tag::List(vec![Tag::Compound(block_state)]));

        let mut biome_indices = SimpleBitStorage::new(4, 64);
        for index in 0..64 {
            biome_indices.set(index, (index % 9) as i32);
        }
        let mut biomes_nbt = CompoundTag::new();
        biomes_nbt.put(
            "palette",
            Tag::List(
                names
                    .iter()
                    .rev()
                    .map(|name| Tag::String(name.to_string()))
                    .collect(),
            ),
        );
        biomes_nbt.put_long_array("data", BitStorage::Simple(biome_indices).get_raw());

        let mut section = CompoundTag::new();
        section.put_byte("Y", 0);
        section.put("block_states", Tag::Compound(blocks_nbt));
        section.put("biomes", Tag::Compound(biomes_nbt));
        let mut nbt = CompoundTag::new();
        utils::add_current_data_version(&mut nbt);
        nbt.put_int("xPos", 0);
        nbt.put_int("zPos", 0);
        nbt.put_string("Status", "minecraft:full");
        nbt.put("sections", Tag::List(vec![Tag::Compound(section)]));
        let height = SimpleLevelHeightAccessor::create(0, 16);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &palettes,
            data_fixer: None,
            has_sky_light: false,
        };
        let pos = ChunkPos::new(0, 0);
        let data = SerializableChunkData::parse(nbt, pos, &context).unwrap();
        let state = data.chunk.get_block_state(&BlockPos::ZERO);
        assert_eq!(
            state.values.get_by_name("lit"),
            Some(StateValue::Bool(true))
        );
        assert!(matches!(
            data.chunk.sections[0].biomes.get(0, 0, 0),
            Holder::Reference(8)
        ));
        assert!(data.chunk.sections[0]
            .biomes
            .data()
            .configuration
            .always_repack());

        let saved = data
            .into_runtime(None)
            .to_nbt(1, &registries, false)
            .unwrap();
        let restored = SerializableChunkData::parse(saved, pos, &context).unwrap();
        assert_eq!(restored.chunk.get_block_state(&BlockPos::ZERO), state);
        for index in 0..64 {
            let Holder::Reference(id) = restored.chunk.sections[0].biomes.get_by_index(index)
            else {
                panic!("expected biome registry ID at {index}");
            };
            assert_eq!(id, 8 - (index % 9) as i32);
        }
    }
}
