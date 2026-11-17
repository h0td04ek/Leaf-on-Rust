//! Перенос `net.minecraft.world.level.chunk.Palette`,
//! `SingleValuePalette`, `LinearPalette`, `HashMapPalette`,
//! `GlobalPalette`, `Configuration`, `Strategy`.
//!
//! Отличие от Java: идентичность значений — по `PartialEq`/`Hash`
//! (в Java — identity `==`, но реестровые значения там синглтоны,
//! поэтому семантика совпадает).

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// `MissingPaletteEntryException`.
#[derive(Clone, Debug)]
pub struct MissingPaletteEntryException(pub i32);

impl std::fmt::Display for MissingPaletteEntryException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing Palette entry for id {}.", self.0)
    }
}

impl std::error::Error for MissingPaletteEntryException {}

/// `IdMap<T>` — глобальная карта значение↔id (реестр).
pub trait IdMap<T>: Send + Sync {
    /// `byId` — значение по id или None.
    fn by_id(&self, id: i32) -> Option<T>;
    /// `getId` — id значения или -1.
    fn get_id(&self, value: &T) -> i32;
    /// `size`.
    fn size(&self) -> usize;
}

/// Результат `Palette::id_for`.
#[derive(Debug)]
pub enum IdFor<T> {
    /// Значение в палитре / добавлено без роста.
    Id(usize),
    /// Требуется рост до `bits` бит; `value` возвращается для вставки.
    Resize { bits: i32, value: T },
}

/// `PaletteResize<T>` — делегат роста палитры (контейнер реализует).
pub type PaletteResize<'a, T> = dyn FnMut(i32, T) -> usize + 'a;

/// `Palette<T>` — палитра id→значение.
#[derive(Clone)]
pub enum Palette<T: Clone + PartialEq + Eq + Hash + 'static> {
    /// `SingleValuePalette`.
    SingleValue {
        /// Единственное значение (None — не инициализирована).
        value: Option<T>,
    },
    /// `LinearPalette` — линейный массив ёмкостью `1 << bits`;
    /// `values.len() == size` (записи добавляются строго подряд).
    Linear {
        /// Занятые значения в порядке id.
        values: Vec<T>,
        /// `bits`.
        bits: i32,
    },
    /// `HashMapPalette` — insertion-ordered map значение→id.
    HashMap {
        /// Значения в порядке id.
        values: Vec<T>,
        /// Обратный индекс.
        index: HashMap<T, usize>,
        /// `bits`.
        bits: i32,
    },
    /// `GlobalPalette` — значения не сериализуются, id = id в глоб. реестре.
    Global,
}

impl<T: Clone + PartialEq + Eq + Hash + 'static> Palette<T> {
    /// `SingleValuePalette.create`.
    pub fn single_value(values: Vec<T>) -> Self {
        assert!(values.len() <= 1, "Can't initialize SingleValuePalette with {} values.", values.len());
        Palette::SingleValue { value: values.into_iter().next() }
    }

    /// `LinearPalette.create`.
    pub fn linear(bits: i32, values: Vec<T>) -> Self {
        let capacity = 1usize << bits;
        assert!(values.len() <= capacity, "Can't initialize LinearPalette of size {} with {} entries", capacity, values.len());
        Palette::Linear { values, bits }
    }

    /// `HashMapPalette.create`.
    pub fn hash_map(bits: i32, values: Vec<T>) -> Self {
        let mut map = HashMap::with_capacity(values.len());
        for (i, v) in values.iter().enumerate() {
            map.insert(v.clone(), i);
        }
        Palette::HashMap { values, index: map, bits }
    }

    /// `getSize`.
    pub fn get_size(&self) -> usize {
        match self {
            Palette::SingleValue { value } => usize::from(value.is_some()),
            Palette::Linear { values, .. } => values.len(),
            Palette::HashMap { values, .. } => values.len(),
            // GlobalPalette.getSize() = registry.size() — обрабатывается
            // на уровне контейнера через `map.size()`.
            Palette::Global => 0,
        }
    }

    /// `idFor` — вернуть id значения; при переполнении — `Resize`.
    /// Реальный рост выполняет `PalettedContainer::on_resize`.
    pub fn id_for(&mut self, state: T) -> IdFor<T> {
        match self {
            Palette::SingleValue { value } => {
                if let Some(v) = value {
                    if *v == state {
                        return IdFor::Id(0);
                    }
                    return IdFor::Resize { bits: 1, value: state };
                }
                *value = Some(state);
                IdFor::Id(0)
            }
            Palette::Linear { values, bits } => {
                for (i, v) in values.iter().enumerate() {
                    if *v == state {
                        return IdFor::Id(i);
                    }
                }
                let ix = values.len();
                if ix < (1usize << *bits) {
                    values.push(state);
                    return IdFor::Id(ix);
                }
                IdFor::Resize { bits: *bits + 1, value: state }
            }
            Palette::HashMap { values, index, bits } => {
                if let Some(&id) = index.get(&state) {
                    return IdFor::Id(id);
                }
                if values.len() >= (1usize << *bits) {
                    return IdFor::Resize { bits: *bits + 1, value: state };
                }
                let id = values.len();
                index.insert(state.clone(), id);
                values.push(state);
                IdFor::Id(id)
            }
            Palette::Global => IdFor::Id(0), // idFor GlobalPalette обрабатывается через IdMap в контейнере
        }
    }

    /// `valueFor`.
    pub fn value_for(&self, id: usize, map: &dyn IdMap<T>) -> Result<T, MissingPaletteEntryException> {
        match self {
            Palette::SingleValue { value } => {
                if id == 0 {
                    if let Some(v) = value {
                        return Ok(v.clone());
                    }
                }
                Err(MissingPaletteEntryException(id as i32))
            }
            Palette::Linear { values, .. } => {
                if let Some(v) = values.get(id) {
                    Ok(v.clone())
                } else {
                    Err(MissingPaletteEntryException(id as i32))
                }
            }
            Palette::HashMap { values, .. } => {
                if let Some(v) = values.get(id) {
                    Ok(v.clone())
                } else {
                    Err(MissingPaletteEntryException(id as i32))
                }
            }
            Palette::Global => {
                let v = map.by_id(id as i32).ok_or(MissingPaletteEntryException(id as i32))?;
                Ok(v)
            }
        }
    }

    /// `maybeHas`.
    pub fn maybe_has(&self, filter: &dyn Fn(&T) -> bool) -> bool {
        match self {
            Palette::SingleValue { value } => value.as_ref().map(|v| filter(v)).unwrap_or(false),
            Palette::Linear { values, .. } | Palette::HashMap { values, .. } => values.iter().any(|v| filter(v)),
            Palette::Global => true,
        }
    }

    /// `read` — десериализация из буфера через глобальную карту.
    pub fn read(&mut self, buf: &mut FriendlyByteBuf, map: &dyn IdMap<T>) -> Result<(), String> {
        match self {
            Palette::SingleValue { value } => {
                let id = buf.read_var_int();
                *value = Some(
                    map.by_id(id).ok_or_else(|| format!("Missing palette entry for id {}", id))?,
                );
                Ok(())
            }
            Palette::Linear { values, bits } => {
                let n = buf.read_var_int() as usize;
                let capacity = 1usize << *bits;
                assert!(n <= capacity, "Can't read {} entries into LinearPalette of capacity {}", n, capacity);
                values.clear();
                for _ in 0..n {
                    let id = buf.read_var_int();
                    values.push(
                        map.by_id(id)
                            .ok_or_else(|| format!("Missing palette entry for id {}", id))?,
                    );
                }
                Ok(())
            }
            Palette::HashMap { values, index, .. } => {
                values.clear();
                index.clear();
                let n = buf.read_var_int() as usize;
                for _ in 0..n {
                    let id = buf.read_var_int();
                    let v = map
                        .by_id(id)
                        .ok_or_else(|| format!("Missing palette entry for id {}", id))?;
                    index.insert(v.clone(), values.len());
                    values.push(v);
                }
                Ok(())
            }
            Palette::Global => Ok(()),
        }
    }

    /// `write` — сериализация через глобальную карту.
    pub fn write(&self, buf: &mut FriendlyByteBuf, map: &dyn IdMap<T>) {
        match self {
            Palette::SingleValue { value } => {
                let v = value.as_ref().expect("Use of an uninitialized palette");
                buf.write_var_int(map.get_id(v));
            }
            Palette::Linear { values, .. } => {
                buf.write_var_int(values.len() as i32);
                for v in values {
                    buf.write_var_int(map.get_id(v));
                }
            }
            Palette::HashMap { values, .. } => {
                buf.write_var_int(values.len() as i32);
                for v in values {
                    buf.write_var_int(map.get_id(v));
                }
            }
            Palette::Global => {}
        }
    }

    /// `getSerializedSize` — размер в байтах на проводе.
    pub fn get_serialized_size(&self, map: &dyn IdMap<T>) -> usize {
        match self {
            Palette::SingleValue { value } => {
                let v = value.as_ref().expect("Use of an uninitialized palette");
                var_int_size(map.get_id(v))
            }
            Palette::Linear { values, .. } | Palette::HashMap { values, .. } => {
                let mut n = var_int_size(values.len() as i32);
                for v in values {
                    n += var_int_size(map.get_id(v));
                }
                n
            }
            Palette::Global => 0,
        }
    }

    /// `copy`.
    pub fn copy(&self) -> Palette<T> {
        self.clone()
    }

    /// Записи палитры (для `pack`/реэнкода).
    pub fn entries(&self) -> Vec<T> {
        match self {
            Palette::SingleValue { value } => value.iter().cloned().collect(),
            Palette::Linear { values, .. } | Palette::HashMap { values, .. } => values.clone(),
            Palette::Global => Vec::new(),
        }
    }
}

/// `VarInt.getByteSize`.
pub fn var_int_size(mut value: i32) -> usize {
    let mut size = 1;
    while (value & !0x7F) != 0 {
        value >>= 7;
        size += 1;
    }
    size
}

/// `Palette.Factory` — фабрика палитр по типу.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteFactory {
    /// `SINGLE_VALUE_PALETTE_FACTORY`.
    SingleValue,
    /// `LINEAR_PALETTE_FACTORY`.
    Linear,
    /// `HASHMAP_PALETTE_FACTORY`.
    HashMap,
}

/// `Configuration` — конфигурация бита хранения + фабрика палитры.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Configuration {
    /// `Configuration.Simple`.
    Simple {
        /// Фабрика палитры.
        factory: PaletteFactory,
        /// `bits` — и биты памяти, и биты хранения.
        bits: i32,
    },
    /// `Configuration.Global` — глобальная палитра, `alwaysRepack`.
    Global {
        /// `bitsInMemory` — биты хранения (= globalPaletteBits).
        bits_in_memory: i32,
        /// `bitsInStorage` — запрошенные биты палитры.
        bits_in_storage: i32,
    },
}

impl Configuration {
    /// `alwaysRepack`.
    pub fn always_repack(&self) -> bool {
        matches!(self, Configuration::Global { .. })
    }

    /// `bitsInMemory`.
    pub fn bits_in_memory(&self) -> i32 {
        match self {
            Configuration::Simple { bits, .. } => *bits,
            Configuration::Global { bits_in_memory, .. } => *bits_in_memory,
        }
    }

    /// `bitsInStorage`.
    pub fn bits_in_storage(&self) -> i32 {
        match self {
            Configuration::Simple { bits, .. } => *bits,
            Configuration::Global { bits_in_storage, .. } => *bits_in_storage,
        }
    }

    /// `createPalette`.
    pub fn create_palette<T: Clone + PartialEq + Eq + Hash + 'static>(
        &self,
        values: Vec<T>,
    ) -> Palette<T> {
        match self {
            Configuration::Simple { factory, bits } => match factory {
                PaletteFactory::SingleValue => Palette::single_value(values),
                PaletteFactory::Linear => Palette::linear(*bits, values),
                PaletteFactory::HashMap => Palette::hash_map(*bits, values),
            },
            Configuration::Global { .. } => Palette::Global,
        }
    }
}

/// Тип стратегии (`Strategy.createForBlockStates`/`createForBiomes`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategyKind {
    /// Для `BlockState` (16³ = 4096 ячеек, 4 бита на ось).
    BlockStates,
    /// Для `Holder<Biome>` (4³ = 64 ячейки, 2 бита на ось).
    Biomes,
}

/// `Strategy<T>` — стратегия палитры контейнера.
#[derive(Clone)]
pub struct Strategy {
    /// Вид стратегии.
    pub kind: StrategyKind,
    /// `bitsPerAxis`.
    pub bits_per_axis: i32,
    /// `entryCount`.
    pub entry_count: usize,
    /// `globalPaletteBitsInMemory` — min bits для глобальной карты.
    pub global_palette_bits_in_memory: i32,
}

impl Strategy {
    /// `Strategy` конструктор.
    pub fn new(kind: StrategyKind, global_map_size: usize) -> Self {
        let (bits_per_axis, entry_count) = match kind {
            StrategyKind::BlockStates => (4, 4096),
            StrategyKind::Biomes => (2, 64),
        };
        Self {
            kind,
            bits_per_axis,
            entry_count,
            global_palette_bits_in_memory: ceillog2(global_map_size.max(1)) as i32,
        }
    }

    /// `getIndex` — линейный индекс (x,y,z) → ячейка.
    pub fn get_index(&self, x: i32, y: i32, z: i32) -> usize {
        (((y << self.bits_per_axis) | z) << self.bits_per_axis | x) as usize
    }

    /// `getConfigurationForBitCount`.
    pub fn get_configuration_for_bit_count(&self, bits: i32) -> Configuration {
        match self.kind {
            StrategyKind::BlockStates => match bits {
                0 => Configuration::Simple { factory: PaletteFactory::SingleValue, bits: 0 },
                // Leaf/Lithium: 1-2 бита → 4-битная линейная палитра.
                1 | 2 => Configuration::Simple { factory: PaletteFactory::Linear, bits: 4 },
                3 | 4 => Configuration::Simple { factory: PaletteFactory::HashMap, bits: 4 },
                5 => Configuration::Simple { factory: PaletteFactory::HashMap, bits: 5 },
                6 => Configuration::Simple { factory: PaletteFactory::HashMap, bits: 6 },
                7 => Configuration::Simple { factory: PaletteFactory::HashMap, bits: 7 },
                8 => Configuration::Simple { factory: PaletteFactory::HashMap, bits: 8 },
                _ => Configuration::Global {
                    bits_in_memory: self.global_palette_bits_in_memory,
                    bits_in_storage: bits,
                },
            },
            StrategyKind::Biomes => match bits {
                0 => Configuration::Simple { factory: PaletteFactory::SingleValue, bits: 0 },
                1 => Configuration::Simple { factory: PaletteFactory::Linear, bits: 1 },
                2 => Configuration::Simple { factory: PaletteFactory::Linear, bits: 2 },
                3 => Configuration::Simple { factory: PaletteFactory::Linear, bits: 3 },
                _ => Configuration::Global {
                    bits_in_memory: self.global_palette_bits_in_memory,
                    bits_in_storage: bits,
                },
            },
        }
    }

    /// `getConfigurationForPaletteSize`.
    pub fn get_configuration_for_palette_size(&self, size: usize) -> Configuration {
        let bits = ceillog2(size.max(1)) as i32;
        self.get_configuration_for_bit_count(bits)
    }
}

/// `Mth.ceillog2` — минимальное число бит для `size` значений.
pub fn ceillog2(size: usize) -> u32 {
    if size <= 1 {
        return 0;
    }
    let bits = (usize::BITS - (size - 1).leading_zeros()) as u32;
    bits
}

/// `SharedIdMap` — разделяемая глобальная карта.
pub type SharedIdMap<T> = Arc<dyn IdMap<T>>;
