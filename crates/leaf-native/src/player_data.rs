use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use leaf_core::nbt::{self, CompoundTag, Tag};
use leaf_core::util::directory_lock::DirectoryLock;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::item::item_stack::ItemStack;
use leaf_core::world::phys::vec3::Vec3;

use crate::items::item_id;
use crate::wire::invalid;
use crate::world::SPAWN_Y;

const INVENTORY_SLOTS: usize = 41;
const FORMAT_VERSION: i32 = 1;
const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

pub struct PlayerData {
    pub position: Vec3,
    pub inventory: [ItemStack; INVENTORY_SLOTS],
    pub selection: usize,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.5, f64::from(SPAWN_Y), 0.5),
            inventory: std::array::from_fn(|_| ItemStack::empty()),
            selection: 0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl PlayerData {
    pub fn chunk(&self) -> (i32, i32) {
        (
            self.position.x.floor() as i32 >> 4,
            self.position.z.floor() as i32 >> 4,
        )
    }

    pub fn valid_position(position: Vec3) -> bool {
        [position.x, position.y, position.z]
            .iter()
            .all(|value| value.is_finite() && value.abs() <= 30_000_000.0)
    }

    fn from_nbt(tag: &CompoundTag) -> io::Result<Self> {
        if tag.get_int("FormatVersion") != Some(FORMAT_VERSION) {
            return Err(invalid("unsupported native player data format"));
        }
        let Some([Tag::Double(x), Tag::Double(y), Tag::Double(z)]) =
            tag.get("Pos").and_then(Tag::as_list)
        else {
            return Err(invalid("invalid saved position"));
        };
        let position = Vec3::new(*x, *y, *z);
        if !Self::valid_position(position) {
            return Err(invalid("invalid saved position"));
        }
        let Some([Tag::Float(yaw), Tag::Float(pitch)]) = tag.get("Rotation").and_then(Tag::as_list)
        else {
            return Err(invalid("invalid saved rotation"));
        };
        if !yaw.is_finite() || !pitch.is_finite() {
            return Err(invalid("invalid saved rotation"));
        }
        let selection = tag
            .get_int("SelectedItemSlot")
            .filter(|value| (0..9).contains(value))
            .ok_or_else(|| invalid("invalid saved hotbar slot"))? as usize;

        let mut inventory = std::array::from_fn(|_| ItemStack::empty());
        let entries = tag
            .get("Inventory")
            .and_then(Tag::as_list)
            .filter(|slots| slots.len() <= INVENTORY_SLOTS)
            .ok_or_else(|| invalid("invalid saved inventory"))?;
        for entry in entries {
            let item = entry
                .as_compound()
                .ok_or_else(|| invalid("invalid inventory entry"))?;
            let slot = item
                .get_byte("Slot")
                .map(|slot| slot as usize)
                .filter(|&slot| slot < INVENTORY_SLOTS)
                .ok_or_else(|| invalid("invalid inventory slot"))?;
            if !inventory[slot].is_empty() {
                return Err(invalid("duplicate inventory slot"));
            }
            let stack = ItemStack::from_nbt(item).map_err(invalid)?;
            if item_id(&stack).is_none() {
                return Err(invalid("saved item is missing from the native registry"));
            }
            inventory[slot] = stack;
        }

        Ok(Self {
            position,
            inventory,
            selection,
            yaw: *yaw,
            pitch: *pitch,
        })
    }

    fn to_nbt(&self) -> io::Result<CompoundTag> {
        let mut tag = CompoundTag::new();
        tag.put_int("FormatVersion", FORMAT_VERSION);
        tag.put(
            "Pos",
            Tag::List(vec![
                Tag::Double(self.position.x),
                Tag::Double(self.position.y),
                Tag::Double(self.position.z),
            ]),
        );
        tag.put(
            "Rotation",
            Tag::List(vec![Tag::Float(self.yaw), Tag::Float(self.pitch)]),
        );
        tag.put_int("SelectedItemSlot", self.selection as i32);
        let mut inventory = Vec::new();
        for (index, item) in self.inventory.iter().enumerate() {
            if item.is_empty() {
                continue;
            }
            let mut slot = CompoundTag::new();
            slot.put_byte("Slot", index as i8);
            inventory.push(item.save(slot).map_err(invalid)?);
        }
        tag.put("Inventory", Tag::List(inventory));
        Ok(tag)
    }
}

pub struct PlayerStore {
    directory: PathBuf,
    _lock: DirectoryLock,
}

impl PlayerStore {
    pub fn open(world: &Path) -> io::Result<Self> {
        let lock = DirectoryLock::create(world).map_err(io::Error::other)?;
        let directory = world.join("playerdata");
        fs::create_dir_all(&directory)?;
        Ok(Self {
            directory,
            _lock: lock,
        })
    }

    pub fn load(&self, uuid: Uuid) -> io::Result<PlayerData> {
        let path = self.directory.join(format!("{uuid}.dat"));
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PlayerData::default())
            }
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(invalid("native player data is too large"));
        }
        let mut budget = nbt::accounter::NbtAccounter::create(64 * 1024 * 1024);
        let tag =
            nbt::io::read_bytes(&bytes, &mut budget).map_err(|error| invalid(error.to_string()))?;
        PlayerData::from_nbt(&tag)
    }

    pub fn save(
        &self,
        uuid: Uuid,
        player: &PlayerData,
        last_saved: &mut Vec<u8>,
    ) -> io::Result<()> {
        let bytes =
            nbt::io::write(&player.to_nbt()?).map_err(|error| invalid(error.to_string()))?;
        if bytes == *last_saved {
            return Ok(());
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(invalid("native player data is too large"));
        }
        let path = self.directory.join(format!("{uuid}.dat"));
        let temporary = path.with_extension("dat.tmp");
        let result: io::Result<()> = (|| {
            let mut file = File::create(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, &path)?;
            #[cfg(unix)]
            File::open(&self.directory)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result?;
        *last_saved = bytes;
        Ok(())
    }
}
