//! Перенос `net.minecraft.network.chat.Component` + `MutableComponent`.
//!
//! В Java `Component` — интерфейс, `MutableComponent` — единственная
//! реализация. Здесь `Component` — конкретная структура с мутабельными
//! методами (`append`, `set_style`, ...), покрывающими MutableComponent.

use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::chat::component_contents::{
    BlockDataSource, ComponentContents, DataSource, EntityDataSource, ObjectInfo,
    SelectorPattern, StorageDataSource, TranslatableArg, TranslatableContents,
};
use crate::network::chat::formatted_text::FormattedText;
use crate::network::chat::style::Style;
use crate::util::either::Either;

/// `Component` (= Java MutableComponent).
#[derive(Clone, Debug)]
pub struct Component {
    pub contents: ComponentContents,
    pub siblings: Vec<Component>,
    pub style: Style,
}

impl Component {
    /// `MutableComponent.create(contents)`.
    pub fn new(contents: ComponentContents) -> Component {
        Component { contents, siblings: Vec::new(), style: Style::empty() }
    }

    /// `Component.literal(text)`.
    pub fn literal(text: impl Into<String>) -> Component {
        Component::new(ComponentContents::PlainText(text.into()))
    }

    /// `Component.empty()`.
    pub fn empty() -> Component {
        Component::literal("")
    }

    /// `Component.translatable(key)`.
    pub fn translatable(key: impl Into<String>, args: Vec<TranslatableArg>) -> Component {
        Component::new(ComponentContents::Translatable(TranslatableContents::new(
            key, None, args,
        )))
    }

    /// `Component.translatable(key, fallback, args)`.
    pub fn translatable_with_fallback(
        key: impl Into<String>,
        fallback: Option<String>,
        args: Vec<TranslatableArg>,
    ) -> Component {
        Component::new(ComponentContents::Translatable(TranslatableContents::new(
            key, fallback, args,
        )))
    }

    /// `Component.translatableEscape` — не-компонентные аргументы
    /// приводятся к строке.
    pub fn translatable_escape(key: impl Into<String>, args: Vec<TranslatableArg>) -> Component {
        Component::translatable(key, args)
    }

    /// `Component.keybind(name)`.
    pub fn keybind(name: impl Into<String>) -> Component {
        Component::new(ComponentContents::Keybind(name.into()))
    }

    /// `Component.nbt(pathPattern, interpreting, separator, dataSource)`.
    pub fn nbt(
        path_pattern: impl Into<String>,
        interpreting: bool,
        separator: Option<Component>,
        data_source: DataSource,
    ) -> Component {
        Component::new(ComponentContents::Nbt {
            nbt_path_pattern: path_pattern.into(),
            interpreting,
            separator: separator.map(Box::new),
            data_source,
        })
    }

    /// `Component.nbt` с block-источником.
    pub fn nbt_block(
        path_pattern: impl Into<String>,
        interpreting: bool,
        separator: Option<Component>,
        pos_pattern: impl Into<String>,
    ) -> Component {
        Component::nbt(
            path_pattern,
            interpreting,
            separator,
            DataSource::Block(BlockDataSource::new(pos_pattern)),
        )
    }

    /// `Component.nbt` с storage-источником.
    pub fn nbt_storage(
        path_pattern: impl Into<String>,
        interpreting: bool,
        separator: Option<Component>,
        id: crate::resources::identifier::Identifier,
    ) -> Component {
        Component::nbt(
            path_pattern,
            interpreting,
            separator,
            DataSource::Storage(StorageDataSource { id }),
        )
    }

    /// `Component.nbt` с entity-источником.
    pub fn nbt_entity(
        path_pattern: impl Into<String>,
        interpreting: bool,
        separator: Option<Component>,
        selector_pattern: impl Into<String>,
    ) -> Component {
        Component::nbt(
            path_pattern,
            interpreting,
            separator,
            DataSource::Entity(EntityDataSource { selector_pattern: selector_pattern.into() }),
        )
    }

    /// `Component.score(SelectorPattern, objective)`.
    pub fn score_selector(selector: SelectorPattern, objective: impl Into<String>) -> Component {
        Component::new(ComponentContents::Score {
            name: Either::Left(selector),
            objective: objective.into(),
        })
    }

    /// `Component.score(name, objective)`.
    pub fn score_name(name: impl Into<String>, objective: impl Into<String>) -> Component {
        Component::new(ComponentContents::Score {
            name: Either::Right(name.into()),
            objective: objective.into(),
        })
    }

    /// `Component.selector(selectorPattern, separator)`.
    pub fn selector(selector: SelectorPattern, separator: Option<Component>) -> Component {
        Component::new(ComponentContents::Selector {
            selector,
            separator: separator.map(Box::new),
        })
    }

    /// `Component.object(objectInfo)`.
    pub fn object(info: ObjectInfo) -> Component {
        Component::new(ComponentContents::Object(info))
    }

    /// `Component.nullToEmpty(text)`.
    pub fn null_to_empty(text: Option<&str>) -> Component {
        match text {
            Some(t) => Component::literal(t),
            None => Component::empty(),
        }
    }

    /// `translationArg`-хелперы.
    pub fn translation_arg_string(value: impl std::fmt::Display) -> Component {
        Component::literal(value.to_string())
    }
    pub fn translation_arg_uuid(value: crate::util::uuid::Uuid) -> Component {
        Component::literal(value.to_string())
    }

    // ------------------------------------------------------------ доступ к полям

    /// `getContents`.
    pub fn get_contents(&self) -> &ComponentContents {
        &self.contents
    }

    /// `getSiblings`.
    pub fn get_siblings(&self) -> &[Component] {
        &self.siblings
    }

    /// `getStyle`.
    pub fn get_style(&self) -> &Style {
        &self.style
    }

    /// `getString()` — весь текст компонента.
    pub fn get_string(&self) -> String {
        let mut out = String::new();
        self.visit_content(&mut |s| {
            out.push_str(s);
            true
        });
        out
    }

    /// `getString(maxLength)`.
    pub fn get_string_limited(&self, max_length: usize) -> String {
        let mut out = String::new();
        self.visit_content(&mut |s| {
            let remaining = max_length.saturating_sub(out.len());
            if remaining == 0 {
                return false;
            }
            if s.chars().count() <= remaining {
                out.push_str(s);
            } else {
                out.extend(s.chars().take(remaining));
            }
            remaining > 0
        });
        out
    }

    /// `visit(ContentConsumer)`.
    pub fn visit_content<F>(&self, acceptor: &mut F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        if !self.contents.visit_content(acceptor) {
            return false;
        }
        for sibling in &self.siblings {
            if !sibling.visit_content(acceptor) {
                return false;
            }
        }
        true
    }

    /// `visit(StyledContentConsumer, style)`.
    /// `base_style` — внешний стиль.
    pub fn visit_styled<F>(&self, base_style: &Style, acceptor: &mut F) -> bool
    where
        F: FnMut(&Style, &str) -> bool,
    {
        let style = self.style.apply_to(base_style);
        if !self.contents.visit_styled(&style, acceptor) {
            return false;
        }
        for sibling in &self.siblings {
            if !sibling.visit_styled(base_style, acceptor) {
                return false;
            }
        }
        true
    }

    /// `visit` как FormattedText.
    pub fn as_formatted_text(&self) -> FormattedText {
        FormattedText::Component(self.clone())
    }

    /// `tryCollapseToString` — plain text без стилей и siblings.
    pub fn try_collapse_to_string(&self) -> Option<String> {
        if let ComponentContents::PlainText(text) = &self.contents {
            if self.siblings.is_empty() && self.style.is_empty() {
                return Some(text.clone());
            }
        }
        None
    }

    /// `plainCopy` — копия с тем же contents, без siblings и стиля.
    pub fn plain_copy(&self) -> Component {
        Component::new(self.contents.clone())
    }

    /// `copy` — полная копия.
    pub fn copy(&self) -> Component {
        self.clone()
    }

    /// `toFlatList` — развёртка в список plain-компонентов.
    pub fn to_flat_list(&self) -> Vec<Component> {
        self.to_flat_list_styled(&Style::empty())
    }

    pub fn to_flat_list_styled(&self, base_style: &Style) -> Vec<Component> {
        let mut list = Vec::new();
        self.visit_styled(base_style, &mut |style, content| {
            if !content.is_empty() {
                let mut c = Component::literal(content);
                c.style = style.clone();
                list.push(c);
            }
            true
        });
        list
    }

    /// `contains(other)` — вхождение плоского списка.
    pub fn contains(&self, other: &Component) -> bool {
        if self == other {
            return true;
        }
        let list = self.to_flat_list();
        let list1 = other.to_flat_list_styled(&self.style);
        // indexOfSubList
        if list1.is_empty() {
            return true;
        }
        list.windows(list1.len()).any(|w| w == list1.as_slice())
    }

    // ------------------------------------------------------------ мутабельные методы (MutableComponent)

    /// `setStyle(Style)`.
    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    /// `append(String)` — добавляет literal-компонент (пустая строка игнорируется).
    pub fn append_str(&mut self, text: &str) -> &mut Self {
        if !text.is_empty() {
            self.siblings.push(Component::literal(text));
        }
        self
    }

    /// `append(Component)`.
    pub fn append(&mut self, sibling: Component) -> &mut Self {
        self.siblings.push(sibling);
        self
    }

    /// `withStyle(Style)` — применяет style поверх текущего.
    pub fn with_style(&mut self, style: Style) -> &mut Self {
        self.style = style.apply_to(&self.style);
        self
    }

    /// `withStyle(UnaryOperator)`.
    pub fn with_style_fn<F: FnOnce(Style) -> Style>(&mut self, f: F) -> &mut Self {
        self.style = f(self.style.clone());
        self
    }

    /// `withStyle(ChatFormatting...)`.
    pub fn with_formats(&mut self, formats: &[ChatFormatting]) -> &mut Self {
        self.style = self.style.apply_formats(formats);
        self
    }

    /// `withStyle(ChatFormatting)`.
    pub fn with_format(&mut self, format: ChatFormatting) -> &mut Self {
        self.style = self.style.apply_format(format);
        self
    }

    /// `withColor(int)`.
    pub fn with_color(&mut self, color: u32) -> &mut Self {
        self.style = self.style.with_color_rgb(color);
        self
    }

    /// `withoutShadow`.
    pub fn without_shadow(&mut self) -> &mut Self {
        self.style = self.style.without_shadow();
        self
    }

    /// `getVisualOrderText` — пока возвращает plain-представление
    /// (StringDecomposer/Language bidi — отдельный порт).
    pub fn get_visual_order_text(&self) -> String {
        self.get_string()
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.contents == other.contents && self.style == other.style && self.siblings == other.siblings
    }
}
