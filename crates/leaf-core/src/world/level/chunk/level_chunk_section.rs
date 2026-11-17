//! Перенос `net.minecraft.world.level.chunk.LevelChunkSection`.

use super::paletted_container::PalettedContainer;
use crate::world::level::block::state::{BlockState, FluidState};
use crate::world::level::chunk::palette::SharedIdMap;

/// `LevelChunkSection` — секция 16×16×16: палитра блоков + биомов.
#[derive(Clone)]
pub struct LevelChunkSection {
    /// `states` — `PalettedContainer<BlockState>`.
    pub states: PalettedContainer<BlockState>,
    /// `biomes` — `PalettedContainer<Holder<Biome>>` (4³ ячейки).
    pub biomes: PalettedContainer<crate::core::holder::Holder<crate::world::level::biome::Biome>>,
    /// `nonEmptyBlockCount`.
    pub non_empty_block_count: i16,
}

impl LevelChunkSection {
    /// `new` — пустая секция со всеми `default` значениями.
    pub fn new(
        states: PalettedContainer<BlockState>,
        biomes: PalettedContainer<crate::core::holder::Holder<crate::world::level::biome::Biome>>,
    ) -> Self {
        Self { states, biomes, non_empty_block_count: 0 }
    }

    /// `getBlockState`.
    pub fn get_block_state(&self, x: i32, y: i32, z: i32) -> BlockState {
        self.states.get(x, y, z)
    }

    /// `getFluidState`.
    pub fn get_fluid_state(&self, x: i32, y: i32, z: i32) -> FluidState {
        self.states.get(x, y, z).get_fluid_state()
    }

    /// `setBlockState` — обновляет счётчик непустых блоков.
    /// Возвращает `(old_state, new_state)`.
    pub fn set_block_state(&mut self, x: i32, y: i32, z: i32, state: BlockState) -> (BlockState, BlockState) {
        let old = self.states.get_and_set(x, y, z, state.clone());
        self.update_count(&old, &state);
        (old, state)
    }

    fn update_count(&mut self, old: &BlockState, new: &BlockState) {
        if old.is_air() != new.is_air() {
            self.non_empty_block_count += if new.is_air() { -1 } else { 1 };
        }
    }

    /// `getBiomes().get(x,y,z)` — holder биома (кварто-координаты 0..3).
    pub fn get_biome(&self, x: i32, y: i32, z: i32) -> crate::core::holder::Holder<crate::world::level::biome::Biome> {
        self.biomes.get(x, y, z)
    }

    /// `setBiome`.
    pub fn set_biome(&mut self, x: i32, y: i32, z: i32, biome: crate::core::holder::Holder<crate::world::level::biome::Biome>) {
        self.biomes.set(x, y, z, biome);
    }

    /// `hasOnlyAir`.
    pub fn has_only_air(&self) -> bool {
        self.non_empty_block_count == 0
    }

    /// `maybeHas` — предикат по блокам.
    pub fn maybe_has(&self, predicate: &dyn Fn(&BlockState) -> bool) -> bool {
        self.states.maybe_has(predicate)
    }

    /// `read` — десериализация секции из буфера.
    pub fn read(&mut self, buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) -> Result<(), String> {
        self.non_empty_block_count = buf.read_short() as i16;
        let gm = self.states.global_map_arc();
        self.states.read(buf, gm.as_ref())?;
        let mut biomes = self.biomes.recreate();
        let gm2 = biomes.global_map_arc();
        biomes.read(buf, gm2.as_ref())?;
        self.biomes = biomes;
        Ok(())
    }

    /// `write` — сериализация секции (short count + states + biomes).
    pub fn write(&self, buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) {
        buf.write_short(self.non_empty_block_count as i32);
        self.states.write(buf, self.states.global_map());
        self.biomes.write(buf, self.biomes.global_map());
    }

    /// `getSerializedSize`.
    pub fn get_serialized_size(&self) -> usize {
        2 + self.states.get_serialized_size(self.states.global_map())
            + self.biomes.get_serialized_size(self.biomes.global_map())
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        Self {
            states: self.states.copy(),
            biomes: self.biomes.copy(),
            non_empty_block_count: self.non_empty_block_count,
        }
    }
}

/// Фабрики контейнеров для секций — глобальные карты блоков/биомов.
pub fn create_block_states_container(default: BlockState) -> PalettedContainer<BlockState> {
    use super::palette::StrategyKind;
    let size = crate::world::level::block::state::block_state_registry::block_state_count();
    let strategy = super::palette::Strategy::new(StrategyKind::BlockStates, size);
    let map = crate::world::level::block::state::block_state_registry::block_state_id_map();
    PalettedContainer::new(default, strategy, Some(map))
}

/// Фабрика биом-контейнера.
pub fn create_biomes_container(default: crate::core::holder::Holder<crate::world::level::biome::Biome>) -> PalettedContainer<crate::core::holder::Holder<crate::world::level::biome::Biome>> {
    use super::palette::StrategyKind;
    let size = crate::world::level::biome::biome_registry::biome_count();
    let strategy = super::palette::Strategy::new(StrategyKind::Biomes, size);
    let map = crate::world::level::biome::biome_registry::biome_holder_id_map();
    PalettedContainer::<crate::core::holder::Holder<crate::world::level::biome::Biome>>::new(default, strategy, Some(map))
}
