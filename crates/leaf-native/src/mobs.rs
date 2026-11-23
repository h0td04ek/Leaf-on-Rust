#[path = "mob_tracking.rs"]
mod tracking;

pub use tracking::MobTracker;
use tracking::SentMob;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::RwLock;
use std::time::Instant;

use leaf_core::core::{
    block_pos::BlockPos, holder::Holder, particles::particle_options::ParticleOptions,
    particles::particle_types::particle_type_by_identifier,
};
use leaf_core::entity::position_move_rotation::PositionMoveRotation;
use leaf_core::network::protocol::game::{
    clientbound_explode_packet::ClientboundExplodePacket,
    clientbound_sound_packet::ClientboundSoundPacket,
};
use leaf_core::random::{LegacyRandomSource, RandomSource};
use leaf_core::resources::Identifier;
use leaf_core::sounds::{sound_event::SoundEvent, sound_source::SoundSource};
use leaf_core::util::{random::weighted_list::WeightedList, uuid::Uuid};
use leaf_core::world::{level::game_type::GameType, phys::vec3::Vec3};

use crate::blocks::Blocks;
use crate::gameplay::PlayContext;
use crate::items::{named_id, stack};
use crate::mob_types::{self, MobType};
use crate::peers::{clear_sight, clear_sight_with, MobTarget, Peers};
use crate::simulation::SimulationContext;
use crate::workers::Workers;
use crate::world::{chunk, BlockView, NativeWorld};

pub const COMMAND_LIMIT: i32 = 50_000;
const FUSE_TICKS: i64 = 80;
const WORLD_LIMIT: usize = 60_000;
const TNT_LIMIT: usize = 256;
const SPAWNS_PER_TICK: usize = 256;
const SIMULATED_MOBS_PER_PLAYER: usize = 192;
const MOB_WORK_BATCH: usize = 192;
const TRACKED_ADDITIONS_PER_TICK: usize = 512;
const NATURAL_LIMIT: usize = 32;

type Chunk = (i32, i32);

pub struct Rules {
    pub natural_spawns: bool,
    pub hostile_spawns: bool,
    pub tnt: bool,
    pub block_drops: bool,
    pub decay: bool,
}

pub struct Mobs(RwLock<Scene>);

impl Default for Mobs {
    fn default() -> Self {
        Self(RwLock::new(Scene {
            entities: HashMap::new(),
            chunks: HashMap::new(),
            chunk_revision: 0,
            batches: VecDeque::new(),
            explosives: HashSet::new(),
            queued: 0,
            queued_tnt: 0,
            natural: [0; 2],
            next_natural: 60,
            random: LegacyRandomSource::new(94309),
        }))
    }
}

struct Scene {
    entities: HashMap<i32, Mob>,
    chunks: HashMap<Chunk, MobChunk>,
    chunk_revision: u64,
    batches: VecDeque<Batch>,
    explosives: HashSet<i32>,
    queued: usize,
    queued_tnt: usize,
    natural: [usize; 2],
    next_natural: i64,
    random: LegacyRandomSource,
}

#[derive(Default)]
struct MobChunk {
    revision: u64,
    ids: HashSet<i32>,
}

struct Batch {
    kind: &'static MobType,
    origin: Vec3,
    spawned: usize,
    total: usize,
}

#[derive(Clone, Copy)]
struct Mob {
    kind: &'static MobType,
    uuid: Uuid,
    motion: PositionMoveRotation,
    health: f32,
    natural: bool,
    on_ground: bool,
    hurt: u32,
    last_hurt: Option<i64>,
    next_attack: Option<i64>,
    dying: Option<i64>,
    fuse: Option<i64>,
}

impl Mobs {
    pub fn summon(
        &self,
        kind: &'static MobType,
        position: Vec3,
        count: usize,
        ctx: &PlayContext<'_>,
    ) -> Result<(), &'static str> {
        if !valid_position(position) {
            return Err("The summon position is outside this world");
        }
        let mut scene = self.0.write().unwrap();
        if count == 0
            || count > COMMAND_LIMIT as usize
            || scene.entities.len() + scene.queued + count > WORLD_LIMIT
        {
            return Err("Too many entities in the world or in the spawning queue");
        }
        if kind.name == "tnt" && scene.explosives.len() + scene.queued_tnt + count > TNT_LIMIT {
            return Err("Too many primed TNT entities");
        }
        if count == 1 {
            let fuse = (kind.name == "tnt").then(|| ctx.commands.game_time() + FUSE_TICKS);
            scene.add(kind, position, false, fuse, ctx.entity_ids);
        } else {
            scene.batches.push_back(Batch {
                kind,
                origin: position,
                spawned: 0,
                total: count,
            });
            scene.queued += count;
            if kind.name == "tnt" {
                scene.queued_tnt += count;
            }
        }
        Ok(())
    }

    pub fn egg(&self, kind: &'static MobType, position: Vec3, ctx: &PlayContext<'_>) -> bool {
        ctx.world.has_room(position, kind.height, kind.aquatic)
            && self.summon(kind, position, 1, ctx).is_ok()
    }

    pub fn ignite(&self, position: BlockPos, ctx: &PlayContext<'_>) -> bool {
        if !ctx.commands.world.lock().unwrap().mob_rules().tnt {
            return false;
        }
        let mut scene = self.0.write().unwrap();
        if scene.explosives.len() + scene.queued_tnt >= TNT_LIMIT {
            return false;
        }
        let state = ctx.world.block(position);
        if Blocks::vanilla().get(state).name == "tnt"
            && ctx.world.break_block(position, state, ctx.audience)
        {
            scene.prime(
                position,
                ctx.commands.game_time() + FUSE_TICKS,
                &ctx.simulation(),
            );
            return true;
        }
        false
    }

    pub fn power_changed(&self, position: BlockPos, ctx: &PlayContext<'_>) {
        for target in ctx.world.powered_tnt_near(position) {
            self.ignite(target, ctx);
        }
    }

    pub fn attack(&self, id: i32, from: Vec3, mode: GameType, damage: f32, ctx: &PlayContext<'_>) {
        if mode == GameType::Spectator {
            return;
        }
        let mut scene = self.0.write().unwrap();
        let Some(mob) = scene
            .entities
            .get_mut(&id)
            .filter(|m| m.fuse.is_none() && m.dying.is_none())
        else {
            return;
        };
        let target = mob.motion.position;
        let reach: f64 = if mode.is_creative() { 5.0 } else { 3.2 };
        if from.distance_to_sqr(&target) > reach.powi(2)
            || !clear_sight(
                ctx.world,
                from.add(0.0, 1.62, 0.0),
                target.add(0.0, mob.kind.height * 0.6, 0.0),
            )
        {
            return;
        }
        let push = from
            .vector_to(&target)
            .multiply(1.0, 0.0, 1.0)
            .normalize()
            .scale(0.28)
            .add(0.0, 0.24, 0.0);
        if mob.damage(damage, push, ctx.commands.game_time(), &ctx.simulation()) {
            scene.mark_changed(at_chunk(target));
        }
    }

    pub fn tick(
        &self,
        now: i64,
        rules: &Rules,
        ctx: &SimulationContext<'_>,
        peers: &Peers,
        workers: &Workers,
    ) {
        let mut scene = self.0.write().unwrap();
        let mut players = peers.mob_targets();
        players.sort_unstable_by_key(|player| player.id);
        scene.spawn_queued(now, ctx);
        scene.spawn_naturally(now, &players, rules, ctx);

        let mut active = scene.explosives.clone();
        let mut explosions = Vec::new();
        for player in &players {
            active.extend(scene.nearby(player.position, 32.0, SIMULATED_MOBS_PER_PLAYER));
        }
        let mut removed = Vec::new();
        let mut jobs = Vec::with_capacity(active.len());
        let mut active = active.into_iter().collect::<Vec<_>>();
        active.sort_unstable();
        for id in active {
            let mob = scene.entities[&id];
            if mob.fuse.is_some_and(|end| end <= now) {
                if explosions.len() < 8 {
                    if rules.tnt {
                        explosions.push(mob.motion.position.add(0.0, 0.0625, 0.0));
                    }
                    removed.push(id);
                }
                continue;
            }
            if mob.dying.is_some_and(|end| end <= now) || mob.motion.position.y < -64.0 {
                removed.push(id);
                continue;
            }
            if mob.dying.is_none() {
                jobs.push(id);
            }
        }

        let world = ctx.world.snapshot();
        if workers.can_split(jobs.len(), MOB_WORK_BATCH) {
            let jobs = jobs
                .into_iter()
                .map(|id| (id, scene.entities[&id]))
                .collect();
            for (id, updated, attack) in step_mobs(jobs, players, world, now, workers) {
                let current = scene.entities.get_mut(&id).unwrap();
                let previous = SentMob::from(*current);
                *current = updated;
                scene.finish_step(id, previous, updated, attack, peers);
            }
        } else {
            for id in jobs {
                let mob = scene.entities.get_mut(&id).unwrap();
                let previous = SentMob::from(*mob);
                let attack = mob.step(id, now, &players, &world);
                let updated = *mob;
                scene.finish_step(id, previous, updated, attack, peers);
            }
        }
        for id in removed {
            scene.remove(id);
        }
        for center in explosions {
            scene.explode(center, now, rules, ctx, peers, workers);
        }
    }
}

impl Scene {
    fn finish_step(
        &mut self,
        id: i32,
        previous: SentMob,
        updated: Mob,
        attack: Option<MobAttack>,
        peers: &Peers,
    ) {
        let current = SentMob::from(updated);
        if previous != current {
            let old_chunk = at_chunk(previous.motion.position);
            let new_chunk = at_chunk(current.motion.position);
            if old_chunk != new_chunk {
                self.unindex(id, old_chunk);
                self.index(id, new_chunk);
            } else {
                self.mark_changed(new_chunk);
            }
        }
        if let Some(attack) = attack {
            peers.hurt(
                attack.target,
                3.0,
                attack.velocity,
                updated.kind.name.into(),
                Instant::now(),
            );
        }
    }

    fn add(
        &mut self,
        kind: &'static MobType,
        position: Vec3,
        natural: bool,
        fuse: Option<i64>,
        ids: &AtomicI32,
    ) -> i32 {
        let id = ids.fetch_add(1, Ordering::Relaxed);
        let yaw = (id.rem_euclid(360)) as f32;
        let velocity = if fuse.is_some() {
            let angle = f64::from(yaw).to_radians();
            Vec3::new(angle.sin() * 0.02, 0.2, angle.cos() * 0.02)
        } else {
            Vec3::ZERO
        };
        let mob = Mob {
            kind,
            uuid: Uuid::random(),
            motion: PositionMoveRotation::new(position, velocity, yaw, 0.0),
            health: kind.health,
            natural,
            on_ground: false,
            hurt: 0,
            last_hurt: None,
            next_attack: None,
            dying: None,
            fuse,
        };
        self.entities.insert(id, mob);
        self.index(id, at_chunk(position));
        if fuse.is_some() {
            self.explosives.insert(id);
        }
        if natural {
            self.natural[usize::from(kind.hostile)] += 1;
        }
        id
    }

    fn prime(&mut self, position: BlockPos, end: i64, ctx: &SimulationContext<'_>) {
        let center = Vec3::new(
            f64::from(position.x) + 0.5,
            f64::from(position.y),
            f64::from(position.z) + 0.5,
        );
        self.add(
            mob_types::named("tnt").unwrap(),
            center,
            false,
            Some(end),
            ctx.entity_ids,
        );
        let (x, y, z) = ClientboundSoundPacket::pack_position(center.x, center.y, center.z);
        ctx.audience.broadcast(
            ClientboundSoundPacket::new(
                Holder::direct(SoundEvent::new(
                    Identifier::minecraft("entity.tnt.primed"),
                    None,
                )),
                SoundSource::Blocks,
                x,
                y,
                z,
                1.0,
                1.0,
                self.random.next_long(),
            ),
            Some(at_chunk(center)),
        );
    }

    fn index(&mut self, id: i32, coord: Chunk) {
        self.chunks.entry(coord).or_default().ids.insert(id);
        self.mark_changed(coord);
    }

    fn unindex(&mut self, id: i32, coord: Chunk) {
        let chunk = self.chunks.get_mut(&coord).unwrap();
        chunk.ids.remove(&id);
        if chunk.ids.is_empty() {
            self.chunks.remove(&coord);
        } else {
            self.mark_changed(coord);
        }
    }

    fn mark_changed(&mut self, coord: Chunk) {
        self.chunk_revision += 1;
        self.chunks.get_mut(&coord).unwrap().revision = self.chunk_revision;
    }

    fn remove(&mut self, id: i32) {
        let mob = self.entities.remove(&id).unwrap();
        self.unindex(id, at_chunk(mob.motion.position));
        self.explosives.remove(&id);
        if mob.natural {
            self.natural[usize::from(mob.kind.hostile)] -= 1;
        }
    }

    fn nearby(&self, center: Vec3, radius: f64, limit: usize) -> impl Iterator<Item = i32> {
        let min = at_chunk(center.subtract(radius, 0.0, radius));
        let max = at_chunk(center.add(radius, 0.0, radius));
        let mut result = Vec::new();
        for x in min.0..=max.0 {
            for z in min.1..=max.1 {
                if let Some(chunk) = self.chunks.get(&(x, z)) {
                    result.extend(chunk.ids.iter().filter_map(|&id| {
                        let mob = &self.entities[&id];
                        let distance = center.distance_to_sqr(&mob.motion.position);
                        (distance <= radius * radius).then_some((id, mob.fuse.is_some(), distance))
                    }));
                }
            }
        }
        let nearest = |(ai, a_explosive, a_dist): &(i32, bool, f64),
                       (bi, b_explosive, b_dist): &(i32, bool, f64)| {
            b_explosive
                .cmp(a_explosive)
                .then_with(|| a_dist.total_cmp(b_dist))
                .then_with(|| ai.cmp(bi))
        };
        if result.len() > limit {
            result.select_nth_unstable_by(limit, nearest);
            result.truncate(limit);
        }
        result.into_iter().map(|(id, _, _)| id)
    }

    fn spawn_queued(&mut self, now: i64, ctx: &SimulationContext<'_>) {
        for _ in 0..SPAWNS_PER_TICK {
            let Some(mut batch) = self.batches.pop_front() else {
                break;
            };
            let (x, z) = spiral(batch.spawned);
            let radius = (batch.total as f64).sqrt().ceil() * 2.0;
            let block_bound = 29_999_984.0 - radius;
            let position = Vec3::new(
                batch.origin.x.clamp(-block_bound, block_bound) + f64::from(x) * 2.0,
                batch.origin.y,
                batch.origin.z.clamp(-block_bound, block_bound) + f64::from(z) * 2.0,
            );
            let fuse = (batch.kind.name == "tnt").then(|| now + FUSE_TICKS);
            let position = if fuse.is_none()
                && !batch.kind.aquatic
                && !ctx.world.has_room(position, batch.kind.height, false)
            {
                let block = BlockPos::containing_vec3(&position);
                ctx.world.open_surface(block.x, block.z).unwrap_or(position)
            } else {
                position
            };
            self.add(batch.kind, position, false, fuse, ctx.entity_ids);
            batch.spawned += 1;
            self.queued -= 1;
            if fuse.is_some() {
                self.queued_tnt -= 1;
            }
            if batch.spawned < batch.total {
                self.batches.push_back(batch);
            }
        }
    }

    fn spawn_naturally(
        &mut self,
        now: i64,
        players: &[MobTarget],
        rules: &Rules,
        ctx: &SimulationContext<'_>,
    ) {
        if now < self.next_natural {
            return;
        }
        self.next_natural = now + 60;
        let discard = self
            .entities
            .iter()
            .filter_map(|(&id, mob)| {
                (mob.dying.is_some_and(|end| end <= now)
                    || mob.natural
                        && players.iter().all(|p| {
                            p.position.distance_to_sqr(&mob.motion.position) > 128.0 * 128.0
                        }))
                .then_some(id)
            })
            .collect::<Vec<_>>();
        for id in discard {
            self.remove(id);
        }
        let nearby = self.natural[usize::from(rules.hostile_spawns)];
        if !rules.natural_spawns
            || players.is_empty()
            || nearby >= NATURAL_LIMIT
            || self.entities.len() + self.queued >= WORLD_LIMIT
        {
            return;
        }
        let player = players[self.random.next_int_bound(players.len() as i32) as usize];
        let angle = f64::from(self.random.next_float()) * std::f64::consts::TAU;
        let distance = 25.0 + f64::from(self.random.next_float()) * 14.0;
        let position = player
            .position
            .add(angle.sin() * distance, 0.0, angle.cos() * distance);
        let block = BlockPos::containing_vec3(&position);
        let species = if rules.hostile_spawns {
            ["zombie", "skeleton", "spider", "creeper"]
        } else {
            ["pig", "cow", "sheep", "chicken"]
        };
        let kind = mob_types::named(species[self.random.next_int_bound(4) as usize]).unwrap();
        let capacity =
            (NATURAL_LIMIT - nearby).min(WORLD_LIMIT - self.entities.len() - self.queued);
        for (dx, dz) in [(0, 0), (2, 1), (-1, 2)].into_iter().take(capacity) {
            if let Some(position) = ctx.world.open_surface(block.x + dx, block.z + dz) {
                if valid_position(position)
                    && ctx.world.has_room(position, kind.height, false)
                    && players
                        .iter()
                        .all(|p| p.position.distance_to_sqr(&position) >= 24.0 * 24.0)
                {
                    self.add(kind, position, true, None, ctx.entity_ids);
                }
            }
        }
    }

    fn explode(
        &mut self,
        center: Vec3,
        now: i64,
        rules: &Rules,
        ctx: &SimulationContext<'_>,
        peers: &Peers,
        workers: &Workers,
    ) {
        for target in peers
            .mob_targets()
            .into_iter()
            .filter(|target| target.vulnerable)
        {
            if let Some((damage, velocity)) = blast_hit(center, target.position, ctx.world) {
                peers.hurt(
                    target.id,
                    damage,
                    velocity,
                    "explosion".into(),
                    Instant::now(),
                );
            }
        }
        let nearby = self.nearby(center, 8.0, usize::MAX).collect::<Vec<_>>();
        for id in nearby {
            let mob = self.entities[&id];
            if mob.fuse.is_none() {
                if let Some((damage, velocity)) = blast_hit(center, mob.motion.position, ctx.world)
                {
                    if self
                        .entities
                        .get_mut(&id)
                        .unwrap()
                        .damage(damage, velocity, now, ctx)
                    {
                        self.mark_changed(at_chunk(mob.motion.position));
                    }
                }
            }
        }
        let removed = ctx
            .world
            .blast(center, 4.0, &mut self.random, ctx.audience, workers);
        let blocks = Blocks::vanilla();
        for &(pos, state) in &removed {
            let block = blocks.get(state);
            if block.name == "tnt" && self.explosives.len() + self.queued_tnt < TNT_LIMIT {
                let ticks = 10 + self.random.next_int_bound(20);
                self.prime(pos, now + i64::from(ticks), ctx);
            } else if rules.block_drops
                && (!rules.decay || self.random.next_int_bound(4) == 0)
                && block.property(state, "half") != Some("upper")
                && block.property(state, "part") != Some("head")
            {
                if let Some(item) = block.item {
                    let center = Vec3::new(
                        f64::from(pos.x) + 0.5,
                        f64::from(pos.y) + 0.5,
                        f64::from(pos.z) + 0.5,
                    );
                    spawn_item(ctx, item, center, now);
                }
            }
        }
        ctx.audience.broadcast(
            ClientboundExplodePacket::new(
                center,
                4.0,
                removed.len() as i32,
                None,
                ParticleOptions::simple(
                    particle_type_by_identifier(&Identifier::minecraft("explosion_emitter"))
                        .unwrap(),
                ),
                Holder::direct(SoundEvent::new(
                    Identifier::minecraft("entity.generic.explode"),
                    None,
                )),
                WeightedList::empty(),
            ),
            Some(at_chunk(center)),
        );
    }
}

impl Mob {
    fn damage(
        &mut self,
        amount: f32,
        velocity: Vec3,
        now: i64,
        ctx: &SimulationContext<'_>,
    ) -> bool {
        if self.dying.is_some() || self.last_hurt.is_some_and(|last| now - last < 10) {
            return false;
        }
        self.last_hurt = Some(now);
        self.health = (self.health - amount).max(0.0);
        self.motion.delta_movement = velocity;
        self.hurt = self.hurt.wrapping_add(1);
        if self.health == 0.0 {
            self.dying = Some(now + 20);
            let item = match self.kind.name {
                "pig" => Some("porkchop"),
                "cow" => Some("beef"),
                "sheep" => Some("mutton"),
                "chicken" => Some("chicken"),
                "zombie" | "husk" => Some("rotten_flesh"),
                "skeleton" => Some("bone"),
                _ => None,
            };
            if let Some(item) = item {
                if ctx.commands.world.lock().unwrap().mob_drops() {
                    spawn_item(ctx, named_id(item), self.motion.position, now);
                }
            }
        }
        true
    }

    fn step(
        &mut self,
        id: i32,
        now: i64,
        players: &[MobTarget],
        world: &BlockView,
    ) -> Option<MobAttack> {
        let mut attack = None;
        let position = self.motion.position;
        let mut velocity = self.motion.delta_movement;
        if self.fuse.is_none() && self.kind.name != "armor_stand" {
            let target = self
                .kind
                .hostile
                .then(|| {
                    players
                        .iter()
                        .filter(|p| {
                            p.vulnerable && position.distance_to_sqr(&p.position) < 14.0 * 14.0
                        })
                        .min_by(|a, b| {
                            position
                                .distance_to_sqr(&a.position)
                                .total_cmp(&position.distance_to_sqr(&b.position))
                        })
                })
                .flatten();
            let phase = (now as u32).wrapping_add((id as u32).wrapping_mul(17));
            if let Some(target) = target {
                let delta = position.vector_to(&target.position);
                let heading = delta.multiply(1.0, 0.0, 1.0).normalize().scale(0.13);
                velocity.x = heading.x;
                velocity.z = heading.z;
                self.motion.y_rot = (-heading.x.atan2(heading.z).to_degrees()) as f32;
                if delta.length_squared() <= 2.5 * 2.5
                    && self.next_attack.is_none_or(|next| now >= next)
                    && clear_sight_with(
                        position.add(0.0, self.kind.height * 0.75, 0.0),
                        target.position.add(0.0, 1.0, 0.0),
                        |pos| world.block(pos),
                    )
                {
                    self.next_attack = Some(now + 20);
                    attack = Some(MobAttack {
                        target: target.id,
                        velocity: heading.scale(2.0).add(0.0, 0.25, 0.0),
                    });
                }
            } else if phase % 120 < 32 && self.on_ground {
                self.motion.y_rot = ((phase / 120 * 71 + id as u32) % 360) as f32;
                let walk = Vec3::direction_from_rotation(0.0, self.motion.y_rot).scale(0.065);
                velocity.x = walk.x;
                velocity.z = walk.z;
            }
        }
        if self.on_ground
            && velocity == Vec3::ZERO
            && position.y.fract() == 0.0
            && !self.kind.aquatic
            && !self.kind.flying
            && !world.has_room(position.add(0.0, -0.04, 0.0), self.kind.height, false)
        {
            return attack;
        }
        let next = position.add(velocity.x, 0.0, velocity.z);
        let swim = self.kind.aquatic && world.has_room(position, self.kind.height, true);
        let mut next = if world.has_room(next, self.kind.height, swim) {
            next
        } else {
            position
        };
        if !self.kind.flying && !swim {
            velocity.y = (velocity.y - 0.04).max(-0.9);
        }
        let vertical = next.add(0.0, velocity.y, 0.0);
        self.on_ground = false;
        if world.has_room(vertical, self.kind.height, swim) {
            next = vertical;
        } else {
            if velocity.y <= 0.0 {
                let block = BlockPos::containing_vec3(&vertical);
                let floor = Vec3::new(next.x, f64::from(block.y + 1), next.z);
                if world.has_room(floor, self.kind.height, swim) {
                    next = floor;
                }
                self.on_ground = true;
            }
            velocity.y = 0.0;
        }
        self.motion.position = next;
        self.motion.delta_movement = velocity.multiply(0.7, 0.98, 0.7);
        attack
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MobAttack {
    target: i32,
    velocity: Vec3,
}

fn step_mobs(
    jobs: Vec<(i32, Mob)>,
    players: Vec<MobTarget>,
    world: BlockView,
    now: i64,
    workers: &Workers,
) -> impl Iterator<Item = (i32, Mob, Option<MobAttack>)> {
    workers
        .map_ranges(jobs.len(), MOB_WORK_BATCH, move |range| {
            jobs[range]
                .iter()
                .map(|&(id, mut mob)| {
                    let attack = mob.step(id, now, &players, &world);
                    (id, mob, attack)
                })
                .collect::<Vec<_>>()
        })
        .into_iter()
        .flatten()
}

fn spawn_item(ctx: &SimulationContext<'_>, item: i32, position: Vec3, tick: i64) {
    ctx.drops.spawn(
        ctx.entity_ids.fetch_add(1, Ordering::Relaxed),
        stack(item, 1),
        position,
        Vec3::ZERO,
        tick,
    );
}

fn blast_hit(center: Vec3, pos: Vec3, world: &NativeWorld) -> Option<(f32, Vec3)> {
    let displacement = center.vector_to(&pos.add(0.0, 0.35, 0.0));
    let distance = displacement.length();
    if distance >= 8.0 || !clear_sight(world, center, pos.add(0.0, 0.35, 0.0)) {
        return None;
    }
    let exposure = 1.0 - distance / 8.0;
    let damage = ((exposure * exposure + exposure) * 28.0 + 1.0) as f32;
    Some((damage, displacement.normalize().scale(exposure)))
}

fn at_chunk(position: Vec3) -> Chunk {
    chunk(BlockPos::containing_vec3(&position))
}

fn valid_position(position: Vec3) -> bool {
    position.x.is_finite()
        && position.y.is_finite()
        && position.z.is_finite()
        && position.x.abs() <= 29_999_984.0
        && position.z.abs() <= 29_999_984.0
        && NativeWorld::editable(BlockPos::containing_vec3(&position))
}

fn spiral(index: usize) -> (i32, i32) {
    if index == 0 {
        return (0, 0);
    }
    let ring = (((index as f64 + 1.0).sqrt() - 1.0) / 2.0).ceil() as i32;
    let width = ring * 2;
    let offset = (width + 1).pow(2) - 1 - index as i32;
    match offset / width {
        0 => (ring - offset, -ring),
        1 => (-ring, -ring + offset - width),
        2 => (-ring + offset - 2 * width, ring),
        _ => (ring, ring - offset + 3 * width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audience::Audience;
    use crate::commands::Commands;
    use crate::drops::Drops;
    use crate::maps::Maps;
    use crate::operators::Operators;
    use crate::tests::TestWorld;
    use crate::wire::PacketIds;

    #[test]
    fn mob_step_results_are_independent_of_worker_count() {
        let mobs = Mobs::default();
        let mut scene = mobs.0.write().unwrap();
        let ids = AtomicI32::new(1);
        let world = NativeWorld::new(0);
        let audience = Audience::default();
        let obstacle = BlockPos::new(1, 16, 0);
        assert!(world.place_block(obstacle.offset(0, -1, 0), obstacle, 1, |_| false, &audience));
        let snapshot = world.snapshot();
        assert!(world.break_block(obstacle, 1, &audience));
        assert_eq!(snapshot.block(obstacle), 1);
        assert_eq!(world.block(obstacle), 0);

        let player = MobTarget {
            id: 7000,
            position: Vec3::new(1.0, 16.0, 1.0),
            vulnerable: true,
        };
        let jobs = (0..768)
            .map(|i| {
                let kind = if i % 2 == 0 { "pig" } else { "zombie" };
                let pos = Vec3::new(
                    0.5 + f64::from(i % 12) * 0.125,
                    16.0,
                    0.5 + f64::from(i / 12) * 0.125,
                );
                let id = scene.add(mob_types::named(kind).unwrap(), pos, false, None, &ids);
                (id, scene.entities[&id])
            })
            .collect::<Vec<_>>();
        let run = |threads| {
            let workers = Workers::new(threads).unwrap();
            step_mobs(jobs.clone(), vec![player], snapshot.clone(), 20, &workers)
                .map(|(id, mob, attack)| (id, mob.motion, mob.on_ground, mob.next_attack, attack))
                .collect::<Vec<_>>()
        };
        let serial = run(1);
        assert!(serial.iter().any(|entry| entry.4.is_some()));
        assert_eq!(serial, run(4));
    }

    #[test]
    fn queues_spawns_in_groups_and_tracks_all_50k_pigs() {
        let directory = TestWorld::new();
        std::fs::create_dir_all(&directory.0).unwrap();
        let commands = Commands::new(Operators::open(directory.0.join("ops.txt")).unwrap(), 0);
        let maps = Maps::open(directory.0.join("maps.txt")).unwrap();
        let mobs = Mobs::default();
        let ctx = PlayContext {
            world: &NativeWorld::new(0),
            drops: &Drops::default(),
            maps: &maps,
            mobs: &mobs,
            name: "h0td04ek",
            player: Uuid::random(),
            ids: &PacketIds::from_protocol(
                leaf_core::network::protocol::game::game_protocols::clientbound_template().bind(),
            ),
            audience: &Audience::default(),
            commands: &commands,
            entity_ids: &AtomicI32::new(1),
        };
        let origin = Vec3::new(0.5, 16.0, 0.5);
        let pig = mob_types::named("pig").unwrap();
        mobs.summon(pig, origin, 50_000, &ctx).unwrap();
        let mut scene = mobs.0.write().unwrap();
        assert_eq!(scene.entities.len(), 0);
        let now = 0;
        scene.spawn_queued(now, &ctx.simulation());
        assert_eq!(scene.entities.len(), SPAWNS_PER_TICK);
        while scene.queued != 0 {
            scene.spawn_queued(now, &ctx.simulation());
        }
        assert_eq!(scene.entities.len(), 50_000);
        assert!(scene.entities.values().all(|mob| mob.kind.id == pig.id));
        let positions = scene
            .entities
            .values()
            .map(|mob| BlockPos::containing_vec3(&mob.motion.position))
            .collect::<HashSet<_>>();
        assert_eq!(positions.len(), 50_000);
        let loaded = (-32..=32)
            .flat_map(|x| (-32..=32).map(move |z| (x, z)))
            .collect();
        let mut tracker = MobTracker::default();
        let mut observed = HashSet::new();
        loop {
            let changes = tracker.collect(&scene, &loaded, (0, 0));
            assert!(changes.added.len() <= TRACKED_ADDITIONS_PER_TICK);
            assert!(changes.removed.is_empty());
            if changes.added.is_empty() {
                break;
            }
            for (id, _) in changes.added {
                assert!(observed.insert(id));
            }
        }
        assert_eq!(observed.len(), 50_000);
        assert!(observed.into_iter().all(|id| tracker.sees(id)));
    }
}
