//! Перенос `net.minecraft.world.damagesource.DamageTypes` — ключи типов урона
//! и их vanilla-значения по умолчанию.

use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::damagesource::damage_effects::DamageEffects;
use crate::world::damagesource::damage_scaling::DamageScaling;
use crate::world::damagesource::death_message_type::DeathMessageType;
use crate::world::damagesource::damage_type::DamageType;

macro_rules! damage_type_key {
    ($name:ident, $path:literal) => {
        /// `DamageTypes::$name`.
        pub fn $name() -> ResourceKey<DamageType> {
            ResourceKey::create_with_registry_name(
                registries::damage_type(),
                Identifier::with_default_namespace($path).unwrap(),
            )
        }
    };
}

damage_type_key!(in_fire, "in_fire");
damage_type_key!(campfire, "campfire");
damage_type_key!(lightning_bolt, "lightning_bolt");
damage_type_key!(on_fire, "on_fire");
damage_type_key!(lava, "lava");
damage_type_key!(hot_floor, "hot_floor");
damage_type_key!(in_wall, "in_wall");
damage_type_key!(cramming, "cramming");
damage_type_key!(drown, "drown");
damage_type_key!(starve, "starve");
damage_type_key!(cactus, "cactus");
damage_type_key!(fall, "fall");
damage_type_key!(ender_pearl, "ender_pearl");
damage_type_key!(fly_into_wall, "fly_into_wall");
damage_type_key!(fell_out_of_world, "out_of_world");
damage_type_key!(generic, "generic");
damage_type_key!(magic, "magic");
damage_type_key!(wither, "wither");
damage_type_key!(dragon_breath, "dragon_breath");
damage_type_key!(dry_out, "dry_out");
damage_type_key!(sweet_berry_bush, "sweet_berry_bush");
damage_type_key!(freeze, "freeze");
damage_type_key!(stalagmite, "stalagmite");
damage_type_key!(falling_block, "falling_block");
damage_type_key!(falling_anvil, "falling_anvil");
damage_type_key!(falling_stalactite, "falling_stalactite");
damage_type_key!(sting, "sting");
damage_type_key!(mob_attack, "mob_attack");
damage_type_key!(mob_attack_no_aggro, "mob_attack_no_aggro");
damage_type_key!(player_attack, "player_attack");
damage_type_key!(spear, "spear");
damage_type_key!(arrow, "arrow");
damage_type_key!(trident, "trident");
damage_type_key!(mob_projectile, "mob_projectile");
damage_type_key!(spit, "spit");
damage_type_key!(wind_charge, "wind_charge");
damage_type_key!(fireworks, "fireworks");
damage_type_key!(fireball, "fireball");
damage_type_key!(unattributed_fireball, "unattributed_fireball");
damage_type_key!(wither_skull, "wither_skull");
damage_type_key!(thrown, "thrown");
damage_type_key!(indirect_magic, "indirect_magic");
damage_type_key!(thorns, "thorns");
damage_type_key!(explosion, "explosion");
damage_type_key!(player_explosion, "player_explosion");
damage_type_key!(sonic_boom, "sonic_boom");
damage_type_key!(bad_respawn_point, "bad_respawn_point");
damage_type_key!(outside_border, "outside_border");
damage_type_key!(generic_kill, "generic_kill");
damage_type_key!(mace_smash, "mace_smash");

/// Запись vanilla-значения типа урона.
struct VanillaDamageType {
    key_fn: fn() -> ResourceKey<DamageType>,
    msg_id: &'static str,
    scaling: DamageScaling,
    exhaustion: f32,
    effects: DamageEffects,
    death_message_type: DeathMessageType,
}

fn vanilla_table() -> Vec<VanillaDamageType> {
    use DamageEffects as E;
    use DamageScaling as S;
    use DeathMessageType as D;
    let n = S::Never;
    let l = S::WhenCausedByLivingNonPlayer;
    vec![
        VanillaDamageType { key_fn: in_fire, msg_id: "inFire", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: campfire, msg_id: "campfire", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: lightning_bolt, msg_id: "lightningBolt", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: on_fire, msg_id: "onFire", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: lava, msg_id: "lava", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: hot_floor, msg_id: "hotFloor", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: in_wall, msg_id: "inWall", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: cramming, msg_id: "cramming", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: drown, msg_id: "drown", scaling: n, exhaustion: 0.0, effects: E::Drowning, death_message_type: D::Default },
        VanillaDamageType { key_fn: starve, msg_id: "starve", scaling: n, exhaustion: 0.1, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: cactus, msg_id: "cactus", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: fall, msg_id: "fall", scaling: l, exhaustion: 0.1, effects: E::Hurt, death_message_type: D::FallVariants },
        VanillaDamageType { key_fn: ender_pearl, msg_id: "enderPearl", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: fly_into_wall, msg_id: "flyIntoWall", scaling: l, exhaustion: 0.1, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: fell_out_of_world, msg_id: "outOfWorld", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: generic, msg_id: "generic", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: magic, msg_id: "magic", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: wither, msg_id: "wither", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: dragon_breath, msg_id: "dragonBreath", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: dry_out, msg_id: "dryOut", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: sweet_berry_bush, msg_id: "sweetBerryBush", scaling: n, exhaustion: 0.0, effects: E::Poking, death_message_type: D::Default },
        VanillaDamageType { key_fn: freeze, msg_id: "freeze", scaling: n, exhaustion: 0.0, effects: E::Freezing, death_message_type: D::Default },
        VanillaDamageType { key_fn: stalagmite, msg_id: "stalagmite", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: falling_block, msg_id: "fallingBlock", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: falling_anvil, msg_id: "anvil", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: falling_stalactite, msg_id: "fallingStalactite", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: sting, msg_id: "sting", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: mob_attack, msg_id: "mob", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: mob_attack_no_aggro, msg_id: "mob", scaling: l, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: player_attack, msg_id: "player", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: spear, msg_id: "player", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: arrow, msg_id: "arrow", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: trident, msg_id: "trident", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: mob_projectile, msg_id: "mob", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: spit, msg_id: "mob", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: wind_charge, msg_id: "windCharge", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: fireworks, msg_id: "fireworks", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: fireball, msg_id: "fireball", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: unattributed_fireball, msg_id: "onFire", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: wither_skull, msg_id: "witherSkull", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: thrown, msg_id: "thrown", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: indirect_magic, msg_id: "indirectMagic", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: thorns, msg_id: "thorns", scaling: n, exhaustion: 0.1, effects: E::Thorns, death_message_type: D::Default },
        VanillaDamageType { key_fn: explosion, msg_id: "explosion", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: player_explosion, msg_id: "explosion.player", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: sonic_boom, msg_id: "sonic_boom", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: bad_respawn_point, msg_id: "badRespawn", scaling: n, exhaustion: 0.1, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: outside_border, msg_id: "outsideBorder", scaling: n, exhaustion: 0.2, effects: E::Hurt, death_message_type: D::Default },
        VanillaDamageType { key_fn: generic_kill, msg_id: "genericKill", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::IntentionalGameDesign },
        VanillaDamageType { key_fn: mace_smash, msg_id: "mace_smash", scaling: n, exhaustion: 0.0, effects: E::Hurt, death_message_type: D::Default },
    ]
}

/// `DamageTypes.bootstrap` — все типы урона с vanilla-значениями.
pub fn bootstrap() -> Vec<(ResourceKey<DamageType>, DamageType)> {
    vanilla_table()
        .into_iter()
        .map(|v| {
            let key = (v.key_fn)();
            (
                key.clone(),
                DamageType::new(
                    key.id().clone(),
                    v.msg_id,
                    v.scaling,
                    v.exhaustion,
                    v.effects,
                    v.death_message_type,
                ),
            )
        })
        .collect()
}

/// Все key-функции в порядке регистрации.
pub const ALL_KEY_FNS: &[fn() -> ResourceKey<DamageType>] = &[
    in_fire, campfire, lightning_bolt, on_fire, lava, hot_floor, in_wall,
    cramming, drown, starve, cactus, fall, ender_pearl, fly_into_wall,
    fell_out_of_world, generic, magic, wither, dragon_breath, dry_out,
    sweet_berry_bush, freeze, stalagmite, falling_block, falling_anvil,
    falling_stalactite, sting, mob_attack, mob_attack_no_aggro, player_attack,
    spear, arrow, trident, mob_projectile, spit, wind_charge, fireworks,
    fireball, unattributed_fireball, wither_skull, thrown, indirect_magic,
    thorns, explosion, player_explosion, sonic_boom, bad_respawn_point,
    outside_border, generic_kill, mace_smash,
];
