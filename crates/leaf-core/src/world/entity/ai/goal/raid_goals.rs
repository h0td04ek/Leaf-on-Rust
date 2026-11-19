//! Перенос рейд-целей `net.minecraft.world.entity.ai.goal` —
//! `PathfindToRaidGoal` (Raider идёт к центру рейда и рекрутит
//! ближайших рейдеров без рейда).

use crate::core::block_pos::BlockPos;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

/// `PathfindToRaidGoal` — рейдер без цели идёт к центру активного
/// рейда; каждые 20 тиков рекрутит рейдеров в 16 блоках без
/// активного рейда (`Raids.canJoinRaid` → `raid.joinRaid`).
pub struct PathfindToRaidGoal {
    /// `recruitmentTick`.
    pub recruitment_tick: i32,
}

impl PathfindToRaidGoal {
    /// `RECRUITMENT_SEARCH_TICK_DELAY`.
    pub const RECRUITMENT_SEARCH_TICK_DELAY: i32 = 20;
    /// `SPEED_MODIFIER`.
    pub const SPEED_MODIFIER: f64 = 1.0;

    /// `new(mob)` — MOVE-флаг.
    pub fn new() -> Self {
        Self { recruitment_tick: 0 }
    }

    /// `recruitNearby(raid)` — рейдеры в bb(16) без активного
    /// рейда и `Raids.canJoinRaid` → `raid.joinRaid(wave,
    /// raider, null, true)`.
    fn recruit_nearby(&self, mob: &Mob, raid: &Rc<RefCell<crate::world::entity::raid::Raid>>) {
        if !raid.borrow().is_active() {
            return;
        }
        let Some(level) = mob.level() else {
            return;
        };
        let box_ = mob.living.entity.get_bounding_box().inflate(16.0, 16.0, 16.0);
        let wave = raid.borrow().get_groups_spawned();
        let mut candidates: Vec<LivingEntity> = Vec::new();
        for e in level.entities_in_box(&box_) {
            if !e.is_living_entity() {
                continue;
            }
            let le = LivingEntity::from_entity(e);
            // !raider1.hasActiveRaid() && Raids.canJoinRaid(raider1)
            let raid_id = le.raider_current_raid_id();
            let has_active_raid = raid_id >= 0
                && level
                    .raids()
                    .and_then(|r| r.borrow().get(raid_id))
                    .map(|raid_rc| raid_rc.borrow().is_active())
                    .unwrap_or(false);
            if !has_active_raid
                && crate::world::entity::raid::Raids::can_join_raid(&le)
            {
                candidates.push(le);
            }
        }
        for le in candidates {
            // joinRaid(level, groupsSpawned, raider, null, true)
            let health = le.get_health();
            raid.borrow_mut().join_raid(wave, le.entity.uuid, health, true);
            // setCurrentRaid/setWave/canJoinRaid/ticksOutside —
            // на стороне raider (view-хуки).
        }
    }
}

impl Default for PathfindToRaidGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for PathfindToRaidGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.get_target().is_some() || mob.has_controlling_passenger() {
            return false;
        }
        if !mob.has_active_raid() {
            return false;
        }
        let Some(raid) = mob.get_current_raid() else {
            return false;
        };
        if raid.borrow().is_over() {
            return false;
        }
        let in_village = mob
            .level()
            .map(|l| l.is_village_at(mob.living.entity.block_position()))
            .unwrap_or(false);
        !in_village
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        if !mob.has_active_raid() {
            return false;
        }
        let Some(raid) = mob.get_current_raid() else {
            return false;
        };
        if raid.borrow().is_over() {
            return false;
        }
        let in_village = mob
            .level()
            .map(|l| l.is_village_at(mob.living.entity.block_position()))
            .unwrap_or(false);
        !in_village
    }

    fn tick(&mut self, mob: &mut Mob) {
        if !mob.has_active_raid() {
            return;
        }
        let Some(raid) = mob.get_current_raid() else {
            return;
        };
        let tick_count = mob.living.entity.tick_count();
        if tick_count > self.recruitment_tick {
            self.recruitment_tick = tick_count + Self::RECRUITMENT_SEARCH_TICK_DELAY;
            self.recruit_nearby(mob, &raid);
        }
        if !mob.is_path_finding() {
            let center = raid.borrow().get_center();
            // Vec3.atBottomCenterOf(center) = (x+0.5, y, z+0.5).
            let bottom = Vec3::new(
                center.x as f64 + 0.5,
                center.y as f64,
                center.z as f64 + 0.5,
            );
            if let Some(pos_towards) =
                crate::world::entity::ai::util::default_random_pos::get_pos_towards(
                    mob,
                    15,
                    4,
                    bottom,
                    std::f64::consts::FRAC_PI_2,
                )
            {
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_pos(
                        mob,
                        pos_towards.x,
                        pos_towards.y,
                        pos_towards.z,
                        Self::SPEED_MODIFIER,
                    );
                }
            }
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}
