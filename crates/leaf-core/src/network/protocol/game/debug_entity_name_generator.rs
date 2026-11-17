//! `DebugEntityNameGenerator` — генератор debug-имён для entity.

use crate::random::{RandomSource, legacy};
use uuid::Uuid;

/// `DebugEntityNameGenerator`.
pub struct DebugEntityNameGenerator;

impl DebugEntityNameGenerator {
    /// `NAMES_FIRST_PART`.
    const NAMES_FIRST_PART: [&'static str; 32] = [
        "Slim", "Far", "River", "Silly", "Fat", "Thin", "Fish", "Bat",
        "Dark", "Oak", "Sly", "Bush", "Zen", "Bark", "Cry", "Slack",
        "Soup", "Grim", "Hook", "Dirt", "Mud", "Sad", "Hard", "Crook",
        "Sneak", "Stink", "Weird", "Fire", "Soot", "Soft", "Rough",
        "Cling", "Scar",
    ];

    /// `NAMES_SECOND_PART`.
    const NAMES_SECOND_PART: [&'static str; 30] = [
        "Fox", "Tail", "Jaw", "Whisper", "Twig", "Root", "Finder", "Nose",
        "Brow", "Blade", "Fry", "Seek", "Wart", "Tooth", "Foot", "Leaf",
        "Stone", "Fall", "Face", "Tongue", "Voice", "Lip", "Mouth", "Snail",
        "Toe", "Ear", "Hair", "Beard", "Shirt", "Fist",
    ];

    /// `getEntityName(entity)` — для `Entity`.
    /// `Player` → `getPlainTextName`, иначе `customName` или `getEntityName(uuid)`.
    pub fn get_entity_name(
        is_player: bool,
        plain_text_name: Option<&str>,
        custom_name: Option<&str>,
        uuid: Uuid,
    ) -> String {
        if is_player {
            return plain_text_name.unwrap_or_default().to_string();
        }
        if let Some(name) = custom_name {
            return name.to_string();
        }
        Self::get_entity_name_from_uuid(uuid)
    }

    /// `getEntityName(uuid)` — случайное имя из UUID.
    pub fn get_entity_name_from_uuid(uuid: Uuid) -> String {
        let mut random = Self::get_random(uuid);
        let first = Self::get_random_string(&mut *random, &Self::NAMES_FIRST_PART);
        let second = Self::get_random_string(&mut *random, &Self::NAMES_SECOND_PART);
        format!("{}{}", first, second)
    }

    /// `getRandomString(random, strings)`.
    fn get_random_string(random: &mut dyn RandomSource, strings: &[&'static str]) -> &'static str {
        let idx = random.next_int_bound(strings.len() as i32) as usize;
        strings[idx]
    }

    /// `getRandom(uuid)` — `RandomSource.create(uuid.hashCode() >> 2)`.
    fn get_random(uuid: Uuid) -> Box<dyn RandomSource> {
        let hash = uuid.as_u128() as i64;
        legacy::create((hash >> 2) as u64 as i64)
    }
}
