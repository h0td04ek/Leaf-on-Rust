//! Перенос `net.minecraft.world.level.block`.

pub mod rotation;
pub mod entity;
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
}

/// Подмножество `BlockBehaviour.Properties` — флаги,
/// влияющие на wire/предикаты.
#[derive(Clone, Debug, Default)]
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
    /// `new`.
    pub fn new(id: crate::resources::identifier::Identifier) -> Self {
        Self {
            id,
            state_definition: None,
            properties: BlockProperties::default(),
        }
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
        })
    }

}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Block {}

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
