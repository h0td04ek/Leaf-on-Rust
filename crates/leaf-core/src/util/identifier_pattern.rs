//! Перенос `net.minecraft.util.IdentifierPattern`.
//! DFU-поля (`namespacePattern`/`pathPattern` как Optional<Pattern>)
//! опущены — портированы только предикаты.

use crate::resources::Identifier;
use std::sync::Arc;

/// `IdentifierPattern` — паттерн-фильтр для Identifier.
#[derive(Clone)]
pub struct IdentifierPattern {
    namespace_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    path_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl IdentifierPattern {
    /// Создание из regex-паттернов (как в Java: Optional<Pattern>).
    pub fn new(namespace_pattern: Option<&str>, path_pattern: Option<&str>) -> Result<Self, regex::Error> {
        let namespace_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync> = match namespace_pattern {
            Some(p) => {
                let re = regex::Regex::new(p)?;
                Arc::new(move |s: &str| re.is_match(s))
            }
            None => Arc::new(|_| true),
        };
        let path_predicate: Arc<dyn Fn(&str) -> bool + Send + Sync> = match path_pattern {
            Some(p) => {
                let re = regex::Regex::new(p)?;
                Arc::new(move |s: &str| re.is_match(s))
            }
            None => Arc::new(|_| true),
        };
        Ok(Self { namespace_predicate, path_predicate })
    }

    /// Паттерн, пропускающий всё.
    pub fn any() -> Self {
        Self {
            namespace_predicate: Arc::new(|_| true),
            path_predicate: Arc::new(|_| true),
        }
    }

    /// `namespacePredicate()`
    pub fn namespace_predicate(&self) -> &Arc<dyn Fn(&str) -> bool + Send + Sync> {
        &self.namespace_predicate
    }

    /// `pathPredicate()`
    pub fn path_predicate(&self) -> &Arc<dyn Fn(&str) -> bool + Send + Sync> {
        &self.path_predicate
    }

    /// `locationPredicate()` — Identifier → bool.
    pub fn location_predicate(&self) -> impl Fn(&Identifier) -> bool + '_ {
        |location: &Identifier| {
            (self.namespace_predicate)(location.get_namespace())
                && (self.path_predicate)(location.get_path())
        }
    }
}
