//! Перенос `net.minecraft.world.entity.ai.village.poi.PoiTypes` —
//! ключи и vanilla-регистрация типов POI.

use std::rc::Rc;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::level::block::state::block_state::BlockState;

macro_rules! poi_key {
    ($name:ident, $str:literal) => {
        pub fn $name() -> ResourceKey<PoiType> {
            ResourceKey::create_with_registry_name(
                registries::point_of_interest_type(),
                Identifier::minecraft($str),
            )
        }
    };
}

poi_key!(armorer, "armorer");
poi_key!(butcher, "butcher");
poi_key!(cartographer, "cartographer");
poi_key!(cleric, "cleric");
poi_key!(farmer, "farmer");
poi_key!(fisherman, "fisherman");
poi_key!(fletcher, "fletcher");
poi_key!(leatherworker, "leatherworker");
poi_key!(librarian, "librarian");
poi_key!(mason, "mason");
poi_key!(shepherd, "shepherd");
poi_key!(toolsmith, "toolsmith");
poi_key!(weaponsmith, "weaponsmith");
poi_key!(home, "home");
poi_key!(meeting, "meeting");
poi_key!(beehive, "beehive");
poi_key!(bee_nest, "bee_nest");
poi_key!(nether_portal, "nether_portal");
poi_key!(lodestone, "lodestone");
poi_key!(lightning_rod, "lightning_rod");
poi_key!(test_instance, "test_instance");

/// Все ключи в порядке `PoiTypes`-объявления.
pub fn all_keys() -> Vec<ResourceKey<PoiType>> {
    vec![
        armorer(), butcher(), cartographer(), cleric(), farmer(),
        fisherman(), fletcher(), leatherworker(), librarian(), mason(),
        shepherd(), toolsmith(), weaponsmith(), home(), meeting(),
        beehive(), bee_nest(), nether_portal(), lodestone(),
        lightning_rod(), test_instance(),
    ]
}

fn block_id(state: &BlockState) -> &Identifier {
    &state.block.id
}

fn is_block(state: &BlockState, names: &[&str]) -> bool {
    names.iter().any(|n| *block_id(state) == Identifier::minecraft(n))
}

/// Matcher по id блока (state-уровень не различаем — см.
/// комментарий в `PoiType`).
fn matcher(names: &'static [&'static str]) -> Arc<dyn Fn(&BlockState) -> bool + Send + Sync> {
    Arc::new(move |s| is_block(s, names))
}

/// BEDS — все цветные кровати (16 шт.). В Java фильтруется
/// `BedBlock.PART == HEAD`; head-часть определяется по
/// свойству `part` — здесь матчим по префиксу `*_bed`,
/// property-фильтр применяется в `for_state` если доступен.
fn bed_matcher() -> Arc<dyn Fn(&BlockState) -> bool + Send + Sync> {
    Arc::new(|s| {
        let path = block_id(s).get_path();
        path.ends_with("_bed") || path == "bed"
    })
}

fn cauldron_matcher() -> Arc<dyn Fn(&BlockState) -> bool + Send + Sync> {
    matcher(&["cauldron", "lava_cauldron", "water_cauldron", "powder_snow_cauldron"])
}

fn lightning_rod_matcher() -> Arc<dyn Fn(&BlockState) -> bool + Send + Sync> {
    matcher(&[
        "lightning_rod",
        "exposed_lightning_rod",
        "weathered_lightning_rod",
        "oxidized_lightning_rod",
        "waxed_lightning_rod",
        "waxed_exposed_lightning_rod",
        "waxed_weathered_lightning_rod",
        "waxed_oxidized_lightning_rod",
    ])
}

/// `bootstrap` — все vanilla POI-типы (key → PoiType).
/// Возвращает `Holder::Key`-совместимые записи через
/// `Holder::Direct(PoiType)` — реальный registry runtime
/// подключается позже.
pub fn bootstrap() -> Vec<(ResourceKey<PoiType>, PoiType)> {
    let bed = bed_matcher();
    let cauldron = cauldron_matcher();
    let rods = lightning_rod_matcher();
    let def = |key: ResourceKey<PoiType>, m: Arc<dyn Fn(&BlockState) -> bool + Send + Sync>, t: i32, r: i32| {
        let t2 = PoiType::new(key.identifier.clone(), m, t, r);
        (key, t2)
    };
    vec![
        def(armorer(), matcher(&["blast_furnace"]), 1, 1),
        def(butcher(), matcher(&["smoker"]), 1, 1),
        def(cartographer(), matcher(&["cartography_table"]), 1, 1),
        def(cleric(), matcher(&["brewing_stand"]), 1, 1),
        def(farmer(), matcher(&["composter"]), 1, 1),
        def(fisherman(), matcher(&["barrel"]), 1, 1),
        def(fletcher(), matcher(&["fletching_table"]), 1, 1),
        def(leatherworker(), cauldron, 1, 1),
        def(librarian(), matcher(&["lectern"]), 1, 1),
        def(mason(), matcher(&["stonecutter"]), 1, 1),
        def(shepherd(), matcher(&["loom"]), 1, 1),
        def(toolsmith(), matcher(&["smithing_table"]), 1, 1),
        def(weaponsmith(), matcher(&["grindstone"]), 1, 1),
        def(home(), bed, 1, 1),
        def(meeting(), matcher(&["bell"]), 32, 6),
        def(beehive(), matcher(&["beehive"]), 0, 1),
        def(bee_nest(), matcher(&["bee_nest"]), 0, 1),
        def(nether_portal(), matcher(&["nether_portal"]), 0, 1),
        def(lodestone(), matcher(&["lodestone"]), 0, 1),
        def(lightning_rod(), rods, 0, 1),
        def(test_instance(), matcher(&["test_instance_block"]), 0, 1),
    ]
}

static TABLE: OnceLock<Vec<(ResourceKey<PoiType>, PoiType)>> = OnceLock::new();

fn table() -> &'static [(ResourceKey<PoiType>, PoiType)] {
    TABLE.get_or_init(bootstrap)
}

/// `get(key)` — PoiType по registry-ключу.
pub fn get(key: &ResourceKey<PoiType>) -> Option<PoiType> {
    table().iter().find(|(k, _)| k == key).map(|(_, t)| t.clone())
}

/// `forState(state)` — POI-тип для block state.
pub fn for_state(state: &BlockState) -> Option<PoiType> {
    table().iter().find(|(_, t)| t.is(state)).map(|(_, t)| t.clone())
}

/// `hasPoi(state)` — есть ли вообще POI для состояния.
pub fn has_poi(state: &BlockState) -> bool {
    for_state(state).is_some()
}

/// Holder-эквивалент: `Holder::Direct(poiType)` для записи
/// в `PoiRecord`-совместимых структурах.
pub fn holder_of(poi_type: PoiType) -> Holder<PoiType> {
    Holder::Direct(poi_type)
}
