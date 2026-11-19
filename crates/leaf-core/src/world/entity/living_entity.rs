//! Перенос `net.minecraft.world.entity.LivingEntity` — живая сущность.
//!
//! Ядро: здоровье, поглощение, атрибуты, экипировка, эффекты,
//! боевой трекер, анимации атаки/использования, NBT-сериализация.

use std::collections::HashMap;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::entity::equipment_slot::EquipmentSlot;
use crate::entity::humanoid_arm::HumanoidArm;
use crate::network::chat::component::Component;
use crate::world::damagesource::combat_tracker::CombatTracker;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::effect::mob_effect_instance::MobEffectInstance;
use crate::entity::ai::attributes::attribute_map::AttributeMap;
use crate::entity::ai::attributes::attributes as attribute_ids;
use crate::entity::ai::attributes::default_attributes;
use crate::world::damagesource::combat_rules::CombatRules;
use crate::world::effect::mob_effects;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_reference::EntityReference;
use crate::world::entity::entity_type::EntityType;
use crate::world::item::component::blocks_attacks::BlocksAttacks;
use crate::world::item::equipment::equippable::Equippable;
use crate::resources::identifier::Identifier;
use crate::world::item::item_stack::ItemStack;
use crate::world::phys::vec3::Vec3;

/// `LivingEntity` — сущность с health/attributes/equipment/effects.
pub struct LivingEntity {
    /// Базовая entity.
    pub entity: Entity,
    /// `attributes` — `AttributeMap` (AttributeInstance на атрибут).
    pub attributes: AttributeMap,
    /// `combatTracker`.
    pub combat_tracker: CombatTracker,
    /// `activeEffects` — активные эффекты.
    pub active_effects: HashMap<u64, MobEffectInstance>,
    /// `lastEquipmentItems`.
    pub last_equipment_items: HashMap<EquipmentSlot, ItemStack>,
    /// `swinging`.
    pub swinging: bool,
    /// `swingingArm`.
    pub swinging_arm: HumanoidArm,
    /// `swingTime`.
    pub swing_time: i32,
    /// `removeArrowTime`.
    pub remove_arrow_time: i32,
    /// `removeStingerTime`.
    pub remove_stinger_time: i32,
    /// `hurtTime`.
    pub hurt_time: i32,
    /// `hurtDuration`.
    pub hurt_duration: i32,
    /// `invulnerableTime` — тики неуязвимости после урона.
    pub invulnerable_time: i32,
    /// `deathTime`.
    pub death_time: i32,
    /// `oAttackAnim`, `attackAnim`.
    pub o_attack_anim: f32,
    pub attack_anim: f32,
    /// `attackStrengthTicker`.
    pub attack_strength_ticker: i32,
    /// `itemSwapTicker`.
    pub item_swap_ticker: i32,
    /// `yBodyRot`, `yHeadRot` (+old).
    pub y_body_rot: f32,
    pub y_body_rot_o: f32,
    pub y_head_rot: f32,
    pub y_head_rot_o: f32,
    /// `lastHurtByPlayerMemoryTime`.
    pub last_hurt_by_player_memory_time: i32,
    /// `dead`.
    pub dead: bool,
    /// `hurtMarked` — сущность получила урон в этом тике
    /// (TradeWithPlayerGoal.canUse проверяет).
    pub hurt_marked: bool,
    /// `noActionTime`.
    pub no_action_time: i32,
    /// `lastHurt`.
    pub last_hurt: f32,
    /// `jumping`.
    pub jumping: bool,
    /// `xxa`, `yya`, `zza`.
    pub xxa: f32,
    pub yya: f32,
    pub zza: f32,
    /// `effectsDirty`.
    pub effects_dirty: bool,
    /// `lastHurtByMobTimestamp`.
    pub last_hurt_by_mob_timestamp: i32,
    /// `lastHurtByMob` — EntityReference на атаковавшую LivingEntity.
    pub last_hurt_by_mob: Option<crate::world::entity::entity_reference::EntityReference>,
    /// `lastHurtByPlayer` — EntityReference на игрока.
    pub last_hurt_by_player: Option<crate::world::entity::entity_reference::EntityReference>,
    /// `lastHurtMob` — последняя сущность, причинившая урон (любая).
    pub last_hurt_mob: Option<Entity>,
    /// `speed`.
    pub speed: f32,
    /// `noJumpDelay`.
    pub no_jump_delay: i32,
    /// `absorptionAmount`.
    pub absorption_amount: f32,
    /// `useItem`, `useItemRemaining`.
    pub use_item: ItemStack,
    pub use_item_remaining: i32,
    /// `fallFlyTicks`.
    pub fall_fly_ticks: i32,
    /// `lastPos`.
    pub last_pos: Option<BlockPos>,
    /// `brain` — Brain AI (None для не-Brain мобов; Rc т.к.
    /// Brain тикается через &self + RefCell-поля).
    pub brain: Option<std::rc::Rc<crate::world::entity::ai::brain::Brain>>,
    /// `lastDamageSource`, `lastDamageStamp`.
    pub last_damage_source: Option<DamageSource>,
    pub last_damage_stamp: i64,
    /// `autoSpinAttackTicks`, `autoSpinAttackDmg`.
    pub auto_spin_attack_ticks: i32,
    pub auto_spin_attack_dmg: f32,
    /// `xpReward`.
    pub xp_reward: i32,
    /// `wasExperienceConsumed` — опыт уже получен.
    pub was_experience_consumed: bool,
    /// `pose` — текущая поза.
    pub pose: crate::world::entity::pose::Pose,
    /// `isSleeping` — сущность спит.
    pub is_sleeping: bool,
    /// Текущий health.
    pub health: f32,
}

impl LivingEntity {
    /// Shared-flag бит «мёртв» (в Java — через EntityDataAccessor).
    pub const FLAG_DEAD: u8 = 1 << 7;

    /// `new(EntityType, Level)`.
    pub fn new(entity_type: Holder<EntityType>, level: Option<std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>>) -> Self {
        let mut entity = Entity::new(entity_type.clone(), level);
        entity.is_living = true;
        let combat_tracker = CombatTracker::new(entity.clone());
        let type_id = match &entity_type {
            Holder::Direct(t) => Some(t.id.clone()),
            Holder::Key(k) => Some(k.identifier.clone()),
            Holder::Reference(_) => None,
        };
        let supplier = type_id
            .as_ref()
            .and_then(|id| default_attributes::get_supplier(id).cloned())
            .unwrap_or_default();
        let mut attributes = AttributeMap::new(supplier);
        let health = attributes
            .get_value(&attribute_ids::max_health())
            .clamp(0.0, f64::MAX) as f32;
        Self {
            entity,
            attributes,
            combat_tracker,
            active_effects: HashMap::new(),
            last_equipment_items: HashMap::new(),
            swinging: false,
            swinging_arm: HumanoidArm::Right,
            swing_time: 0,
            remove_arrow_time: 0,
            remove_stinger_time: 0,
            hurt_time: 0,
            hurt_duration: 0,
            invulnerable_time: 0,
            death_time: 0,
            o_attack_anim: 0.0,
            attack_anim: 0.0,
            attack_strength_ticker: 0,
            item_swap_ticker: 0,
            y_body_rot: 0.0,
            y_body_rot_o: 0.0,
            y_head_rot: 0.0,
            y_head_rot_o: 0.0,
            last_hurt_by_player_memory_time: 0,
            dead: false,
            hurt_marked: false,
            no_action_time: 0,
            last_hurt: 0.0,
            jumping: false,
            xxa: 0.0,
            yya: 0.0,
            zza: 0.0,
            effects_dirty: true,
            last_hurt_by_mob_timestamp: 0,
            last_hurt_by_mob: None,
            last_hurt_by_player: None,
            last_hurt_mob: None,
            speed: 0.0,
            no_jump_delay: 0,
            absorption_amount: 0.0,
            use_item: ItemStack::empty(),
            use_item_remaining: 0,
            fall_fly_ticks: 0,
            last_pos: None,
            brain: None,
            last_damage_source: None,
            last_damage_stamp: 0,
            auto_spin_attack_ticks: 0,
            auto_spin_attack_dmg: 0.0,
            xp_reward: 0,
            was_experience_consumed: false,
            pose: crate::world::entity::pose::Pose::Standing,
            is_sleeping: false,
            health,
        }
    }

    /// `getHealth()`.
    pub fn get_health(&self) -> f32 {
        self.health
    }

    /// `getMaxHealth()` — `MAX_HEALTH` атрибут.
    pub fn get_max_health(&mut self) -> f32 {
        self.attributes.get_value(&attribute_ids::max_health()) as f32
    }

    /// `isAlive()`.
    pub fn is_alive(&self) -> bool {
        !self.dead && self.get_health() > 0.0
    }

    /// `isDeadOrDying()`.
    pub fn is_dead_or_dying(&self) -> bool {
        self.dead || self.get_health() <= 0.0
    }

    /// `getAbsorptionAmount()`.
    pub fn get_absorption_amount(&self) -> f32 {
        self.absorption_amount
    }

    /// `setAbsorptionAmount(amount)`.
    pub fn set_absorption_amount(&mut self, amount: f32) {
        self.absorption_amount = amount.clamp(0.0, self.get_max_health());
    }

    /// `setHealth(health)` — через attribute в Java.
    pub fn set_health(&mut self, health: f32) {
        self.health = health.clamp(0.0, self.get_max_health());
    }

    /// `heal(amount)`.
    pub fn heal(&mut self, amount: f32) {
        self.set_health(self.get_health() + amount);
    }

    /// `getItemBySlot(slot)`.
    pub fn get_item_by_slot(&self, slot: EquipmentSlot) -> &ItemStack {
        self.last_equipment_items.get(&slot).unwrap_or(ItemStack::empty_static())
    }

    /// `setItemSlot(slot, stack)`.
    pub fn set_item_slot(&mut self, slot: EquipmentSlot, stack: ItemStack) {
        self.last_equipment_items.insert(slot, stack);
        self.effects_dirty = true;
    }

    /// `getMainHandItem()`.
    pub fn get_main_hand_item(&self) -> &ItemStack {
        self.get_item_by_slot(EquipmentSlot::Mainhand)
    }

    /// `getOffhandItem()`.
    pub fn get_offhand_item(&self) -> &ItemStack {
        self.get_item_by_slot(EquipmentSlot::Offhand)
    }

    /// `getUseItem()`.
    pub fn get_use_item(&self) -> &ItemStack {
        &self.use_item
    }

    /// `isUsingItem()`.
    pub fn is_using_item(&self) -> bool {
        !self.use_item.is_empty() && self.use_item_remaining > 0
    }

    /// `startUsingItem(hand)` — берёт предмет из руки, запускает
    /// использование на `getUseDuration` тиков.
    pub fn start_using_item(&mut self, hand: HumanoidArm) {
        let item = self
            .get_item_by_slot(hand.as_equipment_slot())
            .clone();
        if !item.is_empty() && !self.is_using_item() {
            self.use_item_remaining = item.get_use_duration_ticks();
            self.use_item = item;
            self.swinging_arm = hand;
            self.swinging = true;
            self.swing_time = 0;
        }
    }

    /// `isHolding(item)` — предмет в основной или второй руке.
    pub fn is_holding_item(&self, item_id: &crate::resources::identifier::Identifier) -> bool {
        self.is_holding(|stack| stack.is(item_id))
    }

    /// `isHolding(Predicate<ItemStack>)`.
    pub fn is_holding(&self, predicate: impl Fn(&ItemStack) -> bool) -> bool {
        predicate(self.get_main_hand_item()) || predicate(self.get_offhand_item())
    }

    /// `swing(hand)`.
    pub fn swing(&mut self, hand: HumanoidArm) {
        self.swinging_arm = hand;
        self.swinging = true;
        self.swing_time = -1;
        self.entity.set_delta_movement(0.0, 0.0, 0.0);
    }

    /// `tick()`.
    pub fn tick(&mut self) {
        self.entity.tick_count += 1;
        if self.swinging {
            self.swing_time += 1;
            if self.swing_time >= 25 {
                self.swinging = false;
                self.swing_time = 0;
            }
        }
        if self.hurt_time > 0 { self.hurt_time -= 1; }
        if self.hurt_duration > 0 { self.hurt_duration -= 1; }
        if self.dead {
            self.tick_death();
        } else if self.death_time > 0 {
            self.death_time += 1;
        }
        if self.no_action_time > 0 { self.no_action_time -= 1; }
        if self.attack_strength_ticker > 0 { self.attack_strength_ticker -= 1; }
        if self.item_swap_ticker > 0 { self.item_swap_ticker -= 1; }
        if self.remove_arrow_time > 0 { self.remove_arrow_time -= 1; }
        if self.remove_stinger_time > 0 { self.remove_stinger_time -= 1; }
        if self.use_item_remaining > 0 { self.use_item_remaining -= 1; }
        self.combat_tracker.tick();
        self.combat_tracker.mob = self.entity.clone();
        self.o_attack_anim = self.attack_anim;
        self.y_body_rot_o = self.y_body_rot;
        self.y_head_rot_o = self.y_head_rot;
    }

    /// `tickDeath()` — инкремент deathTime; на 20 тиках —
    /// `EntityEvent.POOF` (60) + `remove(KILLED)`.
    pub fn tick_death(&mut self) {
        self.death_time += 1;
        if self.death_time >= 20 && !self.entity.is_removed() {
            if let Some(level) = self.entity.level.clone() {
                // EntityEvent.POOF = 60
                level.broadcast_entity_event(self.entity.id, 60);
            }
            self.remove_living(crate::world::entity::entity::RemovalReason::Killed);
        }
    }

    /// `die(source)` — полная цепочка `LivingEntity.die` (1.21.11):
    /// kill credit, stop sleeping/using, dead-флаг, combat recheck,
    /// `killedEntity`-проверка, `dropAllDeathLoot`, `createWitherRose`,
    /// broadcast `EntityEvent.DEATH`, pose DYING.
    pub fn die(&mut self, source: DamageSource) {
        if !self.entity.is_removed() && !self.dead {
            let causing_entity = source.get_causing_entity().cloned();
            if let Some(mut kill_credit) = self.get_kill_credit() {
                let killed = self.entity.clone();
                kill_credit.award_kill_score(&killed, &source);
            }
            if self.is_sleeping {
                self.stop_sleeping();
            }
            self.stop_using_item();
            self.dead = true;
            self.combat_tracker.recheck_status();
            if let Some(level) = self.entity.level.clone() {
                let killed_ok = match &causing_entity {
                    None => true,
                    Some(e) => e.killed_entity(level.as_ref(), &self.entity, &source),
                };
                if killed_ok {
                    self.entity.game_event(&crate::world::level::gameevent::game_event::entity_die());
                    self.drop_all_death_loot(&level, &source);
                    self.create_wither_rose();
                }
                // EntityEvent.DEATH = 3
                level.broadcast_entity_event(self.entity.id, 3);
            }
            self.set_pose(crate::world::entity::pose::Pose::Dying);
            self.death_time = 0;
        }
    }

    /// `onKilled(source)`.
    pub fn on_killed(entity: &mut Entity, _source: DamageSource) {
        entity.set_shared_flag(LivingEntity::FLAG_DEAD, true);
    }

    /// `hurt(source, amount)` — клиентский алиас `hurtServer`.
    pub fn hurt(&mut self, source: DamageSource, amount: f32) -> bool {
        self.hurt_server(source, amount)
    }

    /// `hurtServer(level, damageSource, amount)` — полная серверная
    /// цепочка урона `LivingEntity.hurtServer` (1.21.11).
    pub fn hurt_server(&mut self, source: DamageSource, amount: f32) -> bool {
        if self.is_invulnerable_to(&source) {
            return false;
        }
        if self.is_dead_or_dying() {
            return false;
        }
        if source.is_fire() && self.has_effect(&mob_effects::fire_resistance()) {
            return false;
        }
        self.no_action_time = 0;
        let mut amount = amount.max(0.0);
        let use_item = self.get_use_item().clone();
        let blocked = self.apply_item_blocking(&source, amount);
        amount -= blocked;
        let flag = blocked > 0.0;
        if source.is_tag(&crate::tags::damage_type_tags::keys::is_freezing())
            && self.entity_type_is_freeze_hurts_extra()
        {
            amount *= 5.0;
        }
        if source.damages_helmet() {
            let head = self.get_item_by_slot(EquipmentSlot::Head).clone();
            if !head.is_empty() {
                self.hurt_helmet(&source, amount);
                amount *= 0.75;
            }
        }
        if !amount.is_finite() {
            amount = f32::MAX;
        }
        let mut flag1 = true;
        if self.invulnerable_time > 10 && !source.bypasses_cooldown() {
            if amount <= self.last_hurt {
                return false;
            }
            self.actually_hurt(&source, amount - self.last_hurt);
            self.last_hurt = amount;
            flag1 = false;
        } else {
            self.last_hurt = amount;
            self.invulnerable_time = 20;
            self.actually_hurt(&source, amount);
            self.hurt_duration = 10;
            self.hurt_time = self.hurt_duration;
        }
        self.resolve_mob_responsible_for_damage(&source);
        self.resolve_player_responsible_for_damage(&source);
        if flag1 {
            if flag {
                if let Some(ba) = use_item.get_component::<BlocksAttacks>(
                    &Identifier::minecraft("blocks_attacks"),
                ) {
                    if let Some(sound) = ba.on_blocked() {
                        self.entity.play_sound(Some(sound), 1.0, 0.8 + rand_f32() * 0.4);
                    }
                }
            }
            if !source.no_impact() && (!flag || amount > 0.0) {
                self.mark_hurt();
            }
            if !source.no_knockback() {
                let mut d = 0.0f64;
                let mut d1 = 0.0f64;
                if let Some(pos) = source.get_source_position() {
                    d = pos.x - self.entity.pos.x;
                    d1 = pos.z - self.entity.pos.z;
                }
                self.knockback(0.4, d, d1);
                if !flag {
                    self.indicate_damage(d as f32, d1 as f32);
                }
            }
        }
        if self.is_dead_or_dying() {
            if !self.check_totem_death_protection(&source) {
                if flag1 {
                    self.make_sound(self.get_death_sound());
                    self.play_secondary_hurt_sound(&source);
                }
                self.die(source.clone());
            }
        } else if flag1 {
            self.play_hurt_sound(&source);
            self.play_secondary_hurt_sound(&source);
        }
        let flag2 = !flag || amount > 0.0;
        if flag2 {
            self.last_damage_source = Some(source.clone());
            self.last_damage_stamp = self.entity.tick_count as i64;
            // `mobEffectInstance.onMobHurt` для всех активных эффектов
            let effects: Vec<MobEffectInstance> =
                self.active_effects.values().cloned().collect();
            let level = self.entity.level.clone();
            let level_ref: Option<&dyn crate::world::level::level_reader::LevelReader> =
                level.as_deref();
            for inst in &effects {
                inst.on_mob_hurt(level_ref, self, &source, amount);
            }
        }
        flag2
    }

    /// `isInvulnerableTo(source)` — `isInvulnerable() &&
    /// !BYPASSES_INVULNERABILITY`.
    pub fn is_invulnerable_to(&self, source: &DamageSource) -> bool {
        self.entity.is_invulnerable() && !source.bypasses_invulnerability()
    }

    /// `applyItemBlocking(level, damageSource, damageAmount)` —
    /// блок щитом/предметом с `blocks_attacks`.
    pub fn apply_item_blocking(
        &mut self,
        damage_source: &DamageSource,
        damage_amount: f32,
    ) -> f32 {
        if damage_amount <= 0.0 {
            return 0.0;
        }
        let Some(item_blocking_with) = self.get_item_blocking_with().cloned() else {
            return 0.0;
        };
        let Some(blocks_attacks) = item_blocking_with.get_component::<BlocksAttacks>(
            &Identifier::minecraft("blocks_attacks"),
        ) else {
            return 0.0;
        };
        if let Some(bypassed_by) = &blocks_attacks.bypassed_by {
            if damage_source.is_tag(bypassed_by) {
                return 0.0;
            }
        }
        let acos = match damage_source.get_source_position() {
            Some(source_pos) => {
                let view = self.entity.calculate_view_vector(0.0, self.y_head_rot);
                let mut d = source_pos.subtract_vec(&self.entity.pos);
                d.y = 0.0;
                if d.length_squared() > 1.0e-7 {
                    d = d.normalize();
                    view.dot_vec(&d).clamp(-1.0, 1.0).acos() as f32
                } else {
                    std::f32::consts::PI
                }
            }
            None => std::f32::consts::PI,
        };
        let blocked =
            blocks_attacks.resolve_blocked_damage(damage_source, damage_amount, acos as f64);
        if blocked > 0.0 && self.entity.is_player() {
            let item_damage = blocks_attacks.item_damage.apply(blocked);
            if item_damage > 0 {
                let mut stack = self.get_item_blocking_with().cloned().unwrap_or_else(ItemStack::empty);
                let slot = self.get_used_item_hand().as_equipment_slot();
                stack.hurt_and_break(item_damage, self, Some(slot));
                self.set_item_slot(slot, stack);
            }
        }
        if blocked > 0.0
            && !damage_source.is_projectile()
            && damage_source
                .get_causing_entity()
                .map(|e| e.is_living_entity())
                .unwrap_or(false)
        {
            self.block_using_item(damage_source);
        }
        blocked
    }

    /// `getItemBlockingWith()` — предмет, которым блокируют.
    pub fn get_item_blocking_with(&self) -> Option<&ItemStack> {
        if !self.is_using_item() {
            return None;
        }
        if self
            .use_item
            .has_component(&Identifier::minecraft("blocks_attacks"))
        {
            let ba = self.use_item.get_component::<BlocksAttacks>(
                &Identifier::minecraft("blocks_attacks"),
            );
            if let Some(ba) = ba {
                let used = self.use_item.get_use_duration_ticks()
                    - self.use_item_remaining;
                if used >= ba.block_delay_ticks() {
                    return Some(&self.use_item);
                }
            }
        }
        None
    }

    /// `isBlocking()`.
    pub fn is_blocking(&self) -> bool {
        self.get_item_blocking_with().is_some()
    }

    /// `playSecondaryHurtSound` — звук thorns.
    pub fn play_secondary_hurt_sound(&mut self, damage_source: &DamageSource) {
        if damage_source
            .get_type()
            .value()
            .map(|dt| dt.id == Identifier::minecraft("thorns"))
            .unwrap_or(false)
        {
            let sound = crate::sounds::sound_events::thorns_hit();
            self.entity.play_sound(Some(&sound), 1.0, 1.0);
        }
    }

    /// `resolveMobResponsibleForDamage` — `lastHurtByMob`.
    pub fn resolve_mob_responsible_for_damage(&mut self, damage_source: &DamageSource) {
        if let Some(causing) = damage_source.get_causing_entity() {
            if causing.is_living_entity()
                && !damage_source.is_tag(&crate::tags::damage_type_tags::keys::no_anger())
            {
                self.last_hurt_by_mob_timestamp =
                    self.entity.tick_count + DAMAGE_SOURCE_TIMEOUT as i32;
            }
        }
    }

    /// `resolvePlayerResponsibleForDamage` — `lastHurtByPlayer`.
    pub fn resolve_player_responsible_for_damage(&mut self, damage_source: &DamageSource) {
        if let Some(causing) = damage_source.get_causing_entity() {
            if causing.is_player() {
                self.last_hurt_by_player_memory_time = 100;
            }
        }
    }

    /// `blockUsingItem(level, entity)` — контратакующий knockback.
    pub fn block_using_item(&mut self, _damage_source: &DamageSource) {}

    /// `blockedByItem(entity)` — отбрасывание блокирующего.
    pub fn blocked_by_item(&mut self, _entity: &LivingEntity) {}

    /// `checkTotemDeathProtection` — тотем бессмертия.
    pub fn check_totem_death_protection(&mut self, damage_source: &DamageSource) -> bool {
        if damage_source.bypasses_invulnerability() {
            return false;
        }
        let mut found: Option<crate::world::item::component::death_protection::DeathProtection> = None;
        for hand_is_main in [true, false] {
            let stack = if hand_is_main {
                self.get_main_hand_item().clone()
            } else {
                self.get_offhand_item().clone()
            };
            if let Some(dp) = stack.get_component::<crate::world::item::component::death_protection::DeathProtection>(
                &Identifier::minecraft("death_protection"),
            ) {
                let mut s = stack;
                s.shrink(1);
                if hand_is_main {
                    self.set_item_slot(EquipmentSlot::Mainhand, s);
                } else {
                    self.set_item_slot(EquipmentSlot::Offhand, s);
                }
                found = Some(dp);
                break;
            }
        }
        let has_toten = found.is_some();
        if let Some(dp) = found {
            self.set_health(1.0);
            for effect in &dp.death_effects {
                effect.apply(self);
            }
        }
        has_toten
    }

    /// `playHurtSound` — звук получения урона.
    pub fn play_hurt_sound(&mut self, damage_source: &DamageSource) {
        let sound = self.get_hurt_sound(damage_source);
        self.make_sound(sound);
    }

    /// `makeSound(sound)`.
    pub fn make_sound(&mut self, sound: Option<crate::sounds::sound_event::SoundEvent>) {
        if let Some(s) = sound {
            let holder = crate::core::holder::Holder::Direct(s);
            self.entity.play_sound(Some(&holder), self.get_voice_volume(), self.get_voice_pitch());
        }
    }

    /// `hurtHelmet` — базовая пустая (переопределяется в ArmorStand/Player).
    pub fn hurt_helmet(&mut self, _damage_source: &DamageSource, _damage_amount: f32) {}

    /// `hurtArmor` — базовая пустая.
    pub fn hurt_armor(&mut self, _damage_source: &DamageSource, _damage_amount: f32) {}

    /// `doHurtEquipment` — износ экипировки при уроне.
    pub fn do_hurt_equipment(
        &mut self,
        damage_source: &DamageSource,
        damage_amount: f32,
        slots: &[EquipmentSlot],
    ) {
        if damage_amount <= 0.0 {
            return;
        }
        let amount = ((damage_amount / 4.0).max(1.0)) as i32;
        for &slot in slots {
            let stack = self.get_item_by_slot(slot).clone();
            let equippable = stack.get_component::<Equippable>(
                &Identifier::minecraft("equippable"),
            );
            if let Some(eq) = equippable {
                if eq.damage_on_hurt
                    && stack.is_damageable_item()
                    && stack.can_be_hurt_by(damage_source)
                {
                    let mut s = stack;
                    s.hurt_and_break(amount, self, Some(slot));
                    self.set_item_slot(slot, s);
                }
            }
        }
    }

    /// `getDamageAfterArmorAbsorb` — броня + toughness.
    pub fn get_damage_after_armor_absorb(
        &mut self,
        damage_source: &DamageSource,
        damage_amount: f32,
    ) -> f32 {
        if !damage_source.bypasses_armor() {
            self.hurt_armor(damage_source, damage_amount);
            let armor = self.get_armor_value() as f32;
            let toughness =
                self.get_attribute_value_by_id(&attribute_ids::armor_toughness()) as f32;
            CombatRules::get_damage_after_absorb(
                &self.entity,
                damage_amount,
                damage_source,
                armor,
                toughness,
                None,
            )
        } else {
            damage_amount
        }
    }

    /// `getDamageAfterMagicAbsorb` — resistance + чары защиты.
    pub fn get_damage_after_magic_absorb(
        &mut self,
        damage_source: &DamageSource,
        mut damage_amount: f32,
    ) -> f32 {
        if damage_source.bypasses_effects() {
            return damage_amount;
        }
        if self.has_effect(&mob_effects::resistance())
            && !damage_source.bypasses_resistance()
        {
            let amplifier = self
                .get_effect_instance(&mob_effects::resistance())
                .map(|i| i.amplifier())
                .unwrap_or(0);
            let reduction = ((amplifier + 1) * 5).min(25);
            damage_amount =
                (damage_amount * (25 - reduction) as f32 / 25.0).max(0.0);
        }
        if damage_amount <= 0.0 {
            return 0.0;
        }
        if damage_source.bypasses_enchantments() {
            return damage_amount;
        }
        let damage_protection =
            crate::world::item::enchantment::enchantment_helper::get_damage_protection(
                self, damage_source,
            );
        if damage_protection > 0.0 {
            damage_amount = CombatRules::get_damage_after_magic_absorb(
                damage_amount,
                damage_protection,
            );
        }
        damage_amount
    }

    /// `actuallyHurt` — применение урона после всех поглощений.
    pub fn actually_hurt(&mut self, damage_source: &DamageSource, amount: f32) {
        if self.is_invulnerable_to(damage_source) {
            return;
        }
        let amount = self.get_damage_after_armor_absorb(damage_source, amount);
        let amount = self.get_damage_after_magic_absorb(damage_source, amount);
        // Vanilla: absorbed = min(absorption, amount); healthDamage = amount - absorbedAbsorbed
        let absorption = self.get_absorption_amount();
        let absorbed = absorption.min(amount);
        self.set_absorption_amount(absorption - absorbed);
        let health_damage = amount - absorbed;
        if health_damage > 0.0 {
            self.combat_tracker
                .record_damage(damage_source.clone(), health_damage);
            self.set_health(self.get_health() - health_damage);
            self.entity.game_event(
                &crate::world::level::gameevent::game_event::entity_damage(),
            );
        }
    }

    /// `onEquippedItemBroken(stack, slot)` — предмет сломан.
    pub fn on_equipped_item_broken(
        &mut self,
        stack: &ItemStack,
        slot: Option<EquipmentSlot>,
    ) {
        if let Some(sound) = stack.get_component::<crate::core::holder::Holder<crate::sounds::sound_event::SoundEvent>>(
            &Identifier::minecraft("break_sound"),
        ) {
            self.entity.play_sound(Some(&sound), 1.0, 1.0);
        }
        let _ = slot;
    }

    /// `entityTypeIsFreezeHurtsExtra` — `FREEZE_HURTS_EXTRA_TYPES` тег.
    fn entity_type_is_freeze_hurts_extra(&self) -> bool {
        self.entity
            .entity_type
            .value()
            .map(|t| {
                crate::tags::entity_type_tags::entity_type_is(
                    t,
                    crate::tags::entity_type_tags::keys::freeze_hurts_extra_types(),
                )
            })
            .unwrap_or(false)
    }

    /// `isInvertedHealAndHarm()` — `INVERTED_HEALING_AND_HARM` тег.
    pub fn is_inverted_heal_and_harm(&self) -> bool {
        self.entity
            .entity_type
            .value()
            .map(|t| {
                crate::tags::entity_type_tags::entity_type_is(
                    t,
                    crate::tags::entity_type_tags::keys::inverted_healing_and_harm(),
                )
            })
            .unwrap_or(false)
    }

    /// `randomTeleport(diameter)` — случайный телепорт в радиусе.
    /// Используется `TeleportRandomlyConsumeEffect`.
    pub fn teleport_randomly(&mut self, diameter: f32) {
        let d = diameter as f64;
        let nx = self.entity.pos.x + (rand::random::<f64>() - 0.5) * d;
        let ny = self.entity.pos.y + (rand::random::<f64>() - 0.5) * d.min(4.0);
        let nz = self.entity.pos.z + (rand::random::<f64>() - 0.5) * d;
        self.entity.pos = crate::world::phys::vec3::Vec3::new(nx, ny, nz);
        self.entity.set_delta_movement(0.0, 0.0, 0.0);
        self.no_action_time = 0;
    }

    /// `markHurt()` — `animateHurt` + shared flag.
    pub fn mark_hurt(&mut self) {
        self.hurt_duration = 10;
        self.hurt_time = self.hurt_duration;
        self.entity.set_shared_flag(0x02, true);
    }

    /// `knockback(strength, x, z)` — реальный knockback с
    /// `knockback_resistance` атрибутом.
    pub fn knockback(&mut self, strength: f64, x: f64, z: f64) {
        let strength = strength
            * (1.0 - self.get_attribute_value_by_id(&attribute_ids::knockback_resistance()));
        if strength <= 0.0 {
            return;
        }
        let delta = self.entity.get_delta_movement();
        let mut x = x;
        let mut z = z;
        while x * x + z * z < 1.0e-5 {
            x = (rand_f64() - rand_f64()) * 0.01;
            z = (rand_f64() - rand_f64()) * 0.01;
        }
        let len = (x * x + z * z).sqrt();
        let vx = x / len * strength;
        let vz = z / len * strength;
        let vy = if self.entity.is_on_ground() {
            (delta.y / 2.0 + strength).min(0.4)
        } else {
            delta.y
        };
        self.entity
            .set_delta_movement(delta.x / 2.0 - vx, vy, delta.z / 2.0 - vz);
    }

    /// `indicateDamage(xDistance, zDistance)`.
    pub fn indicate_damage(&mut self, x_distance: f32, z_distance: f32) {
        let _ = (x_distance, z_distance);
    }

    /// `getArmorValue()` — `ARMOR` атрибут (floor).
    pub fn get_armor_value(&mut self) -> i32 {
        self.attributes.get_value(&attribute_ids::armor()).floor() as i32
    }

    /// `getArmorCoverPercentage()` — доля надетых humanoid-armor
    /// слотов от всех armor-слотов (`i1 / i`).
    pub fn get_armor_cover_percentage(&mut self) -> f32 {
        let mut i = 0i32;
        let mut i1 = 0i32;
        for slot in crate::entity::equipment_slot::EquipmentSlot::VALUES {
            if slot.get_type()
                == crate::entity::equipment_slot::EquipmentSlotType::HumanoidArmor
            {
                let stack = self.get_item_by_slot(slot);
                if !stack.is_empty() {
                    i1 += 1;
                }
                i += 1;
            }
        }
        if i > 0 {
            i1 as f32 / i as f32
        } else {
            0.0
        }
    }


    /// `getKillCredit()`.
    pub fn get_kill_credit(&self) -> Option<Entity> {
        self.combat_tracker
            .entries
            .last()
            .and_then(|e| e.source.get_causing_entity().cloned())
    }

    /// `dropAllDeathLoot(level, damageSource)` — полная цепочка:
    /// `shouldDropLoot` → `dropFromLootTable` + `dropCustomDeathLoot`,
    /// затем `dropEquipment` + `dropExperience`.
    pub fn drop_all_death_loot(
        &mut self,
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        source: &DamageSource,
    ) {
        let recently_hit = self.last_hurt_by_player_memory_time > 0;
        if self.should_drop_loot(level.as_ref()) {
            self.drop_from_loot_table(level.as_ref(), source, recently_hit);
            self.drop_custom_death_loot(level.as_ref(), source, recently_hit);
        }
        self.drop_equipment(level.as_ref());
        self.drop_experience_rewards(level, source.get_causing_entity());
    }

    /// `shouldDropLoot(level)` — база `true` (Mob/Player уточняют).
    pub fn should_drop_loot(
        &self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
    ) -> bool {
        !self.entity.is_spectator() && !self.dead
    }

    /// `dropEquipment(level)` — база пустая (Mob переопределяет:
    /// дроп экипировки по шансам/лут-таблице).
    pub fn drop_equipment(
        &mut self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
    ) {
    }

    /// `dropCustomDeathLoot(level, source, recentlyHit)` — база пустая.
    pub fn drop_custom_death_loot(
        &mut self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
        _source: &DamageSource,
        _recently_hit: bool,
    ) {
    }

    /// `dropDeathLoot(level, source)` — Paper-хук, база пустая.
    pub fn drop_death_loot(
        &mut self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
        _source: &DamageSource,
    ) {
    }

    /// `getLootTable()` — `Optional<ResourceKey<LootTable>>`;
    /// база пустая (Mob возвращает свой loot table).
    pub fn get_loot_table(
        &self,
    ) -> Option<crate::resources::identifier::Identifier> {
        None
    }

    /// `getLootTableSeed()`.
    pub fn get_loot_table_seed(&self) -> i64 {
        0
    }

    /// `dropFromLootTable(level, source, playerKill)` —
    /// резолвит `getLootTable` и делегирует в overload.
    pub fn drop_from_loot_table(
        &mut self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        source: &DamageSource,
        player_kill: bool,
    ) {
        if let Some(table) = self.get_loot_table() {
            self.drop_from_loot_table_key(level, source, player_kill, &table);
        }
    }

    /// `dropFromLootTable(level, source, playerKill, lootTable)` —
    /// строит `LootParams` (THIS_ENTITY/ORIGIN/DAMAGE_SOURCE/
    /// ATTACKING_ENTITY/DIRECT_ATTACKING_ENTITY/LAST_DAMAGE_PLAYER)
    /// и спавнит результаты через `spawnAtLocation`.
    pub fn drop_from_loot_table_key(
        &mut self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        source: &DamageSource,
        _player_kill: bool,
        table: &crate::resources::identifier::Identifier,
    ) {
        use crate::world::level::storage::loot::loot_context::LootParams;
        use crate::world::level::storage::loot::parameters::loot_context_params as params;
        let this_entity = self.entity.clone();
        let mut loot_params = LootParams::new()
            .with_parameter(params::this_entity(), this_entity)
            .with_parameter(params::origin(), self.entity.pos)
            .with_parameter(params::damage_source(), source.clone());
        if let Some(attacker) = source.get_causing_entity() {
            loot_params = loot_params.with_parameter(params::attacking_entity(), attacker.clone());
        }
        if let Some(direct) = source.get_direct_entity() {
            loot_params = loot_params.with_parameter(params::direct_attacking_entity(), direct.clone());
        }
        let items = level.generate_loot(table, &loot_params);
        for stack in items {
            if !stack.is_empty() {
                self.entity.spawn_at_location(stack);
            }
        }
    }

    /// `dropFromLootTable(..., dropConsumer)` — вариант с consumer'ом.
    pub fn drop_from_loot_table_with(
        &mut self,
        level: &dyn crate::world::level::level_reader::LevelReader,
        source: &DamageSource,
        player_kill: bool,
        table: &crate::resources::identifier::Identifier,
        mut drop_consumer: impl FnMut(&mut Self, crate::world::item::item_stack::ItemStack),
    ) {
        use crate::world::level::storage::loot::loot_context::LootParams;
        use crate::world::level::storage::loot::parameters::loot_context_params as params;
        let this_entity = self.entity.clone();
        let mut loot_params = LootParams::new()
            .with_parameter(params::this_entity(), this_entity)
            .with_parameter(params::origin(), self.entity.pos)
            .with_parameter(params::damage_source(), source.clone());
        if let Some(attacker) = source.get_causing_entity() {
            loot_params = loot_params.with_parameter(params::attacking_entity(), attacker.clone());
        }
        if let Some(direct) = source.get_direct_entity() {
            loot_params = loot_params.with_parameter(params::direct_attacking_entity(), direct.clone());
        }
        let _ = player_kill;
        let items = level.generate_loot(table, &loot_params);
        for stack in items {
            if !stack.is_empty() {
                drop_consumer(self, stack);
            }
        }
    }

    /// `dropExperience(level, entity)` — `ExperienceOrb.award`
    /// с проверками `wasExperienceConsumed`/`isAlwaysExperienceDropper`/
    /// `MOB_DROPS` game rule.
    pub fn drop_experience_rewards(
        &mut self,
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        entity: Option<&Entity>,
    ) {
        let _ = entity;
        if !self.was_experience_consumed
            && (self.is_always_experience_dropper()
                || (self.last_hurt_by_player_memory_time > 0
                    && self.should_drop_experience()
                    && level.mob_drops()))
        {
            let reward = self.get_experience_reward_for(level.as_ref(), entity);
            crate::world::entity::experience_orb::ExperienceOrb::award(
                Some(level),
                self.entity.pos,
                reward,
            );
        }
    }

    /// `isAlwaysExperienceDropper()` — база false (Player переопределяет).
    pub fn is_always_experience_dropper(&self) -> bool {
        false
    }

    /// `getExperienceReward(level, entity)` — `xpReward`
    /// (+ Looting/прогрессии добавятся с enchantment-эффектами).
    pub fn get_experience_reward_for(
        &self,
        _level: &dyn crate::world::level::level_reader::LevelReader,
        _entity: Option<&Entity>,
    ) -> i32 {
        self.xp_reward
    }

    /// `createWitherRose(entitySource)` — только для WitherBoss;
    /// базовая проверка по entity-type id `wither_boss`.
    pub fn create_wither_rose(&mut self) {
        if let Some(level) = self.entity.level.clone() {
            // WitherBoss не портирован — проверяем тип убийцы через
            // kill credit (entity_type id == wither_boss).
            let is_wither = self
                .get_kill_credit()
                .and_then(|e| e.entity_type.value().map(|t| t.id.clone()))
                .map(|id| id.get_path() == "wither_boss")
                .unwrap_or(false);
            if !is_wither || !level.mob_griefing() {
                return;
            }
            // WitherRose block не портирован — дропаем ItemEntity с
            // предметом wither_rose (vanilla fallback при невозможности
            // установить блок).
            let item_id = Identifier::with_default_namespace("wither_rose").unwrap();
            if let Some(item) = crate::world::item::item::item_by_identifier(&item_id) {
                let stack = ItemStack::new(Holder::Direct(item.as_ref().clone()), 1);
                self.entity.spawn_at_location(stack);
            }
        }
    }

    /// `awardKillScore(killedEntity, source)` — база no-op
    /// (Player начисляет очки убийства).
    pub fn award_kill_score(&mut self, _killed_entity: &Entity, _source: &DamageSource) {}

    /// `getExperienceReward()`.
    pub fn get_experience_reward(&self) -> i32 {
        self.xp_reward
    }

    /// `getMainArm()`.
    pub fn get_main_arm(&self) -> HumanoidArm {
        self.swinging_arm
    }

    /// `addEffect(instance)` → `addEffect(instance, None)` —
    /// полная цепочка `LivingEntity.addEffect` (1.21.11).
    /// Возвращает `true` если эффект изменил состояние.
    pub fn add_effect(&mut self, instance: MobEffectInstance) -> bool {
        self.add_effect_with_source(instance, None)
    }

    /// `addEffect(instance, entity)`.
    pub fn add_effect_with_source(
        &mut self,
        instance: MobEffectInstance,
        _source: Option<&Entity>,
    ) -> bool {
        if !self.can_be_affected(&instance) {
            return false;
        }
        let key = Self::effect_key(&instance);
        let existing = self.active_effects.get(&key).cloned();
        let mut flag = false;
        match existing {
            None => {
                self.active_effects.insert(key, instance.clone());
                self.on_effect_added(&instance);
                flag = true;
                instance.on_effect_added(self);
            }
            Some(mut prev) => {
                if prev.update(&instance) {
                    self.on_effect_updated(&prev, true);
                    flag = true;
                    self.active_effects.insert(key, prev);
                }
            }
        }
        instance.on_effect_started(self);
        flag
    }

    /// `canBeAffected(effectInstance)` — entity-type теги
    /// IMMUNE_TO_INFESTED / IMMUNE_TO_OOZING / IGNORES_POISON_AND_REGEN.
    pub fn can_be_affected(&self, effect_instance: &MobEffectInstance) -> bool {
        use crate::tags::entity_type_tags::{entity_type_is, keys};
        let entity_type = match self.entity.entity_type.value() {
            Some(t) => t,
            None => return true,
        };
        if entity_type_is(entity_type, keys::immune_to_infested())
            && effect_instance.is(&mob_effects::infested())
        {
            return false;
        }
        if entity_type_is(entity_type, keys::immune_to_oozing())
            && effect_instance.is(&mob_effects::oozing())
        {
            return false;
        }
        if entity_type_is(entity_type, keys::ignores_poison_and_regen())
            && (effect_instance.is(&mob_effects::regeneration())
                || effect_instance.is(&mob_effects::poison()))
        {
            return false;
        }
        true
    }

    /// `forceAddEffect(effectInstance, entity)` — обход update-логики.
    pub fn force_add_effect(&mut self, instance: MobEffectInstance) {
        if !self.can_be_affected(&instance) {
            return;
        }
        let key = Self::effect_key(&instance);
        let prev = self.active_effects.insert(key, instance.clone());
        match prev {
            None => {
                self.on_effect_added(&instance);
            }
            Some(_) => {
                self.on_effect_updated(&instance, true);
            }
        }
    }

    /// `onEffectAdded(effectInstance)` — атрибутные модификаторы
    /// + effectsDirty (sendEffectToPassengers — до порта пакетов).
    fn on_effect_added(&mut self, effect_instance: &MobEffectInstance) {
        self.effects_dirty = true;
        effect_instance.add_attribute_modifiers(&mut self.attributes);
    }

    /// `onEffectUpdated(effectInstance, forced)` — при forced
    /// пересоздаются атрибутные модификаторы.
    fn on_effect_updated(&mut self, effect_instance: &MobEffectInstance, forced: bool) {
        self.effects_dirty = true;
        if forced {
            if let Some(e) = effect_instance.effect.value() {
                e.remove_attribute_modifiers(&mut self.attributes);
                e.add_attribute_modifiers(&mut self.attributes, effect_instance.amplifier());
            }
        }
    }

    /// `onEffectsRemoved(effects)` — снятие модификаторов,
    /// `stopUsingItem`, effectsDirty.
    /// (В ванили `onMobRemoved` вызывается отдельно через
    /// `triggerOnDeathMobEffects` при удалении сущности.)
    fn on_effects_removed(&mut self, effects: &[MobEffectInstance]) {
        self.effects_dirty = true;
        for inst in effects {
            inst.remove_attribute_modifiers(&mut self.attributes);
        }
        self.stop_using_item();
    }

    /// `triggerOnDeathMobEffects(level, removalReason)` —
    /// `onMobRemoved` для всех активных эффектов + очистка.
    /// Вызывается из `remove(KILLED|DISCARDED)` на сервере.
    pub fn trigger_on_death_mob_effects(
        &mut self,
        level: &std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>,
        reason: crate::world::entity::entity::RemovalReason,
    ) {
        let level_ref: Option<&dyn crate::world::level::level_reader::LevelReader> =
            Some(level.as_ref());
        let effects: Vec<MobEffectInstance> =
            self.active_effects.values().cloned().collect();
        for inst in &effects {
            // inst.on_mob_removed требует &mut self — вызываем через клон
            // с последующим применением к self нельзя (borrow); делаем
            // через временный приём: эффект не зависит от полей LivingEntity
            // кроме тех, что доступны через entity — передаём self.
            let mut inst2 = inst.clone();
            inst2.on_mob_removed(level_ref, self, reason);
        }
        self.active_effects.clear();
    }

    /// `remove(reason)` — override: death mob effects при
    /// KILLED/DISCARDED + `super.remove`.
    pub fn remove_living(&mut self, reason: crate::world::entity::entity::RemovalReason) {
        if matches!(
            reason,
            crate::world::entity::entity::RemovalReason::Killed
                | crate::world::entity::entity::RemovalReason::Discarded
        ) {
            if let Some(level) = self.entity.level.clone() {
                self.trigger_on_death_mob_effects(&level, reason);
            }
        }
        self.entity.remove(reason);
    }

    fn effect_key(instance: &MobEffectInstance) -> u64 {
        Self::holder_key(&instance.effect)
    }

    /// хэш registry-ключа holder'а эффекта (как `Holder.hashCode`
    /// по underlying id — у нас детерминированный хэш Identifier).
    fn holder_key(effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> u64 {
        let id = effect
            .value()
            .map(|e| e.id.clone())
            .or_else(|| {
                effect
                    .key()
                    .map(|k| k.id().clone())
            });
        id.map(|id| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&id, &mut h);
            std::hash::Hasher::finish(&h)
        })
        .unwrap_or(0)
    }

    /// `hasEffect(effect)`.
    pub fn has_effect(&self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        effect.value().map(|e| {
            self.active_effects.values().any(|i| {
                i.effect.value().map(|x| x.id == e.id).unwrap_or(false)
            })
        }).unwrap_or(false)
    }

    /// `getEffect(effect)` — активный инстанс по holder'у эффекта.
    pub fn get_effect_instance(
        &self,
        effect: &Holder<crate::world::effect::mob_effect::MobEffect>,
    ) -> Option<&MobEffectInstance> {
        effect.value().and_then(|e| {
            self.active_effects.values().find(|i| {
                i.effect.value().map(|x| x.id == e.id).unwrap_or(false)
            })
        })
    }

    /// `getActiveEffects()`.
    pub fn get_active_effects(&self) -> impl Iterator<Item = &MobEffectInstance> {
        self.active_effects.values()
    }

    /// `getDeathSound()`.
    pub fn get_death_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `getHurtSound(source)`.
    pub fn get_hurt_sound(&self, _source: &DamageSource) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `getAmbientSound()`.
    pub fn get_ambient_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `getEatingSound()`.
    pub fn get_eating_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `getFallDamageSound()`.
    pub fn get_fall_damage_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `getSwimSound()`.
    pub fn get_swim_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        None
    }

    /// `isPushable()`.
    pub fn is_pushable(&self) -> bool {
        !self.dead && !self.is_spectator()
    }

    /// `isSpectator()`.
    pub fn is_spectator(&self) -> bool {
        self.entity.is_spectator()
    }

    /// `canBeSeenAsEnemy()`.
    /// `canBeSeenAsEnemy()` — `!isInvulnerable() && canBeSeenByAnyone()`.
    pub fn can_be_seen_as_enemy(&self) -> bool {
        !self.entity.invulnerable && self.can_be_seen_by_anyone()
    }

    /// `isFallFlying()`.
    pub fn is_fall_flying(&self) -> bool {
        self.fall_fly_ticks > 0
    }

    /// `shouldDropExperience()`.
    pub fn should_drop_experience(&self) -> bool {
        self.xp_reward > 0
    }

    /// `getEquipmentSlotsFor` — слоты для синхронизации.
    pub fn get_equipment_slots(&self) -> Vec<EquipmentSlot> {
        vec![
            EquipmentSlot::Mainhand,
            EquipmentSlot::Offhand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Chest,
            EquipmentSlot::Head,
        ]
    }

    /// `collectEquipmentChanges()`.
    pub fn collect_equipment_changes(&mut self) -> Vec<(EquipmentSlot, ItemStack)> {
        let mut changes = Vec::new();
        for slot in self.get_equipment_slots() {
            let current = self.get_item_by_slot(slot).clone();
            let last = self.last_equipment_items.get(&slot).cloned().unwrap_or_else(ItemStack::empty);
            if !ItemStack::matches(&current, &last) {
                changes.push((slot, current.clone()));
                self.last_equipment_items.insert(slot, current);
            }
        }
        changes
    }

    /// `getAttributeValue(attribute)`.
    pub fn get_attribute_value(&mut self, attribute: &Holder<crate::entity::ai::attributes::Attribute>) -> f64 {
        match attribute.value() {
            Some(a) => self.attributes.get_value(&a.id),
            None => attribute
                .key()
                .map(|k| self.attributes.get_value(&k.identifier))
                .unwrap_or(0.0),
        }
    }

    /// `getAttributeValue` по registry-id атрибута.
    pub fn get_attribute_value_by_id(&mut self, attribute: &Identifier) -> f64 {
        self.attributes.get_value(attribute)
    }

    /// `getAttributeValue` read-only (cached, без dirty-пересчёта)
    /// — для &self-контекстов (pathfinding/AI read-paths).
    pub fn get_attribute_value_by_id_ro(&self, attribute: &Identifier) -> f64 {
        self.attributes.get_value_ro(attribute)
    }

    /// `getAttributeBaseValue(attribute)`.
    pub fn get_attribute_base_value_by_id(&mut self, attribute: &Identifier) -> f64 {
        self.attributes.get_base_value(attribute)
    }

    /// `getBlockSpeedFactor()`.
    pub fn get_block_speed_factor(&self) -> f32 {
        1.0
    }

    /// `getFieldOfViewModifier()`.
    pub fn get_field_of_view_modifier(&self) -> f32 {
        1.0
    }

    /// `getLuck()`.
    pub fn get_luck(&self) -> f32 {
        0.0
    }

    /// `getYRot()`.
    pub fn get_y_rot(&self) -> f32 {
        self.entity.get_y_rot()
    }

    /// `getXRot()`.
    pub fn get_y_rot_living(&self) -> f32 {
        self.get_y_rot()
    }

    /// `getXRot()`.
    pub fn get_x_rot(&self) -> f32 {
        self.entity.get_x_rot()
    }

    /// `getLookAngle()`.
    pub fn get_look_angle(&self) -> Vec3 {
        // Entity не имеет getLookAngle — вычисляем из rotation
        let (yaw, pitch) = (self.entity.get_y_rot().to_radians(), self.entity.get_x_rot().to_radians());
        Vec3::new((-yaw.sin() * pitch.cos()) as f64, (-pitch.sin()) as f64, (yaw.cos() * pitch.cos()) as f64)
    }


    /// `isDamageSourceBlocked(source)`.
    pub fn is_damage_source_blocked(&self, _source: &DamageSource) -> bool {
        false
    }

    /// `getCurrentSwingDuration()`.
    pub fn get_current_swing_duration(&self) -> i32 {
        if self.swinging { 25 } else { 0 }
    }

    /// `getUsedItemHand()`.
    pub fn get_used_item_hand(&self) -> HumanoidArm {
        self.swinging_arm
    }

    /// `hasPose(pose)`.
    pub fn has_pose(&self, pose: crate::world::entity::pose::Pose) -> bool {
        self.pose == pose
    }

    /// `setPose(pose)`.
    pub fn set_pose(&mut self, pose: crate::world::entity::pose::Pose) {
        self.pose = pose;
    }

    /// `stopUsingItem()` — сброс use-item состояния.
    pub fn stop_using_item(&mut self) {
        if self.is_using_item() {
            self.use_item = crate::world::item::item_stack::ItemStack::empty();
            self.use_item_remaining = 0;
        }
    }

    /// `isSleeping()`.
    pub fn is_sleeping_entity(&self) -> bool {
        self.is_sleeping
    }

    /// `stopSleeping()`.
    pub fn stop_sleeping(&mut self) {
        self.is_sleeping = false;
    }

    /// `hasCustomName()`.
    pub fn has_custom_name(&self) -> bool {
        self.entity.custom_name.is_some()
    }

    /// `getKillCredit()` уже реализован; `awardKillScore` выше.

    /// `getItemInHand(hand)`.
    pub fn get_item_in_hand(&self, hand: HumanoidArm) -> &ItemStack {
        match hand {
            HumanoidArm::Right => self.get_main_hand_item(),
            _ => self.get_offhand_item(),
        }
    }

    /// `getVehicle()`.
    /// `AbstractSchoolingFish.canBeFollowed()` — вожак стаи.
    /// Default: false (рыба переопределяет).
    pub fn mob_can_be_followed(&self) -> bool {
        false
    }

    /// `AbstractSchoolingFish.isFollower()` — член стаи.
    pub fn mob_is_follower(&self) -> bool {
        false
    }

    /// `Player.hasInfiniteMaterials()` — креатив/спектатор для
    /// RunAroundLikeCrazyGoal tame-check. Default: false.
    pub fn player_has_infinite_materials(&self) -> bool {
        false
    }

    /// `Llama.inCaravan()` — view-хук для FollowCaravanGoal
    /// (LlamaFollowCaravanGoal работает по Entity/LivingEntity-view).
    pub fn mob_in_caravan(&self) -> bool {
        false
    }

    /// `Llama.getCaravanHead()` — view-хук.
    pub fn mob_get_caravan_head(&self) -> Option<Entity> {
        None
    }

    /// `Llama.hasCaravanTail()` — view-хук.
    pub fn mob_llama_has_caravan_tail(&self) -> bool {
        false
    }

    /// `Raider.canBeLeader()`.
    pub fn raider_can_be_leader(&self) -> bool {
        false
    }

    /// `Raider.setPatrolLeader(flag)`.
    pub fn raider_set_patrol_leader(&mut self, _leader: bool) {}

    /// `Raider.isPatrolLeader()`.
    pub fn raider_is_patrol_leader(&self) -> bool {
        false
    }

    /// `Raider.setCurrentRaid(raidId)` — id рейда или -1.
    pub fn raider_set_current_raid_id(&mut self, _raid_id: i32) {}

    /// `Raider.getCurrentRaid()` разрешается через level;
    /// здесь храним id.
    pub fn raider_current_raid_id(&self) -> i32 {
        -1
    }

    /// `Raider.setWave(wave)` / `getWave()`.
    pub fn raider_set_wave(&mut self, _wave: i32) {}
    /// `Raider.getWave()`.
    pub fn raider_wave(&self) -> i32 {
        0
    }

    /// `Raider.setCanJoinRaid(flag)` / `canJoinRaid()`.
    pub fn raider_set_can_join_raid(&mut self, _can: bool) {}
    /// `Raider.canJoinRaid()`.
    pub fn raider_can_join_raid(&self) -> bool {
        false
    }

    /// `Raider.setTicksOutsideRaid` / `getTicksOutsideRaid`.
    pub fn raider_set_ticks_outside_raid(&mut self, _ticks: i32) {}
    /// `Raider.getTicksOutsideRaid()`.
    pub fn raider_ticks_outside_raid(&self) -> i32 {
        0
    }

    /// `Raider.getNoActionTime()` (как Mob, view-вариант).
    pub fn get_no_action_time(&self) -> i32 {
        0
    }

    /// `Entity.hasMovedHorizontallyRecently()` — игрок в лодке
    /// недавно двигался (FollowBoatGoal). Default: false.
    pub fn player_moved_horizontally_recently(&self) -> bool {
        false
    }

    /// `Player.getAbilities().flying` — LandOnOwnersShoulderGoal.
    /// Default: false.
    pub fn entity_abilities_flying(&self) -> bool {
        false
    }

    pub fn get_vehicle(&self) -> Option<Entity> {
        self.entity.get_vehicle()
    }

    /// `isPassengerOfSameVehicle(entity)`.
    pub fn is_passenger_of_same_vehicle(&self, entity: &Entity) -> bool {
        self.entity.is_passenger_of_same_vehicle(entity)
    }

    /// `canUseSlot(slot)`.
    pub fn can_use_slot(&self, _slot: EquipmentSlot) -> bool {
        true
    }

    /// `getAttributeModifiers(attribute)`.
    pub fn get_attribute_modifiers(
        &self,
        _attribute: &Holder<crate::entity::ai::attributes::Attribute>,
    ) -> Vec<crate::entity::ai::attributes::AttributeModifier> {
        Vec::new()
    }

    /// `removeAllEffects()` — удаление всех + `onEffectsRemoved`.
    pub fn remove_all_effects(&mut self) -> Vec<MobEffectInstance> {
        let effects: Vec<MobEffectInstance> =
            self.active_effects.drain().map(|(_, v)| v).collect();
        self.on_effects_removed(&effects);
        effects
    }

    /// `removeEffectNoUpdate(effect)` — удаление без onEffectsRemoved.
    pub fn remove_effect_no_update(
        &mut self,
        effect: &Holder<crate::world::effect::mob_effect::MobEffect>,
    ) -> Option<MobEffectInstance> {
        let key = Self::holder_key(effect);
        self.active_effects.remove(&key)
    }

    /// `removeEffect(effect)` — удаление + `onEffectsRemoved`.
    pub fn remove_effect(&mut self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        if let Some(inst) = self.remove_effect_no_update(effect) {
            self.on_effects_removed(&[inst]);
            return true;
        }
        false
    }

    /// `addAdditionalSaveData(tag)`.
    pub fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("Health", Tag::Float(self.health));
        tag.put("AbsorptionAmount", Tag::Float(self.absorption_amount));
        tag.put("HurtTime", Tag::Int(self.hurt_time));
        tag.put("HurtByTimestamp", Tag::Int(self.last_hurt_by_mob_timestamp));
        if let Some(r) = &self.last_hurt_by_player {
            r.store(tag, "last_hurt_by_player");
            tag.put("last_hurt_by_player_memory_time", Tag::Int(self.last_hurt_by_player_memory_time));
        }
        if let Some(r) = &self.last_hurt_by_mob {
            r.store(tag, "last_hurt_by_mob");
        }
        tag.put("DeathTime", Tag::Int(self.death_time));
        tag.put("NoActionTime", Tag::Int(self.no_action_time));
        tag.put("FallFlying", Tag::Byte(if self.fall_fly_ticks > 0 { 1 } else { 0 }));
        if !self.active_effects.is_empty() {
            let effects: Vec<Tag> = self.active_effects.values().map(|i| i.to_nbt()).collect();
            tag.put("ActiveEffects", Tag::List(effects));
        }
        let mut equipment = CompoundTag::new();
        for (slot, stack) in &self.last_equipment_items {
            if !stack.is_empty() {
                equipment.put(&format!("{:?}", slot).to_lowercase(), stack.save(CompoundTag::new()));
            }
        }
        tag.put("equipment", Tag::Compound(equipment));
    }

    /// `readAdditionalSaveData(tag)`.
    pub fn read_additional_save_data(&mut self, tag: &CompoundTag) {
        if let Some(Tag::Float(h)) = tag.get("Health") { self.health = *h; }
        if let Some(Tag::Float(a)) = tag.get("AbsorptionAmount") { self.absorption_amount = *a; }
        if let Some(Tag::Int(ht)) = tag.get("HurtTime") { self.hurt_time = *ht; }
        if let Some(Tag::Int(dt)) = tag.get("DeathTime") { self.death_time = *dt; }
        if let Some(Tag::Int(ht)) = tag.get("HurtByTimestamp") { self.last_hurt_by_mob_timestamp = *ht; }
        self.last_hurt_by_player = crate::world::entity::entity_reference::EntityReference::read(tag, "last_hurt_by_player");
        if let Some(Tag::Int(mt)) = tag.get("last_hurt_by_player_memory_time") { self.last_hurt_by_player_memory_time = *mt; }
        self.last_hurt_by_mob = crate::world::entity::entity_reference::EntityReference::read(tag, "last_hurt_by_mob");
        if let Some(Tag::Int(na)) = tag.get("NoActionTime") { self.no_action_time = *na; }
        if let Some(Tag::Byte(ff)) = tag.get("FallFlying") { self.fall_fly_ticks = if *ff != 0 { 1 } else { 0 }; }
        if let Some(Tag::List(list)) = tag.get("ActiveEffects") {
            for item in list {
                if let Tag::Compound(c) = item {
                    if let Some(inst) = MobEffectInstance::from_nbt(c) {
                        let _ = self.add_effect(inst);
                    }
                }
            }
        }
    }

    /// `tickEffects()` — серверный цикл `MobEffectInstance.tickServer`
    /// (урон/хил/истечение) + `onEffectsRemoved` для снятых эффектов
    /// + `onEffectUpdated` каждые 600 тиков длительности.
    pub fn tick_effects(&mut self) {
        let mut effects = std::mem::take(&mut self.active_effects);
        let level = self.entity.level.clone();
        let level_ref: Option<&dyn crate::world::level::level_reader::LevelReader> =
            level.as_deref();
        let mut removed: Vec<MobEffectInstance> = Vec::new();
        let mut to_remove: Vec<u64> = Vec::new();
        for (key, inst) in effects.iter_mut() {
            let mut updated = false;
            let alive = inst.tick_server(level_ref, self, &mut || updated = true);
            if !alive {
                to_remove.push(*key);
                removed.push(inst.clone());
            } else if inst.duration() % 600 == 0 {
                self.on_effect_updated(inst, false);
            }
            if updated {
                self.effects_dirty = true;
            }
        }
        for key in to_remove {
            effects.remove(&key);
        }
        self.active_effects = effects;
        if !removed.is_empty() {
            self.on_effects_removed(&removed);
        }
    }

    /// `causeFallDamage(fallDistance, multiplier, source)`.
    pub fn cause_fall_damage(&mut self, fall_distance: f32, multiplier: f32, source: DamageSource) -> bool {
        if fall_distance <= 3.0 {
            return false;
        }
        let damage = (fall_distance - 3.0) * multiplier;
        self.entity.fall_distance = fall_distance;
        self.hurt(source, damage)
    }

    /// `travel`.
    pub fn travel(&mut self, _travel_vector: Vec3) {}

    /// `LivingEntity.moveRelative(friction, movementInput)` —
    /// `getInputVector(input, 1, yRot)` + `deltaMovement.add(input*friction)`.
    /// Входной вектор поворачивается на `-yaw` (как `Vec3.yRot`).
    pub fn move_relative_friction(&mut self, friction: f32, input: Vec3) {
        let yaw = -(self.entity.get_y_rot() as f64) * std::f64::consts::PI / 180.0;
        let (sin, cos) = yaw.sin_cos();
        // `Vec3.yRot(angle)`: x' = x*cos + z*sin; z' = -x*sin + z*cos.
        let ix = input.x * cos + input.z * sin;
        let iz = -input.x * sin + input.z * cos;
        let cur = self.entity.get_delta_movement();
        self.entity.set_delta_movement(
            cur.x + ix * friction as f64,
            cur.y + input.y * friction as f64,
            cur.z + iz * friction as f64,
        );
    }

    /// `getJumpPower()`.
    pub fn get_jump_power(&self) -> f32 {
        0.42
    }

    /// `getStepHeight()`.
    pub fn get_step_height(&self) -> f32 {
        0.6
    }

    /// `getScale()`.
    pub fn get_scale(&self) -> f32 {
        1.0
    }

    /// `getVoicePitch()`.
    pub fn get_voice_pitch(&self) -> f32 {
        1.0
    }

    /// `getVoiceVolume()`.
    pub fn get_voice_volume(&self) -> f32 {
        1.0
    }

    /// `playAmbientSound()`.
    pub fn play_ambient_sound(&mut self) {}

    /// `isSunBurnTick()`.
    pub fn is_sun_burn_tick(&self) -> bool {
        false
    }

    /// `causeFoodExhaustion(amount)` — база no-op
    /// (Player переопределяет через FoodData).
    pub fn cause_food_exhaustion(&mut self, _amount: f32) {}

    /// `eat`-saturation ветка SaturationMobEffect — база no-op
    /// (Player: `getFoodData().eat(foodLevel, 1.0F)`).
    pub fn eat_saturation(&mut self, _food_level: i32) {}

    /// `isAffectedByPotions()`.
    pub fn is_affected_by_potions(&self) -> bool {
        true
    }

    /// `onEffectRemoved`.
    pub fn on_effect_removed(&mut self, _effect: &MobEffectInstance) {}

    /// `removeEffects`.
    pub fn remove_effects(&mut self) -> Vec<MobEffectInstance> {
        self.remove_all_effects()
    }

    /// `isCurrentlyGlowing()`.
    pub fn is_currently_glowing(&self) -> bool {
        false
    }

    /// `setGlowingTag`.
    pub fn set_glowing_tag(&mut self, _glowing: bool, _caused_by_living: bool) {}

    /// `getEntityInteractionRange`.
    pub fn get_entity_interaction_range(&self) -> f64 {
        3.0
    }

    /// `getAttackRangeWith(target)`.
    pub fn get_attack_range_with(&self, _target: &Entity) -> f64 {
        3.0
    }

    /// `canAttackType(type)`.
    pub fn can_attack_type(&self, _entity_type: &Holder<EntityType>) -> bool {
        true
    }

    /// `Villager.getPlayerReputation(player)` — репутация у
    /// игрока (DefendVillageTargetGoal). Default: 0.
    pub fn player_reputation(&self, _player: &LivingEntity) -> i32 {
        0
    }

    /// `isAlliedTo(entity)` — сравнение по Team (null team → false;
    /// та же команда → allied). UUID-самоссылка — тоже allied.
    pub fn is_allied_to(&self, entity: &Entity) -> bool {
        if self.entity.uuid == entity.uuid {
            return true;
        }
        match (self.get_team(), entity.get_team()) {
            (Some(t), Some(o)) => t.name == o.name,
            _ => false,
        }
    }

    /// `getTeam()`.
    pub fn get_team(&self) -> Option<crate::world::scores::team::PlayerTeam> {
        None
    }

    /// `onEnterCombat`/`onLeaveCombat`.
    pub fn on_enter_combat(&mut self) {}
    pub fn on_leave_combat(&mut self) {}

    /// `getTicksSinceLastSwing`.
    pub fn get_ticks_since_last_swing(&self) -> i32 {
        if self.swinging { self.swing_time } else { 0 }
    }

    /// `getMainHandItemStack`.
    pub fn get_main_hand_item_stack(&self) -> &ItemStack {
        self.get_main_hand_item()
    }

    /// `isActuallyUsingItem`.
    pub fn is_actually_using_item(&self) -> bool {
        self.is_using_item() && self.get_use_item().get_use_duration_ticks() > self.use_item_remaining
    }

    /// `releaseUsingItem`.
    pub fn release_using_item(&mut self) {
        self.use_item_remaining = 0;
    }

    /// `completeUsingItem`.
    pub fn complete_using_item(&mut self) {}

    /// `setFallFlying(ticks)`.
    pub fn set_fall_flying(&mut self, ticks: i32) {
        self.fall_fly_ticks = ticks;
    }

    /// `getFallFlyingTicks`.
    pub fn get_fall_flying_ticks(&self) -> i32 {
        self.fall_fly_ticks
    }

    /// `tryToForgeTrim`.
    pub fn try_to_forge_trim(&mut self, _slot: EquipmentSlot, _upgrade: &ItemStack) -> bool {
        false
    }

    /// `getItemSlot`.
    pub fn get_item_slot(&self, slot: EquipmentSlot) -> &ItemStack {
        self.get_item_by_slot(slot)
    }

    /// `setItemSlotAndDropWhenKilled`.
    pub fn set_item_slot_and_drop_when_killed(&mut self, slot: EquipmentSlot, stack: ItemStack) {
        self.set_item_slot(slot, stack);
    }

    /// `getRandom()` — `Entity.random`.
    pub fn entity_random(&self) -> std::sync::Arc<std::sync::Mutex<Box<dyn crate::random::RandomSource>>> {
        self.entity.random.clone()
    }

    /// `detectEquipmentUpdates`.
    pub fn detect_equipment_updates(&mut self) {
        self.effects_dirty = true;
    }

    /// `getActiveItem()` — `isUsingItem ? useItem : mainHandItem`.
    pub fn get_active_item(&self) -> ItemStack {
        if self.is_using_item() {
            self.get_use_item().clone()
        } else {
            self.get_main_hand_item().clone()
        }
    }

    /// `getWeaponItem()` — основной предмет атаки.
    pub fn get_weapon_item(&self) -> ItemStack {
        self.get_main_hand_item().clone()
    }

    /// `entityAttackRange()` — `ATTACK_RANGE` активного предмета
    /// или `AttackRange.defaultFor(this)`.
    pub fn entity_attack_range(&self) -> crate::world::item::component::attack_range::AttackRange {
        use crate::resources::identifier::Identifier;
        self.get_active_item()
            .components
            .get_by_id::<crate::world::item::component::attack_range::AttackRange>(
                &Identifier::minecraft("attack_range"),
            )
            .copied()
            .unwrap_or_else(|| {
                crate::world::item::component::attack_range::AttackRange::default_for(self)
            })
    }

    /// `hasLineOfSight(entity)` — `hasLineOfSight(entity,
    /// COLLIDER, NONE, entity.getEyeY())`.
    pub fn has_line_of_sight(&self, entity: &Entity) -> bool {
        self.has_line_of_sight_at(entity, entity.get_eye_y())
    }

    /// `hasLineOfSight(entity, block, fluid, y)` — та же лига,
    /// расстояние ≤ 128², `level.clip(...) == MISS`.
    pub fn has_line_of_sight_at(&self, entity: &Entity, y: f64) -> bool {
        let (Some(level), Some(other_level)) = (&self.entity.level, &entity.level)
        else {
            return false;
        };
        if !std::sync::Arc::ptr_eq(level, other_level)
            && level.dimension_id() != other_level.dimension_id()
        {
            return false;
        }
        let eye = Vec3::new(self.entity.pos.x, self.get_eye_y(), self.entity.pos.z);
        let target = Vec3::new(entity.pos.x, y, entity.pos.z);
        if target.distance_to_sqr(&eye) > 128.0 * 128.0 {
            return false;
        }
        let hit = crate::world::level::clip::clip(
            level.as_ref(),
            eye,
            target,
            crate::world::level::clip::ClipBlock::Collider,
            crate::world::level::clip::ClipFluid::None,
        );
        hit.hit_type() == crate::world::phys::hit_result::HitResultType::MISS
    }

    /// `isWithinMeleeAttackRange`.
    pub fn is_within_melee_attack_range(&self, _target: &Entity) -> bool {
        false
    }

    /// `isWithinItemUseRange`.
    pub fn is_within_item_use_range(&self, _target: &Entity) -> bool {
        false
    }

    /// `getTicksSinceLastKineticHit`.
    pub fn get_ticks_since_last_kinetic_hit(&self) -> i64 {
        0
    }

    /// `lerpHeadTo`.
    pub fn lerp_head_to(&mut self, _y_rot: f32, _x_rot: f32, _steps: i32) {}

    /// `lerpBodyRotation`.
    pub fn lerp_body_rotation(&mut self, _rot: f32, _steps: i32) {}

    /// `handleEntityEvent`.
    pub fn handle_entity_event(&mut self, _id: u8, _event_parameter: i32) {}

    /// `getXpNeededForNextLevel`.
    pub fn get_xp_needed_for_next_level(&self, _level: i32) -> i32 {
        7
    }

    /// `getEnchantedDamage`.
    pub fn get_enchanted_damage(&self, _source: &DamageSource, base: f32) -> f32 {
        base
    }

    /// `getKnockback(target, damageSource)` — в Java параметр
    /// называется `attacker`, но это цель (получающая knockback).
    /// `(ATTACK_KNOCKBACK + enchant) / 2`.
    pub fn get_knockback(&mut self, target: &LivingEntity, damage_source: &DamageSource) -> f32 {
        let f = self
            .get_attribute_value_by_id(&crate::entity::ai::attributes::attributes::attack_knockback())
            as f32;
        if self.entity.level.is_some() {
            let weapon = self.get_weapon_item();
            crate::world::item::enchantment::enchantment_helper::modify_knockback(
                &weapon,
                target,
                damage_source,
                f,
            ) / 2.0
        } else {
            f / 2.0
        }
    }

    /// `causeExtraKnockback(target, strength, currentMovement)` —
    /// knockback цели + замедление себя (×0.6 по XZ).
    pub fn cause_extra_knockback(
        &mut self,
        target: &mut LivingEntity,
        strength: f32,
        _current_movement: Vec3,
    ) {
        if strength > 0.0 {
            let y_rot = self.entity.get_y_rot() as f64 * std::f64::consts::PI / 180.0;
            target.knockback(
                strength as f64,
                crate::util::mth::sin(y_rot) as f64,
                -crate::util::mth::cos(y_rot) as f64,
            );
            let d = self.entity.get_delta_movement();
            let m = d.multiply(0.6, 1.0, 0.6);
            self.entity.set_delta_movement(m.x, m.y, m.z);
        }
    }

    /// `lungeForwardMaybe()` — `EnchantmentHelper.doLungeEffects`.
    pub fn lunge_forward_maybe(&mut self) {
        if self.entity.level.is_some() {
            crate::world::item::enchantment::enchantment_helper::do_lunge_effects(self);
        }
    }

    /// `playAttackSound()` — базовая реализация пуста
    /// (звуки атаки у конкретных мобов).
    pub fn play_attack_sound(&mut self) {}

    /// `doHurtTarget` — базовая (LivingEntity): только
    /// `setLastHurtMob(target)` и `false`.
    pub fn do_hurt_target(&mut self, target: &mut LivingEntity, _source: &DamageSource) -> bool {
        self.set_last_hurt_mob(Some(target.entity.clone()));
        false
    }

    /// `getAttackKnockback`.
    pub fn get_attack_knockback(&self, _target: &Entity, _damage: f32) -> f32 {
        0.0
    }

    /// `getDamageAfterFallFall`.
    pub fn get_damage_after_fall_fall(&self, _distance: f32) -> f32 {
        0.0
    }

    /// `getWaterSlowDown`.
    pub fn get_water_slow_down(&self) -> f32 {
        0.8
    }

    /// `getLavaSlowDown`.
    pub fn get_lava_slow_down(&self) -> f32 {
        0.5
    }

    /// `getBlockJumpFactor`.
    pub fn get_block_jump_factor(&self) -> f32 {
        1.0
    }

    /// `maxUpStep`.
    pub fn max_up_step(&self) -> f32 {
        0.6
    }

    /// `getGravity`.
    pub fn get_gravity(&self) -> f64 {
        0.08
    }

    /// `getFluidFallingAdjustedMovement`.
    pub fn get_fluid_falling_adjusted_movement(
        &self,
        gravity: f64,
        _is_falling: bool,
        _fluid_state: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> f64 {
        gravity
    }

    /// `getFrictionInfluencedSpeed`.
    pub fn get_friction_influenced_speed(&self, speed: f32) -> f32 {
        speed
    }

    /// `getRelativePortalPosition`.
    pub fn get_relative_portal_position(
        &self,
        _direction: crate::core::direction::Direction,
        _area: f64,
    ) -> Vec3 {
        Vec3::ZERO
    }

    /// `setRelativePortalPosition`.
    pub fn set_relative_portal_position(&mut self, _portal_entry: Vec3) {}

    /// `canDisableShield`.
    pub fn can_disable_shield(&self, _shield_user: &Entity) -> bool {
        false
    }

    /// `isDamageSourceBlocked` + shield check.
    pub fn is_damage_source_blocked_shield(&self, _source: &DamageSource) -> bool {
        false
    }

    /// `getProjectileDeflection`.
    pub fn get_projectile_deflection(&self, _projectile: &Entity) -> Option<Vec3> {
        None
    }

    /// `onProjectileHit`.
    pub fn on_projectile_hit(&mut self, _projectile: &Entity) {}

    /// `playBlockFallSound`.
    pub fn play_block_fall_sound(&mut self) {}

    /// `getHurtDir`.
    pub fn get_hurt_dir(&self) -> f32 {
        0.0
    }

    /// `setHurtDir`.
    pub fn set_hurt_dir(&mut self, _dir: f32) {}

    /// `getLastHurtByPlayer` — EntityReference-резолв → LivingEntity-view.
    pub fn get_last_hurt_by_player(&self) -> Option<LivingEntity> {
        crate::world::entity::entity_reference::EntityReference::get_living_entity(
            self.last_hurt_by_player.as_ref(),
            self.entity.level.as_ref(),
        )
    }

    /// `setLastHurtByPlayer(player, memoryTime)`.
    pub fn set_last_hurt_by_player(&mut self, player: Option<Entity>, memory_time: i32) {
        self.last_hurt_by_player = player.as_ref().map(EntityReference::of);
        self.last_hurt_by_player_memory_time = memory_time;
    }

    /// `getLastHurtByMob` — EntityReference-резолв → LivingEntity.
    pub fn get_last_hurt_by_mob(&self) -> Option<LivingEntity> {
        crate::world::entity::entity_reference::EntityReference::get_living_entity(
            self.last_hurt_by_mob.as_ref(),
            self.entity.level.as_ref(),
        )
    }

    /// `setLastHurtByMob(mob)`.
    pub fn set_last_hurt_by_mob(&mut self, mob: Option<Entity>) {
        self.last_hurt_by_mob = mob.as_ref().map(EntityReference::of);
        if mob.is_some() {
            self.last_hurt_by_mob_timestamp = self.entity.tick_count;
        }
    }

    /// `getLastHurtByMobTimestamp`.
    pub fn get_last_hurt_by_mob_timestamp(&self) -> i32 {
        self.last_hurt_by_mob_timestamp
    }

    /// `setLastHurtByMobTimestamp`.
    pub fn set_last_hurt_by_mob_timestamp(&mut self, timestamp: i32) {
        self.last_hurt_by_mob_timestamp = timestamp;
    }

    /// `getLastHurtMob`.
    pub fn get_last_hurt_mob(&self) -> Option<Entity> {
        self.last_hurt_mob.clone()
    }

    /// `setLastHurtMob`.
    pub fn set_last_hurt_mob(&mut self, mob: Option<Entity>) {
        self.last_hurt_mob = mob;
    }

    /// `getLastHurtMobTimestamp`.
    pub fn get_last_hurt_mob_timestamp(&self) -> i32 {
        self.last_hurt_by_mob_timestamp
    }

    /// `setLastHurtMobTimestamp`.
    pub fn set_last_hurt_mob_timestamp(&mut self, timestamp: i32) {
        self.last_hurt_by_mob_timestamp = timestamp;
    }

    /// `isBlocking` variant.
    pub fn is_blocking_item(&self) -> bool {
        false
    }

    /// `isDamageBlock` variant.
    pub fn is_damage_blocked(&self, _source: &DamageSource) -> bool {
        false
    }

    /// `getUseItemHand`.
    pub fn get_use_item_hand(&self) -> HumanoidArm {
        self.swinging_arm
    }

    /// `getOffhandItem` variant.
    pub fn offhand_item(&self) -> &ItemStack {
        self.get_offhand_item()
    }

    /// `getMainHandItem` variant.
    pub fn main_hand_item(&self) -> &ItemStack {
        self.get_main_hand_item()
    }

    /// `getUseItemRemainingTicks`.
    pub fn get_use_item_remaining_ticks(&self) -> i32 {
        self.use_item_remaining
    }

    /// `getTicksUsingItem`.
    pub fn get_ticks_using_item(&self) -> i32 {
        if self.is_using_item() {
            self.get_use_item().get_use_duration_ticks() - self.use_item_remaining
        } else {
            0
        }
    }

    /// `isVisuallySwimming`.
    pub fn is_visually_swimming(&self) -> bool {
        false
    }

    /// `setSwimming`.
    pub fn set_swimming(&mut self, _swimming: bool) {}

    /// `isSwimming`.
    pub fn is_swimming(&self) -> bool {
        false
    }

    /// `isInWall`.
    pub fn is_in_wall(&self) -> bool {
        false
    }

    /// `isInWater`.
    pub fn is_in_water(&self) -> bool {
        self.entity.is_in_water()
    }

    /// `isInWaterOrBubble`.
    pub fn is_in_water_or_bubble(&self) -> bool {
        self.is_in_water()
    }

    /// `isUnderWater`.
    pub fn is_under_water(&self) -> bool {
        self.entity.is_under_water()
    }

    /// `canStandOnFluid` — базовая реализация `false`
    /// (Strider переопределяет на lava).
    pub fn can_stand_on_fluid(
        &self,
        _fluid: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> bool {
        false
    }

    /// `isInLava`.
    pub fn is_in_lava(&self) -> bool {
        self.entity.is_in_lava()
    }

    /// `isOnFire`.
    pub fn is_on_fire(&self) -> bool {
        self.entity.is_on_fire()
    }

    /// `setRemainingFireTicks`.
    pub fn set_remaining_fire_ticks(&mut self, ticks: i32) {
        self.entity.set_remaining_fire_ticks(ticks);
    }

    /// `getRemainingFireTicks`.
    pub fn get_remaining_fire_ticks(&self) -> i32 {
        self.entity.get_remaining_fire_ticks()
    }

    /// `isFreezing` — `ticksFrozen >= getTicksRequiredToFreeze`.
    pub fn is_freezing(&self) -> bool {
        self.entity.ticks_frozen >= self.get_ticks_required_to_freeze()
    }

    /// `getTicksFrozen`.
    pub fn get_ticks_frozen(&self) -> i32 {
        self.entity.ticks_frozen
    }

    /// `setTicksFrozen`.
    pub fn set_ticks_frozen(&mut self, ticks: i32) {
        self.entity.ticks_frozen = ticks;
    }

    /// `isFullyFrozen` — `percentFrozen >= 0.99F`.
    pub fn is_fully_frozen(&self) -> bool {
        self.entity.percent_frozen >= 0.99
    }

    /// `getPercentFrozen`.
    pub fn get_percent_frozen(&self) -> f32 {
        self.entity.percent_frozen
    }

    /// `setPercentFrozen`.
    pub fn set_percent_frozen(&mut self, percent: f32) {
        self.entity.percent_frozen = percent.clamp(0.0, 1.0);
    }

    /// `getTicksRequiredToFreeze` — 140 тиков (vanilla default).
    pub fn get_ticks_required_to_freeze(&self) -> i32 {
        140
    }

    /// `onClimbable`.
    pub fn on_climbable(&self) -> bool {
        false
    }

    /// `isIgnoringBlockTriggers`.
    pub fn is_ignoring_block_triggers(&self) -> bool {
        false
    }

    /// `setIgnoreBlockTriggers`.
    pub fn set_ignore_block_triggers(&mut self, _ignore: bool) {}

    /// `getPersistentAngerTarget`.
    pub fn get_persistent_anger_target(&self) -> Option<Entity> {
        None
    }

    /// `getTypeSpecificMaxFireTicks`.
    pub fn get_type_specific_max_fire_ticks(&self) -> i32 {
        100
    }

    /// `getMaxFallDistance`.
    pub fn get_max_fall_distance(&self) -> f32 {
        3.0
    }

    /// `fallDamage` variant.
    pub fn fall_damage(&mut self, fall_distance: f32, source: DamageSource) -> bool {
        self.cause_fall_damage(fall_distance, 1.0, source)
    }

    /// `moveRelative`.
    pub fn move_relative(&mut self, _strafe: f32, _up: f32, _forward: f32) {}

    /// `getAttackAnim`.
    pub fn get_attack_anim(&self, _partial_tick: f32) -> f32 {
        self.attack_anim
    }

    /// `getAttackSpeed()` — скорость атаки из атрибута `attack_speed`.
    pub fn get_attack_speed(&mut self) -> f32 {
        self.get_attribute_value_by_id(&attribute_ids::attack_speed()) as f32
    }

    /// `getMaxAttackStrength()` — максимум шкалы атаки (= attack speed).
    pub fn get_max_attack_strength(&mut self) -> f32 {
        self.get_attack_speed()
    }

    /// `getAttackStrengthScale(partialTick)` — прогресс перезарядки
    /// атаки в [0, 1].
    pub fn get_attack_strength_scale(&mut self, partial_tick: f32) -> f32 {
        let speed = self.get_max_attack_strength();
        ((self.attack_strength_ticker as f32 + partial_tick) / speed).clamp(0.0, 1.0)
    }

    /// `canPerformAttack()` — готова ли сущность к следующей атаке.
    pub fn can_perform_attack(&mut self) -> bool {
        self.get_attack_strength_scale(0.5) > 0.9
    }

    /// `getSwingProgress`.
    pub fn get_swing_progress(&self, partial_tick: f32) -> f32 {
        if self.swinging {
            (self.swing_time as f32 + partial_tick) / 6.0
        } else {
            0.0
        }
    }

    /// `isVisuallyCrawling`.
    pub fn is_visually_crawling(&self) -> bool {
        false
    }

    /// `setRecordPlaying`.
    pub fn set_record_playing(&mut self, _record: bool) {}

    /// `isPickable`.
    pub fn is_pickable(&self) -> bool {
        !self.dead
    }

    /// `canCollideWith`.
    pub fn can_collide_with(&self, _entity: &Entity) -> bool {
        false
    }

    /// `isAttackable`.
    pub fn is_attackable(&self) -> bool {
        true
    }

    /// `getAirSupply`.
    pub fn get_air_supply(&self) -> i32 {
        300
    }

    /// `setAirSupply`.
    pub fn set_air_supply(&mut self, _supply: i32) {}

    /// `decreaseAirSupply`.
    pub fn decrease_air_supply(&mut self, _amount: i32) {}

    /// `getMaxAirSupply`.
    pub fn get_max_air_supply(&self) -> i32 {
        300
    }

    /// `getType`.
    pub fn get_type(&self) -> Holder<EntityType> {
        self.entity.get_type().clone()
    }

    /// `getEncodeId`.
    pub fn get_encode_id(&self) -> i32 {
        self.entity.get_id()
    }

    /// `getUUID`.
    pub fn get_uuid(&self) -> crate::util::uuid::Uuid {
        self.entity.get_uuid()
    }

    /// `getName`.
    pub fn get_name(&self) -> Component {
        self.entity.get_name().cloned().unwrap_or_else(|| self.entity.get_display_name())
    }

    /// `getDisplayName`.
    pub fn get_display_name(&self) -> Component {
        self.entity.get_display_name()
    }

    /// `getStringUUID`.
    pub fn get_string_uuid(&self) -> String {
        self.entity.get_uuid().to_string()
    }

    /// `distanceTo`.
    pub fn distance_to(&self, entity: &Entity) -> f64 {
        self.entity.distance_to_sqr(entity).sqrt()
    }

    /// `distanceToSqr`.
    pub fn distance_to_sqr(&self, entity: &Entity) -> f64 {
        self.entity.distance_to_sqr(entity)
    }

    /// `getEyeY`.
    pub fn get_eye_y(&self) -> f64 {
        self.entity.pos.y + self.entity.eye_height as f64
    }

    /// `getEyePosition`.
    pub fn get_eye_position(&self) -> Vec3 {
        self.entity.get_eye_position()
    }

    /// `getBoundingBox`.
    pub fn get_bounding_box(&self) -> crate::world::phys::aabb::Aabb {
        self.entity.get_bounding_box()
    }

    /// `getBbWidth`.
    pub fn get_bb_width(&self) -> f32 {
        (self.entity.bounding_box.max_x - self.entity.bounding_box.min_x) as f32
    }

    /// `getBbHeight`.
    pub fn get_bb_height(&self) -> f32 {
        (self.entity.bounding_box.max_y - self.entity.bounding_box.min_y) as f32
    }

    /// `getBlockY`.
    pub fn get_block_y(&self) -> i32 {
        BlockPos::containing_vec3(&self.entity.pos).get_y()
    }

    /// `getBlockX`.
    pub fn get_block_x(&self) -> i32 {
        BlockPos::containing_vec3(&self.entity.pos).get_x()
    }

    /// `getBlockZ`.
    pub fn get_block_z(&self) -> i32 {
        BlockPos::containing_vec3(&self.entity.pos).get_z()
    }

    /// `blockPosition`.
    pub fn block_position(&self) -> BlockPos {
        BlockPos::containing_vec3(&self.entity.pos)
    }

    /// `position`.
    pub fn position(&self) -> Vec3 {
        self.entity.get_pos()
    }

    /// `getX`, `getY`, `getZ`.
    pub fn get_x(&self) -> f64 {
        self.entity.get_pos().x
    }
    pub fn get_y(&self) -> f64 {
        self.entity.get_pos().y
    }
    pub fn get_z(&self) -> f64 {
        self.entity.get_pos().z
    }

    /// `setPos`.
    pub fn set_pos(&mut self, pos: Vec3) {
        self.entity.set_pos(pos.x, pos.y, pos.z);
    }

    /// `setPosRaw`.
    pub fn set_pos_raw(&mut self, x: f64, y: f64, z: f64) {
        self.entity.set_pos(x, y, z);
    }

    /// `getDeltaMovement`.
    pub fn get_delta_movement(&self) -> Vec3 {
        self.entity.get_delta_movement()
    }

    /// `setDeltaMovement`.
    pub fn set_delta_movement(&mut self, delta: Vec3) {
        self.entity.set_delta_movement(delta.x, delta.y, delta.z);
    }

    /// `getSpeed`.
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// `setSpeed`.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// `setZza`/`setYya`/`setXxa` — входные значения движения
    /// (поля `zza`/`yya`/`xxa`).
    pub fn set_zza(&mut self, amount: f32) {
        self.zza = amount;
    }
    pub fn set_yya(&mut self, amount: f32) {
        self.yya = amount;
    }
    pub fn set_xxa(&mut self, amount: f32) {
        self.xxa = amount;
    }
    pub fn get_zza(&self) -> f32 {
        self.zza
    }
    pub fn get_yya(&self) -> f32 {
        self.yya
    }
    pub fn get_xxa(&self) -> f32 {
        self.xxa
    }

    /// `isAffectedByFluids` — базовая реализация `true`
    /// (переопределяется у летунов/призраков).
    pub fn is_affected_by_fluids(&self) -> bool {
        true
    }

    /// `getY(double)` — Y + fraction*height.
    pub fn get_y_fraction(&self, fraction: f64) -> f64 {
        self.entity.get_y() + self.entity.dimensions.height as f64 * fraction
    }

    /// `setYRot`, `setXRot`.
    pub fn set_y_rot(&mut self, rot: f32) {
        self.entity.set_y_rot(rot);
    }
    pub fn set_x_rot(&mut self, rot: f32) {
        self.entity.set_x_rot(rot);
    }

    /// `isAlive` variant.
    pub fn alive(&self) -> bool {
        self.is_alive()
    }

    /// `isDeadOrDying` variant.
    pub fn dead_or_dying(&self) -> bool {
        self.is_dead_or_dying()
    }

    /// `getHealth` variant.
    pub fn health(&self) -> f32 {
        self.get_health()
    }

    /// `getMaxHealth` variant.
    pub fn max_health(&mut self) -> f32 {
        self.get_max_health()
    }

    /// `getAbsorptionAmount` variant.
    pub fn absorption_amount(&self) -> f32 {
        self.get_absorption_amount()
    }

    /// `setAbsorptionAmount` variant.
    pub fn set_absorption_amount_living(&mut self, amount: f32) {
        self.set_absorption_amount(amount);
    }

    /// `heal` variant.
    pub fn heal_amount(&mut self, amount: f32) {
        self.heal(amount);
    }

    /// `hurt` variant.
    pub fn hurt_entity(&mut self, source: DamageSource, amount: f32) -> bool {
        self.hurt(source, amount)
    }

    /// `die` variant.
    pub fn die_entity(&mut self, source: DamageSource) {
        self.die(source.clone());
    }

    /// `onKilled` variant.
    pub fn on_killed_entity(entity: &mut Entity, source: DamageSource) {
        Self::on_killed(entity, source);
    }

    /// `isInvulnerableTo` variant.
    pub fn invulnerable_to(&self, source: &DamageSource) -> bool {
        self.is_invulnerable_to(source)
    }

    /// `isPushable` variant.
    pub fn is_pushable_living(&self) -> bool {
        self.is_pushable()
    }

    /// `isSpectator` variant.
    pub fn spectator(&self) -> bool {
        self.is_spectator()
    }

    /// `canBeSeenAsEnemy` variant.
    pub fn seen_as_enemy(&self) -> bool {
        self.can_be_seen_as_enemy()
    }

    /// `isFallFlying` variant.
    pub fn fall_flying_check(&self) -> bool {
        self.is_fall_flying()
    }

    /// `isUsingItem` variant.
    pub fn using_item_check(&self) -> bool {
        self.is_using_item()
    }

    /// `shouldDropExperience` variant.
    pub fn drop_experience(&self) -> bool {
        self.should_drop_experience()
    }

    /// `getExperienceReward` variant.
    pub fn experience_reward(&self) -> i32 {
        self.get_experience_reward()
    }

    /// `getMainArm` variant.
    pub fn main_arm_check(&self) -> HumanoidArm {
        self.get_main_arm()
    }

    /// `addEffect` variant.
    pub fn add_effect_instance(&mut self, instance: MobEffectInstance) {
        let _ = self.add_effect(instance);
    }

    /// `hasEffect` variant.
    pub fn has_effect_check(&self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        self.has_effect(effect)
    }

    /// `getActiveEffects` variant.
    pub fn active_effects_iter(&self) -> impl Iterator<Item = &MobEffectInstance> {
        self.get_active_effects()
    }

    /// `getDeathSound` variant.
    pub fn death_sound_check(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_death_sound()
    }

    /// `getHurtSound` variant.
    pub fn hurt_sound(&self, source: &DamageSource) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_hurt_sound(source)
    }

    /// `getAmbientSound` variant.
    pub fn ambient_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_ambient_sound()
    }

    /// `getEatingSound` variant.
    pub fn eating_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_eating_sound()
    }

    /// `getFallDamageSound` variant.
    pub fn fall_damage_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_fall_damage_sound()
    }

    /// `getSwimSound` variant.
    pub fn swim_sound(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_swim_sound()
    }

    /// `isPushable` final.
    pub fn pushable_final(&self) -> bool {
        self.is_pushable()
    }

    /// `isSpectator` final.
    pub fn spectator_final(&self) -> bool {
        self.is_spectator()
    }

    /// `canBeSeenAsEnemy` final.
    pub fn seen_as_enemy_final(&self) -> bool {
        self.can_be_seen_as_enemy()
    }

    /// `isFallFlying` final.
    pub fn fall_flying_final(&self) -> bool {
        self.is_fall_flying()
    }

    /// `isUsingItem` final.
    pub fn using_item_final(&self) -> bool {
        self.is_using_item()
    }

    /// `shouldDropExperience` final.
    pub fn drop_experience_final(&self) -> bool {
        self.should_drop_experience()
    }

    /// `getExperienceReward` final.
    pub fn experience_reward_final(&self) -> i32 {
        self.get_experience_reward()
    }

    /// `getMainArm` final.
    pub fn main_arm_final(&self) -> HumanoidArm {
        self.get_main_arm()
    }

    /// `addEffect` final.
    pub fn add_effect_final(&mut self, instance: MobEffectInstance) {
        let _ = self.add_effect(instance);
    }

    /// `hasEffect` final.
    pub fn has_effect_final(&self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        self.has_effect(effect)
    }

    /// `getActiveEffects` final.
    pub fn active_effects_final(&self) -> impl Iterator<Item = &MobEffectInstance> {
        self.get_active_effects()
    }

    /// `getDeathSound` final.
    pub fn death_sound_final(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_death_sound()
    }

    /// `getHurtSound` final.
    pub fn hurt_sound_final(&self, source: &DamageSource) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_hurt_sound(source)
    }

    /// `getAmbientSound` final.
    pub fn ambient_sound_final(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_ambient_sound()
    }

    /// `getEatingSound` final.
    pub fn eating_sound_final(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_eating_sound()
    }

    /// `getFallDamageSound` final.
    pub fn fall_damage_sound_final(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_fall_damage_sound()
    }

    /// `getSwimSound` final.
    pub fn swim_sound_final(&self) -> Option<crate::sounds::sound_event::SoundEvent> {
        self.get_swim_sound()
    }

    /// `isPushable` absolute final.
    pub fn pushable_absolute_final(&self) -> bool {
        self.is_pushable()
    }
}

impl Clone for LivingEntity {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity.clone(),
            attributes: self.attributes.clone(),
            combat_tracker: self.combat_tracker.clone(),
            active_effects: self.active_effects.clone(),
            last_equipment_items: self.last_equipment_items.clone(),
            swinging: self.swinging,
            swinging_arm: self.swinging_arm,
            swing_time: self.swing_time,
            remove_arrow_time: self.remove_arrow_time,
            remove_stinger_time: self.remove_stinger_time,
            hurt_time: self.hurt_time,
            hurt_duration: self.hurt_duration,
            invulnerable_time: self.invulnerable_time,
            death_time: self.death_time,
            o_attack_anim: self.o_attack_anim,
            attack_anim: self.attack_anim,
            attack_strength_ticker: self.attack_strength_ticker,
            item_swap_ticker: self.item_swap_ticker,
            y_body_rot: self.y_body_rot,
            y_body_rot_o: self.y_body_rot_o,
            y_head_rot: self.y_head_rot,
            y_head_rot_o: self.y_head_rot_o,
            last_hurt_by_player_memory_time: self.last_hurt_by_player_memory_time,
            dead: self.dead,
            hurt_marked: self.hurt_marked,
            no_action_time: self.no_action_time,
            last_hurt: self.last_hurt,
            jumping: self.jumping,
            xxa: self.xxa,
            yya: self.yya,
            zza: self.zza,
            effects_dirty: self.effects_dirty,
            last_hurt_by_mob_timestamp: self.last_hurt_by_mob_timestamp,
            last_hurt_by_mob: self.last_hurt_by_mob.clone(),
            last_hurt_by_player: self.last_hurt_by_player.clone(),
            last_hurt_mob: self.last_hurt_mob.clone(),
            speed: self.speed,
            no_jump_delay: self.no_jump_delay,
            absorption_amount: self.absorption_amount,
            use_item: self.use_item.clone(),
            use_item_remaining: self.use_item_remaining,
            fall_fly_ticks: self.fall_fly_ticks,
            last_pos: self.last_pos,
            brain: self.brain.clone(),
            last_damage_source: self.last_damage_source.clone(),
            last_damage_stamp: self.last_damage_stamp,
            auto_spin_attack_ticks: self.auto_spin_attack_ticks,
            auto_spin_attack_dmg: self.auto_spin_attack_dmg,
            xp_reward: self.xp_reward,
            was_experience_consumed: self.was_experience_consumed,
            pose: self.pose,
            is_sleeping: self.is_sleeping,
            health: self.health,
        }
    }
}

/// `LivingEntity.DAMAGE_SOURCE_TIMEOUT`.
const DAMAGE_SOURCE_TIMEOUT: u32 = 40;

fn rand_f64() -> f64 {
    rand::random::<f64>()
}

fn rand_f32() -> f32 {
    rand::random::<f32>()
}

impl LivingEntity {
    /// `LivingEntity`-обёртка над существующим `Entity` —
    /// "view" для целей/предикатов, когда уровень хранит
    /// сущности как `Entity` (target-mob из `EntityReference`,
    /// `nearest_entity` и т.п.). Сохраняет позицию/флаги/
    /// entity-type; combat-поля — дефолтные.
    pub fn from_entity(entity: Entity) -> Self {
        let mut le = Self::new(entity.entity_type.clone(), entity.level.clone());
        le.entity = entity;
        le.entity.is_living = true;
        le
    }

    /// `canAttack(target)` — peaceful не даёт атаковать игроков;
    /// цель должна быть видна как враг.
    pub fn can_attack(&self, target: &LivingEntity) -> bool {
        let is_player = crate::world::entity::entity_type::holder_type_id(
            &target.entity.entity_type,
        )
        .map(|id| id == crate::resources::identifier::Identifier::minecraft("player"))
        .unwrap_or(false);
        let peaceful = self
            .entity
            .level
            .as_ref()
            .map(|l| l.difficulty() == crate::world::difficulty::Difficulty::Peaceful)
            .unwrap_or(false);
        (!is_player || !peaceful) && target.can_be_seen_as_enemy()
    }

    /// `canBeSeenByAnyone()` — не spectator и жив.
    pub fn can_be_seen_by_anyone(&self) -> bool {
        !self.is_spectator() && self.is_alive()
    }

    /// `getVisibilityPercent(observer)` — доля видимости цели
    /// для наблюдателя (0..1); INVISIBILITY-эффект → blend factor.
    pub fn get_visibility_percent(&self, _observer: &Entity) -> f64 {
        let key = Self::holder_key(&crate::world::effect::mob_effects::invisibility());
        match self.active_effects.get(&key) {
            Some(inst) => inst.get_blend_factor(self, 0.0) as f64,
            None => 1.0,
        }
    }

    /// `Villager.wantsToSpawnGolem(gameTime)` — villager просит
    /// голема (gossip + panic timer); default false, переопределяется
    /// в `Villager`. Используется `GolemRandomStrollInVillageGoal`.
    pub fn villager_wants_to_spawn_golem(&self, _game_time: i64) -> bool {
        false
    }

    /// `Brain`-хелперы: getMemory/setMemory/eraseMemory/
    /// hasMemoryValue/checkMemory/isMemoryValue через
    /// `self.brain` (None → Java-эквивалент "нет brain").
    pub fn brain_get<T: Clone + 'static>(
        &self,
        ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
    ) -> Option<T> {
        self.brain.as_ref()?.get_memory(ty)
    }

    /// `brain.setMemory(type, value)`.
    pub fn brain_set<T: 'static>(
        &self,
        ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
        value: Option<T>,
    ) {
        if let Some(b) = &self.brain {
            b.set_memory(ty, value);
        }
    }

    /// `brain.setMemoryWithExpiry`.
    pub fn brain_set_with_expiry<T: 'static>(
        &self,
        ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
        value: T,
        ttl: i64,
    ) {
        if let Some(b) = &self.brain {
            b.set_memory_with_expiry(ty, value, ttl);
        }
    }

    /// `brain.eraseMemory(type)`.
    pub fn brain_erase(&self, ty: &crate::world::entity::ai::memory::memory::MemoryModuleType) {
        if let Some(b) = &self.brain {
            b.erase_memory(ty);
        }
    }

    /// `brain.hasMemoryValue(type)`.
    pub fn brain_has(&self, ty: &crate::world::entity::ai::memory::memory::MemoryModuleType) -> bool {
        self.brain.as_ref().map(|b| b.has_memory_value(ty)).unwrap_or(false)
    }

    /// `brain.checkMemory(type, status)`.
    pub fn brain_check(
        &self,
        ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
        status: crate::world::entity::ai::memory::memory::MemoryStatus,
    ) -> bool {
        self.brain.as_ref().map(|b| b.check_memory(ty, status)).unwrap_or(false)
    }

    /// `brain.setMemory(type, value)` — не-Option перегрузка.
    pub fn brain_set_value<T: 'static>(
        &self,
        ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
        value: T,
    ) {
        self.brain_set(ty, Some(value));
    }

    /// `entity.getType()` — EntityType за Holder (None если
    /// holder пуст — эквивалент отсутствия типа).
    pub fn entity_type_value(&self) -> Option<&crate::world::entity::entity_type::EntityType> {
        self.entity.entity_type.value()
    }

    /// `Animal.isFood(stack)` — food-тест для TEMPT-целей.
    pub fn animal_is_food(&self, _stack: &crate::world::item::item_stack::ItemStack) -> bool {
        false
    }

    /// `Warden.canTargetEntity(entity)` — фильтр WardenEntitySensor.
    pub fn warden_can_target_entity(&self, _entity: &LivingEntity) -> bool {
        true
    }

    /// `canUseNonMeleeWeapon(stack)` — может ли использовать
    /// не-melee оружие (лук/арбалет). Default: false.
    pub fn can_use_non_melee_weapon(&self) -> bool {
        false
    }

    /// `ProjectileWeaponItem.getDefaultProjectileRange()` —
    /// дальность не-melee оружия в руках. Default: 0.
    pub fn non_melee_weapon_range(&self) -> f64 {
        0.0
    }

    /// `getLastDamageSource()`.
    pub fn get_last_damage_source(&self) -> Option<DamageSource> {
        self.last_damage_source.clone()
    }

    /// `isAdult()` — AgeableMob-эквивалент (`!isBaby`).
    pub fn is_adult(&self) -> bool {
        !self.entity.is_baby()
    }

    /// `Hoglin.canBeHunted()` — охотничий режим хоглина.
    pub fn can_be_hunted(&self) -> bool {
        false
    }

    /// `Armadillo.isScaredBy(entity)`.
    pub fn armadillo_is_scared_by(&self, _entity: &LivingEntity) -> bool {
        false
    }

    /// `Armadillo.canStayRolledUp()`.
    pub fn armadillo_can_stay_rolled_up(&self) -> bool {
        true
    }

    /// `Frog.canEat(entity)` — лягушка может съесть цель.
    pub fn frog_can_eat(&self, _target: &LivingEntity) -> bool {
        false
    }

    /// `PiglinAi.isWearingSafeArmor(player)`.
    pub fn piglin_ai_is_wearing_safe_armor(&self) -> bool {
        false
    }

    /// `PiglinAi.isPlayerHoldingLovedItem(player)`.
    pub fn piglin_ai_is_player_holding_loved_item(&self) -> bool {
        false
    }

    /// `PiglinAi.isZombified(entityType)` — zombified_piglin/drowned?
    /// В ваниле: `EntityType.PIGLIN`-семейство с zombie-вариантом —
    /// `zombified_piglin`.
    pub fn piglin_ai_is_zombified_type(type_id: &crate::resources::identifier::Identifier) -> bool {
        type_id.path == "zombified_piglin" || type_id.path == "zoglin"
    }

    /// `Villager.getVillagerData().profession().secondaryPoi()` —
    /// блоки вторичного POI профессии; None = нет (sensor стирает
    /// SECONDARY_JOB_SITE как в Java early-return).
    pub fn villager_secondary_poi_blocks(&self) -> Option<Vec<crate::resources::identifier::Identifier>> {
        None
    }

    /// `EntityTracker.isVisibleBy` — проверка, что target есть в
    /// `NEAREST_VISIBLE_LIVING_ENTITIES` памяти Brain'а владельца.
    /// Default `None` = brain не подключён; переопределяется когда
    /// появится `Brain`.
    pub fn brain_nearest_visible_contains(&self, _target: &LivingEntity) -> Option<bool> {
        None
    }

    /// `Brain.isMemoryValue(type, value)` — проверка, что память
    /// `type` содержит значение, равное `value` (по UUID для
    /// сущностей). Default `None` = brain не подключён;
    /// переопределяется в `Brain`-носителе.
    pub fn brain_is_memory_value(
        &self,
        _ty: &crate::world::entity::ai::memory::memory::MemoryModuleType,
        _value: &LivingEntity,
    ) -> Option<bool> {
        None
    }
}
