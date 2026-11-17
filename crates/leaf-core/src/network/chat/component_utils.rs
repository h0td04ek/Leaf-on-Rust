//! Перенос `net.minecraft.network.chat.ComponentUtils`
//! (форматирование списков; entity-aware части — с портом commands/entity).

use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::chat::component::Component;
use crate::network::chat::style::Style;

/// `DEFAULT_SEPARATOR_TEXT`.
pub const DEFAULT_SEPARATOR_TEXT: &str = ", ";

/// `DEFAULT_SEPARATOR` — `", "` серым.
pub fn default_separator() -> Component {
    let mut c = Component::literal(", ");
    c.with_style(Style::empty().with_color_formatting(ChatFormatting::Gray));
    c
}

/// `DEFAULT_NO_STYLE_SEPARATOR`.
pub fn default_no_style_separator() -> Component {
    Component::literal(", ")
}

/// `mergeStyles(Component, Style)` — объединяет стиль компонента с данным.
pub fn merge_styles(component: &Component, style: Style) -> Component {
    if style.is_empty() {
        return component.clone();
    }
    let component_style = &component.style;
    if component_style.is_empty() {
        let mut merged = component.copy();
        merged.style = style;
        return merged;
    }
    if *component_style == style {
        return component.clone();
    }
    let mut merged = component.copy();
    merged.style = component_style.apply_to(&style);
    merged
}

/// `formatList(Collection<Component>, Component)` — join с разделителем.
pub fn format_list<I>(elements: I, separator: Component) -> Component
where
    I: IntoIterator<Item = Component>,
{
    format_list_map(elements, separator, |c| c.clone())
}

/// `formatList(elements, separator, extractor)`.
pub fn format_list_map<I, T, F>(elements: I, separator: Component, extractor: F) -> Component
where
    I: IntoIterator<Item = T>,
    F: Fn(&T) -> Component,
{
    let items: Vec<T> = elements.into_iter().collect();
    if items.is_empty() {
        return Component::empty();
    }
    if items.len() == 1 {
        return extractor(&items[0]);
    }
    let mut result = Component::empty();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.append(separator.clone());
        }
        result.append(extractor(item));
    }
    result
}

/// `wrapInSquareBrackets(Component)`.
pub fn wrap_in_square_brackets(to_wrap: Component) -> Component {
    Component::translatable("chat.square_brackets", vec![
        crate::network::chat::component_contents::TranslatableArg::Component(Box::new(to_wrap)),
    ])
}
