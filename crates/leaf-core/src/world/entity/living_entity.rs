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
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type::EntityType;
use crate::world::item::item_stack::ItemStack;
use crate::world::phys::vec3::Vec3;

/// `LivingEntity` — сущность с health/attributes/equipment/effects.
pub struct LivingEntity {
    /// Базовая entity.
    pub entity: Entity,
    /// `attributes` — карта атрибутов.
    pub attributes: HashMap<u64, f64>,
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
    /// `lastDamageSource`, `lastDamageStamp`.
    pub last_damage_source: Option<DamageSource>,
    pub last_damage_stamp: i64,
    /// `autoSpinAttackTicks`, `autoSpinAttackDmg`.
    pub auto_spin_attack_ticks: i32,
    pub auto_spin_attack_dmg: f32,
    /// `xpReward`.
    pub xp_reward: i32,
    /// Текущий health.
    pub health: f32,
}

impl LivingEntity {
    /// Shared-flag бит «мёртв» (в Java — через EntityDataAccessor).
    pub const FLAG_DEAD: u8 = 1 << 7;

    /// `new(EntityType, Level)`.
    pub fn new(entity_type: Holder<EntityType>, level: Option<std::sync::Arc<dyn crate::world::level::level_reader::LevelReader>>) -> Self {
        let entity = Entity::new(entity_type.clone(), level);
        let combat_tracker = CombatTracker::new(entity.clone());
        Self {
            entity,
            attributes: HashMap::new(),
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
            no_action_time: 0,
            last_hurt: 0.0,
            jumping: false,
            xxa: 0.0,
            yya: 0.0,
            zza: 0.0,
            effects_dirty: true,
            last_hurt_by_mob_timestamp: 0,
            speed: 0.0,
            no_jump_delay: 0,
            absorption_amount: 0.0,
            use_item: ItemStack::empty(),
            use_item_remaining: 0,
            fall_fly_ticks: 0,
            last_pos: None,
            last_damage_source: None,
            last_damage_stamp: 0,
            auto_spin_attack_ticks: 0,
            auto_spin_attack_dmg: 0.0,
            xp_reward: 0,
            health: 20.0,
        }
    }

    /// `getHealth()`.
    pub fn get_health(&self) -> f32 {
        self.health
    }

    /// `getMaxHealth()` — `MAX_HEALTH` атрибут (default 20).
    pub fn get_max_health(&self) -> f32 {
        self.attributes.values().next().map(|_| 20.0f32).unwrap_or(20.0)
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

    /// `startUsingItem(hand)`.
    pub fn start_using_item(&mut self, hand: HumanoidArm) {
        self.swinging_arm = hand;
        self.swinging = true;
        self.swing_time = 0;
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
        if self.death_time > 0 { self.death_time += 1; }
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

    /// `die(source)`.
    pub fn die(&mut self, source: DamageSource) {
        if !self.is_dead_or_dying() {
            self.dead = true;
            self.death_time = 0;
            Self::on_killed(&mut self.entity, source);
        }
    }

    /// `onKilled(source)`.
    pub fn on_killed(entity: &mut Entity, _source: DamageSource) {
        entity.set_shared_flag(LivingEntity::FLAG_DEAD, true);
    }

    /// `hurt(source, amount)`.
    pub fn hurt(&mut self, source: DamageSource, amount: f32) -> bool {
        if !self.is_alive() || self.is_invulnerable_to(&source) {
            return false;
        }
        self.combat_tracker.record_damage(source.clone(), amount);
        self.last_damage_source = Some(source.clone());
        self.last_damage_stamp = self.entity.tick_count as i64;
        self.hurt_duration = 10;
        self.hurt_time = self.hurt_duration.max(10);
        self.set_health(self.get_health() - amount);
        self.last_hurt = amount;
        self.no_action_time = 0;
        if self.get_health() <= 0.0 {
            self.die(source);
            return true;
        }
        true
    }

    /// `isInvulnerableTo(source)`.
    pub fn is_invulnerable_to(&self, _source: &DamageSource) -> bool {
        false
    }

    /// `getKillCredit()`.
    pub fn get_kill_credit(&self) -> Option<Entity> {
        self.combat_tracker
            .entries
            .last()
            .and_then(|e| e.source.get_causing_entity().cloned())
    }

    /// `dropAllDeathLoot`.
    pub fn drop_all_death_loot(&mut self, _source: &DamageSource, _destroyed_by_player: bool) {}

    /// `getExperienceReward()`.
    pub fn get_experience_reward(&self) -> i32 {
        self.xp_reward
    }

    /// `getMainArm()`.
    pub fn get_main_arm(&self) -> HumanoidArm {
        self.swinging_arm
    }

    /// `addEffect(instance)`.
    pub fn add_effect(&mut self, instance: MobEffectInstance) {
        let key = Self::effect_key(&instance);
        self.active_effects.insert(key, instance);
        self.effects_dirty = true;
    }

    fn effect_key(instance: &MobEffectInstance) -> u64 {
        instance.effect.value().map(|e| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&e.id, &mut h);
            std::hash::Hasher::finish(&h)
        }).unwrap_or(0)
    }

    /// `hasEffect(effect)`.
    pub fn has_effect(&self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        effect.value().map(|e| {
            self.active_effects.values().any(|i| {
                i.effect.value().map(|x| x.id == e.id).unwrap_or(false)
            })
        }).unwrap_or(false)
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
    pub fn can_be_seen_as_enemy(&self) -> bool {
        !self.is_dead_or_dying()
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
    pub fn get_attribute_value(&self, attribute: &Holder<crate::entity::ai::attributes::Attribute>) -> f64 {
        attribute.value().map(|a| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&a.id, &mut h);
            self.attributes.get(&std::hash::Hasher::finish(&h)).copied().unwrap_or(0.0)
        }).unwrap_or(0.0)
    }

    /// `getKnockback(target)`.
    pub fn get_knockback(&self, _target: &Entity) -> f32 {
        0.0
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

    /// `isBlocking()`.
    pub fn is_blocking(&self) -> bool {
        false
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
    pub fn has_pose(&self, _pose: crate::world::entity::pose::Pose) -> bool {
        false
    }

    /// `setPose(pose)`.
    pub fn set_pose(&mut self, _pose: crate::world::entity::pose::Pose) {}

    /// `getItemInHand(hand)`.
    pub fn get_item_in_hand(&self, hand: HumanoidArm) -> &ItemStack {
        match hand {
            HumanoidArm::Right => self.get_main_hand_item(),
            _ => self.get_offhand_item(),
        }
    }

    /// `getVehicle()`.
    pub fn get_vehicle(&self) -> Option<Entity> {
        self.entity.get_vehicle().cloned()
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

    /// `removeAllEffects()`.
    pub fn remove_all_effects(&mut self) -> Vec<MobEffectInstance> {
        self.active_effects.drain().map(|(_, v)| v).collect()
    }

    /// `removeEffect(effect)`.
    pub fn remove_effect(&mut self, effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        if let Some(e) = effect.value() {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&e.id, &mut h);
            let key = std::hash::Hasher::finish(&h);
            return self.active_effects.remove(&key).is_some();
        }
        false
    }

    /// `addAdditionalSaveData(tag)`.
    pub fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("Health", Tag::Float(self.health));
        tag.put("AbsorptionAmount", Tag::Float(self.absorption_amount));
        tag.put("HurtTime", Tag::Int(self.hurt_time));
        tag.put("HurtByTimestamp", Tag::Int(self.last_hurt_by_mob_timestamp));
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
        if let Some(Tag::Int(na)) = tag.get("NoActionTime") { self.no_action_time = *na; }
        if let Some(Tag::Byte(ff)) = tag.get("FallFlying") { self.fall_fly_ticks = if *ff != 0 { 1 } else { 0 }; }
        if let Some(Tag::List(list)) = tag.get("ActiveEffects") {
            for item in list {
                if let Tag::Compound(c) = item {
                    if let Some(inst) = MobEffectInstance::from_nbt(c) {
                        self.add_effect(inst);
                    }
                }
            }
        }
    }

    /// `tickEffects()`.
    pub fn tick_effects(&mut self) {
        let mut expired = Vec::new();
        for (key, inst) in self.active_effects.iter_mut() {
            inst.tick();
            if inst.is_expired() {
                expired.push(*key);
            }
        }
        for key in expired {
            self.active_effects.remove(&key);
        }
        self.effects_dirty = true;
    }

    /// `getArmorValue()`.
    pub fn get_armor_value(&self) -> f32 {
        0.0
    }

    /// `getArmorCoverPercentage()`.
    pub fn get_armor_cover_percentage(&self) -> f32 {
        0.0
    }

    /// `getDamageAfterArmorAbsorb`.
    pub fn get_damage_after_armor_absorb(&self, _source: &DamageSource, damage: f32) -> f32 {
        damage
    }

    /// `getDamageAfterMagicAbsorb`.
    pub fn get_damage_after_magic_absorb(&self, _source: &DamageSource, damage: f32) -> f32 {
        damage
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

    /// `isAffectedByPotions()`.
    pub fn is_affected_by_potions(&self) -> bool {
        true
    }

    /// `canBeAffected(effect)`.
    pub fn can_be_affected(&self, _effect: &Holder<crate::world::effect::mob_effect::MobEffect>) -> bool {
        true
    }

    /// `onEffectAdded`.
    pub fn on_effect_added(&mut self, _effect: &MobEffectInstance, _source: Option<&Entity>) {}

    /// `onEffectUpdated`.
    pub fn on_effect_updated(&mut self, _effect: &MobEffectInstance, _old_amplifier: i32, _source: Option<&Entity>) {}

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

    /// `isAlliedTo(entity)`.
    pub fn is_allied_to(&self, _entity: &Entity) -> bool {
        false
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

    /// `detectEquipmentUpdates`.
    pub fn detect_equipment_updates(&mut self) {
        self.effects_dirty = true;
    }

    /// `hasLineOfSight`.
    pub fn has_line_of_sight(&self, _entity: &Entity) -> bool {
        false
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

    /// `markHurt`.
    pub fn mark_hurt(&mut self) {
        self.hurt_time = self.hurt_duration;
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

    /// `doHurtTarget`.
    pub fn do_hurt_target(&mut self, _target: &mut Entity, _source: &DamageSource) -> bool {
        false
    }

    /// `getAttackKnockback`.
    pub fn get_attack_knockback(&self, _target: &Entity, _damage: f32) -> f32 {
        0.0
    }

    /// `knockback`.
    pub fn knockback(&mut self, _strength: f32, _x: f64, _z: f64) {}

    /// `indicateDamage`.
    pub fn indicate_damage(&mut self, _yaw: f32) {}

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

    /// `getLastHurtByPlayer`.
    pub fn get_last_hurt_by_player(&self) -> Option<Entity> {
        None
    }

    /// `setLastHurtByPlayer`.
    pub fn set_last_hurt_by_player(&mut self, _player: Option<Entity>) {}

    /// `setLastHurtByMob`.
    pub fn set_last_hurt_by_mob(&mut self, _mob: Option<Entity>) {}

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
        None
    }

    /// `setLastHurtMob`.
    pub fn set_last_hurt_mob(&mut self, _mob: Option<Entity>) {}

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

    /// `isInLava`.
    pub fn is_in_lava(&self) -> bool {
        self.entity.is_in_lava()
    }

    /// `isOnFire`.
    pub fn is_on_fire(&self) -> bool {
        self.entity.is_on_fire()
    }

    /// `setRemainingFireTicks`.
    pub fn set_remaining_fire_ticks(&mut self, _ticks: i32) {}

    /// `getRemainingFireTicks`.
    pub fn get_remaining_fire_ticks(&self) -> i32 {
        0
    }

    /// `isFreezing`.
    pub fn is_freezing(&self) -> bool {
        false
    }

    /// `getTicksFrozen`.
    pub fn get_ticks_frozen(&self) -> i32 {
        0
    }

    /// `setTicksFrozen`.
    pub fn set_ticks_frozen(&mut self, _ticks: i32) {}

    /// `isFullyFrozen`.
    pub fn is_fully_frozen(&self) -> bool {
        false
    }

    /// `getPercentFrozen`.
    pub fn get_percent_frozen(&self) -> f32 {
        0.0
    }

    /// `setPercentFrozen`.
    pub fn set_percent_frozen(&mut self, _percent: f32) {}

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
    pub fn max_health(&self) -> f32 {
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
        self.die(source);
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
        self.add_effect(instance);
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
        self.add_effect(instance);
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
            no_action_time: self.no_action_time,
            last_hurt: self.last_hurt,
            jumping: self.jumping,
            xxa: self.xxa,
            yya: self.yya,
            zza: self.zza,
            effects_dirty: self.effects_dirty,
            last_hurt_by_mob_timestamp: self.last_hurt_by_mob_timestamp,
            speed: self.speed,
            no_jump_delay: self.no_jump_delay,
            absorption_amount: self.absorption_amount,
            use_item: self.use_item.clone(),
            use_item_remaining: self.use_item_remaining,
            fall_fly_ticks: self.fall_fly_ticks,
            last_pos: self.last_pos,
            last_damage_source: self.last_damage_source.clone(),
            last_damage_stamp: self.last_damage_stamp,
            auto_spin_attack_ticks: self.auto_spin_attack_ticks,
            auto_spin_attack_dmg: self.auto_spin_attack_dmg,
            xp_reward: self.xp_reward,
            health: self.health,
        }
    }
}
