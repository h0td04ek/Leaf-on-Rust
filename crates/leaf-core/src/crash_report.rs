//! Перенос `net.minecraft.CrashReport` + `CrashReportCategory`
//! (минимальная модель: заголовок, детали категорий, suppressed).
//!
//! `systemDetails` (полный сбор системной информации) и
//! `CrashReportDetail`-lazy-семантика сохранены через boxed-замыкания.

use std::collections::BTreeMap;
use std::fmt;

/// Поставщик детали краш-репорта (`Supplier<String>`).
pub type DetailSupplier = Box<dyn Fn() -> String>;

/// `CrashReportCategory` — категория с именованными деталями.
#[derive(Default)]
pub struct CrashReportCategory {
    pub name: String,
    /// Порядок добавления + значения (lazy).
    pub details: Vec<(String, DetailSupplier)>,
}

/// Трейты для заполнения категории/репорта из пользовательского кода
/// (`PacketListener.fillCrashReport` и т.п.).
pub trait CrashReportCategorySink {
    /// `setDetail(name, supplier)`.
    fn set_detail(&mut self, name: &str, supplier: DetailSupplier);

    /// Удобный вариант со строковым значением.
    fn set_detail_str(&mut self, name: &str, value: String) {
        self.set_detail(name, Box::new(move || value.clone()));
    }
}

impl std::fmt::Debug for CrashReportCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrashReportCategory")
            .field("name", &self.name)
            .field("details", &self.details.len())
            .finish()
    }
}

impl CrashReportCategorySink for CrashReportCategory {
    fn set_detail(&mut self, name: &str, supplier: DetailSupplier) {
        // Java: повторный setDetail перезаписывает — заменяем.
        if let Some(entry) = self.details.iter_mut().find(|(n, _)| n == name) {
            entry.1 = supplier;
        } else {
            self.details.push((name.to_string(), supplier));
        }
    }
}

/// `CrashReportSink` — интерфейс добавления категорий.
pub trait CrashReportSink {
    /// `addCategory(name, consumer)`.
    fn add_category(&mut self, name: &str, populate: &mut dyn FnMut(&mut dyn CrashReportCategorySink));
}

/// `CrashReport`.
#[derive(Debug, Default)]
pub struct CrashReport {
    /// `title` — заголовок ("Main thread packet handler" и т.п.).
    pub title: String,
    /// `description` — описание.
    pub description: Option<String>,
    /// `exception` — строковое представление throwable.
    pub exception: Option<String>,
    /// `suppressed` — подавленные исключения.
    pub suppressed: Vec<String>,
    /// `details` — категории.
    pub categories: BTreeMap<String, CrashReportCategory>,
}

impl CrashReport {
    /// `forThrowable(throwable, title)`.
    pub fn for_throwable(error: &dyn std::error::Error, title: &str) -> CrashReport {
        let mut report = CrashReport { title: title.to_string(), ..Default::default() };
        report.exception = Some(error.to_string());
        report
    }

    /// `forThrowable(throwable)` — title по умолчанию.
    pub fn for_throwable_default(error: &dyn std::error::Error) -> CrashReport {
        Self::for_throwable(error, "Unknown cause")
    }

    /// `addCategory(name)` → заполнение через sink.
    pub fn add_category<F>(&mut self, name: &str, populate: F) -> &mut CrashReportCategory
    where
        F: FnOnce(&mut dyn CrashReportCategorySink),
    {
        let mut cat = CrashReportCategory { name: name.to_string(), ..Default::default() };
        populate(&mut cat);
        self.categories.insert(name.to_string(), cat);
        self.categories.get_mut(name).unwrap()
    }

    /// Копия с разрешёнными (вычисленными) значениями деталей —
    /// аналог повторного заполнения существующего CrashReport.
    pub fn clone_resolved(&self) -> CrashReport {
        let mut categories = BTreeMap::new();
        for (name, cat) in &self.categories {
            let mut copy = CrashReportCategory { name: cat.name.clone(), details: Vec::new() };
            for (dname, supplier) in &cat.details {
                let value = supplier();
                copy.details.push((dname.clone(), Box::new(move || value.clone()) as DetailSupplier));
            }
            categories.insert(name.clone(), copy);
        }
        CrashReport {
            title: self.title.clone(),
            description: self.description.clone(),
            exception: self.exception.clone(),
            suppressed: self.suppressed.clone(),
            categories,
        }
    }

    /// `getTitle`.
    pub fn get_title(&self) -> &str {
        &self.title
    }
}

impl CrashReportSink for CrashReport {
    fn add_category(&mut self, name: &str, populate: &mut dyn FnMut(&mut dyn CrashReportCategorySink)) {
        let mut cat = CrashReportCategory { name: name.to_string(), ..Default::default() };
        populate(&mut cat);
        self.categories.insert(name.to_string(), cat);
    }
}

impl fmt::Display for CrashReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "---- {} ----", self.title)?;
        if let Some(desc) = &self.description {
            writeln!(f, "Description: {}", desc)?;
        }
        if let Some(exc) = &self.exception {
            writeln!(f, "Exception: {}", exc)?;
        }
        for cat in self.categories.values() {
            writeln!(f, "\n-- {} --", cat.name)?;
            for (name, supplier) in &cat.details {
                writeln!(f, "\t{}: {}", name, supplier())?;
            }
        }
        Ok(())
    }
}
