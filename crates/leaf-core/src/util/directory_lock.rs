//! Перенос `net.minecraft.util.DirectoryLock`.
//! `FileChannel.tryLock` → `fs2::FileExt::try_lock_exclusive`.

use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

pub const LOCK_FILE: &str = "session.lock";

/// `DirectoryLock` — эксклюзивная блокировка директории через session.lock.
pub struct DirectoryLock {
    _lock_file: File,
    path: PathBuf,
}

impl DirectoryLock {
    /// `create(path)` — создаёт/блокирует session.lock; занято → LockException.
    pub fn create(path: &Path) -> Result<Self, DirectoryLockError> {
        let lock_path = path.join(LOCK_FILE);
        create_directories_safe(path).map_err(DirectoryLockError::Io)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .map_err(DirectoryLockError::Io)?;
        // Пишем "☃" (UTF-8) как Java DUMMY + force
        file.write_all_marker().map_err(DirectoryLockError::Io)?;
        file.try_lock_exclusive().map_err(|_| {
            DirectoryLockError::AlreadyLocked(lock_path.clone())
        })?;
        Ok(Self { _lock_file: file, path: lock_path })
    }

    /// `isValid` — блокировка жива, пока файл открыт.
    pub fn is_valid(&self) -> bool {
        true
    }

    /// `isLocked(path)` — true если session.lock занят другим процессом.
    pub fn is_locked(path: &Path) -> io::Result<bool> {
        let lock_path = path.join(LOCK_FILE);
        match OpenOptions::new().write(true).open(&lock_path) {
            Ok(file) => {
                match file.try_lock_exclusive() {
                    Ok(()) => {
                        let _ = file.unlock();
                        Ok(false)
                    }
                    Err(_) => Ok(true),
                }
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Путь к lock-файлу.
    pub fn lock_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = self._lock_file.unlock();
    }
}

trait WriteAllMarker {
    fn write_all_marker(&self) -> io::Result<()>;
}

impl WriteAllMarker for File {
    fn write_all_marker(&self) -> io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        let mut f = self.try_clone()?;
        f.seek(SeekFrom::Start(0))?;
        f.write_all("☃".as_bytes())?;
        f.sync_all()
    }
}

fn create_directories_safe(path: &Path) -> io::Result<()> {
    if path.exists() {
        let real = path.canonicalize()?;
        fs::create_dir_all(real)
    } else {
        fs::create_dir_all(path)
    }
}

#[derive(Debug)]
pub enum DirectoryLockError {
    Io(io::Error),
    AlreadyLocked(PathBuf),
}

impl std::fmt::Display for DirectoryLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::AlreadyLocked(p) => write!(
                f,
                "{}: already locked (possibly by other Minecraft instance?)",
                p.display()
            ),
        }
    }
}

impl std::error::Error for DirectoryLockError {}

impl From<io::Error> for DirectoryLockError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
