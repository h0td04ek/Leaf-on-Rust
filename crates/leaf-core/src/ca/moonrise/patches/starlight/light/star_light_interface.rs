//! `ca.spottedleaf.moonrise.patches.starlight.light.StarLightInterface` —
//! фасад StarLight: держит оба движка (sky/block), очереди задач и
//! layer-listener'ы. Полный порт Java-класса, включая `LightQueue`,
//! `ClientLightQueue`, `ServerLightQueue` и `ChunkTasks`.
//!
//! Планирование server-задач делегируется moonrise chunk-system scheduler'у
//! через [`StarLightTaskScheduler`] — сам scheduler портируется вместе с
//! chunk system; до его подключения `scheduler` равен `None` и задачи
//! накапливаются без исполнения (как в Java при отсутствии scheduler'а
//! очередь просто не планируется).

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::ca::moonrise::common::util::world_util::WorldUtil;
use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::data_layer::DataLayer;
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::light_layer::LightLayer;
use crate::world::level::lighting::layer_light_event_listener::{
    DummyLightLayerEventListener, LayerLightEventListener, LightEventListener,
};
use crate::world::level::lighting::level_light_engine::LevelLightEngine;

use super::star_light_engine::{EngineKind, StarLightEngine};
use crate::ca::moonrise::patches::starlight::chunk::StarlightChunk;

thread_local! {
    static SKY_PROPAGATOR: std::cell::RefCell<Option<StarLightEngine>> =
        const { std::cell::RefCell::new(None) };
    static BLOCK_PROPAGATOR: std::cell::RefCell<Option<StarLightEngine>> =
        const { std::cell::RefCell::new(None) };
}

/// `ca.spottedleaf.concurrentutil.executor.PrioritisedExecutor.Priority`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// `COMPLETING(-1)` — задача завершена/завершается; не для планирования.
    Completing = -1,
    /// `BLOCKING` — блокирует time-critical поток.
    Blocking,
    /// `HIGHEST`.
    Highest,
    /// `HIGHER`.
    Higher,
    /// `HIGH`.
    High,
    /// `NORMAL` — приоритет по умолчанию.
    Normal,
    /// `LOW`.
    Low,
    /// `LOWER`.
    Lower,
    /// `LOWEST`.
    Lowest,
    /// `IDLE`.
    Idle,
}

/// `PrioritisedExecutor.PrioritisedTask` — задача планировщика.
pub trait PrioritisedLightTask: Send + Sync {
    /// `queue()`.
    fn queue(&self);
    /// `cancel()`.
    fn cancel(&self) -> bool;
    /// `getPriority()`.
    fn get_priority(&self) -> Priority;
    /// `lowerPriority(priority)`.
    fn lower_priority(&self, priority: Priority);
    /// `setPriority(priority)`.
    fn set_priority(&self, priority: Priority);
    /// `raisePriority(priority)`.
    fn raise_priority(&self, priority: Priority);
}

/// Мост к moonrise chunk-system scheduler'у
/// (`moonrise$getChunkTaskScheduler().radiusAwareScheduler`).
/// Реализация появится с портом `ChunkTaskScheduler`; интерфейс полный.
pub trait StarLightTaskScheduler: Send + Sync {
    /// `radiusAwareScheduler.createTask(chunkX, chunkZ, writeRadius, task, priority)`.
    fn create_task(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        write_radius: i32,
        task: Arc<dyn Fn() + Send + Sync>,
        priority: Priority,
    ) -> Arc<dyn PrioritisedLightTask>;
}

// ============================================================
// ChunkTasks
// ============================================================

/// `LightQueue.ChunkTasks` — задачи одного чанка.
pub struct ChunkTasks {
    /// `chunkCoordinate`.
    pub chunk_coordinate: i64,
    /// `changedPositions`.
    changed_positions: HashSet<BlockPos>,
    /// `changedSectionSet` — `Boolean[]` → `Vec<Option<bool>>`.
    changed_section_set: Option<Vec<Option<bool>>>,
    /// `queuedEdgeChecksSky`/`Block` — `ShortOpenHashSet` → `HashSet<i16>`.
    queued_edge_checks_sky: Option<HashSet<i16>>,
    /// `queuedEdgeChecksBlock`.
    queued_edge_checks_block: Option<HashSet<i16>>,
    /// `lightTasks` — `BooleanSupplier` → `Fn() -> bool`.
    light_tasks: Option<Vec<Arc<dyn Fn() -> bool + Send + Sync>>>,
    /// `onComplete` — `MultiThreadedQueue<Runnable>` → FIFO.
    on_complete: VecDeque<Box<dyn FnOnce() + Send>>,
}

/// Handle на `ChunkTasks` (Java возвращает сам объект).
pub type ChunkTasksHandle = Arc<Mutex<ChunkTasks>>;

impl ChunkTasks {
    fn new(chunk_coordinate: i64) -> Self {
        Self {
            chunk_coordinate,
            changed_positions: HashSet::new(),
            changed_section_set: None,
            queued_edge_checks_sky: None,
            queued_edge_checks_block: None,
            light_tasks: None,
            on_complete: VecDeque::new(),
        }
    }

    /// `addChangedPosition(pos)`.
    pub fn add_changed_position(&mut self, pos: BlockPos) {
        self.changed_positions.insert(pos);
    }

    /// `setChangedSection(y, newEmptyValue)`.
    /// `newEmptyValue == None` соответствует Java `null` (секция без изменений).
    pub fn set_changed_section(
        &mut self,
        y: i32,
        new_empty_value: Option<bool>,
        min_section: i32,
        total_sections: i32,
    ) {
        if self.changed_section_set.is_none() {
            self.changed_section_set = Some(vec![None; total_sections as usize]);
        }
        let set = self.changed_section_set.as_mut().unwrap();
        let idx = (y - min_section) as usize;
        if idx < set.len() {
            set[idx] = new_empty_value;
        }
    }

    /// `addLightTask(lightTask)`.
    pub fn add_light_task(&mut self, light_task: Arc<dyn Fn() -> bool + Send + Sync>) {
        self.light_tasks.get_or_insert_with(Vec::new).push(light_task);
    }

    /// `addEdgeChecksSky(values)`.
    pub fn add_edge_checks_sky(&mut self, values: &[i16]) {
        self.queued_edge_checks_sky
            .get_or_insert_with(HashSet::new)
            .extend(values.iter().copied());
    }

    /// `addEdgeChecksBlock(values)`.
    pub fn add_edge_checks_block(&mut self, values: &[i16]) {
        self.queued_edge_checks_block
            .get_or_insert_with(HashSet::new)
            .extend(values.iter().copied());
    }

    /// `addOnCompleteCallback(callback)`.
    pub fn add_on_complete_callback(&mut self, callback: Box<dyn FnOnce() + Send>) {
        self.on_complete.push_back(callback);
    }

    /// `runTasks()` — выполняет световые задачи чанка.
    pub fn run_tasks(handle: &ChunkTasksHandle, interface: &Arc<StarLightInterface>) {
        // фаза 1: забираем входные данные
        let (positions, mut section_changes, edge_sky, edge_block, light_tasks) = {
            let mut this = handle.lock().unwrap();
            (
                std::mem::take(&mut this.changed_positions),
                this.changed_section_set.take(),
                this.queued_edge_checks_sky.take(),
                this.queued_edge_checks_block.take(),
                this.light_tasks.take(),
            )
        };

        let mut lit_chunk = false;
        if let Some(tasks) = &light_tasks {
            for run in tasks {
                if run() {
                    lit_chunk = true;
                    break;
                }
            }
        }

        if !lit_chunk {
            let mut sky_engine = interface.get_sky_light_engine();
            let mut block_engine = interface.get_block_light_engine();

            let chunk_coordinate = handle.lock().unwrap().chunk_coordinate;
            let chunk_x = (chunk_coordinate >> 32) as i32;
            let chunk_z = chunk_coordinate as i32;

            let has_changes =
                !positions.is_empty() || section_changes.is_some();

            if let Some(engine) = sky_engine.as_mut() {
                if has_changes {
                    engine.blocks_changed_in_chunk(
                        interface.get_light_access(),
                        chunk_x,
                        chunk_z,
                        &positions,
                        section_changes.as_mut().map(|v| v.as_mut_slice()),
                    );
                }
                if let Some(sections) = edge_sky {
                    let v: Vec<i16> = sections.into_iter().collect();
                    engine.check_chunk_edges_with_sections(
                        interface.get_light_access(),
                        chunk_x,
                        chunk_z,
                        &v,
                    );
                }
            }
            if let Some(engine) = block_engine.as_mut() {
                if has_changes {
                    engine.blocks_changed_in_chunk(
                        interface.get_light_access(),
                        chunk_x,
                        chunk_z,
                        &positions,
                        section_changes.as_mut().map(|v| v.as_mut_slice()),
                    );
                }
                if let Some(sections) = edge_block {
                    let v: Vec<i16> = sections.into_iter().collect();
                    engine.check_chunk_edges_with_sections(
                        interface.get_light_access(),
                        chunk_x,
                        chunk_z,
                        &v,
                    );
                }
            }

            interface.release_sky_light_engine(sky_engine);
            interface.release_block_light_engine(block_engine);
        }

        // onComplete — FIFO
        loop {
            let run = handle.lock().unwrap().on_complete.pop_front();
            match run {
                Some(f) => f(),
                None => break,
            }
        }
    }
}

// ============================================================
// LightQueue
// ============================================================

/// `StarLightInterface.LightQueue` — очередь световых задач.
pub trait LightQueue: Send + Sync {
    /// `isEmpty()`.
    fn is_empty(&self) -> bool;
    /// `queueBlockChange(pos)`.
    fn queue_block_change(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: BlockPos,
    ) -> ChunkTasksHandle;
    /// `queueSectionChange(pos, newEmptyValue)`.
    fn queue_section_change(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        new_empty_value: bool,
    ) -> ChunkTasksHandle;
    /// `queueChunkSkylightEdgeCheck(pos, sections)`.
    fn queue_chunk_skylight_edge_check(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle;
    /// `queueChunkBlocklightEdgeCheck(pos, sections)`.
    fn queue_chunk_blocklight_edge_check(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle;
}

/// `StarLightInterface.ClientLightQueue`.
pub struct ClientLightQueue {
    /// `chunkTasks` — `LinkedOpenHashMap` → insertion-ordered (map + FIFO-ключи).
    chunk_tasks: Mutex<(HashMap<i64, ChunkTasksHandle>, VecDeque<i64>)>,
}

impl ClientLightQueue {
    fn new() -> Self {
        Self {
            chunk_tasks: Mutex::new((HashMap::new(), VecDeque::new())),
        }
    }

    /// `drainTasks()`.
    pub fn drain_tasks(&self, interface: &Arc<StarLightInterface>) {
        loop {
            let task = {
                let mut map = self.chunk_tasks.lock().unwrap();
                // removeFirst — самый старый ключ
                let mut found = None;
                while let Some(key) = map.1.pop_front() {
                    if let Some(t) = map.0.remove(&key) {
                        found = Some(t);
                        break;
                    }
                }
                found
            };
            match task {
                Some(t) => ChunkTasks::run_tasks(&t, interface),
                None => break,
            }
        }
    }

    /// `getOrCreateChunkTasks(chunkCoordinate)`.
    fn get_or_create(&self, key: i64) -> ChunkTasksHandle {
        let mut map = self.chunk_tasks.lock().unwrap();
        if let Some(t) = map.0.get(&key) {
            return t.clone();
        }
        let tasks: ChunkTasksHandle = Arc::new(Mutex::new(ChunkTasks::new(key)));
        map.0.insert(key, tasks.clone());
        map.1.push_back(key);
        tasks
    }
}

impl LightQueue for ClientLightQueue {
    fn is_empty(&self) -> bool {
        self.chunk_tasks.lock().unwrap().0.is_empty()
    }

    fn queue_block_change(
        &self,
        _interface: &Arc<StarLightInterface>,
        pos: BlockPos,
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.get_x() >> 4, pos.get_z() >> 4);
        let tasks = self.get_or_create(key);
        tasks.lock().unwrap().add_changed_position(pos);
        tasks
    }

    fn queue_section_change(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        new_empty_value: bool,
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let tasks = self.get_or_create(key);
        let (min_section, total) = (interface.min_section, interface.max_section - interface.min_section + 1);
        tasks.lock().unwrap().set_changed_section(
            pos.y,
            Some(new_empty_value),
            min_section,
            total,
        );
        tasks
    }

    fn queue_chunk_skylight_edge_check(
        &self,
        _interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let tasks = self.get_or_create(key);
        tasks.lock().unwrap().add_edge_checks_sky(sections);
        tasks
    }

    fn queue_chunk_blocklight_edge_check(
        &self,
        _interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let tasks = self.get_or_create(key);
        tasks.lock().unwrap().add_edge_checks_block(sections);
        tasks
    }
}

/// `StarLightInterface.ServerLightQueue`.
pub struct ServerLightQueue {
    /// `chunkTasks`.
    chunk_tasks: Mutex<HashMap<i64, Arc<ServerChunkTasks>>>,
    /// `scheduler` — moonrise chunk-task scheduler (опционален до порта
    /// chunk system; без него задачи накапливаются, но не планируются).
    scheduler: Option<Arc<dyn StarLightTaskScheduler>>,
    /// Self-Weak для создания задач из `&self`-методов трейта.
    me: OnceLock<Weak<ServerLightQueue>>,
}

/// `ServerLightQueue.ServerChunkTasks`.
pub struct ServerChunkTasks {
    /// `chunkTasks` — общий handle на `ChunkTasks`.
    inner: ChunkTasksHandle,
    /// `ticketAdded`.
    ticket_added: AtomicBool,
    /// `task` — `PrioritisedTask` планировщика.
    task: Mutex<Option<Arc<dyn PrioritisedLightTask>>>,
}

impl ServerChunkTasks {
    fn new(
        chunk_coordinate: i64,
        scheduler: Option<&dyn StarLightTaskScheduler>,
        priority: Priority,
        queue: Weak<ServerLightQueue>,
        interface: Weak<StarLightInterface>,
    ) -> Self {
        let inner: ChunkTasksHandle = Arc::new(Mutex::new(ChunkTasks::new(chunk_coordinate)));
        let task = scheduler.map(|s| {
            let chunk_x = (chunk_coordinate >> 32) as i32;
            let chunk_z = chunk_coordinate as i32;
            let write_radius = ChunkStatus::Light.write_radius();
            let inner_clone = inner.clone();
            let queue_clone = queue.clone();
            let interface_clone = interface.clone();
            s.create_task(
                chunk_x,
                chunk_z,
                write_radius,
                Arc::new(move || {
                    // ServerChunkTasks.run(): удалить себя из очереди и выполнить
                    if let Some(q) = queue_clone.upgrade() {
                        q.chunk_tasks.lock().unwrap().remove(&chunk_coordinate);
                    }
                    if let Some(interface) = interface_clone.upgrade() {
                        ChunkTasks::run_tasks(&inner_clone, &interface);
                    }
                }),
                priority,
            )
        });
        Self {
            inner,
            ticket_added: AtomicBool::new(false),
            task: Mutex::new(task),
        }
    }

    /// `run()` — вызывается планировщиком; логика в замыкании выше.
    /// Оставлен для паритета с Java API.
    #[allow(dead_code)]
    fn run(&self, interface: &Arc<StarLightInterface>) {
        ChunkTasks::run_tasks(&self.inner, interface);
    }

    /// `markTicketAdded()`.
    pub fn mark_ticket_added(&self) -> bool {
        !self.ticket_added.swap(true, Ordering::SeqCst)
    }

    /// `schedule()`.
    pub fn schedule(&self) {
        if let Some(t) = self.task.lock().unwrap().as_ref() {
            t.queue();
        }
    }

    /// `cancel()`.
    pub fn cancel(&self) -> bool {
        self.task
            .lock()
            .unwrap()
            .as_ref()
            .map(|t| t.cancel())
            .unwrap_or(false)
    }
}

impl ServerLightQueue {
    fn new(scheduler: Option<Arc<dyn StarLightTaskScheduler>>) -> Self {
        Self {
            chunk_tasks: Mutex::new(HashMap::new()),
            scheduler,
            me: OnceLock::new(),
        }
    }

    /// `Weak` на себя (устанавливается после `Arc::new`).
    pub fn init_self_weak(self: &Arc<Self>) {
        let _ = self.me.set(Arc::downgrade(self));
    }

    fn arc_self(&self) -> Arc<ServerLightQueue> {
        self.me.get().and_then(|w| w.upgrade()).expect(
            "ServerLightQueue::init_self_weak was not called",
        )
    }

    /// `computeOrCreateChunkTasks(chunkPos, priority)`.
    fn compute_or_create(
        self: &Arc<Self>,
        interface: &Arc<StarLightInterface>,
        key: i64,
        priority: Priority,
    ) -> Arc<ServerChunkTasks> {
        let mut map = self.chunk_tasks.lock().unwrap();
        if let Some(ret) = map.get(&key) {
            return ret.clone();
        }
        let ret = Arc::new(ServerChunkTasks::new(
            key,
            self.scheduler.as_deref(),
            priority,
            Arc::downgrade(self),
            Arc::downgrade(interface),
        ));
        map.insert(key, ret.clone());
        ret
    }

    /// `lowerPriority(chunkX, chunkZ, priority)`.
    pub fn lower_priority(&self, chunk_x: i32, chunk_z: i32, priority: Priority) {
        let key = ChunkPos::as_long_static(chunk_x, chunk_z);
        let map = self.chunk_tasks.lock().unwrap();
        if let Some(task) = map.get(&key) {
            if let Some(t) = task.task.lock().unwrap().as_ref() {
                t.lower_priority(priority);
            }
        }
    }

    /// `setPriority(chunkX, chunkZ, priority)`.
    pub fn set_priority(&self, chunk_x: i32, chunk_z: i32, priority: Priority) {
        let key = ChunkPos::as_long_static(chunk_x, chunk_z);
        let map = self.chunk_tasks.lock().unwrap();
        if let Some(task) = map.get(&key) {
            if let Some(t) = task.task.lock().unwrap().as_ref() {
                t.set_priority(priority);
            }
        }
    }

    /// `raisePriority(chunkX, chunkZ, priority)`.
    pub fn raise_priority(&self, chunk_x: i32, chunk_z: i32, priority: Priority) {
        let key = ChunkPos::as_long_static(chunk_x, chunk_z);
        let map = self.chunk_tasks.lock().unwrap();
        if let Some(task) = map.get(&key) {
            if let Some(t) = task.task.lock().unwrap().as_ref() {
                t.raise_priority(priority);
            }
        }
    }

    /// `getPriority(chunkX, chunkZ)` — `Priority.COMPLETING` если задачи нет.
    pub fn get_priority(&self, chunk_x: i32, chunk_z: i32) -> Priority {
        let key = ChunkPos::as_long_static(chunk_x, chunk_z);
        let map = self.chunk_tasks.lock().unwrap();
        match map.get(&key) {
            Some(task) => task
                .task
                .lock()
                .unwrap()
                .as_ref()
                .map(|t| t.get_priority())
                .unwrap_or(Priority::Completing),
            None => Priority::Completing,
        }
    }

    /// `queueChunkLightTask(chunkPos, lightTask, priority)` — Leaf-метод.
    /// Возвращает `ServerChunkTasks`.
    pub fn queue_chunk_light_task(
        self: &Arc<Self>,
        interface: &Arc<StarLightInterface>,
        pos: ChunkPos,
        light_task: Arc<dyn Fn() -> bool + Send + Sync>,
        priority: Priority,
    ) -> Arc<ServerChunkTasks> {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let ret = self.compute_or_create(interface, key, priority);
        ret.inner.lock().unwrap().add_light_task(light_task);
        ret.schedule();
        ret
    }
}

impl LightQueue for ServerLightQueue {
    fn is_empty(&self) -> bool {
        self.chunk_tasks.lock().unwrap().is_empty()
    }

    fn queue_block_change(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: BlockPos,
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.get_x() >> 4, pos.get_z() >> 4);
        let queue = self.arc_self();
        let ret = queue.compute_or_create(interface, key, Priority::Normal);
        ret.inner.lock().unwrap().add_changed_position(pos);
        ret.schedule();
        ret.inner.clone()
    }

    fn queue_section_change(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        new_empty_value: bool,
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let queue = self.arc_self();
        let ret = queue.compute_or_create(interface, key, Priority::Normal);
        let (min_section, total) =
            (interface.min_section, interface.max_section - interface.min_section + 1);
        ret.inner.lock().unwrap().set_changed_section(
            pos.y,
            Some(new_empty_value),
            min_section,
            total,
        );
        ret.schedule();
        ret.inner.clone()
    }

    fn queue_chunk_skylight_edge_check(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let queue = self.arc_self();
        let ret = queue.compute_or_create(interface, key, Priority::Normal);
        ret.inner.lock().unwrap().add_edge_checks_sky(sections);
        ret.schedule();
        ret.inner.clone()
    }

    fn queue_chunk_blocklight_edge_check(
        &self,
        interface: &Arc<StarLightInterface>,
        pos: SectionPos,
        sections: &[i16],
    ) -> ChunkTasksHandle {
        let key = ChunkPos::as_long_static(pos.x, pos.z);
        let queue = self.arc_self();
        let ret = queue.compute_or_create(interface, key, Priority::Normal);
        ret.inner.lock().unwrap().add_edge_checks_block(sections);
        ret.schedule();
        ret.inner.clone()
    }
}

// ============================================================
// StarLightInterface
// ============================================================

/// `StarLightInterface` — фасад освещения Sky/Block Starlight.
pub struct StarLightInterface {
    /// `lightAccess`.
    pub light_access: Arc<dyn LightChunkGetter>,
    /// `world` — `None` = пустой мир.
    world: Option<Arc<dyn LevelReader>>,
    /// `lightQueue`.
    light_queue: LightQueueKind,
    /// `skyReader`.
    sky_reader: OnceLock<Arc<dyn LayerLightEventListener>>,
    /// `blockReader`.
    block_reader: OnceLock<Arc<dyn LayerLightEventListener>>,
    /// `isClientSide`.
    is_client_side: bool,
    /// `minSection`/`maxSection`.
    pub min_section: i32,
    /// `maxSection`.
    pub max_section: i32,
    /// `minLightSection`.
    pub min_light_section: i32,
    /// `maxLightSection`.
    pub max_light_section: i32,
    /// `lightEngine`.
    pub light_engine: Option<Arc<LevelLightEngine>>,
    /// `hasBlockLight`/`hasSkyLight`.
    has_block_light: bool,
    /// `hasSkyLight`.
    has_sky_light: bool,
}

enum LightQueueKind {
    Client(Arc<ClientLightQueue>),
    Server(Arc<ServerLightQueue>),
}

impl StarLightInterface {
    /// `new(lightAccess, world, hasSkyLight, hasBlockLight, lightEngine)`.
    /// Возвращает `Arc`, чтобы readers могли держать `Weak` на интерфейс.
    /// `world` соответствует `lightAccess.getLevel()` в Java — передаётся
    /// явно, т.к. `LightChunkGetter::get_level` отдаёт `&dyn` без `Arc`.
    pub fn new_arc(
        light_access: Arc<dyn LightChunkGetter>,
        world: Option<Arc<dyn LevelReader>>,
        has_sky_light: bool,
        has_block_light: bool,
        light_engine: Option<Arc<LevelLightEngine>>,
    ) -> Arc<Self> {
        let is_client_side = match &world {
            None => true,
            Some(w) => w.is_client_side(),
        };
        let (min_section, max_section, min_light_section, max_light_section) = match &world {
            None => (-4, 19, -5, 20),
            Some(w) => (
                WorldUtil::get_min_section(w.as_ref()),
                WorldUtil::get_max_section(w.as_ref()),
                WorldUtil::get_min_light_section(w.as_ref()),
                WorldUtil::get_max_light_section(w.as_ref()),
            ),
        };

        // server queue только для server-миров (world != null && !isClientSide)
        let is_server = world
            .as_ref()
            .map(|w| !w.is_client_side())
            .unwrap_or(false);
        let scheduler = world
            .as_ref()
            .and_then(|w| w.moonrise_get_chunk_task_scheduler());
        let light_queue = if is_server {
            let q = Arc::new(ServerLightQueue::new(scheduler));
            q.init_self_weak();
            LightQueueKind::Server(q)
        } else {
            LightQueueKind::Client(Arc::new(ClientLightQueue::new()))
        };

        let interface = Arc::new(Self {
            light_access,
            world,
            light_queue,
            sky_reader: OnceLock::new(),
            block_reader: OnceLock::new(),
            is_client_side,
            min_section,
            max_section,
            min_light_section,
            max_light_section,
            light_engine,
            has_block_light,
            has_sky_light,
        });

        // инициализируем reader'ов с Weak-ссылкой на интерфейс;
        // для выключенных слоёв — Dummy (аналог конструктора Java)
        let sky: Arc<dyn LayerLightEventListener> = if has_sky_light {
            Arc::new(StarLightSkyReader {
                interface: Arc::downgrade(&interface),
            })
        } else {
            Arc::new(DummyLightLayerEventListener)
        };
        let block: Arc<dyn LayerLightEventListener> = if has_block_light {
            Arc::new(StarLightBlockReader {
                interface: Arc::downgrade(&interface),
            })
        } else {
            Arc::new(DummyLightLayerEventListener)
        };
        let _ = interface.sky_reader.set(sky);
        let _ = interface.block_reader.set(block);

        interface
    }

    /// `getClientLightQueue()`.
    pub fn get_client_light_queue(&self) -> Option<&Arc<ClientLightQueue>> {
        match &self.light_queue {
            LightQueueKind::Client(q) => Some(q),
            _ => None,
        }
    }

    /// `getServerLightQueue()`.
    pub fn get_server_light_queue(&self) -> Option<&Arc<ServerLightQueue>> {
        match &self.light_queue {
            LightQueueKind::Server(q) => Some(q),
            _ => None,
        }
    }

    /// `hasSkyLight()`.
    pub fn has_sky_light(&self) -> bool {
        self.has_sky_light
    }

    /// `hasBlockLight()`.
    pub fn has_block_light(&self) -> bool {
        self.has_block_light
    }

    /// `getSkyLightValue(blockPos, chunk)`.
    pub fn get_sky_light_value(&self, block_pos: &BlockPos, chunk: Option<&ChunkAccess>) -> u8 {
        if !self.has_sky_light {
            return 0;
        }
        let x = block_pos.get_x();
        let mut y = block_pos.get_y();
        let z = block_pos.get_z();

        let min_section = self.min_section;
        let max_section = self.max_section;
        let min_light_section = self.min_light_section;
        let max_light_section = self.max_light_section;

        let chunk = match chunk {
            Some(c)
                if (self.is_client_side || c.is_light_correct())
                    && c.get_persisted_status().is_or_after(ChunkStatus::Light) =>
            {
                c
            }
            _ => return 15,
        };

        let mut section_y = y >> 4;

        if section_y > max_light_section {
            return 15;
        }

        if section_y < min_light_section {
            section_y = min_light_section;
            y = section_y << 4;
        }

        let nibbles = chunk.starlight_get_sky_nibbles();
        let immediate = nibbles
            .get((section_y - min_light_section) as usize)
            .and_then(|n| n.as_ref());

        let immediate_is_null = match immediate {
            Some(n) => n.lock().unwrap().is_null_nibble_visible(),
            None => true,
        };

        if !immediate_is_null {
            if let Some(n) = immediate {
                return n.lock().unwrap().get_visible_xyz(x, y, z);
            }
        }

        let emptiness_map = match chunk.starlight_get_sky_emptiness_map() {
            Some(m) => m,
            None => return 15,
        };

        // are we above this chunk's lowest empty section?
        let mut lowest_y = min_light_section - 1;
        let mut curr_y = max_section;
        {
            let map = emptiness_map.lock().unwrap();
            while curr_y >= min_section {
                if !map[(curr_y - min_section) as usize] {
                    // should always be full lit here
                    lowest_y = curr_y;
                    break;
                }
                curr_y -= 1;
            }
        }

        if section_y > lowest_y {
            return 15;
        }

        // this nibble is going to depend solely on the skylight data above it
        // find first non-null data above (there does exist one, as we just found it above)
        let mut above_y = section_y + 1;
        while above_y <= max_light_section {
            if let Some(Some(n)) = nibbles.get((above_y - min_light_section) as usize) {
                let n = n.lock().unwrap();
                if !n.is_null_nibble_visible() {
                    return n.get_visible_xyz(x, 0, z);
                }
            }
            above_y += 1;
        }

        // should never reach here
        15
    }

    /// `getBlockLightValue(blockPos, chunk)`.
    pub fn get_block_light_value(&self, block_pos: &BlockPos, chunk: Option<&ChunkAccess>) -> u8 {
        if !self.has_block_light {
            return 0;
        }
        let y = block_pos.get_y();
        let cy = y >> 4;

        let min_light_section = self.min_light_section;
        let max_light_section = self.max_light_section;

        if cy < min_light_section || cy > max_light_section {
            return 0;
        }

        let chunk = match chunk {
            Some(c) => c,
            None => return 0,
        };

        let nibbles = chunk.starlight_get_block_nibbles();
        match nibbles
            .get((cy - min_light_section) as usize)
            .and_then(|n| n.as_ref())
        {
            Some(n) => n
                .lock()
                .unwrap()
                .get_visible_xyz(block_pos.get_x(), y, block_pos.get_z()),
            None => 0,
        }
    }

    /// `getRawBrightness(pos, ambientDarkness)`.
    pub fn get_raw_brightness(&self, pos: &BlockPos, ambient_darkness: i32) -> i32 {
        let chunk = self.get_any_chunk_now(pos.get_x() >> 4, pos.get_z() >> 4);

        let sky = self.get_sky_light_value(pos, chunk.as_deref()) as i32 - ambient_darkness;
        // Don't fetch the block light level if the skylight level is 15,
        // since the value will never be higher.
        if sky == 15 {
            return 15;
        }
        let block = self.get_block_light_value(pos, chunk.as_deref()) as i32;
        sky.max(block)
    }

    /// `getSkyReader()`.
    pub fn get_sky_reader(&self) -> Arc<dyn LayerLightEventListener> {
        self.sky_reader.get().cloned().unwrap()
    }

    /// `getBlockReader()`.
    pub fn get_block_reader(&self) -> Arc<dyn LayerLightEventListener> {
        self.block_reader.get().cloned().unwrap()
    }

    /// `isClientSide()`.
    pub fn is_client_side(&self) -> bool {
        self.is_client_side
    }

    /// `getAnyChunkNow(chunkX, chunkZ)`.
    pub fn get_any_chunk_now(&self, chunk_x: i32, chunk_z: i32) -> Option<Arc<ChunkAccess>> {
        let world = self.world.as_ref()?;
        world.moonrise_get_any_chunk_if_loaded(chunk_x, chunk_z)
    }

    /// `hasUpdates()`.
    pub fn has_updates(&self) -> bool {
        !self.light_queue_is_empty()
    }

    fn light_queue_is_empty(&self) -> bool {
        match &self.light_queue {
            LightQueueKind::Client(q) => q.is_empty(),
            LightQueueKind::Server(q) => q.is_empty(),
        }
    }

    /// `getWorld()`.
    pub fn get_world(&self) -> Option<&Arc<dyn LevelReader>> {
        self.world.as_ref()
    }

    /// `getLightAccess()`.
    pub fn get_light_access(&self) -> &dyn LightChunkGetter {
        self.light_access.as_ref()
    }

    /// `getLightAccess()` как `Arc` (для внешнего wiring'а).
    pub fn get_light_access_arc(&self) -> &Arc<dyn LightChunkGetter> {
        &self.light_access
    }

    /// `getSkyLightEngine()` — движок из thread-local пула.
    pub fn get_sky_light_engine(&self) -> Option<StarLightEngine> {
        if !self.has_sky_light || self.world.is_none() {
            return None;
        }
        let mut ret = SKY_PROPAGATOR.with(|p| p.borrow_mut().take());
        if ret.is_none() {
            ret = Some(StarLightEngine::new(EngineKind::Sky));
        }
        let mut engine = ret.unwrap();
        engine.set_world(self.world.as_deref());
        Some(engine)
    }

    /// `releaseSkyLightEngine(engine)`.
    pub fn release_sky_light_engine(&self, engine: Option<StarLightEngine>) {
        if let Some(mut e) = engine {
            e.set_world(None);
            if !self.has_sky_light || self.world.is_none() {
                return;
            }
            SKY_PROPAGATOR.with(|p| {
                let mut p = p.borrow_mut();
                if p.is_none() {
                    *p = Some(e);
                }
            });
        } else if !(self.has_sky_light && self.world.is_some()) {
            return;
        }
    }

    /// `getBlockLightEngine()`.
    pub fn get_block_light_engine(&self) -> Option<StarLightEngine> {
        if !self.has_block_light || self.world.is_none() {
            return None;
        }
        let mut ret = BLOCK_PROPAGATOR.with(|p| p.borrow_mut().take());
        if ret.is_none() {
            ret = Some(StarLightEngine::new(EngineKind::Block));
        }
        let mut engine = ret.unwrap();
        engine.set_world(self.world.as_deref());
        Some(engine)
    }

    /// `releaseBlockLightEngine(engine)`.
    pub fn release_block_light_engine(&self, engine: Option<StarLightEngine>) {
        if let Some(mut e) = engine {
            e.set_world(None);
            if !self.has_block_light || self.world.is_none() {
                return;
            }
            BLOCK_PROPAGATOR.with(|p| {
                let mut p = p.borrow_mut();
                if p.is_none() {
                    *p = Some(e);
                }
            });
        } else if !(self.has_block_light && self.world.is_some()) {
            return;
        }
    }

    /// `blockChange(pos)` → `LightQueue.ChunkTasks`.
    pub fn block_change(self: &Arc<Self>, pos: &BlockPos) -> Option<ChunkTasksHandle> {
        let world = self.world.as_ref()?;
        let min_y = WorldUtil::get_min_block_y(world.as_ref());
        let max_y = WorldUtil::get_max_block_y(world.as_ref());
        if pos.get_y() < min_y || pos.get_y() > max_y {
            // empty world / out of bounds
            return None;
        }
        Some(self.queue_block_change(pos))
    }

    fn queue_block_change(self: &Arc<Self>, pos: &BlockPos) -> ChunkTasksHandle {
        match &self.light_queue {
            LightQueueKind::Client(q) => q.queue_block_change(self, *pos),
            LightQueueKind::Server(q) => q.queue_block_change(self, *pos),
        }
    }

    /// `sectionChange(pos, newEmptyValue)`.
    pub fn section_change(
        self: &Arc<Self>,
        pos: SectionPos,
        new_empty_value: bool,
    ) -> Option<ChunkTasksHandle> {
        if self.world.is_none() {
            // empty world
            return None;
        }
        Some(match &self.light_queue {
            LightQueueKind::Client(q) => q.queue_section_change(self, pos, new_empty_value),
            LightQueueKind::Server(q) => q.queue_section_change(self, pos, new_empty_value),
        })
    }

    /// `forceLoadInChunk(chunk, emptySections)`.
    pub fn force_load_in_chunk(
        self: &Arc<Self>,
        chunk: &ChunkAccess,
        empty_sections: &mut [Option<bool>],
    ) {
        let mut sky_engine = self.get_sky_light_engine();
        let mut block_engine = self.get_block_light_engine();

        if let Some(engine) = sky_engine.as_mut() {
            engine.force_handle_empty_section_changes(
                self.get_light_access(),
                chunk,
                empty_sections,
            );
        }
        if let Some(engine) = block_engine.as_mut() {
            engine.force_handle_empty_section_changes(
                self.get_light_access(),
                chunk,
                empty_sections,
            );
        }

        self.release_sky_light_engine(sky_engine);
        self.release_block_light_engine(block_engine);
    }

    /// `loadInChunk(chunkX, chunkZ, emptySections)`.
    pub fn load_in_chunk(
        self: &Arc<Self>,
        chunk_x: i32,
        chunk_z: i32,
        empty_sections: &mut [Option<bool>],
    ) {
        let mut sky_engine = self.get_sky_light_engine();
        let mut block_engine = self.get_block_light_engine();

        if let Some(engine) = sky_engine.as_mut() {
            engine.handle_empty_section_changes(
                self.get_light_access(),
                chunk_x,
                chunk_z,
                empty_sections,
            );
        }
        if let Some(engine) = block_engine.as_mut() {
            engine.handle_empty_section_changes(
                self.get_light_access(),
                chunk_x,
                chunk_z,
                empty_sections,
            );
        }

        self.release_sky_light_engine(sky_engine);
        self.release_block_light_engine(block_engine);
    }

    /// `lightChunk(chunk, emptySections)`.
    pub fn light_chunk(
        self: &Arc<Self>,
        chunk: &ChunkAccess,
        empty_sections: &mut [Option<bool>],
    ) {
        let mut sky_engine = self.get_sky_light_engine();
        let mut block_engine = self.get_block_light_engine();

        if let Some(engine) = sky_engine.as_mut() {
            engine.light(self.get_light_access(), chunk, empty_sections);
        }
        if let Some(engine) = block_engine.as_mut() {
            engine.light(self.get_light_access(), chunk, empty_sections);
        }

        self.release_sky_light_engine(sky_engine);
        self.release_block_light_engine(block_engine);
    }

    /// `relightChunks(chunks, chunkLightCallback, onComplete)`.
    /// Sky-движок получает callback только если block-движка нет.
    pub fn relight_chunks(
        self: &Arc<Self>,
        chunks: &HashSet<ChunkPos>,
        chunk_light_callback: Option<&mut dyn FnMut(ChunkPos)>,
        on_complete: Option<&mut dyn FnMut(i32)>,
    ) {
        let mut sky_engine = self.get_sky_light_engine();
        let mut block_engine = self.get_block_light_engine();

        let has_block = block_engine.is_some();
        if has_block {
            // sky получает callback только если block-движка нет
            if let Some(engine) = sky_engine.as_mut() {
                engine.relight_chunks(self.get_light_access(), chunks, None, None);
            }
            if let Some(engine) = block_engine.as_mut() {
                engine.relight_chunks(
                    self.get_light_access(),
                    chunks,
                    chunk_light_callback,
                    on_complete,
                );
            }
        } else if let Some(engine) = sky_engine.as_mut() {
            engine.relight_chunks(
                self.get_light_access(),
                chunks,
                chunk_light_callback,
                on_complete,
            );
        }

        self.release_sky_light_engine(sky_engine);
        self.release_block_light_engine(block_engine);
    }

    /// `checkChunkEdges(chunkX, chunkZ)`.
    pub fn check_chunk_edges(self: &Arc<Self>, chunk_x: i32, chunk_z: i32) {
        self.check_sky_edges(chunk_x, chunk_z);
        self.check_block_edges(chunk_x, chunk_z);
    }

    /// `checkSkyEdges(chunkX, chunkZ)`.
    pub fn check_sky_edges(self: &Arc<Self>, chunk_x: i32, chunk_z: i32) {
        let mut sky_engine = self.get_sky_light_engine();
        if let Some(engine) = sky_engine.as_mut() {
            engine.check_chunk_edges(self.get_light_access(), chunk_x, chunk_z);
        }
        self.release_sky_light_engine(sky_engine);
    }

    /// `checkBlockEdges(chunkX, chunkZ)`.
    pub fn check_block_edges(self: &Arc<Self>, chunk_x: i32, chunk_z: i32) {
        let mut block_engine = self.get_block_light_engine();
        if let Some(engine) = block_engine.as_mut() {
            engine.check_chunk_edges(self.get_light_access(), chunk_x, chunk_z);
        }
        self.release_block_light_engine(block_engine);
    }

    /// `propagateChanges()` — только для client queue.
    pub fn propagate_changes(self: &Arc<Self>) {
        if let LightQueueKind::Client(q) = &self.light_queue {
            q.drain_tasks(self);
        }
        // else: invalid usage, although we won't throw because mods...
    }
}

// ============================================================
// Reader'ы
// ============================================================

/// `skyReader` — `LayerLightEventListener` для sky-слоя.
struct StarLightSkyReader {
    interface: Weak<StarLightInterface>,
}

impl LayerLightEventListener for StarLightSkyReader {
    fn get_data_layer_data(&self, pos: SectionPos) -> Option<DataLayer> {
        let interface = self.interface.upgrade()?;
        let chunk = interface.get_any_chunk_now(pos.x, pos.z)?;
        if (!interface.is_client_side() && !chunk.is_light_correct())
            || !chunk.get_persisted_status().is_or_after(ChunkStatus::Light)
        {
            return None;
        }

        let section_y = pos.y;
        if section_y > interface.max_light_section || section_y < interface.min_light_section {
            return None;
        }

        if chunk.starlight_get_sky_emptiness_map().is_none() {
            return None;
        }

        let nibbles = chunk.starlight_get_sky_nibbles();
        let nibble = nibbles.get((section_y - interface.min_light_section) as usize)?;
        nibble
            .as_ref()
            .and_then(|n| n.lock().unwrap().to_vanilla_nibble())
    }

    fn get_light_value(&self, pos: BlockPos) -> i32 {
        let interface = match self.interface.upgrade() {
            Some(i) => i,
            None => return 0,
        };
        let chunk = interface.get_any_chunk_now(pos.get_x() >> 4, pos.get_z() >> 4);
        interface.get_sky_light_value(&pos, chunk.as_deref()) as i32
    }
}

impl LightEventListener for StarLightSkyReader {
    fn check_block(&self, pos: BlockPos) {
        if let Some(interface) = self.interface.upgrade() {
            if let Some(engine) = &interface.light_engine {
                engine.check_light(&pos, LightLayer::Sky);
            }
        }
    }

    fn has_light_work(&self) -> bool {
        self.interface
            .upgrade()
            .map(|i| i.has_updates())
            .unwrap_or(false)
    }

    fn run_light_updates(&self) -> i32 {
        unimplemented!("runLightUpdates is not supported by StarLightInterface")
    }

    fn update_section_status(&self, pos: SectionPos, not_ready: bool) {
        if let Some(interface) = self.interface.upgrade() {
            interface.section_change(pos, not_ready);
        }
    }

    fn set_light_enabled(&self, _chunk_pos: ChunkPos, _light_enabled: bool) {
        unimplemented!("setLightEnabled is not supported by StarLightInterface")
    }

    fn propagate_light_sources(&self, _chunk_pos: ChunkPos) {
        unimplemented!("propagateLightSources is not supported by StarLightInterface")
    }
}

/// `blockReader` — `LayerLightEventListener` для block-слоя.
struct StarLightBlockReader {
    interface: Weak<StarLightInterface>,
}

impl LayerLightEventListener for StarLightBlockReader {
    fn get_data_layer_data(&self, pos: SectionPos) -> Option<DataLayer> {
        let interface = self.interface.upgrade()?;
        let chunk = interface.get_any_chunk_now(pos.x, pos.z)?;

        if pos.y < interface.min_light_section || pos.y > interface.max_light_section {
            return None;
        }

        let nibbles = chunk.starlight_get_block_nibbles();
        let nibble = nibbles.get((pos.y - interface.min_light_section) as usize)?;
        nibble
            .as_ref()
            .and_then(|n| n.lock().unwrap().to_vanilla_nibble())
    }

    fn get_light_value(&self, pos: BlockPos) -> i32 {
        let interface = match self.interface.upgrade() {
            Some(i) => i,
            None => return 0,
        };
        let chunk = interface.get_any_chunk_now(pos.get_x() >> 4, pos.get_z() >> 4);
        interface.get_block_light_value(&pos, chunk.as_deref()) as i32
    }
}

impl LightEventListener for StarLightBlockReader {
    fn check_block(&self, pos: BlockPos) {
        if let Some(interface) = self.interface.upgrade() {
            if let Some(engine) = &interface.light_engine {
                engine.check_light(&pos, LightLayer::Block);
            }
        }
    }

    fn has_light_work(&self) -> bool {
        self.interface
            .upgrade()
            .map(|i| i.has_updates())
            .unwrap_or(false)
    }

    fn run_light_updates(&self) -> i32 {
        unimplemented!("runLightUpdates is not supported by StarLightInterface")
    }

    fn update_section_status(&self, pos: SectionPos, not_ready: bool) {
        if let Some(interface) = self.interface.upgrade() {
            interface.section_change(pos, not_ready);
        }
    }

    fn set_light_enabled(&self, _chunk_pos: ChunkPos, _light_enabled: bool) {
        unimplemented!("setLightEnabled is not supported by StarLightInterface")
    }

    fn propagate_light_sources(&self, _chunk_pos: ChunkPos) {
        unimplemented!("propagateLightSources is not supported by StarLightInterface")
    }
}
