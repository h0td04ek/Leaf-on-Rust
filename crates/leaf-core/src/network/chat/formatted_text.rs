//! Перенос `net.minecraft.network.chat.FormattedText`.
//!
//! `ContentConsumer`/`StyledContentConsumer` с `Optional<T>` заменены
//! на bool-колбэки (`false` = STOP_ITERATION, аналог early-return).

use crate::network::chat::component::Component;
use crate::network::chat::style::Style;

/// `FormattedText` — последовательность (style, text) кусков.
#[derive(Clone, Debug, PartialEq)]
pub enum FormattedText {
    /// `FormattedText.of(String)`.
    Plain(String),
    /// Компонент (style + contents + siblings).
    Component(Component),
    /// `FormattedText.composite(...)`.
    Composite(Vec<FormattedText>),
}

impl FormattedText {
    /// `FormattedText.EMPTY` — пустой текст.
    pub fn empty() -> FormattedText {
        FormattedText::Plain(String::new())
    }

    /// `FormattedText.of(text)`.
    pub fn of(text: impl Into<String>) -> FormattedText {
        FormattedText::Plain(text.into())
    }

    /// `FormattedText.composite(elements)`.
    pub fn composite(elements: Vec<FormattedText>) -> FormattedText {
        FormattedText::Composite(elements)
    }

    /// `getString()` — конкатенация всех кусков.
    pub fn get_string(&self) -> String {
        let mut out = String::new();
        self.visit_content(&mut |s| {
            out.push_str(s);
            true
        });
        out
    }

    /// `visit(ContentConsumer)` — обход без стилей.
    /// Возвращает `false`, если потребитель запросил остановку.
    pub fn visit_content<F>(&self, acceptor: &mut F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        match self {
            FormattedText::Plain(text) => acceptor(text),
            FormattedText::Component(c) => c.visit_content(acceptor),
            FormattedText::Composite(parts) => {
                for part in parts {
                    if !part.visit_content(acceptor) {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// `visit(StyledContentConsumer, style)` — обход со стилями.
    /// `base_style` — внешний стиль, к которому применяются стили компонентов.
    pub fn visit_styled<F>(&self, base_style: &Style, acceptor: &mut F) -> bool
    where
        F: FnMut(&Style, &str) -> bool,
    {
        match self {
            FormattedText::Plain(text) => acceptor(base_style, text),
            FormattedText::Component(c) => c.visit_styled(base_style, acceptor),
            FormattedText::Composite(parts) => {
                for part in parts {
                    if !part.visit_styled(base_style, acceptor) {
                        return false;
                    }
                }
                true
            }
        }
    }
}
