//! `net.minecraft.world.ticks.LevelTicks` — менеджер тиков уровня:
//! сбор тиков из чанков, drain по приоритету, лимит за тик,
//! `willTickThisTick`, `clearArea`/`copyArea`.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::core::vec3i::Vec3i;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;

use super::level_chunk_ticks::{LevelChunkTicks, OnTickAdded};
use super::scheduled_tick::{intra_tick_drain_order_cmp, ScheduledTick, TickKey, TickType};
use super::tick_access::{LevelTickAccess, TickAccess};

/// `LevelTicks<T>` — состояние за `Arc<Mutex>`, чтобы `onTickAdded`
/// callback мог обновлять `nextTickForContainer` (Java — прямой вызов
/// метода на этом же объекте).
pub struct LevelTicks<T: TickType> {
    /// `tickCheck` — `LongPredicate` (чанк должен тикать).
    tick_check: Arc<dyn Fn(i64) -> bool + Send + Sync>,
    /// Разделяемое состояние.
    state: Arc<Mutex<LevelTicksState<T>>>,
}

/// Внутреннее состояние `LevelTicks`.
struct LevelTicksState<T: TickType> {
    /// `allContainers` — chunkPos.toLong → LevelChunkTicks.
    all_containers: HashMap<i64, Arc<LevelChunkTicks<T>>>,
    /// `nextTickForContainer` — chunkPos.toLong → triggerTick головы.
    next_tick_for_container: HashMap<i64, i64>,
    /// `containersToTick` — PriorityQueue по INTRA_TICK_DRAIN_ORDER голов.
    containers_to_tick: BinaryHeap<Reverse<ContainerOrd<T>>>,
    /// `toRunThisTick`.
    to_run_this_tick: VecDeque<ScheduledTick<T>>,
    /// `alreadyRunThisTick`.
    already_run_this_tick: Vec<ScheduledTick<T>>,
    /// `toRunThisTickSet`.
    to_run_this_tick_set: HashSet<TickKey>,
}

/// Обёртка контейнера для priority-очереди; Ord читает `peek()` живьём
/// (как Java PriorityQueue с компаратором по peek).
struct ContainerOrd<T: TickType>(Arc<LevelChunkTicks<T>>);

impl<T: TickType> PartialEq for ContainerOrd<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: TickType> Eq for ContainerOrd<T> {}
impl<T: TickType> PartialOrd for ContainerOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: TickType> Ord for ContainerOrd<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse-семантика через вызов извне: здесь — прямой порядок
        // INTRA_TICK_DRAIN_ORDER по головам контейнеров.
        let pa = self.0.peek();
        let pb = other.0.peek();
        match (pa, pb) {
            (Some(a), Some(b)) => intra_tick_drain_order_cmp(&a, &b),
            (None, None) => std::cmp::Ordering::Equal,
            // в Java peek() == null в очереди не бывает; None считаем "меньше"
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
        }
    }
}

impl<T: TickType + Clone> LevelTicks<T> {
    /// `new(tickCheck)`.
    pub fn new(tick_check: Arc<dyn Fn(i64) -> bool + Send + Sync>) -> Self {
        Self {
            tick_check,
            state: Arc::new(Mutex::new(LevelTicksState {
                all_containers: HashMap::new(),
                next_tick_for_container: HashMap::new(),
                containers_to_tick: BinaryHeap::new(),
                to_run_this_tick: VecDeque::new(),
                already_run_this_tick: Vec::new(),
                to_run_this_tick_set: HashSet::new(),
            })),
        }
    }

    /// `addContainer(chunkPos, chunkTicks)`.
    /// `chunk_ticks` — уже созданный `Arc<Mutex<LevelChunkTicks>>`;
    /// если `None` — создаётся пустой.
    pub fn add_container(
        &self,
        chunk_pos: ChunkPos,
        chunk_ticks: Option<Arc<LevelChunkTicks<T>>>,
    ) -> Arc<LevelChunkTicks<T>> {
        let packed = chunk_pos.as_long();
        let container = chunk_ticks.unwrap_or_else(|| Arc::new(LevelChunkTicks::new()));

        // onTickAdded: если добавленный тик стал головой — обновить
        // nextTickForContainer (аналог chunkScheduleUpdater).
        let state_weak = Arc::downgrade(&self.state);
        let container_weak = Arc::downgrade(&container);
        let cb: OnTickAdded<T> = Arc::new(move |_| {
            let (Some(state), Some(container)) = (state_weak.upgrade(), container_weak.upgrade())
            else {
                return;
            };
            let mut s = state.lock().unwrap();
            if s.all_containers
                .get(&packed)
                .is_some_and(|c| Arc::ptr_eq(c, &container))
            {
                if let Some(head) = container.peek() {
                    s.next_tick_for_container.insert(packed, head.trigger_tick);
                }
            }
        });

        let mut s = self.state.lock().unwrap();
        if let Some(previous) = s.all_containers.insert(packed, container.clone()) {
            previous.set_on_tick_added(None);
        }
        container.set_on_tick_added(Some(cb));
        if let Some(head) = container.peek() {
            s.next_tick_for_container.insert(packed, head.trigger_tick);
        } else {
            s.next_tick_for_container.remove(&packed);
        }
        container
    }

    /// `removeContainer(chunkPos)`.
    pub fn remove_container(&self, chunk_pos: ChunkPos) {
        let packed = chunk_pos.as_long();
        let mut s = self.state.lock().unwrap();
        if let Some(container) = s.all_containers.remove(&packed) {
            s.next_tick_for_container.remove(&packed);
            container.set_on_tick_added(None);
        }
    }

    /// `updateContainerScheduling(tick)`.
    fn update_container_scheduling(&self, tick: &ScheduledTick<T>) {
        self.state.lock().unwrap().next_tick_for_container.insert(
            ChunkPos::as_long_static(tick.pos.get_x() >> 4, tick.pos.get_z() >> 4),
            tick.trigger_tick,
        );
    }

    /// `tick(gameTime, maxAllowedTicks, ticker)`.
    pub fn tick(
        &self,
        game_time: i64,
        max_allowed_ticks: usize,
        ticker: &mut dyn FnMut(BlockPos, T),
    ) {
        // collect
        self.collect_ticks(game_time, max_allowed_ticks);
        // run
        self.run_collected_ticks(ticker);
        // cleanup
        self.cleanup_after_tick();
    }

    /// `collectTicks`.
    fn collect_ticks(&self, game_time: i64, max_allowed_ticks: usize) {
        self.sort_containers_to_tick(game_time);
        self.drain_containers(game_time, max_allowed_ticks);
        self.reschedule_leftover_containers();
    }

    /// `sortContainersToTick`.
    fn sort_containers_to_tick(&self, game_time: i64) {
        let mut s = self.state.lock().unwrap();
        let keys: Vec<i64> = s.next_tick_for_container.keys().copied().collect();
        for key in keys {
            let Some(&long_value) = s.next_tick_for_container.get(&key) else {
                continue;
            };
            if long_value > game_time {
                continue;
            }
            let container_opt = s.all_containers.get(&key).cloned();
            match container_opt {
                None => {
                    s.next_tick_for_container.remove(&key);
                }
                Some(container) => {
                    let head = container.peek();
                    match head {
                        None => {
                            s.next_tick_for_container.remove(&key);
                        }
                        Some(tick) if tick.trigger_tick > game_time => {
                            s.next_tick_for_container.insert(key, tick.trigger_tick);
                        }
                        Some(_) => {
                            if (self.tick_check)(key) {
                                s.next_tick_for_container.remove(&key);
                                s.containers_to_tick.push(Reverse(ContainerOrd(container)));
                            }
                        }
                    }
                }
            }
        }
    }

    /// `drainContainers`.
    fn drain_containers(&self, game_time: i64, max_allowed_ticks: usize) {
        loop {
            if !self.can_schedule_more_ticks(max_allowed_ticks) {
                break;
            }
            let container = {
                let mut s = self.state.lock().unwrap();
                s.containers_to_tick.pop().map(|r| r.0)
            };
            let Some(container) = container else { break };

            // poll одного тика из контейнера
            let first = container.0.poll();
            let Some(first) = first else { continue };
            self.state.lock().unwrap().to_run_this_tick.push_back(first);

            self.drain_from_current_container(&container.0, game_time, max_allowed_ticks);

            let head = container.0.peek();
            if let Some(tick) = head {
                if tick.trigger_tick <= game_time && self.can_schedule_more_ticks(max_allowed_ticks)
                {
                    self.state
                        .lock()
                        .unwrap()
                        .containers_to_tick
                        .push(Reverse(container));
                } else {
                    self.update_container_scheduling(&tick);
                }
            }
        }
    }

    /// `rescheduleLeftoverContainers`.
    fn reschedule_leftover_containers(&self) {
        let mut s = self.state.lock().unwrap();
        let leftovers = std::mem::take(&mut s.containers_to_tick);
        for Reverse(c) in leftovers {
            if let Some(head) = c.0.peek() {
                s.next_tick_for_container.insert(
                    ChunkPos::as_long_static(head.pos.get_x() >> 4, head.pos.get_z() >> 4),
                    head.trigger_tick,
                );
            }
        }
    }

    /// `drainFromCurrentContainer`.
    fn drain_from_current_container(
        &self,
        container: &Arc<LevelChunkTicks<T>>,
        game_time: i64,
        max_allowed_ticks: usize,
    ) {
        if !self.can_schedule_more_ticks(max_allowed_ticks) {
            return;
        }
        // peek следующего контейнера в очереди для INTRA_TICK_DRAIN_ORDER
        let next_container_head = {
            let s = self.state.lock().unwrap();
            s.containers_to_tick.peek().and_then(|r| r.0 .0.peek())
        };

        loop {
            if !self.can_schedule_more_ticks(max_allowed_ticks) {
                break;
            }
            let head = container.peek();
            let Some(head) = head else { break };
            if head.trigger_tick > game_time {
                break;
            }
            if let Some(next) = &next_container_head {
                if intra_tick_drain_order_cmp(&head, next).is_gt() {
                    break;
                }
            }
            let tick = container.poll();
            if let Some(tick) = tick {
                self.state.lock().unwrap().to_run_this_tick.push_back(tick);
            }
        }
    }

    /// `scheduleForThisTick` — уже инлайнен при drain.

    /// `canScheduleMoreTicks`.
    fn can_schedule_more_ticks(&self, max_allowed_ticks: usize) -> bool {
        self.state.lock().unwrap().to_run_this_tick.len() < max_allowed_ticks
    }

    /// `runCollectedTicks`.
    fn run_collected_ticks(&self, ticker: &mut dyn FnMut(BlockPos, T)) {
        loop {
            let tick = self.state.lock().unwrap().to_run_this_tick.pop_front();
            let Some(tick) = tick else { break };
            {
                let mut s = self.state.lock().unwrap();
                s.to_run_this_tick_set.remove(&tick.key());
                s.already_run_this_tick.push(tick.clone());
            }
            ticker(tick.pos, tick.type_.clone());
        }
    }

    /// `cleanupAfterTick`.
    fn cleanup_after_tick(&self) {
        let mut s = self.state.lock().unwrap();
        // toRunThisTick уже пуст (всё заполловано)
        s.containers_to_tick.clear();
        s.already_run_this_tick.clear();
        s.to_run_this_tick_set.clear();
    }

    /// `forContainersInArea(area, action)`.
    fn for_containers_in_area(
        &self,
        area: &BoundingBox,
        action: &mut dyn FnMut(i64, &Arc<LevelChunkTicks<T>>),
    ) {
        let section_min_x = SectionPos::block_to_section_coord(area.min_x_());
        let section_min_z = SectionPos::block_to_section_coord(area.min_z_());
        let section_max_x = SectionPos::block_to_section_coord(area.max_x_());
        let section_max_z = SectionPos::block_to_section_coord(area.max_z_());
        let s = self.state.lock().unwrap();
        for i in section_min_x..=section_max_x {
            for j in section_min_z..=section_max_z {
                let packed = ChunkPos::as_long_static(i, j);
                if let Some(c) = s.all_containers.get(&packed) {
                    action(packed, c);
                }
            }
        }
    }

    /// `clearArea(area)`.
    pub fn clear_area(&self, area: &BoundingBox) {
        // собираем контейнеры в области через forContainersInArea
        let mut collected = Vec::new();
        self.for_containers_in_area(area, &mut |pos, c| collected.push((pos, c.clone())));
        for (pos, container) in collected {
            let before = container.peek();
            container.remove_if(&|t: &ScheduledTick<T>| area.is_inside_pos(t.pos));
            let after = container.peek();
            if tick_head_changed(&before, &after) {
                if let Some(t) = &after {
                    self.update_container_scheduling(t);
                } else {
                    self.state
                        .lock()
                        .unwrap()
                        .next_tick_for_container
                        .remove(&pos);
                }
            }
        }
        let mut s = self.state.lock().unwrap();
        let area = area.clone();
        s.already_run_this_tick
            .retain(|t| !area.is_inside_pos(t.pos));
        s.to_run_this_tick.retain(|t| !area.is_inside_pos(t.pos));
        s.to_run_this_tick_set
            .retain(|key| !area.is_inside_pos(key.pos));
    }

    /// `copyArea(area, offset)`.
    pub fn copy_area(&self, area: &BoundingBox, offset: Vec3i) {
        self.copy_area_from_self(area, offset);
    }

    fn copy_area_from_self(&self, area: &BoundingBox, offset: Vec3i) {
        // собираем все тики в области: alreadyRun + toRun + контейнеры
        let mut list: Vec<ScheduledTick<T>> = Vec::new();
        {
            let s = self.state.lock().unwrap();
            for t in &s.already_run_this_tick {
                if area.is_inside_pos(t.pos) {
                    list.push(t.clone());
                }
            }
            for t in &s.to_run_this_tick {
                if area.is_inside_pos(t.pos) {
                    list.push(t.clone());
                }
            }
        }
        let mut containers = Vec::new();
        self.for_containers_in_area(area, &mut |_, c| containers.push(c.clone()));
        for c in containers {
            for t in c.get_all() {
                if area.is_inside_pos(t.pos) {
                    list.push(t);
                }
            }
        }
        // subTickOrder: min → max+1 (сдвиг), как в Java
        if list.is_empty() {
            return;
        }
        let min = list.iter().map(|t| t.sub_tick_order).min().unwrap();
        let max = list.iter().map(|t| t.sub_tick_order).max().unwrap();
        for t in list {
            let new_pos = t.pos.offset(offset.x, offset.y, offset.z);
            self.schedule(ScheduledTick::new(
                t.type_.clone(),
                new_pos,
                t.trigger_tick,
                t.priority,
                t.sub_tick_order - min + max + 1,
            ));
        }
    }

    /// `copyAreaFrom(levelTicks, area, offset)`.
    pub fn copy_area_from(&self, other: &LevelTicks<T>, area: &BoundingBox, offset: Vec3i) {
        let mut list: Vec<ScheduledTick<T>> = Vec::new();
        {
            let s = other.state.lock().unwrap();
            for t in &s.already_run_this_tick {
                if area.is_inside_pos(t.pos) {
                    list.push(t.clone());
                }
            }
            for t in &s.to_run_this_tick {
                if area.is_inside_pos(t.pos) {
                    list.push(t.clone());
                }
            }
        }
        let mut containers = Vec::new();
        other.for_containers_in_area(area, &mut |_, c| containers.push(c.clone()));
        for c in containers {
            for t in c.get_all() {
                if area.is_inside_pos(t.pos) {
                    list.push(t);
                }
            }
        }
        if list.is_empty() {
            return;
        }
        let min = list.iter().map(|t| t.sub_tick_order).min().unwrap();
        let max = list.iter().map(|t| t.sub_tick_order).max().unwrap();
        for t in list {
            let new_pos = t.pos.offset(offset.x, offset.y, offset.z);
            self.schedule(ScheduledTick::new(
                t.type_.clone(),
                new_pos,
                t.trigger_tick,
                t.priority,
                t.sub_tick_order - min + max + 1,
            ));
        }
    }

    /// Доступ к контейнеру чанка (для LevelChunk::getBlockTicks).
    pub fn get_container(&self, chunk_pos: ChunkPos) -> Option<Arc<LevelChunkTicks<T>>> {
        self.state
            .lock()
            .unwrap()
            .all_containers
            .get(&chunk_pos.as_long())
            .cloned()
    }

    /// `count()`.
    pub fn count(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .all_containers
            .values()
            .map(|c| c.count())
            .sum()
    }
}

impl<T: TickType + Clone> TickAccess<T> for LevelTicks<T> {
    fn schedule(&self, tick: ScheduledTick<T>) {
        let packed = ChunkPos::as_long_static(tick.pos.get_x() >> 4, tick.pos.get_z() >> 4);
        let container = self
            .state
            .lock()
            .unwrap()
            .all_containers
            .get(&packed)
            .cloned();
        match container {
            Some(c) => c.schedule(tick),
            None => {
                // Util.logAndPauseIfInIde — логируем через eprintln в debug
                #[cfg(debug_assertions)]
                eprintln!(
                    "Trying to schedule tick in not loaded position {:?}",
                    tick.pos
                );
            }
        }
    }

    fn has_scheduled_tick(&self, pos: BlockPos, type_: &T) -> bool {
        let packed = ChunkPos::as_long_static(pos.get_x() >> 4, pos.get_z() >> 4);
        self.state
            .lock()
            .unwrap()
            .all_containers
            .get(&packed)
            .map(|c| c.has_scheduled_tick(pos, type_))
            .unwrap_or(false)
    }

    fn count(&self) -> usize {
        LevelTicks::count(self)
    }
}

impl<T: TickType + Clone> LevelTickAccess<T> for LevelTicks<T> {
    fn will_tick_this_tick(&self, pos: BlockPos, type_: &T) -> bool {
        self.calculate_tick_set_if_needed();
        self.state
            .lock()
            .unwrap()
            .to_run_this_tick_set
            .contains(&TickKey::of(pos, type_))
    }
}

impl<T: TickType + Clone> LevelTicks<T> {
    /// `calculateTickSetIfNeeded`.
    fn calculate_tick_set_if_needed(&self) {
        let mut s = self.state.lock().unwrap();
        if s.to_run_this_tick_set.is_empty() && !s.to_run_this_tick.is_empty() {
            s.to_run_this_tick_set = s.to_run_this_tick.iter().map(|t| t.key()).collect();
        }
    }
}

// helper: changed-head check для clear_area
fn tick_head_changed<T: TickType + Clone>(
    before: &Option<ScheduledTick<T>>,
    after: &Option<ScheduledTick<T>>,
) -> bool {
    match (before, after) {
        (None, None) => false,
        (Some(a), Some(b)) => !a.tick_eq_full(b),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::level::scheduled_tick_access::BlockTickType;
    use crate::world::ticks::{SavedTick, TickPriority};

    fn ty() -> BlockTickType {
        BlockTickType {
            id: "minecraft:stone".to_owned(),
        }
    }

    #[test]
    fn runs_earliest_ticks_and_resumes_after_the_per_tick_limit() {
        let ticks = LevelTicks::new(Arc::new(|_| true));
        let west = ChunkPos::new(-1, 0);
        let east = ChunkPos::new(0, 0);
        let late = BlockPos::new(-2, 0, 0);
        let early = BlockPos::new(-1, 0, 0);
        let high = BlockPos::ZERO;
        let saved = LevelChunkTicks::with_pending_ticks(vec![
            SavedTick::new(ty(), late, 20, TickPriority::ExtremelyHigh),
            SavedTick::new(ty(), early, 4, TickPriority::Normal),
        ]);
        ticks.add_container(west, Some(Arc::new(saved))).unpack(100);
        ticks.add_container(east, None);
        ticks.schedule(ScheduledTick::new(ty(), high, 104, TickPriority::High, 0));

        let mut run = Vec::new();
        ticks.tick(103, 8, &mut |pos, _| run.push(pos));
        assert!(run.is_empty());
        ticks.tick(104, 1, &mut |pos, _| run.push(pos));
        assert_eq!(run, [high]);
        ticks.tick(105, 8, &mut |pos, _| run.push(pos));
        assert_eq!(run, [high, early]);
        ticks.tick(120, 8, &mut |pos, _| run.push(pos));
        assert_eq!(run, [high, early, late]);
    }

    #[test]
    fn replaces_containers_without_running_detached_ticks() {
        let ticks = LevelTicks::new(Arc::new(|_| true));
        let pos = ChunkPos::new(0, 0);
        let old = ticks.add_container(pos, None);
        old.schedule(ScheduledTick::with_default_priority(
            ty(),
            BlockPos::ZERO,
            10,
            0,
        ));

        let new = ticks.add_container(pos, None);
        old.schedule(ScheduledTick::with_default_priority(
            ty(),
            BlockPos::new(1, 0, 0),
            11,
            0,
        ));
        new.schedule(ScheduledTick::with_default_priority(
            ty(),
            BlockPos::new(2, 0, 0),
            13,
            0,
        ));

        let mut run = Vec::new();
        ticks.tick(11, 64, &mut |pos, _| run.push(pos));
        assert!(run.is_empty());
        ticks.tick(13, 64, &mut |pos, _| run.push(pos));
        assert_eq!(run, [BlockPos::new(2, 0, 0)]);
    }
}
