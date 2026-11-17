//! Перенос `net.minecraft.world.scores.DisplaySlot`.

/// `DisplaySlot` — слоты отображения счёта.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DisplaySlot {
    /// `LIST`.
    List = 0,
    /// `SIDEBAR`.
    Sidebar = 1,
    /// `BELOW_NAME`.
    BelowName = 2,
    /// `TEAM_BLACK`.
    TeamBlack = 3,
    /// `TEAM_DARK_BLUE`.
    TeamDarkBlue = 4,
    /// `TEAM_DARK_GREEN`.
    TeamDarkGreen = 5,
    /// `TEAM_DARK_AQUA`.
    TeamDarkAqua = 6,
    /// `TEAM_DARK_RED`.
    TeamDarkRed = 7,
    /// `TEAM_DARK_PURPLE`.
    TeamDarkPurple = 8,
    /// `TEAM_GOLD`.
    TeamGold = 9,
    /// `TEAM_GRAY`.
    TeamGray = 10,
    /// `TEAM_DARK_GRAY`.
    TeamDarkGray = 11,
    /// `TEAM_BLUE`.
    TeamBlue = 12,
    /// `TEAM_GREEN`.
    TeamGreen = 13,
    /// `TEAM_AQUA`.
    TeamAqua = 14,
    /// `TEAM_RED`.
    TeamRed = 15,
    /// `TEAM_LIGHT_PURPLE`.
    TeamLightPurple = 16,
    /// `TEAM_YELLOW`.
    TeamYellow = 17,
    /// `TEAM_WHITE`.
    TeamWhite = 18,
}

/// Все значения (`values()`).
pub const DISPLAY_SLOTS: [DisplaySlot; 19] = [
    DisplaySlot::List,
    DisplaySlot::Sidebar,
    DisplaySlot::BelowName,
    DisplaySlot::TeamBlack,
    DisplaySlot::TeamDarkBlue,
    DisplaySlot::TeamDarkGreen,
    DisplaySlot::TeamDarkAqua,
    DisplaySlot::TeamDarkRed,
    DisplaySlot::TeamDarkPurple,
    DisplaySlot::TeamGold,
    DisplaySlot::TeamGray,
    DisplaySlot::TeamDarkGray,
    DisplaySlot::TeamBlue,
    DisplaySlot::TeamGreen,
    DisplaySlot::TeamAqua,
    DisplaySlot::TeamRed,
    DisplaySlot::TeamLightPurple,
    DisplaySlot::TeamYellow,
    DisplaySlot::TeamWhite,
];

impl DisplaySlot {
    /// `values()`.
    pub fn values() -> &'static [DisplaySlot] {
        &DISPLAY_SLOTS
    }

    /// `id()`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `getSerializedName()` (`StringRepresentable`).
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            DisplaySlot::List => "list",
            DisplaySlot::Sidebar => "sidebar",
            DisplaySlot::BelowName => "below_name",
            DisplaySlot::TeamBlack => "sidebar.team.black",
            DisplaySlot::TeamDarkBlue => "sidebar.team.dark_blue",
            DisplaySlot::TeamDarkGreen => "sidebar.team.dark_green",
            DisplaySlot::TeamDarkAqua => "sidebar.team.dark_aqua",
            DisplaySlot::TeamDarkRed => "sidebar.team.dark_red",
            DisplaySlot::TeamDarkPurple => "sidebar.team.dark_purple",
            DisplaySlot::TeamGold => "sidebar.team.gold",
            DisplaySlot::TeamGray => "sidebar.team.gray",
            DisplaySlot::TeamDarkGray => "sidebar.team.dark_gray",
            DisplaySlot::TeamBlue => "sidebar.team.blue",
            DisplaySlot::TeamGreen => "sidebar.team.green",
            DisplaySlot::TeamAqua => "sidebar.team.aqua",
            DisplaySlot::TeamRed => "sidebar.team.red",
            DisplaySlot::TeamLightPurple => "sidebar.team.light_purple",
            DisplaySlot::TeamYellow => "sidebar.team.yellow",
            DisplaySlot::TeamWhite => "sidebar.team.white",
        }
    }

    /// `BY_ID` — `ByIdMap.continuous` + `OutOfBoundsStrategy.ZERO`:
    /// вне диапазона → первый элемент.
    pub fn by_id(id: i32) -> DisplaySlot {
        if (0..DISPLAY_SLOTS.len() as i32).contains(&id) {
            DISPLAY_SLOTS[id as usize]
        } else {
            DISPLAY_SLOTS[0]
        }
    }
}
