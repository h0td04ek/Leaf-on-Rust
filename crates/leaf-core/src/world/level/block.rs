//! Перенос `net.minecraft.world.level.block`.

pub mod rotation;
pub mod entity;
pub mod jigsaw_block;
pub mod mirror;
pub mod structure_mode;
pub mod test_block_mode;
pub mod state;

pub use rotation::Rotation;
pub use mirror::Mirror;
pub use structure_mode::StructureMode;
pub use test_block_mode::TestBlockMode;

/// `net.minecraft.world.level.block.Block` — реестровая запись
/// блока (`BlockBehaviour`-наследник в Java; здесь — данные:
/// id + `StateDefinition` + поведенческие флаги добавляются
/// по мере порта `BlockBehaviour`).
#[derive(Clone)]
pub struct Block {
    /// Registry-ключ блока (`minecraft:stone` и т.п.).
    pub id: crate::resources::identifier::Identifier,

    /// `StateDefinition` блока (None до bootstrap'а).
    pub state_definition: Option<std::sync::Arc<state::StateDefinition>>,

    /// Поведенческие флаги `BlockBehaviour.Properties`
    /// (hasCollision, isAir, destroysDrops и т.п.) — добавляются
    /// инкрементально по мере порта BlockBehaviour.
    pub properties: BlockProperties,

    /// `BlockKind` — класс блока для pathfinding/особых проверок
    /// (в Java — `instanceof DoorBlock`/`BaseRailBlock` и т.п.).
    pub kind: BlockKind,
}

/// `Block.UpdateFlags` — битовые флаги обновления при setBlock.
pub mod update_flags {
    /// `UPDATE_NEIGHBORS`.
    pub const UPDATE_NEIGHBORS: i32 = 1;
    /// `UPDATE_CLIENTS`.
    pub const UPDATE_CLIENTS: i32 = 2;
    /// `UPDATE_INVISIBLE`.
    pub const UPDATE_INVISIBLE: i32 = 4;
    /// `UPDATE_IMMEDIATE`.
    pub const UPDATE_IMMEDIATE: i32 = 8;
    /// `UPDATE_KNOWN_SHAPE`.
    pub const UPDATE_KNOWN_SHAPE: i32 = 16;
    /// `UPDATE_SUPPRESS_DROPS`.
    pub const UPDATE_SUPPRESS_DROPS: i32 = 32;
    /// `UPDATE_MOVE_BY_PISTON`.
    pub const UPDATE_MOVE_BY_PISTON: i32 = 64;
    /// `UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE`.
    pub const UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE: i32 = 128;
    /// `UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS`.
    pub const UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS: i32 = 256;
    /// `UPDATE_SKIP_ON_PLACE`.
    pub const UPDATE_SKIP_ON_PLACE: i32 = 512;
    /// `UPDATE_NONE`.
    pub const UPDATE_NONE: i32 = UPDATE_INVISIBLE | UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS;
    /// `UPDATE_ALL`.
    pub const UPDATE_ALL: i32 = UPDATE_NEIGHBORS | UPDATE_CLIENTS;
    /// `UPDATE_ALL_IMMEDIATE`.
    pub const UPDATE_ALL_IMMEDIATE: i32 = UPDATE_NEIGHBORS | UPDATE_CLIENTS | UPDATE_IMMEDIATE;
    /// `UPDATE_SKIP_ALL_SIDEEFFECTS`.
    pub const UPDATE_SKIP_ALL_SIDEEFFECTS: i32 =
        UPDATE_KNOWN_SHAPE | UPDATE_SUPPRESS_DROPS | UPDATE_SKIP_BLOCK_ENTITY_SIDEEFFECTS | UPDATE_SKIP_ON_PLACE;
    /// `UPDATE_LIMIT` — recursion limit для setBlock.
    pub const UPDATE_LIMIT: i32 = 512;
}

/// `BlockKind` — тип блока, влияющий на `PathType` и
/// проходимость (аналог instanceof-проверок в
/// `WalkNodeEvaluator.getPathTypeFromState`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockKind {
    /// Обычный блок.
    #[default]
    Normal,
    /// `DoorBlock` — `wooden` = `DoorType.canOpenByHand()`.
    Door { wooden: bool },
    /// `FenceGateBlock`.
    FenceGate,
    /// `BaseRailBlock`.
    BaseRail,
    /// `LeavesBlock`.
    Leaves,
}

pub type ProjectileHitHandler = std::sync::Arc<
    dyn Fn(&dyn crate::world::level::level_reader::LevelReader,
        &state::BlockState,
        &crate::world::phys::BlockHitResult,
        &crate::world::entity::projectile::Projectile)
        + Send
        + Sync,
>;

/// Подмножество `BlockBehaviour.Properties` — флаги,
/// влияющие на wire/предикаты.
#[derive(Clone, Default)]
pub struct BlockProperties {
    /// `isAir`.
    pub is_air: bool,
    /// `hasCollision`.
    pub has_collision: bool,
    /// `canOcclude`.
    pub can_occlude: bool,
    /// `isRandomlyTicking`.
    pub is_randomly_ticking: bool,
    /// `destroyTime` (None = неразрушим, -1 по умолчанию в Java
    /// у флагов без override).
    pub destroy_time: Option<f32>,
    /// `explosionResistance`.
    pub explosion_resistance: Option<f32>,
    /// `liquid` — блок-жидкость (`LiquidBlock` в Java).
    pub liquid: bool,
    /// `isRedstoneConductor` — StatePredicate (default:
    /// `isCollisionShapeFullBlock`).
    pub is_redstone_conductor: Option<std::sync::Arc<dyn Fn(&state::BlockState, &dyn crate::world::level::level_reader::LevelReader, crate::core::block_pos::BlockPos) -> bool + Send + Sync>>,

    /// `isValidSpawn` — StateArgumentPredicate<EntityType>
    /// (default: `isFaceSturdy(UP) && lightEmission < 14`).
    pub is_valid_spawn: Option<std::sync::Arc<dyn Fn(&state::BlockState, &dyn crate::world::level::level_reader::LevelReader, crate::core::block_pos::BlockPos, &crate::entity::entity_type::EntityType) -> bool + Send + Sync>>,

    /// `isPathfindable` — переопределение проходимости
    /// (`None` → дефолт `BlockBehaviour.isPathfindable`).
    pub pathfindable: Option<bool>,
    /// `collisionShape` — форма коллизии блока
    /// (`None` → полный куб для твёрдых, пусто для air/liquid).
    pub collision_shape: Option<crate::world::phys::shapes::VoxelShape>,
    /// `isSignalSource` — `BlockBehaviour.Properties.isSignalSource`.
    pub signal_source: bool,

    /// `getSignal` — `BlockBehaviour.getSignal` (default: 0).
    /// `Fn(&BlockState, &dyn LevelReader, BlockPos, Direction) -> i32`.
    pub get_signal: Option<std::sync::Arc<dyn Fn(&state::BlockState, &dyn crate::world::level::level_reader::LevelReader, crate::core::block_pos::BlockPos, crate::core::direction::Direction) -> i32 + Send + Sync>>,

    /// `getDirectSignal` — `BlockBehaviour.getDirectSignal` (default: 0).
    pub get_direct_signal: Option<std::sync::Arc<dyn Fn(&state::BlockState, &dyn crate::world::level::level_reader::LevelReader, crate::core::block_pos::BlockPos, crate::core::direction::Direction) -> i32 + Send + Sync>>,

    /// `updateShape` — `BlockBehaviour.updateShape`
    /// (default: identity — возвращает входное состояние).
    pub update_shape: Option<std::sync::Arc<dyn Fn(&state::BlockState, &dyn crate::world::level::level_reader::LevelReader, &dyn crate::world::level::scheduled_tick_access::ScheduledTickAccess, crate::core::block_pos::BlockPos, crate::core::direction::Direction, crate::core::block_pos::BlockPos, &state::BlockState, std::sync::Arc<dyn crate::random::RandomSource>) -> state::BlockState + Send + Sync>>,

    /// `BlockBehaviour.onProjectileHit`.
    pub on_projectile_hit: Option<ProjectileHitHandler>,

    /// `replaceable` — `BlockBehaviour.Properties.replaceable`.
    pub replaceable: bool,
    // `isLitCampfire`-подобные блоки: `lit`-свойство читается
    // из `BlockState.values` (campfire/soul_campfire).
}

/// Глобальный реестр блоков (`BuiltInRegistries.BLOCK`) —
/// упорядоченный список для holderRegistry-кодека.
pub fn block_registry() -> &'static std::sync::RwLock<Vec<std::sync::Arc<Block>>> {
    static REG: std::sync::OnceLock<std::sync::RwLock<Vec<std::sync::Arc<Block>>>> = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

/// Регистрация блока — вызывается из bootstrap'а в порядке vanilla.
/// Возвращает registry-id.
pub fn register_block(block: Block) -> i32 {
    let mut reg = block_registry().write().unwrap();
    let id = reg.len() as i32;
    reg.push(std::sync::Arc::new(block));
    id
}

/// Блок по registry-id.
pub fn block_by_id(id: i32) -> Option<std::sync::Arc<Block>> {
    block_registry().read().unwrap().get(id as usize).cloned()
}

/// Блок по идентификатору (`minecraft:stone`).
pub fn block_by_identifier(id: &crate::resources::identifier::Identifier) -> Option<std::sync::Arc<Block>> {
    block_registry()
        .read()
        .unwrap()
        .iter()
        .find(|b| &b.id == id)
        .cloned()
}

impl Block {
    /// `Block.isShapeFullBlock(VoxelShape)` — форма совпадает
    /// с единичным кубом блока.
    pub fn is_shape_full_block(shape: &crate::world::phys::shapes::VoxelShape) -> bool {
        if shape.is_empty() {
            return false;
        }
        match shape.bounds() {
            Some(b) => {
                (b.min_x - 0.0).abs() < 1e-6
                    && (b.min_y - 0.0).abs() < 1e-6
                    && (b.min_z - 0.0).abs() < 1e-6
                    && (b.max_x - 1.0).abs() < 1e-6
                    && (b.max_y - 1.0).abs() < 1e-6
                    && (b.max_z - 1.0).abs() < 1e-6
            }
            None => false,
        }
    }

    /// `canSurvive` — базовая реализация `BlockBehaviour.canSurvive`
    /// (true; переопределяется конкретными блоками при их порте).
    pub fn can_survive(&self, _state: &state::BlockState, _level: &dyn crate::world::level::level_reader::LevelReader, _pos: crate::core::block_pos::BlockPos) -> bool {
        true
    }

    /// `new`.
    pub fn new(id: crate::resources::identifier::Identifier) -> Self {
        Self {
            id,
            state_definition: None,
            properties: BlockProperties::default(),
            kind: BlockKind::Normal,
        }
    }

    /// `withKind`.
    pub fn with_kind(mut self, kind: BlockKind) -> Self {
        self.kind = kind;
        self
    }

    /// `withStateDefinition`.
    pub fn with_state_definition(mut self, def: std::sync::Arc<state::StateDefinition>) -> Self {
        self.state_definition = Some(def);
        self
    }

    /// `stateDefinition`.
    pub fn state_definition(&self) -> Option<&std::sync::Arc<state::StateDefinition>> {
        self.state_definition.as_ref()
    }

    /// `defaultBlockState` — состояние со значениями по умолчанию
    /// (первое значение каждого свойства — Java `StateHolder`
    /// default берёт первое допустимое значение).
    pub fn default_block_state(&self) -> Option<state::BlockState> {
        let def = self.state_definition.as_ref()?;
        let mut values = state::StateHolder::new();
        for prop in def.properties() {
            if let Some(v) = prop.possible_values().first().and_then(|s| prop.parse(s)) {
                values.set_erased(prop.name(), v);
            }
        }
        Some(state::BlockState {
            block: std::sync::Arc::new(self.clone()),
            definition: def.clone(),
            values,
            fluid_state: crate::world::level::block::state::fluid_state::FluidState::empty(),
        })
    }

}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Block {}

impl std::hash::Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Block").field(&self.id).finish()
    }
}

impl Block {
    /// `Block.STREAM_CODEC` — `holderRegistry(BLOCK)`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, crate::core::holder::Holder<Block>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<Block>()
    }
}
