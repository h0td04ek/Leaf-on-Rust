use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

pub const OWNER: &str = "h0td04ek";
pub const ALL: u8 = 0;
pub const GAMEMASTERS: u8 = 2;
pub const OWNERS: u8 = 4;
pub const PERMISSION_EVENT: u8 = 24;

pub fn valid_name(name: &str) -> bool {
    (1..=16).contains(&name.len())
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub struct Operators {
    path: PathBuf,
    names: BTreeSet<String>,
}

impl Operators {
    pub fn open(path: PathBuf) -> io::Result<Self> {
        let names = match fs::read_to_string(&path) {
            Ok(contents) => contents
                .lines()
                .map(str::trim)
                .filter(|name| !name.is_empty() && !name.starts_with('#'))
                .map(|name| {
                    valid_name(name)
                        .then(|| name.to_owned())
                        .ok_or_else(|| io::Error::other("invalid name in ops.txt"))
                })
                .collect::<io::Result<_>>()?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let names = BTreeSet::from([OWNER.to_owned()]);
                Self::save(&path, &names)?;
                names
            }
            Err(error) => return Err(error),
        };
        Ok(Self { path, names })
    }

    pub fn level(&self, name: &str) -> u8 {
        if self.names.contains(name) {
            OWNERS
        } else {
            ALL
        }
    }

    pub fn change(&mut self, name: &str, grant: bool) -> io::Result<bool> {
        if !valid_name(name) {
            return Err(io::Error::other("invalid player name"));
        }
        let mut next = self.names.clone();
        let changed = if grant {
            next.insert(name.to_owned())
        } else {
            next.remove(name)
        };
        if changed {
            if next.is_empty() {
                return Err(io::Error::other("cannot remove the last operator"));
            }
            Self::save(&self.path, &next)?;
            self.names = next;
        }
        Ok(changed)
    }

    fn save(path: &std::path::Path, names: &BTreeSet<String>) -> io::Result<()> {
        let temporary = path.with_extension("txt.tmp");
        let mut file = File::create(&temporary)?;
        for name in names {
            writeln!(file, "{name}")?;
        }
        file.sync_all()?;
        drop(file);
        fs::rename(temporary, path)
    }
}
