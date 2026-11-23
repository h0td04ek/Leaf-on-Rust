use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use leaf_core::core::{block_pos::BlockPos, direction::Direction};
use leaf_core::nbt::Tag;
use leaf_core::world::level::chunk_pos::ChunkPos;
use leaf_core::world::phys::vec3::Vec3;
use leaf_core::world::ticks::scheduled_tick::{SavedTick, TickNbt, TickType};
use leaf_core::world::ticks::{LevelTicks, ScheduledTick, TickAccess};

use crate::audience::Audience;
use crate::blocks::Blocks;

use super::{
    broadcast_blocks, chunk, fluid_faces, set_state, state_at, update, BlockView, Chunk, Edits,
    NativeWorld, AIR,
};

const MAX_FLUID_TICKS: usize = 1024;
const HORIZONTAL: [Direction; 4] = [
    Direction::NORTH,
    Direction::SOUTH,
    Direction::WEST,
    Direction::EAST,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fluid {
    Water,
    Lava,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Liquid {
    fluid: Fluid,
    level: i32,
}

impl Liquid {
    fn at(state: i32) -> Option<Self> {
        if fluid_faces::waterlogged(state) {
            return Some(Self {
                fluid: Fluid::Water,
                level: 0,
            });
        }
        let ids = FluidBlocks::get();
        [(Fluid::Water, ids.water), (Fluid::Lava, ids.lava)]
            .into_iter()
            .find_map(|(fluid, source)| {
                let level = state - source;
                (0..16).contains(&level).then_some(Self { fluid, level })
            })
    }

    fn distance(self) -> i32 {
        if self.level >= 8 {
            0
        } else {
            self.level
        }
    }
}

impl Fluid {
    fn name(self) -> &'static str {
        match self {
            Self::Water => "minecraft:water",
            Self::Lava => "minecraft:lava",
        }
    }

    fn state(self, level: i32) -> i32 {
        let ids = FluidBlocks::get();
        match self {
            Self::Water => ids.water + level,
            Self::Lava => ids.lava + level,
        }
    }

    fn delay(self) -> i64 {
        match self {
            Self::Water => 5,
            Self::Lava => 30,
        }
    }

    fn drop_off(self) -> i32 {
        match self {
            Self::Water => 1,
            Self::Lava => 2,
        }
    }

    fn slope_distance(self) -> u8 {
        match self {
            Self::Water => 4,
            Self::Lava => 2,
        }
    }
}

impl TickType for Fluid {
    fn tick_hash(&self) -> u64 {
        *self as u64
    }
    fn tick_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl TickNbt for Fluid {
    fn write_nbt(&self) -> Tag {
        Tag::String(self.name().into())
    }

    fn read_nbt(tag: &Tag) -> Option<Self> {
        match tag.as_string()? {
            "minecraft:water" | "minecraft:flowing_water" => Some(Self::Water),
            "minecraft:lava" | "minecraft:flowing_lava" => Some(Self::Lava),
            _ => None,
        }
    }
}

struct FluidBlocks {
    water: i32,
    lava: i32,
    stone: i32,
    obsidian: i32,
    cobblestone: i32,
    soul_soil: i32,
    blue_ice: i32,
    basalt: i32,
}

impl FluidBlocks {
    fn get() -> &'static Self {
        static BLOCKS: OnceLock<FluidBlocks> = OnceLock::new();
        BLOCKS.get_or_init(|| {
            let blocks = Blocks::vanilla();
            let id = |name| blocks.named(name).unwrap().default;
            Self {
                water: id("water"),
                lava: id("lava"),
                stone: id("stone"),
                obsidian: id("obsidian"),
                cobblestone: id("cobblestone"),
                soul_soil: id("soul_soil"),
                blue_ice: id("blue_ice"),
                basalt: id("basalt"),
            }
        })
    }
}

#[derive(Clone, Copy)]
pub struct FluidRules {
    pub water_sources: bool,
    pub lava_sources: bool,
}

impl FluidRules {
    fn can_form_source(self, fluid: Fluid) -> bool {
        match fluid {
            Fluid::Water => self.water_sources,
            Fluid::Lava => self.lava_sources,
        }
    }
}

pub(super) struct FluidUpdates {
    ticks: Arc<LevelTicks<Fluid>>,
    clock: i64,
    order: i64,
}

impl Default for FluidUpdates {
    fn default() -> Self {
        Self {
            ticks: Arc::new(LevelTicks::new(Arc::new(|_| true))),
            clock: 0,
            order: 0,
        }
    }
}

impl FluidUpdates {
    pub fn restore(&mut self, coord: Chunk, mut pending: Vec<SavedTick<Fluid>>) {
        if pending.is_empty() {
            return;
        }
        // Keep ordering across restored chunk containers and newly scheduled ticks.
        pending.sort_by_key(|tick| tick.delay);
        let container = self
            .ticks
            .add_container(ChunkPos::new(coord.0, coord.1), None);
        for tick in pending {
            self.order += 1;
            container.schedule(tick.unpack(self.clock, self.order));
        }
    }

    pub fn pending(&self, coord: Chunk, now: i64) -> Vec<SavedTick<Fluid>> {
        let Some(container) = self.ticks.get_container(ChunkPos::new(coord.0, coord.1)) else {
            return Vec::new();
        };
        let mut ticks = container.get_all();
        ticks.sort_by_key(|tick| (tick.trigger_tick, tick.priority, tick.sub_tick_order));
        ticks
            .into_iter()
            .map(|tick| tick.to_saved_tick(now))
            .collect()
    }

    fn schedule(&mut self, pos: BlockPos, fluid: Fluid) -> bool {
        let (x, z) = chunk(pos);
        let chunk = ChunkPos::new(x, z);
        let container = self
            .ticks
            .get_container(chunk)
            .unwrap_or_else(|| self.ticks.add_container(chunk, None));
        if container.has_scheduled_tick(pos, &fluid) {
            return false;
        }
        self.order += 1;
        container.schedule(ScheduledTick::with_default_priority(
            fluid,
            pos,
            self.clock + fluid.delay(),
            self.order,
        ));
        true
    }
}

pub(super) fn notify(edits: &mut Edits, pos: BlockPos, old: i32, new: i32) {
    let ids = FluidBlocks::get();
    let blocks = Blocks::vanilla();
    let solid_shape_changed = blocks.get(old).replaceable != blocks.get(new).replaceable
        || fluid_faces::shape_changed(old, new);
    if !solid_shape_changed
        && Liquid::at(old).is_none()
        && Liquid::at(new).is_none()
        && ![old, new]
            .into_iter()
            .any(|state| matches!(state, id if id == ids.soul_soil || id == ids.blue_ice))
    {
        return;
    }
    reschedule(edits, pos);
}

pub(super) fn reschedule(edits: &mut Edits, pos: BlockPos) {
    for neighbor in
        std::iter::once(pos).chain(Direction::VALUES.into_iter().map(|side| pos.relative(side)))
    {
        if !inside_world(neighbor) {
            continue;
        }
        if let Some(liquid) = Liquid::at(state_at(edits, neighbor)) {
            if edits.fluids.schedule(neighbor, liquid.fluid) {
                edits.dirty.insert(chunk(neighbor));
            }
        }
    }
}

impl NativeWorld {
    pub fn tick_fluids(&self, now: i64, rules: FluidRules, audience: &Audience) {
        let mut edits = self.edits.lock().unwrap();
        edits.fluids.clock = now;
        let ticks = Arc::clone(&edits.fluids.ticks);
        let mut updates = HashMap::new();
        let mut touched = HashSet::new();
        ticks.tick(now, MAX_FLUID_TICKS, &mut |pos, fluid| {
            let coord = chunk(pos);
            touched.insert(coord);
            edits.dirty.insert(coord);
            Flow {
                edits: &mut edits,
                updates: &mut updates,
                rules,
            }
            .tick(pos, fluid);
        });
        for (x, z) in touched {
            let pos = ChunkPos::new(x, z);
            if ticks
                .get_container(pos)
                .is_some_and(|container| container.count() == 0)
            {
                ticks.remove_container(pos);
            }
        }
        broadcast_blocks(audience, updates);
    }

    pub fn take_fluid(&self, pos: BlockPos, audience: &Audience) -> Option<Fluid> {
        if !inside_world(pos) {
            return None;
        }
        let mut edits = self.edits.lock().unwrap();
        let state = state_at(&edits, pos);
        let liquid = Liquid::at(state).filter(|liquid| liquid.level == 0)?;
        let replacement = if fluid_faces::waterlogged(state) {
            waterlogged_state(state, false)
        } else {
            AIR
        };
        update(&mut edits, pos, replacement, audience);
        Some(liquid.fluid)
    }

    pub fn pour_bucket(
        &self,
        clicked: BlockPos,
        adjacent: BlockPos,
        fluid: Fluid,
        audience: &Audience,
    ) -> Option<BlockPos> {
        let mut edits = self.edits.lock().unwrap();
        if fluid == Fluid::Water && inside_world(clicked) {
            let state = state_at(&edits, clicked);
            if fluid_faces::can_waterlog(state) {
                if fluid_faces::waterlogged(state) {
                    return None;
                }
                update(
                    &mut edits,
                    clicked,
                    waterlogged_state(state, true),
                    audience,
                );
                return Some(clicked);
            }
        }
        pour_fluid(&mut edits, adjacent, fluid, audience).then_some(adjacent)
    }

    #[cfg(test)]
    fn pour_fluid(&self, pos: BlockPos, fluid: Fluid, audience: &Audience) -> bool {
        pour_fluid(&mut self.edits.lock().unwrap(), pos, fluid, audience)
    }
}

fn pour_fluid(edits: &mut Edits, pos: BlockPos, fluid: Fluid, audience: &Audience) -> bool {
    if !can_enter(edits, pos, fluid, 0) {
        return false;
    }
    let previous = Liquid::at(state_at(edits, pos));
    let state = if let (Fluid::Water, Some(other)) = (fluid, previous) {
        if other.fluid == Fluid::Lava {
            let ids = FluidBlocks::get();
            if other.level == 0 {
                ids.obsidian
            } else {
                ids.cobblestone
            }
        } else {
            fluid.state(0)
        }
    } else {
        fluid.state(0)
    };
    update(edits, pos, state, audience);
    true
}

fn waterlogged_state(state: i32, wet: bool) -> i32 {
    Blocks::vanilla().get(state).state(
        state,
        &[("waterlogged", if wet { "true" } else { "false" })],
    )
}

pub(super) fn waterlog_on_place(previous: i32, placed: i32) -> i32 {
    if previous == Fluid::Water.state(0) && fluid_faces::can_waterlog(placed) {
        waterlogged_state(placed, true)
    } else {
        placed
    }
}

pub(super) fn after_break(state: i32) -> i32 {
    if fluid_faces::waterlogged(state) {
        Fluid::Water.state(0)
    } else {
        AIR
    }
}

fn inside_world(pos: BlockPos) -> bool {
    NativeWorld::editable(pos)
        && (-30_000_000..30_000_000).contains(&pos.x)
        && (-30_000_000..30_000_000).contains(&pos.z)
}

fn can_enter(edits: &Edits, pos: BlockPos, fluid: Fluid, level: i32) -> bool {
    if !inside_world(pos) {
        return false;
    }
    let previous = state_at(edits, pos);
    match Liquid::at(previous) {
        Some(other) if other.fluid == fluid => match level {
            0 => other.level != 0,
            1..=7 => (level + 1..8).contains(&other.level),
            _ => (1..8).contains(&other.level),
        },
        Some(_) => level == 0 || fluid == Fluid::Water || level >= 8,
        None => Blocks::vanilla().get(previous).replaceable,
    }
}

fn slope_can_pass(edits: &Edits, pos: BlockPos, fluid: Fluid) -> bool {
    if !inside_world(pos) {
        return false;
    }
    let previous = state_at(edits, pos);
    match Liquid::at(previous) {
        Some(other) if other.fluid == fluid => other.level != 0,
        Some(_) => fluid == Fluid::Water,
        None => Blocks::vanilla().get(previous).replaceable,
    }
}

struct Flow<'a> {
    edits: &'a mut Edits,
    updates: &'a mut HashMap<BlockPos, i32>,
    rules: FluidRules,
}

impl Flow<'_> {
    fn set(&mut self, pos: BlockPos, state: i32) {
        set_state(self.edits, pos, state);
        self.updates.insert(pos, state);
    }

    fn pass(&self, from: BlockPos, direction: Direction) -> bool {
        let state = state_at(self.edits, from);
        let next = state_at(self.edits, from.relative(direction));
        fluid_faces::can_pass(state, next, direction)
    }

    fn tick(&mut self, pos: BlockPos, fluid: Fluid) {
        let Some(liquid) =
            Liquid::at(state_at(self.edits, pos)).filter(|liquid| liquid.fluid == fluid)
        else {
            return;
        };
        if fluid == Fluid::Lava {
            if let Some(solid) = self.solidify_lava(pos, liquid.level) {
                self.set(pos, solid);
                return;
            }
        }

        let level = if liquid.level == 0 {
            0
        } else if let Some(level) = self.incoming_level(pos, fluid) {
            if level != liquid.level {
                self.set(pos, fluid.state(level));
            }
            level
        } else {
            self.set(pos, AIR);
            return;
        };

        let below = pos.relative(Direction::DOWN);
        if self.pass(pos, Direction::DOWN) {
            if can_enter(self.edits, below, fluid, 8) {
                self.spread(below, fluid, 8);
                return;
            }
            if Liquid::at(state_at(self.edits, below))
                .is_some_and(|lower| lower.fluid == fluid && lower.level != 0)
            {
                return;
            }
        }
        let next = if level >= 8 { 0 } else { level } + fluid.drop_off();
        if next > 7 {
            return;
        }
        let candidates = HORIZONTAL
            .into_iter()
            .filter_map(|side| {
                let neighbor = pos.relative(side);
                (can_enter(self.edits, neighbor, fluid, next) && self.pass(pos, side)).then(|| {
                    let distance = self.slope_distance(
                        neighbor,
                        fluid,
                        side.get_opposite(),
                        fluid.slope_distance(),
                    );
                    (neighbor, distance)
                })
            })
            .collect::<Vec<_>>();
        let best = candidates
            .iter()
            .map(|&(_, distance)| distance)
            .min()
            .unwrap_or(u8::MAX);
        for (neighbor, distance) in candidates {
            if distance == best {
                self.spread(neighbor, fluid, next);
            }
        }
    }

    fn solidify_lava(&self, pos: BlockPos, level: i32) -> Option<i32> {
        let ids = FluidBlocks::get();
        let neighbors = HORIZONTAL.into_iter().chain([Direction::UP]);
        if neighbors.clone().any(|side| {
            Liquid::at(state_at(self.edits, pos.relative(side)))
                .is_some_and(|liquid| liquid.fluid == Fluid::Water)
        }) {
            return Some(if level == 0 {
                ids.obsidian
            } else {
                ids.cobblestone
            });
        }
        (state_at(self.edits, pos.relative(Direction::DOWN)) == ids.soul_soil
            && neighbors
                .into_iter()
                .any(|side| state_at(self.edits, pos.relative(side)) == ids.blue_ice))
        .then_some(ids.basalt)
    }

    fn incoming_level(&self, pos: BlockPos, fluid: Fluid) -> Option<i32> {
        let from = |side| {
            Liquid::at(state_at(self.edits, pos.relative(side)))
                .filter(|other| other.fluid == fluid && self.pass(pos, side))
        };
        let horizontal = HORIZONTAL.into_iter().filter_map(from);
        let mut sources = 0;
        let mut strongest = 8;
        for liquid in horizontal {
            sources += usize::from(liquid.level == 0);
            strongest = strongest.min(liquid.distance());
        }
        if sources >= 2 && self.rules.can_form_source(fluid) {
            let below = state_at(self.edits, pos.relative(Direction::DOWN));
            if !Blocks::vanilla().get(below).replaceable
                || from(Direction::DOWN).is_some_and(|liquid| liquid.level == 0)
            {
                return Some(0);
            }
        }
        if from(Direction::UP).is_some() {
            return Some(8);
        }
        let level = strongest + fluid.drop_off();
        (level <= 7).then_some(level)
    }

    fn slope_distance(&self, pos: BlockPos, fluid: Fluid, back: Direction, remaining: u8) -> u8 {
        if can_enter(self.edits, pos.relative(Direction::DOWN), fluid, 8)
            && self.pass(pos, Direction::DOWN)
        {
            return 0;
        }
        if remaining == 1 {
            return u8::MAX;
        }
        HORIZONTAL
            .into_iter()
            .filter(|&side| side != back)
            .filter_map(|side| {
                let next = pos.relative(side);
                (slope_can_pass(self.edits, next, fluid) && self.pass(pos, side)).then(|| {
                    self.slope_distance(next, fluid, side.get_opposite(), remaining - 1)
                        .saturating_add(1)
                })
            })
            .min()
            .unwrap_or(u8::MAX)
    }

    fn spread(&mut self, pos: BlockPos, fluid: Fluid, level: i32) {
        let state = state_at(self.edits, pos);
        let ids = FluidBlocks::get();
        match Liquid::at(state).filter(|other| other.fluid != fluid) {
            Some(other) if fluid == Fluid::Water => {
                self.set(
                    pos,
                    if other.level == 0 {
                        ids.obsidian
                    } else {
                        ids.cobblestone
                    },
                );
            }
            Some(_) => self.set(pos, ids.stone),
            None => self.set(pos, fluid.state(level)),
        }
    }
}

impl BlockView {
    pub fn fluid_at(&self, position: Vec3) -> Option<Fluid> {
        let pos = BlockPos::containing_vec3(&position);
        let liquid = Liquid::at(self.block(pos))?;
        let relative_y = position.y - f64::from(pos.y);
        let height = f64::from(8 - liquid.distance()) / 9.0;
        if relative_y < height
            || Liquid::at(self.block(pos.relative(Direction::UP)))
                .is_some_and(|above| above.fluid == liquid.fluid)
        {
            Some(liquid.fluid)
        } else {
            None
        }
    }
}

pub enum BucketHit {
    Source(BlockPos),
    Face {
        clicked: BlockPos,
        adjacent: BlockPos,
    },
}

pub fn bucket_hit(
    world: &BlockView,
    from: Vec3,
    direction: Vec3,
    reach: f64,
    collecting: bool,
) -> Option<BucketHit> {
    let start = [from.x, from.y, from.z];
    let dir = [direction.x, direction.y, direction.z];
    let mut cell = start.map(|value| value.floor() as i32);
    let step = dir.map(|value| if value < 0.0 { -1 } else { 1 });
    let interval = dir.map(|value| 1.0 / value.abs());
    let mut next = std::array::from_fn::<_, 3, _>(|axis| {
        if dir[axis] == 0.0 {
            f64::INFINITY
        } else {
            let boundary = f64::from(cell[axis] + i32::from(step[axis] > 0));
            (boundary - start[axis]) / dir[axis]
        }
    });
    let mut previous = None;
    loop {
        let position = BlockPos::new(cell[0], cell[1], cell[2]);
        let state = world.block(position);
        if collecting && Liquid::at(state).is_some_and(|liquid| liquid.level == 0) {
            return Some(BucketHit::Source(position));
        }
        if !Blocks::vanilla().get(state).replaceable {
            return if collecting {
                None
            } else {
                previous.map(|adjacent| BucketHit::Face {
                    clicked: position,
                    adjacent,
                })
            };
        }
        let axis = (0..3)
            .min_by(|&left, &right| next[left].total_cmp(&next[right]))
            .unwrap();
        if next[axis] > reach {
            return None;
        }
        previous = Some(position);
        cell[axis] += step[axis];
        next[axis] += interval[axis];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestWorld;
    use crate::world::Generation;

    const RULES: FluidRules = FluidRules {
        water_sources: true,
        lava_sources: false,
    };

    #[test]
    fn falling_water_spreads_and_drains_after_removing_the_source() {
        let world = NativeWorld::new(0);
        let audience = Audience::default();
        let top = BlockPos::new(0, 18, 0);
        assert!(world.pour_fluid(top, Fluid::Water, &audience));
        let below = top.offset(0, -1, 0);
        world.tick_fluids(4, RULES, &audience);
        assert_eq!(world.block(below), AIR);
        world.tick_fluids(5, RULES, &audience);
        assert_eq!(world.block(below), Fluid::Water.state(8));
        for tick in 6..=15 {
            world.tick_fluids(tick, RULES, &audience);
        }
        let side = top.offset(1, -2, 0);
        assert_eq!(world.block(side), Fluid::Water.state(1));
        assert_eq!(world.take_fluid(side, &audience), None);
        assert_eq!(world.take_fluid(top, &audience), Some(Fluid::Water));
        for tick in 16..=150 {
            world.tick_fluids(tick, RULES, &audience);
        }
        for pos in [top, below, side] {
            assert_eq!(world.block(pos), AIR, "{pos:?}");
        }
    }

    #[test]
    fn reacts_to_lava_and_obeys_the_source_conversion_rule() {
        let world = NativeWorld::new(0);
        let audience = Audience::default();
        let source = BlockPos::new(0, 16, 0);
        let stream = BlockPos::new(15, 16, 0);
        let falls = BlockPos::new(30, 16, 0);
        let basin = BlockPos::new(45, 16, 0);
        let put = |pos, fluid| assert!(world.pour_fluid(pos, fluid, &audience));
        put(source, Fluid::Lava);
        put(source.offset(1, 0, 0), Fluid::Water);
        world
            .fill(
                stream,
                stream,
                Fluid::Lava.state(2),
                crate::world::FillMode::Keep,
                1,
                &audience,
            )
            .unwrap();
        put(stream.offset(1, 0, 0), Fluid::Water);
        put(falls, Fluid::Water);
        put(falls.offset(0, 1, 0), Fluid::Lava);
        put(basin.offset(-1, 0, 0), Fluid::Water);
        put(basin.offset(1, 0, 0), Fluid::Water);
        let disabled = FluidRules {
            water_sources: false,
            lava_sources: false,
        };
        for tick in 1..=35 {
            world.tick_fluids(tick, disabled, &audience);
        }
        let ids = FluidBlocks::get();
        assert_eq!(world.block(source), ids.obsidian);
        assert_eq!(world.block(stream), ids.cobblestone);
        assert_eq!(world.block(falls), ids.stone);
        assert_eq!(world.block(basin), Fluid::Water.state(1));

        world.take_fluid(basin.offset(1, 0, 0), &audience);
        put(basin.offset(1, 0, 0), Fluid::Water);
        for tick in 36..=41 {
            world.tick_fluids(tick, RULES, &audience);
        }
        assert_eq!(world.block(basin), Fluid::Water.state(0));
    }

    #[test]
    fn slabs_preserve_waterlogged_states_and_only_open_faces_leak() {
        let directory = TestWorld::new();
        let path = directory.0.join("native-region");
        let audience = Audience::default();
        let world = NativeWorld::open([0; 10], Generation::FLAT, path.clone()).unwrap();
        let block = Blocks::vanilla().named("oak_slab").unwrap();
        let bottom = BlockPos::new(0, 18, 0);
        let top = BlockPos::new(12, 18, 0);
        let upper = block.state(block.default, &[("type", "top")]);
        world
            .fill(
                bottom,
                bottom,
                block.default,
                crate::world::FillMode::Keep,
                1,
                &audience,
            )
            .unwrap();
        world
            .fill(top, top, upper, crate::world::FillMode::Keep, 1, &audience)
            .unwrap();
        for pos in [bottom, top] {
            assert_eq!(
                world.pour_bucket(pos, pos.offset(0, 1, 0), Fluid::Water, &audience),
                Some(pos)
            );
        }
        let wet_top = block.state(upper, &[("waterlogged", "true")]);
        let wet_bottom = block.state(block.default, &[("waterlogged", "true")]);
        assert_eq!(world.block(top), wet_top);
        assert_eq!(world.block(bottom), wet_bottom);
        assert_eq!(
            world
                .snapshot()
                .fluid_at(bottom.get_bottom_center().add(0.0, 0.7, 0.0)),
            Some(Fluid::Water)
        );
        world.tick_fluids(5, RULES, &audience);
        assert_eq!(world.block(bottom.offset(0, -1, 0)), AIR);
        assert_eq!(world.block(top.offset(0, -1, 0)), Fluid::Water.state(8));
        world.save(5).unwrap();
        drop(world);

        let world = NativeWorld::open([0; 10], Generation::FLAT, path).unwrap();
        assert_eq!(world.block(top), wet_top);
        assert_eq!(world.take_fluid(top, &audience), Some(Fluid::Water));
        assert_eq!(world.block(top), upper);
        assert_eq!(world.take_fluid(top, &audience), None);
        assert!(world.break_block(bottom, wet_bottom, &audience));
        assert_eq!(world.block(bottom), Fluid::Water.state(0));

        let source = BlockPos::new(22, 16, 0);
        world
            .fill(
                source,
                source,
                Fluid::Water.state(0),
                crate::world::FillMode::Keep,
                1,
                &audience,
            )
            .unwrap();
        assert!(world.place_block(
            source.offset(0, -1, 0),
            source,
            block.default,
            |_| false,
            &audience
        ));
        assert_eq!(world.block(source), wet_bottom);
    }

    #[test]
    fn saves_fluid_ticks_in_anvil_with_the_remaining_delay() {
        let directory = TestWorld::new();
        let path = directory.0.join("native-region");
        let audience = Audience::default();
        let top = BlockPos::new(-16, 18, 1);
        let below = top.offset(0, -1, 0);
        let world = NativeWorld::open([0; 10], Generation::FLAT, path.clone()).unwrap();
        assert!(world.pour_fluid(top, Fluid::Water, &audience));
        world.tick_fluids(2, RULES, &audience);
        world.save(2).unwrap();
        drop(world);

        let world = NativeWorld::open([0; 10], Generation::FLAT, path).unwrap();
        world.tick_fluids(2, RULES, &audience);
        assert_eq!(world.block(below), AIR);
        world.tick_fluids(3, RULES, &audience);
        assert_eq!(world.block(below), Fluid::Water.state(8));
    }
}
