//! Перенос `net.minecraft.world.level.gameevent.GameEvent` —
//! record `GameEvent(int notificationRadius)` + все константы
//! `GameEvent.*` с ванильными radius (default 16, jukebox 10,
//! shriek 32).

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `GameEvent` — запись реестра `Registries.GAME_EVENT`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameEvent {
    /// registry-ключ (`minecraft:entity_die` и т.п.).
    pub id: Identifier,
    /// `notificationRadius`.
    pub notification_radius: i32,
}

impl GameEvent {
    /// `new(id)` — radius 16 (дефолт `register(name)`).
    pub fn new(id: Identifier) -> Self {
        Self { id, notification_radius: 16 }
    }

    /// `register(name, notificationRadius)`.
    pub fn with_radius(id: Identifier, notification_radius: i32) -> Self {
        Self { id, notification_radius }
    }

    /// `getNotificationRadius`.
    pub fn notification_radius(&self) -> i32 {
        self.notification_radius
    }

    /// holder этого события (`Holder::Key` по registry-ключу).
    pub fn holder(&self) -> Holder<GameEvent> {
        Holder::Key(ResourceKey::create_with_registry_name(
            registries::game_event(),
            self.id.clone(),
        ))
    }

    /// id-литерал для удобства.
    fn id(name: &str) -> Identifier {
        Identifier::with_default_namespace(name).unwrap()
    }
}

macro_rules! game_event {
    ($name:ident, $path:literal) => {
        /// `GameEvent::$name`.
        pub fn $name() -> GameEvent {
            GameEvent::new(GameEvent::id($path))
        }
    };
    ($name:ident, $path:literal, $radius:expr) => {
        /// `GameEvent::$name`.
        pub fn $name() -> GameEvent {
            GameEvent::with_radius(GameEvent::id($path), $radius)
        }
    };
}

game_event!(block_activate, "block_activate");
game_event!(block_attach, "block_attach");
game_event!(block_change, "block_change");
game_event!(block_close, "block_close");
game_event!(block_deactivate, "block_deactivate");
game_event!(block_destroy, "block_destroy");
game_event!(block_detach, "block_detach");
game_event!(block_open, "block_open");
game_event!(block_place, "block_place");
game_event!(container_close, "container_close");
game_event!(container_open, "container_open");
game_event!(drink, "drink");
game_event!(eat, "eat");
game_event!(elytra_glide, "elytra_glide");
game_event!(entity_damage, "entity_damage");
game_event!(entity_die, "entity_die");
game_event!(entity_dismount, "entity_dismount");
game_event!(entity_interact, "entity_interact");
game_event!(entity_mount, "entity_mount");
game_event!(entity_place, "entity_place");
game_event!(entity_action, "entity_action");
game_event!(equip, "equip");
game_event!(explode, "explode");
game_event!(flap, "flap");
game_event!(fluid_pickup, "fluid_pickup");
game_event!(fluid_place, "fluid_place");
game_event!(hit_ground, "hit_ground");
game_event!(instrument_play, "instrument_play");
game_event!(item_interact_finish, "item_interact_finish");
game_event!(item_interact_start, "item_interact_start");
game_event!(jukebox_play, "jukebox_play", 10);
game_event!(jukebox_stop_play, "jukebox_stop_play", 10);
game_event!(lightning_strike, "lightning_strike");
game_event!(note_block_play, "note_block_play");
game_event!(prime_fuse, "prime_fuse");
game_event!(projectile_land, "projectile_land");
game_event!(projectile_shoot, "projectile_shoot");
game_event!(sculk_sensor_tendrils_clicking, "sculk_sensor_tendrils_clicking");
game_event!(shear, "shear");
game_event!(shriek, "shriek", 32);
game_event!(splash, "splash");
game_event!(step, "step");
game_event!(swim, "swim");
game_event!(teleport, "teleport");
game_event!(unequip, "unequip");
game_event!(resonate_1, "resonate_1");
game_event!(resonate_2, "resonate_2");
game_event!(resonate_3, "resonate_3");
game_event!(resonate_4, "resonate_4");
game_event!(resonate_5, "resonate_5");
game_event!(resonate_6, "resonate_6");
game_event!(resonate_7, "resonate_7");
game_event!(resonate_8, "resonate_8");
game_event!(resonate_9, "resonate_9");
game_event!(resonate_10, "resonate_10");
game_event!(resonate_11, "resonate_11");
game_event!(resonate_12, "resonate_12");
game_event!(resonate_13, "resonate_13");
game_event!(resonate_14, "resonate_14");
game_event!(resonate_15, "resonate_15");

/// Все события в порядке регистрации ванили.
pub fn all() -> Vec<GameEvent> {
    vec![
        block_activate(), block_attach(), block_change(), block_close(),
        block_deactivate(), block_destroy(), block_detach(), block_open(),
        block_place(), container_close(), container_open(), drink(), eat(),
        elytra_glide(), entity_damage(), entity_die(), entity_dismount(),
        entity_interact(), entity_mount(), entity_place(), entity_action(),
        equip(), explode(), flap(), fluid_pickup(), fluid_place(),
        hit_ground(), instrument_play(), item_interact_finish(),
        item_interact_start(), jukebox_play(), jukebox_stop_play(),
        lightning_strike(), note_block_play(), prime_fuse(), projectile_land(),
        projectile_shoot(), sculk_sensor_tendrils_clicking(), shear(),
        shriek(), splash(), step(), swim(), teleport(), unequip(),
        resonate_1(), resonate_2(), resonate_3(), resonate_4(), resonate_5(),
        resonate_6(), resonate_7(), resonate_8(), resonate_9(), resonate_10(),
        resonate_11(), resonate_12(), resonate_13(), resonate_14(), resonate_15(),
    ]
}
