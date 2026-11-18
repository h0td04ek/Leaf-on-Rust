//! Перенос `net.minecraft.world.damagesource.DamageSources` — фабрика источников урона.

use crate::core::holder::Holder;
use crate::resources::resource_key::ResourceKey;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::damagesource::damage_type::DamageType;
use crate::world::damagesource::damage_types;
use crate::core::registry::registries;
use crate::world::entity::entity::Entity;
use crate::world::phys::vec3::Vec3;

/// `DamageSources` — фабрика источников урона, привязанная к реестру `DAMAGE_TYPE`.
pub struct DamageSources {
    /// `damageTypes` — реестр типов урона (lookup по `ResourceKey`).
    damage_types: Box<dyn Fn(&ResourceKey<DamageType>) -> Holder<DamageType> + Send + Sync>,
}

impl DamageSources {
    /// `new(registry)` — создаёт фабрику с resolver'ом реестра.
    pub fn new<F>(resolver: F) -> Self
    where
        F: Fn(&ResourceKey<DamageType>) -> Holder<DamageType> + Send + Sync + 'static,
    {
        Self { damage_types: Box::new(resolver) }
    }

    /// `source(key)` — базовый источник.
    fn source(&self, key: &ResourceKey<DamageType>) -> DamageSource {
        DamageSource::new((self.damage_types)(key))
    }

    /// `source(key, entity)` — источник с сущностью-причиной.
    fn source_entity(&self, key: &ResourceKey<DamageType>, entity: &Entity) -> DamageSource {
        DamageSource::with_entities((self.damage_types)(key), Some(entity.clone()), None)
    }

    /// `source(key, causing, direct)` — источник с двумя сущностями.
    fn source_entities(
        &self,
        key: &ResourceKey<DamageType>,
        causing: &Entity,
        direct: &Entity,
    ) -> DamageSource {
        DamageSource::with_entities((self.damage_types)(key), Some(causing.clone()), Some(direct.clone()))
    }

    /// `source(key, direct, causing-optional)` — projectile-вариант.
    fn source_projectile(
        &self,
        key: &ResourceKey<DamageType>,
        direct: &Entity,
        causing: Option<&Entity>,
    ) -> DamageSource {
        DamageSource::with_entities(
            (self.damage_types)(key),
            causing.map(|e| e.clone()),
            Some(direct.clone()),
        )
    }

    /// `inFire()`.
    pub fn in_fire(&self) -> DamageSource { self.source(&damage_types::in_fire()) }
    /// `campfire()`.
    pub fn campfire(&self) -> DamageSource { self.source(&damage_types::campfire()) }
    /// `lightningBolt()`.
    pub fn lightning_bolt(&self) -> DamageSource { self.source(&damage_types::lightning_bolt()) }
    /// `onFire()`.
    pub fn on_fire(&self) -> DamageSource { self.source(&damage_types::on_fire()) }
    /// `lava()`.
    pub fn lava(&self) -> DamageSource { self.source(&damage_types::lava()) }
    /// `hotFloor()`.
    pub fn hot_floor(&self) -> DamageSource { self.source(&damage_types::hot_floor()) }
    /// `inWall()`.
    pub fn in_wall(&self) -> DamageSource { self.source(&damage_types::in_wall()) }
    /// `cramming()`.
    pub fn cramming(&self) -> DamageSource { self.source(&damage_types::cramming()) }
    /// `drown()`.
    pub fn drown(&self) -> DamageSource { self.source(&damage_types::drown()) }
    /// `starve()`.
    pub fn starve(&self) -> DamageSource { self.source(&damage_types::starve()) }
    /// `cactus()`.
    pub fn cactus(&self) -> DamageSource { self.source(&damage_types::cactus()) }
    /// `fall()`.
    pub fn fall(&self) -> DamageSource { self.source(&damage_types::fall()) }
    /// `enderPearl()`.
    pub fn ender_pearl(&self) -> DamageSource { self.source(&damage_types::ender_pearl()) }
    /// `flyIntoWall()`.
    pub fn fly_into_wall(&self) -> DamageSource { self.source(&damage_types::fly_into_wall()) }
    /// `fellOutOfWorld()`.
    pub fn fell_out_of_world(&self) -> DamageSource { self.source(&damage_types::fell_out_of_world()) }
    /// `generic()`.
    pub fn generic(&self) -> DamageSource { self.source(&damage_types::generic()) }
    /// `magic()`.
    pub fn magic(&self) -> DamageSource { self.source(&damage_types::magic()) }
    /// `wither()`.
    pub fn wither(&self) -> DamageSource { self.source(&damage_types::wither()) }
    /// `dragonBreath()`.
    pub fn dragon_breath(&self) -> DamageSource { self.source(&damage_types::dragon_breath()) }
    /// `dryOut()`.
    pub fn dry_out(&self) -> DamageSource { self.source(&damage_types::dry_out()) }
    /// `sweetBerryBush()`.
    pub fn sweet_berry_bush(&self) -> DamageSource { self.source(&damage_types::sweet_berry_bush()) }
    /// `freeze()`.
    pub fn freeze(&self) -> DamageSource { self.source(&damage_types::freeze()) }
    /// `stalagmite()`.
    pub fn stalagmite(&self) -> DamageSource { self.source(&damage_types::stalagmite()) }

    /// `fallingBlock(entity)`.
    pub fn falling_block(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::falling_block(), entity)
    }
    /// `anvil(entity)`.
    pub fn anvil(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::falling_anvil(), entity)
    }
    /// `fallingStalactite(entity)`.
    pub fn falling_stalactite(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::falling_stalactite(), entity)
    }
    /// `sting(entity)`.
    pub fn sting(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::sting(), entity)
    }
    /// `mobAttack(mob)`.
    pub fn mob_attack(&self, mob: &Entity) -> DamageSource {
        self.source_entity(&damage_types::mob_attack(), mob)
    }
    /// `noAggroMobAttack(mob)`.
    pub fn no_aggro_mob_attack(&self, mob: &Entity) -> DamageSource {
        self.source_entity(&damage_types::mob_attack_no_aggro(), mob)
    }
    /// `playerAttack(player)`.
    pub fn player_attack(&self, player: &Entity) -> DamageSource {
        self.source_entity(&damage_types::player_attack(), player)
    }
    /// `arrow(arrow, shooter)`.
    pub fn arrow(&self, arrow: &Entity, shooter: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::arrow(), arrow, shooter)
    }
    /// `trident(trident, thrower)`.
    pub fn trident(&self, trident: &Entity, thrower: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::trident(), trident, thrower)
    }
    /// `mobProjectile(projectile, thrower)`.
    pub fn mob_projectile(&self, projectile: &Entity, thrower: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::mob_projectile(), projectile, thrower)
    }
    /// `spit(spit, thrower)`.
    pub fn spit(&self, spit: &Entity, thrower: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::spit(), spit, thrower)
    }
    /// `windCharge(windCharge, thrower)`.
    pub fn wind_charge(&self, wind_charge: &Entity, thrower: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::wind_charge(), wind_charge, thrower)
    }
    /// `fireworks(firework, shooter)`.
    pub fn fireworks(&self, firework: &Entity, shooter: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::fireworks(), firework, shooter)
    }
    /// `fireball(fireball, thrower)`.
    pub fn fireball(&self, fireball: &Entity, thrower: Option<&Entity>) -> DamageSource {
        self.source_projectile(&damage_types::fireball(), fireball, thrower)
    }
    /// `witherSkull(witherSkull, shooter)`.
    pub fn wither_skull(&self, wither_skull: &Entity, shooter: &Entity) -> DamageSource {
        self.source_entities(&damage_types::wither_skull(), shooter, wither_skull)
    }
    /// `thrown(causingEntity, directEntity)`.
    pub fn thrown(&self, causing: &Entity, direct: &Entity) -> DamageSource {
        self.source_entities(&damage_types::thrown(), causing, direct)
    }
    /// `indirectMagic(causingEntity, directEntity)`.
    pub fn indirect_magic(&self, causing: &Entity, direct: &Entity) -> DamageSource {
        self.source_entities(&damage_types::indirect_magic(), causing, direct)
    }
    /// `thorns(entity)`.
    pub fn thorns(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::thorns(), entity)
    }
    /// `explosion(explosion)` — упрощённо: без explosion-объекта.
    pub fn explosion_source(&self) -> DamageSource {
        self.source(&damage_types::explosion())
    }
    /// `explosion(causingEntity, directEntity)`.
    pub fn explosion_entities(&self, causing: Option<&Entity>, direct: Option<&Entity>) -> DamageSource {
        DamageSource::with_entities(
            (self.damage_types)(&damage_types::explosion()),
            causing.map(|e| e.clone()),
            direct.map(|e| e.clone()),
        )
    }
    /// `sonicBoom(entity)`.
    pub fn sonic_boom(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::sonic_boom(), entity)
    }
    /// `badRespawnPointExplosion(position)`.
    pub fn bad_respawn_point_explosion(&self, position: Vec3) -> DamageSource {
        DamageSource::with_position((self.damage_types)(&damage_types::bad_respawn_point()), position)
    }
    /// `outOfBorder()`.
    pub fn out_of_border(&self) -> DamageSource {
        self.source(&damage_types::outside_border())
    }
    /// `genericKill()`.
    pub fn generic_kill(&self) -> DamageSource {
        self.source(&damage_types::generic_kill())
    }
    /// `mace(entity)`.
    pub fn mace(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::mace_smash(), entity)
    }
    /// `spear(entity)`.
    pub fn spear(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::spear(), entity)
    }
    /// `sweetBerryBushEntity(entity)` — вариант с сущностью.
    pub fn sweet_berry_bush_entity(&self, entity: &Entity) -> DamageSource {
        self.source_entity(&damage_types::sweet_berry_bush(), entity)
    }
}

impl Clone for DamageSources {
    fn clone(&self) -> Self {
        // Resolver — Fn-объект; клонируем через Arc-подобную обёртку не можем,
        // поэтому DamageSources не клонируется напрямую — используем Arc снаружи.
        unreachable!("DamageSources должен оборачиваться в Arc")
    }
}

// Реализация Clone невозможна для Box<dyn Fn> — убираем derive и документируем Arc-использование.
impl std::fmt::Debug for DamageSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DamageSources").finish()
    }
}

/// Удобный конструктор из `RegistryAccessMap`: резолвит ключи через
/// `get_holder`, при отсутствии — `Holder::Key`.
pub fn from_registry_access_map(
    access: &crate::core::registry_access::RegistryAccess,
) -> DamageSources {
    let registry = access.registry_or_throw::<DamageType>(&registries::damage_type());
    DamageSources::new(move |key| {
        registry
            .get_holder(key)
            .unwrap_or_else(|| Holder::Key(key.clone()))
    })
}
