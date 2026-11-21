//! Перенос `net.minecraft.world.level.chunk.PalettedContainer`
//! (+ `PalettedContainerRO` для read-пути).

use crate::util::bit_storage::{BitStorage, SimpleBitStorage};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::chunk::palette::{
    Configuration, IdFor, IdMap, Palette, PaletteFactory, SharedIdMap, Strategy,
};
use std::hash::Hash;
use std::sync::Arc;

/// `PalettedContainer<T>` — хранилище `entryCount` значений типа `T`
/// через битовую палитру.
#[derive(Clone)]
pub struct PalettedContainer<T: Clone + PartialEq + Eq + Hash + 'static> {
    /// Текущие данные (конфигурация + битовое хранилище + палитра).
    data: Data<T>,
    /// Стратегия (геометрия + конфиги).
    strategy: Strategy,
    /// `defaultValue`.
    pub default_value: T,
    /// Глобальная карта для `GlobalPalette` (реестр значений).
    global_map: Option<SharedIdMap<T>>,
}

/// `PalettedContainer.Data`.
#[derive(Clone)]
pub struct Data<T: Clone + PartialEq + Eq + Hash + 'static> {
    /// Конфигурация, породившая данные.
    pub configuration: Configuration,
    /// Битовое хранилище id палитры.
    pub storage: BitStorage,
    /// Палитра id→значение.
    pub palette: Palette<T>,
}

impl<T: Clone + PartialEq + Eq + Hash + 'static> PalettedContainer<T> {
    /// `PalettedContainer(defaultValue, strategy)`.
    /// `global_map` — IdMap глобального реестра (нужна для GlobalPalette
    /// и сериализации палитр).
    pub fn new(default_value: T, strategy: Strategy, global_map: Option<SharedIdMap<T>>) -> Self {
        let configuration = strategy.get_configuration_for_bit_count(0);
        let storage = Self::create_storage(configuration.bits_in_memory(), strategy.entry_count);
        let palette = configuration.create_palette(vec![default_value.clone()]);
        Self {
            data: Data { configuration, storage, palette },
            strategy,
            default_value,
            global_map,
        }
    }

    /// Глобальная карта или panic (как `strategy.globalMap()` в Java —
    /// всегда присутствует для chunk-контейнеров).
    pub fn global_map(&self) -> &dyn IdMap<T> {
        self.global_map
            .as_deref()
            .expect("PalettedContainer requires global IdMap")
    }

    /// Global IdMap как Arc — owned доступ без borrow конфликтов.
    pub fn global_map_arc(&self) -> SharedIdMap<T> {
        self.global_map
            .clone()
            .expect("PalettedContainer requires global IdMap")
    }

    fn create_storage(bits_in_memory: i32, entry_count: usize) -> BitStorage {
        if bits_in_memory == 0 {
            BitStorage::Zero { size: entry_count }
        } else {
            BitStorage::Simple(SimpleBitStorage::new(bits_in_memory, entry_count))
        }
    }

    /// Восстанавливает палитру и упакованные индексы из NBT секции.
    pub fn unpack(&mut self, values: Vec<T>, words: Option<&[i64]>) -> Result<(), String> {
        let size = self.strategy.entry_count;
        if values.is_empty() || values.len() > size {
            return Err(format!("invalid palette size {}", values.len()));
        }
        let configuration = self.strategy.get_configuration_for_palette_size(values.len());
        let bits = configuration.bits_in_storage();
        let storage = if bits == 0 {
            BitStorage::Zero { size }
        } else {
            let words = words.ok_or_else(|| "missing data for a multi-value palette".to_owned())?;
            let packed = SimpleBitStorage::from_raw(bits, size, words.to_vec())?;
            if (0..size).any(|index| packed.get(index) as usize >= values.len()) {
                return Err("palette indices exceed the encoded palette size".to_owned());
            }
            if configuration.always_repack() {
                let map = self.global_map();
                let ids: Vec<_> = values.iter().map(|value| map.get_id(value)).collect();
                let memory_bits = configuration.bits_in_memory();
                if ids.iter().any(|&id| id < 0 || (id as u64) >= (1u64 << memory_bits)) {
                    return Err("palette contains an unregistered value".to_owned());
                }
                let mut global = Self::create_storage(memory_bits, size);
                for index in 0..size {
                    global.set(index, ids[packed.get(index) as usize]);
                }
                global
            } else {
                BitStorage::Simple(packed)
            }
        };
        let palette = configuration.create_palette(values);
        self.data = Data { configuration, storage, palette };
        Ok(())
    }

    /// Собирает локальную дисковую палитру даже для контейнера с глобальными ID.
    pub fn pack(&self) -> (Vec<T>, Vec<i64>) {
        if self.data.configuration.bits_in_memory() == 0 {
            return (vec![self.get_by_index(0)], Vec::new());
        }

        let mut values = Vec::new();
        let mut ids = std::collections::HashMap::new();
        let indices: Vec<_> = (0..self.strategy.entry_count)
            .map(|index| {
                let value = self.get_by_index(index);
                *ids.entry(value.clone()).or_insert_with(|| {
                    let id = values.len() as i32;
                    values.push(value);
                    id
                })
            })
            .collect();
        let bits = self.strategy.get_configuration_for_palette_size(values.len()).bits_in_storage();
        let mut storage = Self::create_storage(bits, indices.len());
        for (index, id) in indices.into_iter().enumerate() {
            storage.set(index, id);
        }
        (values, storage.get_raw().to_vec())
    }

    /// `get(x, y, z)`.
    pub fn get(&self, x: i32, y: i32, z: i32) -> T {
        self.get_by_index(self.strategy.get_index(x, y, z))
    }

    /// `getUnchecked` по линейному индексу.
    pub fn get_by_index(&self, index: usize) -> T {
        let palette_id = self.data.storage.get(index);
        self.read_palette(&self.data, palette_id as usize)
    }

    fn read_palette(&self, data: &Data<T>, id: usize) -> T {
        match &data.palette {
            Palette::Global => data
                .palette
                .value_for(id, self.global_map())
                .unwrap_or_else(|e| panic!("{}", e)),
            _ => data
                .palette
                .value_for(id, self.global_map())
                .unwrap_or_else(|e| panic!("{}", e)),
        }
    }

    /// `set(x, y, z, value)` — возвращает старое значение.
    pub fn set(&mut self, x: i32, y: i32, z: i32, value: T) -> T {
        let index = self.strategy.get_index(x, y, z);
        self.set_by_index(index, value)
    }

    /// `set` по линейному индексу.
    pub fn set_by_index(&mut self, index: usize, value: T) -> T {
        self.get_and_set_by_index(index, value)
    }

    /// `setUnchecked` — без чтения старого.
    pub fn set_by_index_unchecked(&mut self, index: usize, value: T) {
        let palette_id = self.palette_id_for(value);
        self.data.storage.set(index, palette_id as i32);
    }

    /// `getAndSet` по линейному индексу.
    pub fn get_and_set_by_index(&mut self, index: usize, value: T) -> T {
        let old_palette_id = self.data.storage.get(index);
        let old = self.read_palette(&self.data, old_palette_id as usize);
        let palette_id = self.palette_id_for(value);
        self.data.storage.set(index, palette_id as i32);
        old
    }

    /// `getAndSet(x,y,z)`.
    pub fn get_and_set(&mut self, x: i32, y: i32, z: i32, value: T) -> T {
        let index = self.strategy.get_index(x, y, z);
        self.get_and_set_by_index(index, value)
    }

    /// `idFor` значения через палитру с обработкой роста (`onResize`).
    fn palette_id_for(&mut self, value: T) -> usize {
        if matches!(self.data.palette, Palette::Global) {
            // GlobalPalette.idFor — id в глобальном реестре.
            let id = self.global_map().get_id(&value);
            return if id == -1 { 0 } else { id as usize };
        }
        match self.data.palette.id_for(value.clone()) {
            IdFor::Id(id) => id,
            IdFor::Resize { bits, value } => self.on_resize(bits, value),
        }
    }

    /// `onResize` — пересоздать данные под `bits` и вставить `addedValue`,
    /// вернуть id добавленного значения.
    fn on_resize(&mut self, bits: i32, added_value: T) -> usize {
        let config = self.strategy.get_configuration_for_bit_count(bits);
        let entry_count = self.strategy.entry_count;
        let mut new_storage = Self::create_storage(config.bits_in_memory(), entry_count);
        let mut new_palette = config.create_palette(Vec::new());

        // Перенос всех значений из старых данных.
        let old_data = self.data.clone();
        for i in 0..entry_count {
            let old_id = old_data.storage.get(i);
            let value = match &old_data.palette {
                Palette::Global => old_data
                    .palette
                    .value_for(old_id as usize, self.global_map())
                    .expect("palette entry"),
                _ => old_data
                    .palette
                    .value_for(old_id as usize, self.global_map())
                    .expect("palette entry"),
            };
            let new_id = Self::palette_insert_no_resize(&mut new_palette, value, self.global_map());
            new_storage.set(i, new_id as i32);
        }
        // Вставка добавляемого значения.
        let added_id = Self::palette_insert_no_resize(&mut new_palette, added_value, self.global_map());

        self.data = Data { configuration: config, storage: new_storage, palette: new_palette };
        added_id
    }

    /// Вставка в палитру без resize — конфиг выбран с достаточной ёмкостью.
    fn palette_insert_no_resize(palette: &mut Palette<T>, value: T, map: &dyn IdMap<T>) -> usize {
        match palette.id_for(value.clone()) {
            IdFor::Id(id) => id,
            IdFor::Resize { .. } => {
                // Для GlobalPalette не бывает; для Simple — растём рекурсивно
                // через полную пересборку не нужна: new_palette создан под
                // config.bits, а palette size ≤ 2^bits гарантировано.
                // На всякий случай — обработаем через value_for по id.
                panic!("Unexpected palette resize during container onResize")
            }
        }
    }

    /// `read` — десериализация.
    pub fn read(&mut self, buf: &mut FriendlyByteBuf, map: &dyn IdMap<T>) -> Result<(), String> {
        let bits = buf.read_byte() as i32;
        // `createOrReuseData(this.data, bits)`
        let config = self.strategy.get_configuration_for_bit_count(bits);
        self.data = Data {
            configuration: config,
            storage: Self::create_storage(config.bits_in_memory(), self.strategy.entry_count),
            palette: config.create_palette(Vec::new()),
        };
        self.data.palette.read(buf, map)?;
        // `readFixedSizeLongArray` — читает ровно столько long'ов,
        // сколько занимает storage.
        let raw_len = self.data.storage.get_raw().len();
        let longs = buf.read_fixed_size_long_array(raw_len);
        self.data.storage.get_raw_mut().copy_from_slice(&longs);
        Ok(())
    }

    /// `write` — сериализация.
    pub fn write(&self, buf: &mut FriendlyByteBuf, map: &dyn IdMap<T>) {
        self.data.write(buf, map);
    }

    /// `getSerializedSize`.
    pub fn get_serialized_size(&self, map: &dyn IdMap<T>) -> usize {
        self.data.get_serialized_size(map)
    }

    /// `bitsPerEntry`.
    pub fn bits_per_entry(&self) -> i32 {
        self.data.storage.get_bits()
    }

    /// `maybeHas`.
    pub fn maybe_has(&self, predicate: &dyn Fn(&T) -> bool) -> bool {
        self.data.palette.maybe_has(predicate)
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// `recreate` — новый контейнер со значением по умолчанию текущей палитры.
    pub fn recreate(&self) -> Self {
        let default = self.data.palette.value_for(0, self.global_map()).unwrap_or_else(|_| self.default_value.clone());
        Self::new(default, self.strategy.clone(), self.global_map.clone())
    }

    /// `count` — карта значение→количество.
    pub fn count(&self, mut count_consumer: impl FnMut(T, i32)) {
        let palette_size = match &self.data.palette {
            Palette::Global => self.global_map().size(),
            p => p.get_size(),
        };
        if palette_size == 1 {
            let v = self
                .data
                .palette
                .value_for(0, self.global_map())
                .unwrap_or_else(|_| self.default_value.clone());
            count_consumer(v, self.data.storage.get_size() as i32);
        } else {
            let mut counts: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
            self.data.storage.get_all(|id| *counts.entry(id as usize).or_insert(0) += 1);
            for (id, count) in counts {
                let v = self.read_palette(&self.data, id);
                count_consumer(v, count);
            }
        }
    }

    /// Прямой доступ к данным.
    pub fn data(&self) -> &Data<T> {
        &self.data
    }

    /// Изменяемый доступ к данным.
    pub fn data_mut(&mut self) -> &mut Data<T> {
        &mut self.data
    }

    /// Стратегия.
    pub fn strategy(&self) -> &Strategy {
        &self.strategy
    }

    /// Глобальная карта (Arc).
    pub fn shared_global_map(&self) -> Option<&SharedIdMap<T>> {
        self.global_map.as_ref()
    }
}

impl<T: Clone + PartialEq + Eq + Hash + 'static> Data<T> {
    /// `Data.getSerializedSize` = 1 (байт бит) + палитра + long[] * 8.
    pub fn get_serialized_size(&self, map: &dyn IdMap<T>) -> usize {
        1 + self.palette.get_serialized_size(map) + self.storage.get_raw().len() * 8
    }

    /// `Data.write` — байт битов + палитра + фиксированный long[] массив.
    pub fn write(&self, buf: &mut FriendlyByteBuf, map: &dyn IdMap<T>) {
        buf.write_byte(self.storage.get_bits());
        self.palette.write(buf, map);
        buf.write_fixed_size_long_array(self.storage.get_raw());
    }
}

/// Пустая IdMap — значение не должно искаться (non-global палитры
/// value_for не обращается к карте).
pub struct EmptyIdMap;

impl<T: Clone + PartialEq + Eq + Hash + 'static> IdMap<T> for EmptyIdMap {
    fn by_id(&self, _id: i32) -> Option<T> {
        None
    }
    fn get_id(&self, _value: &T) -> i32 {
        -1
    }
    fn size(&self) -> usize {
        0
    }
}

/// Удобный алиас для Arc IdMap.
pub fn arc_map<T: Clone + PartialEq + Eq + Hash + 'static>(m: impl IdMap<T> + 'static) -> SharedIdMap<T> {
    Arc::new(m)
}
