use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use leaf_core::network::protocol::game::clientbound_map_item_data_packet::ClientboundMapItemDataPacket;
use leaf_core::resources::Identifier;
use leaf_core::world::item::item_stack::ItemStack;
use leaf_core::world::level::saveddata::maps::{MapId, MapPatch};

use crate::world::FlatWorld;

const MAX_SCALE: u8 = 4;
pub const MAP_SIZE: i32 = 128;

#[derive(Clone, Copy)]
struct Map {
    x: i32,
    z: i32,
    scale: u8,
}

pub struct Maps {
    path: PathBuf,
    entries: Mutex<Vec<Map>>,
}

impl Maps {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let entries = match fs::read_to_string(&path) {
            Ok(data) => data
                .lines()
                .map(|line| {
                    let mut fields = line.split_ascii_whitespace();
                    let mut field = || {
                        fields
                            .next()
                            .and_then(|value| value.parse::<i32>().ok())
                            .ok_or_else(|| io::Error::other("invalid entry in maps.txt"))
                    };
                    let (x, z, scale) = (field()?, field()?, field()?);
                    if !(0..=i32::from(MAX_SCALE)).contains(&scale)
                        || x.abs_diff(0) > 30_002_048
                        || z.abs_diff(0) > 30_002_048
                        || fields.next().is_some()
                    {
                        return Err(io::Error::other("invalid entry in maps.txt"));
                    }
                    Ok(Map {
                        x,
                        z,
                        scale: scale as u8,
                    })
                })
                .collect::<io::Result<_>>()?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        Ok(Self {
            path,
            entries: Mutex::new(entries),
        })
    }

    pub fn create(&self, x: i32, z: i32) -> io::Result<MapId> {
        self.add(x, z, 0)
    }

    pub fn can_expand(&self, id: MapId) -> bool {
        self.get(id).is_some_and(|map| map.scale < MAX_SCALE)
    }

    pub fn expand(&self, id: MapId) -> io::Result<MapId> {
        let map = self
            .get(id)
            .filter(|map| map.scale < MAX_SCALE)
            .ok_or_else(|| io::Error::other("map cannot be expanded"))?;
        self.add(map.x, map.z, map.scale + 1)
    }

    fn get(&self, id: MapId) -> Option<Map> {
        self.entries.lock().unwrap().get(id.0 as usize).copied()
    }

    fn add(&self, x: i32, z: i32, scale: u8) -> io::Result<MapId> {
        let width = MAP_SIZE << scale;
        let center = |coordinate: i32| {
            (coordinate + MAP_SIZE / 2).div_euclid(width) * width + (width - MAP_SIZE) / 2
        };
        let added = Map {
            x: center(x),
            z: center(z),
            scale,
        };
        let mut maps = self.entries.lock().unwrap();
        let id = MapId(i32::try_from(maps.len()).map_err(io::Error::other)?);
        let temporary = self.path.with_extension("txt.tmp");
        let mut file = File::create(&temporary)?;
        for map in maps.iter().chain(std::iter::once(&added)) {
            writeln!(file, "{} {} {}", map.x, map.z, map.scale)?;
        }
        file.sync_all()?;
        drop(file);
        fs::rename(temporary, &self.path)?;
        maps.push(added);
        Ok(id)
    }

    pub fn render(&self, id: MapId, world: &FlatWorld) -> Option<ClientboundMapItemDataPacket> {
        let map = self.get(id)?;
        let step = 1 << map.scale;
        let offset = MAP_SIZE / 2 * step;
        let pixels = world.map_colors(map.x - offset, map.z - offset, step);
        Some(ClientboundMapItemDataPacket::new(
            id,
            map.scale as i8,
            false,
            None,
            Some(MapPatch::new(0, 0, MAP_SIZE, MAP_SIZE, pixels)),
        ))
    }
}

pub fn map_id(item: &ItemStack) -> Option<MapId> {
    item.get_component::<MapId>(&Identifier::minecraft("map_id"))
}

pub fn set_map_id(item: &mut ItemStack, id: MapId) {
    item.set_component(&Identifier::minecraft("map_id"), Some(id));
}
