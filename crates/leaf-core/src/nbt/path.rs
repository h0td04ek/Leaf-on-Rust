//! Перенос `net.minecraft.commands.arguments.NbtPathArgument` —
//! путь внутри NBT (`foo.bar[0].baz`, `*`, `{k=v}`).

use crate::nbt::tag::Tag;

/// Элемент пути `NbtPath`.
#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    /// `CompoundChildNode` — ключ составного тега.
    Key(String),
    /// `IndexedElementNode` — `[i]` элемент списка.
    Index(i32),
    /// `AllElementsNode` — `*` все элементы списка.
    All,
    /// `MatchElementNode` — `{k=v}` элементы списка по совпадению.
    Match(Vec<(String, Tag)>),
}

/// `NbtPath` — распарсенный путь.
#[derive(Clone, Debug, PartialEq)]
pub struct NbtPath {
    segments: Vec<PathSegment>,
}

impl NbtPath {
    /// `parse` — разбор строки пути.
    pub fn parse(path: &str) -> Result<Self, String> {
        let mut segments = Vec::new();
        let mut chars = path.chars().peekable();
        // Первый сегмент — ключ (до '.' или '[').
        let mut key = String::new();
        while let Some(&c) = chars.peek() {
            if c == '.' || c == '[' {
                break;
            }
            key.push(c);
            chars.next();
        }
        if !key.is_empty() {
            segments.push(PathSegment::Key(key));
        }
        loop {
            match chars.peek().copied() {
                Some('.') => {
                    chars.next();
                    let mut key = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '.' || c == '[' {
                            break;
                        }
                        key.push(c);
                        chars.next();
                    }
                    if key.is_empty() {
                        return Err("Empty node name".to_string());
                    }
                    segments.push(PathSegment::Key(key));
                }
                Some('[') => {
                    chars.next();
                    match chars.peek().copied() {
                        Some('*') => {
                            chars.next();
                            segments.push(PathSegment::All);
                        }
                        Some('{') => {
                            chars.next();
                            let mut body = String::new();
                            let mut depth = 1;
                            while let Some(c) = chars.next() {
                                if c == '{' { depth += 1; }
                                if c == '}' {
                                    depth -= 1;
                                    if depth == 0 { break; }
                                }
                                body.push(c);
                            }
                            let mut pairs = Vec::new();
                            for part in body.split(',') {
                                let mut it = part.splitn(2, '=');
                                let k = it.next().unwrap_or("").trim().to_string();
                                let v = it.next().unwrap_or("").trim();
                                if k.is_empty() || v.is_empty() {
                                    return Err("Invalid match node".to_string());
                                }
                                let tag = crate::nbt::snbt_parse::parse(v)
                                    .map_err(|e| format!("Invalid match value: {e:?}"))?;
                                pairs.push((k, tag));
                            }
                            segments.push(PathSegment::Match(pairs));
                        }
                        Some(_) => {
                            let mut num = String::new();
                            while let Some(&c) = chars.peek() {
                                if c == ']' { break; }
                                num.push(c);
                                chars.next();
                            }
                            if chars.next() != Some(']') {
                                return Err("Missing closing bracket".to_string());
                            }
                            let idx: i32 = num.trim().parse()
                                .map_err(|_| "Invalid index".to_string())?;
                            if idx < 0 {
                                return Err("Negative index".to_string());
                            }
                            segments.push(PathSegment::Index(idx));
                        }
                        None => return Err("Unexpected end".to_string()),
                    }
                }
                _ => break,
            }
        }
        if segments.is_empty() {
            return Err("Empty path".to_string());
        }
        Ok(Self { segments })
    }

    /// `segments()`.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// `get(compoundTag)` — все теги, удовлетворяющие пути.
    pub fn get<'a>(&self, root: &'a Tag) -> Vec<&'a Tag> {
        let mut current: Vec<&Tag> = vec![root];
        for seg in &self.segments {
            let mut next: Vec<&Tag> = Vec::new();
            for tag in current {
                match seg {
                    PathSegment::Key(k) => {
                        if let Some(c) = tag.as_compound() {
                            if let Some(t) = c.get(k) {
                                next.push(t);
                            }
                        }
                    }
                    PathSegment::Index(i) => {
                        if let Some(list) = tag.as_list() {
                            if let Some(t) = list.get(*i as usize) {
                                next.push(t);
                            }
                        }
                    }
                    PathSegment::All => {
                        if let Some(list) = tag.as_list() {
                            next.extend(list.iter());
                        }
                    }
                    PathSegment::Match(pairs) => {
                        if let Some(list) = tag.as_list() {
                            for item in list {
                                let matches = pairs.iter().all(|(k, v)| {
                                    item.as_compound()
                                        .and_then(|c| c.get(k))
                                        .map(|t| t == v)
                                        .unwrap_or(false)
                                });
                                if matches {
                                    next.push(item);
                                }
                            }
                        }
                    }
                }
            }
            current = next;
        }
        current
    }

    /// `getSingle` — первый результат или None.
    pub fn get_single<'a>(&self, root: &'a Tag) -> Option<&'a Tag> {
        self.get(root).into_iter().next()
    }
}
