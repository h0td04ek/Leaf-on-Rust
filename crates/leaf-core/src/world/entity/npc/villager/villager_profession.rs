//! Перенос `net.minecraft.world.entity.npc.villager.VillagerProfession` —
//! профессия жителя: heldJobSite/acquirableJobSite предикаты,
//! requestedItems, secondaryPoi, workSound.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::network::chat::component::Component;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::sounds::sound_event::SoundEvent;
use crate::sounds::sound_events;
use crate::world::entity::ai::village::poi::poi_manager::holder_is;
use crate::world::entity::ai::village::poi::poi_record::resolve_poi_type;
use crate::world::entity::ai::village::poi::poi_type::PoiType;

/// `Predicate<Holder<PoiType>>` — предикат по типу POI.
pub type PoiPredicate = Arc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync>;

macro_rules! villager_profession_key {
    ($name:ident, $str:literal) => {
        pub fn $name() -> ResourceKey<VillagerProfession> {
            ResourceKey::create_with_registry_name(
                registries::villager_profession(),
                Identifier::minecraft($str),
            )
        }
    };
}

villager_profession_key!(none, "none");
villager_profession_key!(armorer, "armorer");
villager_profession_key!(butcher, "butcher");
villager_profession_key!(cartographer, "cartographer");
villager_profession_key!(cleric, "cleric");
villager_profession_key!(farmer, "farmer");
villager_profession_key!(fisherman, "fisherman");
villager_profession_key!(fletcher, "fletcher");
villager_profession_key!(leatherworker, "leatherworker");
villager_profession_key!(librarian, "librarian");
villager_profession_key!(mason, "mason");
villager_profession_key!(nitwit, "nitwit");
villager_profession_key!(shepherd, "shepherd");
villager_profession_key!(toolsmith, "toolsmith");
villager_profession_key!(weaponsmith, "weaponsmith");

/// `VillagerProfession(name, heldJobSite, acquirableJobSite,
/// requestedItems, secondaryPoi, workSound)`.
#[derive(Clone)]
pub struct VillagerProfession {
    /// registry-id профессии (`minecraft:armorer` и т.д.).
    pub id: Identifier,
    /// `name` — translatable component.
    pub name: Component,
    /// `heldJobSite` — POI-типы, которые профессия «держит».
    pub held_job_site: PoiPredicate,
    /// `acquirableJobSite` — POI-типы, которые можно занять.
    pub acquirable_job_site: PoiPredicate,
    /// `requestedItems` — предметы для сделок (id).
    pub requested_items: HashSet<Identifier>,
    /// `secondaryPoi` — id блоков второстепенных POI (farmer
    /// → farmland).
    pub secondary_poi: HashSet<Identifier>,
    /// `workSound` — звук работы (Holder<SoundEvent> или id).
    pub work_sound: Option<Identifier>,
}

impl VillagerProfession {
    /// `id` — registry identifier.
    pub fn key_id(&self) -> &Identifier {
        &self.id
    }
}

fn sound_id(h: Holder<SoundEvent>) -> Identifier {
    match h {
        Holder::Key(k) => k.identifier,
        Holder::Direct(se) => se.location().clone(),
        Holder::Reference(_) => Identifier::minecraft("empty"),
    }
}

fn poi_is(key: &ResourceKey<PoiType>) -> PoiPredicate {
    let key = key.clone();
    Arc::new(move |h| holder_is(h, &key))
}

fn none_predicate() -> PoiPredicate {
    Arc::new(|_| false)
}

fn all_acquirable_jobs() -> PoiPredicate {
    // `PoiTypeTags.ACQUIRABLE_JOB_SITE` — все job-site POI
    // кроме HOME/MEETING/BEEHIVE/BEE_NEST/NETHER_PORTAL/
    // LODESTONE/LIGHTNING_ROD/TEST_INSTANCE.
    Arc::new(|h| {
        use crate::world::entity::ai::village::poi::poi_types;
        let Some(t) = resolve_poi_type(h) else { return false };
        // acquirable = все job-site типы (armorer..weaponsmith)
        // — HOME и MEETING исключены; BEEHIVE и пр. имеют
        // maxTickets=0 → не acquirable в Java через тег.
        matches!(
            t.id.get_path(),
            "armorer" | "butcher" | "cartographer" | "cleric" | "farmer"
                | "fisherman" | "fletcher" | "leatherworker" | "librarian"
                | "mason" | "shepherd" | "toolsmith" | "weaponsmith"
        )
    })
}

/// Запись таблицы: (key, profession).
pub struct ProfessionEntry {
    pub key: ResourceKey<VillagerProfession>,
    pub profession: VillagerProfession,
}

static TABLE: OnceLock<Vec<ProfessionEntry>> = OnceLock::new();

fn table() -> &'static [ProfessionEntry] {
    TABLE.get_or_init(|| {
        let mut v: Vec<ProfessionEntry> = Vec::new();
        let mut add = |key: ResourceKey<VillagerProfession>,
                       held: PoiPredicate,
                       acquirable: PoiPredicate,
                       requested: &[&str],
                       secondary: &[&str],
                       sound: Option<Identifier>| {
            let name = Component::translatable(
                format!(
                    "entity.minecraft.villager.{}",
                    key.identifier.get_path()
                ),
                Vec::new(),
            );
            v.push(ProfessionEntry {
                key: key.clone(),
                profession: VillagerProfession {
                    id: key.identifier.clone(),
                    name,
                    held_job_site: held,
                    acquirable_job_site: acquirable,
                    requested_items: requested
                        .iter()
                        .map(|s| Identifier::minecraft(s))
                        .collect(),
                    secondary_poi: secondary
                        .iter()
                        .map(|s| Identifier::minecraft(s))
                        .collect(),
                    work_sound: sound,
                },
            });
        };
        use crate::world::entity::ai::village::poi::poi_types as pt;
        add(none(), none_predicate(), all_acquirable_jobs(), &[], &[], None);
        add(armorer(), poi_is(&pt::armorer()), poi_is(&pt::armorer()), &[], &[], Some(sound_id(sound_events::villager_work_armorer())));
        add(butcher(), poi_is(&pt::butcher()), poi_is(&pt::butcher()), &[], &[], Some(sound_id(sound_events::villager_work_butcher())));
        add(cartographer(), poi_is(&pt::cartographer()), poi_is(&pt::cartographer()), &[], &[], Some(sound_id(sound_events::villager_work_cartographer())));
        add(cleric(), poi_is(&pt::cleric()), poi_is(&pt::cleric()), &[], &[], Some(sound_id(sound_events::villager_work_cleric())));
        add(
            farmer(),
            poi_is(&pt::farmer()),
            poi_is(&pt::farmer()),
            &["wheat", "wheat_seeds", "beetroot_seeds", "bone_meal"],
            &["farmland"],
            Some(sound_id(sound_events::villager_work_farmer())),
        );
        add(fisherman(), poi_is(&pt::fisherman()), poi_is(&pt::fisherman()), &[], &[], Some(sound_id(sound_events::villager_work_fisherman())));
        add(fletcher(), poi_is(&pt::fletcher()), poi_is(&pt::fletcher()), &[], &[], Some(sound_id(sound_events::villager_work_fletcher())));
        add(leatherworker(), poi_is(&pt::leatherworker()), poi_is(&pt::leatherworker()), &[], &[], Some(sound_id(sound_events::villager_work_leatherworker())));
        add(librarian(), poi_is(&pt::librarian()), poi_is(&pt::librarian()), &[], &[], Some(sound_id(sound_events::villager_work_librarian())));
        add(mason(), poi_is(&pt::mason()), poi_is(&pt::mason()), &[], &[], Some(sound_id(sound_events::villager_work_mason())));
        add(nitwit(), none_predicate(), none_predicate(), &[], &[], None);
        add(shepherd(), poi_is(&pt::shepherd()), poi_is(&pt::shepherd()), &[], &[], Some(sound_id(sound_events::villager_work_shepherd())));
        add(toolsmith(), poi_is(&pt::toolsmith()), poi_is(&pt::toolsmith()), &[], &[], Some(sound_id(sound_events::villager_work_toolsmith())));
        add(weaponsmith(), poi_is(&pt::weaponsmith()), poi_is(&pt::weaponsmith()), &[], &[], Some(sound_id(sound_events::villager_work_weaponsmith())));
        v
    })
}

/// `get(key)` — профессия по ключу.
pub fn get(key: &ResourceKey<VillagerProfession>) -> Option<VillagerProfession> {
    table()
        .iter()
        .find(|e| &e.key == key)
        .map(|e| e.profession.clone())
}

/// Все ключи.
pub fn all_keys() -> Vec<ResourceKey<VillagerProfession>> {
    table().iter().map(|e| e.key.clone()).collect()
}

/// `listElements`-эквивалент для AssignProfessionFromJobSite:
/// профессии, чей heldJobSite принимает данный POI-holder.
pub fn professions_for_poi(poi: &Holder<PoiType>) -> Vec<ResourceKey<VillagerProfession>> {
    table()
        .iter()
        .filter(|e| (e.profession.held_job_site)(poi))
        .map(|e| e.key.clone())
        .collect()
}

impl std::fmt::Debug for VillagerProfession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VillagerProfession").finish_non_exhaustive()
    }
}
