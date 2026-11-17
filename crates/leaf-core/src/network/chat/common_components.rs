//! Перенос `net.minecraft.network.chat.CommonComponents`.
//!
//! Java-поля — `static final Component`; здесь — функции (Component
//! не Sync из-за RefCell-кэшей в hover-информации).

use crate::network::chat::component::Component;
use crate::network::chat::component_contents::TranslatableArg;

fn tr(key: &str) -> Component {
    Component::translatable(key, Vec::new())
}

fn tr1(key: &str, arg: impl Into<TranslatableArg>) -> Component {
    Component::translatable(key, vec![arg.into()])
}

/// `EMPTY`.
pub fn empty() -> Component {
    Component::empty()
}

/// `OPTION_ON`.
pub fn option_on() -> Component {
    tr("options.on")
}

/// `OPTION_OFF`.
pub fn option_off() -> Component {
    tr("options.off")
}

/// `GUI_DONE`.
pub fn gui_done() -> Component {
    tr("gui.done")
}

/// `GUI_CANCEL`.
pub fn gui_cancel() -> Component {
    tr("gui.cancel")
}

/// `GUI_YES`.
pub fn gui_yes() -> Component {
    tr("gui.yes")
}

/// `GUI_NO`.
pub fn gui_no() -> Component {
    tr("gui.no")
}

/// `GUI_OK`.
pub fn gui_ok() -> Component {
    tr("gui.ok")
}

/// `GUI_PROCEED`.
pub fn gui_proceed() -> Component {
    tr("gui.proceed")
}

/// `GUI_CONTINUE`.
pub fn gui_continue() -> Component {
    tr("gui.continue")
}

/// `GUI_BACK`.
pub fn gui_back() -> Component {
    tr("gui.back")
}

/// `GUI_TO_TITLE`.
pub fn gui_to_title() -> Component {
    tr("gui.toTitle")
}

/// `GUI_ACKNOWLEDGE`.
pub fn gui_acknowledge() -> Component {
    tr("gui.acknowledge")
}

/// `GUI_OPEN_IN_BROWSER`.
pub fn gui_open_in_browser() -> Component {
    tr("chat.link.open")
}

/// `GUI_COPY_TO_CLIPBOARD`.
pub fn gui_copy_to_clipboard() -> Component {
    tr("chat.copy")
}

/// `GUI_COPY_LINK_TO_CLIPBOARD`.
pub fn gui_copy_link_to_clipboard() -> Component {
    tr("gui.copy_link_to_clipboard")
}

/// `GUI_DISCONNECT`.
pub fn gui_disconnect() -> Component {
    tr("menu.disconnect")
}

/// `GUI_RETURN_TO_MENU`.
pub fn gui_return_to_menu() -> Component {
    tr("menu.returnToMenu")
}

/// `TRANSFER_CONNECT_FAILED`.
pub fn transfer_connect_failed() -> Component {
    tr("connect.failed.transfer")
}

/// `CONNECT_FAILED`.
pub fn connect_failed() -> Component {
    tr("connect.failed")
}

/// `NEW_LINE`.
pub fn new_line() -> Component {
    Component::literal("\n")
}

/// `NARRATION_SEPARATOR`.
pub fn narration_separator() -> Component {
    Component::literal(". ")
}

/// `ELLIPSIS`.
pub fn ellipsis() -> Component {
    Component::literal("...")
}

/// `SPACE`.
pub fn space() -> Component {
    Component::literal(" ")
}

/// `days(long)`.
pub fn days(days: i64) -> Component {
    tr1("gui.days", TranslatableArg::Text(days.to_string()))
}

/// `hours(long)`.
pub fn hours(hours: i64) -> Component {
    tr1("gui.hours", TranslatableArg::Text(hours.to_string()))
}

/// `minutes(long)`.
pub fn minutes(minutes: i64) -> Component {
    tr1("gui.minutes", TranslatableArg::Text(minutes.to_string()))
}

/// `optionStatus(boolean)`.
pub fn option_status(is_enabled: bool) -> Component {
    if is_enabled { option_on() } else { option_off() }
}

/// `disconnectButtonLabel(boolean)`.
pub fn disconnect_button_label(is_single_player: bool) -> Component {
    if is_single_player { gui_return_to_menu() } else { gui_disconnect() }
}

/// `optionStatus(Component, boolean)`.
pub fn option_status_composed(message: Component, composed: bool) -> Component {
    Component::translatable(
        if composed { "options.on.composed" } else { "options.off.composed" },
        vec![TranslatableArg::Component(Box::new(message))],
    )
}

/// `optionNameValue(Component, Component)`.
pub fn option_name_value(caption: Component, value_message: Component) -> Component {
    Component::translatable(
        "options.generic_value",
        vec![
            TranslatableArg::Component(Box::new(caption)),
            TranslatableArg::Component(Box::new(value_message)),
        ],
    )
}

/// `joinForNarration(Component...)`.
pub fn join_for_narration(components: &[Component]) -> Component {
    let mut result = Component::empty();
    for (i, c) in components.iter().enumerate() {
        result.append(c.clone());
        if i != components.len() - 1 {
            result.append(narration_separator());
        }
    }
    result
}

/// `joinLines(Component...)` — `ComponentUtils.formatList(lines, NEW_LINE)`.
pub fn join_lines(lines: &[Component]) -> Component {
    crate::network::chat::component_utils::format_list(lines.iter().cloned(), new_line())
}
