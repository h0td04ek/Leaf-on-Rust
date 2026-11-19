//! Перенос `net.minecraft.world.entity.npc.villager.Villager` +
//! `VillagerDataHolder` — villager-специфика `Mob`.
//!//! В Java `Villager extends AbstractVillager extends
//! AgeableMob` — здесь villager-поля живут в `Mob`
//! (`villager_data`, `villager_xp`, `gossips`, `offers` и т.д.),
//! а этот модуль — `impl Mob` методы, соответствующие
//! `Villager`-API. Поведения работают с `&mut Mob` напрямую.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::global_pos::GlobalPos;
use crate::core::holder::Holder;
use crate::entity::schedule::activity::{self, Activity};
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use crate::world::entity::ai::brain::{ActivitySchedule, Brain};
use crate::world::entity::ai::gossip::gossip::GossipContainer;
use crate::world::entity::ai::memory::memory::{self, MemoryModuleType};
use crate::world::entity::mob::Mob;
use crate::world::entity::npc::villager::villager_data::{self, VillagerData};
use crate::world::entity::npc::villager::villager_profession::{self, VillagerProfession};
use crate::world::entity::npc::villager::villager_type::VillagerType;
use crate::world::item::trading::merchant_offers::MerchantOffers;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;

/// `Villager.POI_MEMORIES`-ключи.
pub fn poi_memories() -> Vec<MemoryModuleType> {
    vec![
        memory::keys::job_site(),
        memory::keys::potential_job_site(),
        memory::keys::secondary_job_site(),
        memory::keys::home(),
        memory::keys::meeting_point(),
    ]
}

/// `Villager.createDefaultVillagerData()`.
pub fn create_default_villager_data() -> VillagerData {
    villager_data::default_villager_data()
}

/// `Villager` — инициализация villager-полей на `Mob`
/// (аналог конструктора `Villager(type, level, villagerType)`).
pub fn init_villager(mob: &mut Mob, ty: Holder<VillagerType>) {
    mob.villager_data = Some(VillagerData::new(
        ty,
        villager_data::holder_for_profession(&villager_profession::none()),
        1,
    ));
    mob.gossips = Some(GossipContainer::new());
    mob.villager_xp = 0;
}

/// `isVillager`-проверка.
pub fn is_villager(mob: &Mob) -> bool {
    mob.villager_data.is_some()
}

/// Расписание активности жителя — упрощённая модель
/// `EnvironmentAttributes.VILLAGER_ACTIVITY`/`BABY_VILLAGER_ACTIVITY`.
struct VillagerActivitySchedule {
    baby: bool,
}

impl ActivitySchedule for VillagerActivitySchedule {
    fn activity_at(&self, level: &dyn LevelReader, _pos: Vec3) -> Arc<Activity> {
        if self.baby {
            return activity::activity_by_name("play")
                .or_else(|| activity::activity_by_name("idle"))
                .expect("activity registry not bootstrapped");
        }
        let t = level.day_time().rem_euclid(24000);
        let name = if t >= 12542 && t < 23460 { "rest" } else { "idle" };
        activity::activity_by_name(name)
            .or_else(|| activity::activity_by_name("idle"))
            .expect("activity registry not bootstrapped")
    }
}

impl Mob {
    /// `getVillagerData` — паникует если не villager (как Java
    /// ClassCastException при неверном использовании).
    pub fn get_villager_data(&self) -> &VillagerData {
        self.villager_data.as_ref().expect("not a villager")
    }

    /// `setVillagerData` — offers сбрасываются при смене
    /// профессии.
    pub fn set_villager_data(&mut self, data: VillagerData) {
        if let Some(old) = &self.villager_data {
            if !profession_eq(&old.profession, &data.profession) {
                self.offers = None;
            }
        }
        self.villager_data = Some(data);
    }

    /// `getVillagerXp`.
    pub fn get_villager_xp(&self) -> i32 {
        self.villager_xp
    }

    /// `setVillagerXp`.
    pub fn set_villager_xp(&mut self, xp: i32) {
        self.villager_xp = xp;
    }

    /// `rewardTradeXp`-эквивалент: xp + level-up.
    pub fn add_villager_xp(&mut self, xp: i32) {
        self.villager_xp += xp;
        loop {
            let level = self.get_villager_data().level;
            if !VillagerData::can_level_up(level)
                || self.villager_xp < VillagerData::get_max_xp_per_level(level)
            {
                break;
            }
            self.villager_xp -= VillagerData::get_max_xp_per_level(level);
            let new_data = self.get_villager_data().with_level(level + 1);
            self.set_villager_data(new_data);
        }
    }

    /// `refreshBrain(level)` — stopAll + copyWithoutBehaviors +
    /// повторная регистрация goal-пакетов.
    pub fn refresh_brain(&mut self, level: &Arc<dyn LevelReader>) {
        if let Some(brain_rc) = self.living.brain.clone() {
            brain_rc.stop_all(level, self);
            let new_brain = Rc::new(brain_rc.copy_without_behaviors());
            self.living.brain = Some(new_brain.clone());
            self.register_brain_goals(level, &new_brain);
        }
    }

    /// `registerBrainGoals` — activities по профессии/возрасту.
    /// Полные пакеты — через `goal_packages` (VillagerGoalPackages).
    pub fn register_brain_goals(&mut self, level: &Arc<dyn LevelReader>, brain: &Brain) {
        let baby = self.living.entity.is_baby();
        brain.set_schedule(Some(Rc::new(VillagerActivitySchedule { baby })));
        if let Some(packages) = self.goal_packages.clone() {
            packages(level, self, brain);
        }
        brain.set_core_activities(HashSet::from([
            activity::activity_by_name("core")
                .or_else(|| activity::activity_by_name("idle"))
                .expect("activity registry not bootstrapped"),
        ]));
        if let Some(def) = activity::activity_by_name("idle") {
            brain.set_default_activity(def.clone());
            brain.set_active_activity_if_possible(def);
        }
        let pos = self.living.entity.position();
        brain.update_activity_from_schedule(level.day_time(), pos, level.as_ref());
    }

    /// `assignProfessionWhenSpawned`.
    pub fn assign_profession_when_spawned(&self) -> bool {
        self.assign_profession_when_spawned
    }

    /// `playWorkSound` — id звука профессии (caller играет через
    /// level.play_sound).
    pub fn villager_work_sound(&self) -> Option<Identifier> {
        let data = self.villager_data.as_ref()?;
        if let Holder::Key(k) = &data.profession {
            villager_profession::get(k).and_then(|p| p.work_sound)
        } else {
            None
        }
    }

    /// `releaseAllPois` — освободить все POI-билеты из памяти.
    pub fn release_all_pois(&mut self, level: &Arc<dyn LevelReader>) {
        for module in poi_memories() {
            self.release_poi(level, &module);
        }
    }

    /// `releasePoi(moduleType)`.
    pub fn release_poi(
        &mut self,
        level: &Arc<dyn LevelReader>,
        module: &MemoryModuleType,
    ) {
        let mem: Option<GlobalPos> = self.living.brain_get(module);
        if let Some(gp) = mem {
            if let Some(pm) = level.poi_manager() {
                let _ = pm.release(gp.pos);
            }
        }
        self.living.brain_erase(module);
    }

    /// `canBreed` — villager-специфика.
    pub fn can_breed_villager(&self) -> bool {
        self.living.entity.is_adult() && !self.living.entity.is_in_love()
    }

    /// `hasExcessFood` — FOOD_POINTS (bread=4, potato/carrot/
    /// beetroot=1) сумма >= 12. Инвентарь — через
    /// `villager_inventory` когда портируется.
    pub fn villager_food_points(&self) -> i32 {
        0
    }

    /// `wantsMoreFood`.
    pub fn wants_more_food(&self) -> bool {
        self.villager_food_points() < 12
    }

    /// `wantsToSpawnGolem(gameTime)` — базовая проверка;
    /// полная — в `VillagerPanicTrigger`.
    pub fn wants_to_spawn_golem(&self, _game_time: i64) -> bool {
        false
    }

    /// `updateTrades` — VillagerTrades (отдельный порт).
    pub fn update_trades(&mut self, _level: &Arc<dyn LevelReader>) {}

    /// `gossip(level, target, gameTime)` — decay обоих
    /// контейнеров (упрощённо).
    pub fn villager_gossip(&mut self, target: &mut Mob, _game_time: i64) {
        if let Some(g) = &mut self.gossips {
            g.decay();
        }
        if let Some(g) = &mut target.gossips {
            g.decay();
        }
    }

    /// `spawnGolemIfNeeded` — level-хук.
    pub fn spawn_golem_if_needed(&self, level: &Arc<dyn LevelReader>, game_time: i64, min_villager_amount: i32) {
        level.spawn_golem_if_needed(&self.living.entity, game_time, min_villager_amount);
    }

    /// `getOffers`/`setOffers`.
    pub fn get_offers(&self) -> Option<&MerchantOffers> {
        self.offers.as_ref()
    }
    pub fn set_offers(&mut self, offers: MerchantOffers) {
        self.offers = Some(offers);
    }

    /// `setTradingPlayer`/`getTradingPlayer`.
    pub fn set_trading_player(&mut self, player: Option<Uuid>) {
        self.trading_player = player;
    }
    pub fn get_trading_player(&self) -> Option<Uuid> {
        self.trading_player
    }

    /// `isTrading`.
    pub fn is_trading(&self) -> bool {
        self.trading_player.is_some()
    }

    /// `stopTrading`.
    pub fn stop_trading(&mut self) {
        self.trading_player = None;
    }
}

/// `profession().equals(other)` — по registry-ключу/id.
fn profession_eq(a: &Holder<VillagerProfession>, b: &Holder<VillagerProfession>) -> bool {
    match (a, b) {
        (Holder::Key(x), Holder::Key(y)) => x == y,
        (Holder::Key(x), Holder::Direct(y)) | (Holder::Direct(y), Holder::Key(x)) => {
            x.identifier == y.id
        }
        (Holder::Direct(x), Holder::Direct(y)) => x.id == y.id,
        _ => false,
    }
}

