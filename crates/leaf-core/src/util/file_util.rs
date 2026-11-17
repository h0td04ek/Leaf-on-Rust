//! Перенос `net.minecraft.util.FileUtil` (портабельная часть).
//! `sanitizeName` требует `SharedConstants.ILLEGAL_FILE_CHARACTERS` —
//! список illegal chars Minecraft; здесь ванильный набор.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static COPY_COUNTER_PATTERN: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("(?i)(<name>.*) \\((<count>\\d*)\\)").unwrap());
static RESERVED_WINDOWS_FILENAMES: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new("(?i).*\\.|(?:COM|CLOCK\\$|CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\\..*)?").unwrap()
});
static STRICT_PATH_SEGMENT_CHECK: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new("[-._a-z0-9]+").unwrap());

const MAX_FILE_NAME: usize = 255;

/// `sanitizeName` — заменяет illegal chars на '_'.
pub fn sanitize_name(name: &str) -> String {
    // SharedConstants.ILLEGAL_FILE_CHARACTERS (vanilla): / \ ? % * : | " < >
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if matches!(c, '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>') {
            out.push('_');
        } else {
            out.push(c);
        }
    }
    // name.replaceAll("[./\"]", "_") — уже покрыто выше для ./"/
    out
}

/// `findAvailableName(dirPath, fileName, fileFormat)` → Result<String>.
pub fn find_available_name(dir_path: &Path, file_name: &str, file_format: &str) -> std::io::Result<String> {
    let mut file_name = sanitize_name(file_name);
    if !is_path_part_portable(&file_name) {
        file_name = format!("_{}_", file_name);
    }
    let mut i = 0i32;
    let caps_owned = COPY_COUNTER_PATTERN.captures(&file_name).map(|c| {
        (c.get(1).map(|m| m.as_str().to_string()), c.get(2).map(|m| m.as_str().to_string()))
    });
    if let Some((Some(name), Some(count))) = caps_owned {
        file_name = name;
        i = count.parse().unwrap_or(0);
    }
    if file_name.chars().count() > MAX_FILE_NAME - file_format.len() {
        file_name = file_name.chars().take(MAX_FILE_NAME - file_format.len()).collect();
    }
    loop {
        let mut string = file_name.clone();
        if i != 0 {
            let suffix = format!(" ({})", i);
            let limit = MAX_FILE_NAME - suffix.chars().count();
            if file_name.chars().count() > limit {
                string = file_name.chars().take(limit).collect();
            }
            string = format!("{}{}", string, suffix);
        }
        string = format!("{}{}", string, file_format);
        let path = dir_path.join(&string);
        // createDirectory + deleteIfExists — атомарная проверка занятости
        match std::fs::create_dir(&path) {
            Ok(()) => {
                let _ = std::fs::remove_dir(&path);
                return Ok(path
                    .strip_prefix(dir_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/"));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                i += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// `isPathNormalized` — path == path.normalize().
pub fn is_path_normalized(path: &Path) -> bool {
    path.normalize() == *path
}

trait NormalizePath {
    fn normalize(&self) -> PathBuf;
}

impl NormalizePath for Path {
    /// Упрощённая нормализация: убирает `.` и разрешает `..` лексически.
    fn normalize(&self) -> PathBuf {
        let mut out = PathBuf::new();
        for comp in self.components() {
            match comp {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    out.pop();
                }
                other => out.push(other.as_os_str()),
            }
        }
        out
    }
}

/// `isPathPortable` — все сегменты пути портируемы.
pub fn is_path_portable(path: &Path) -> bool {
    path.iter().all(|seg| is_path_part_portable(&seg.to_string_lossy()))
}

/// `isPathPartPortable` — не зарезервированное Windows-имя.
pub fn is_path_part_portable(path: &str) -> bool {
    !RESERVED_WINDOWS_FILENAMES.is_match(path)
}

/// `createPathToResource(dirPath, locationPath, fileFormat)`.
pub fn create_path_to_resource(dir_path: &Path, location_path: &str, file_format: &str) -> Result<PathBuf, String> {
    let string = format!("{}{}", location_path, file_format);
    let path = Path::new(&string);
    if path.ends_with(file_format) && location_path.is_empty() {
        return Err(format!("empty resource name: {}", string));
    }
    Ok(dir_path.join(path))
}

/// `getFullResourcePath` — нормализация слэшей в '/'.
pub fn get_full_resource_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    // FilenameUtils.getFullPath — часть до последнего '/'
    match normalized.rfind('/') {
        Some(i) => normalized[..=i].to_string(),
        None => String::new(),
    }
}

/// `normalizeResourcePath` — FilenameUtils.normalize.
pub fn normalize_resource_path(path: &str) -> String {
    let unified = path.replace('\\', "/");
    let mut parts: Vec<&str> = Vec::new();
    for seg in unified.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// `decomposePath` → Result<Vec<String>, String>.
pub fn decompose_path(path: &str) -> Result<Vec<String>, String> {
    if !path.contains('/') {
        return match path {
            "" | "." | ".." => Err(format!("Invalid path '{}'", path)),
            _ if !contains_allowed_characters_only(path) => Err(format!("Invalid path '{}'", path)),
            _ => Ok(vec![path.to_string()]),
        };
    }
    let mut list = Vec::new();
    let mut i = 0usize;
    let mut index = path.find('/');
    loop {
        let last = index.is_none();
        let idx = index.unwrap_or(path.len());
        let sub = &path[i..idx];
        if matches!(sub, "" | "." | "..") {
            return Err(format!("Invalid segment '{}' in path '{}'", sub, path));
        }
        if !contains_allowed_characters_only(sub) {
            return Err(format!("Invalid segment '{}' in path '{}'", sub, path));
        }
        list.push(sub.to_string());
        if last {
            return Ok(list);
        }
        i = idx + 1;
        index = path[i..].find('/').map(|r| i + r);
    }
}

/// `resolvePath(path, subdirectories)`.
pub fn resolve_path(path: &Path, subdirectories: &[String]) -> PathBuf {
    match subdirectories.len() {
        0 => path.to_path_buf(),
        1 => path.join(&subdirectories[0]),
        _ => {
            let mut p = path.join(&subdirectories[0]);
            for seg in &subdirectories[1..] {
                p = p.join(seg);
            }
            p
        }
    }
}

fn contains_allowed_characters_only(segment: &str) -> bool {
    STRICT_PATH_SEGMENT_CHECK.is_match(segment)
}

/// `isValidPathSegment`.
pub fn is_valid_path_segment(segment: &str) -> bool {
    segment != ".." && segment != "." && contains_allowed_characters_only(segment)
}

/// `validatePath(elements...)` — panic при невалидном сегменте.
pub fn validate_path(elements: &[&str]) {
    assert!(!elements.is_empty(), "Path must have at least one element");
    for string in elements {
        assert!(is_valid_path_segment(string), "Illegal segment {} in path {:?}", string, elements);
    }
}

/// `createDirectoriesSafe(path)` — createDirectories по realpath если существует.
pub fn create_directories_safe(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        let real = path.canonicalize()?;
        std::fs::create_dir_all(real)
    } else {
        std::fs::create_dir_all(path)
    }
}
