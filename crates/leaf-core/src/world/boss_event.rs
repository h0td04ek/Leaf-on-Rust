//! Перенос `net.minecraft.world.BossEvent` — босс-бары.
//! Вложенные enum'ы `BossBarColor`/`BossBarOverlay` и данные события.

use crate::network::chat::component::Component;
use crate::network::chat::chat_formatting::ChatFormatting;
use crate::util::uuid::Uuid;

/// `BossEvent.BossBarColor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BossBarColor {
    /// `PINK`.
    Pink,
    /// `BLUE`.
    Blue,
    /// `RED`.
    Red,
    /// `GREEN`.
    Green,
    /// `YELLOW`.
    Yellow,
    /// `PURPLE`.
    Purple,
    /// `WHITE`.
    White,
}

/// Все значения (`values()`).
pub const BOSS_BAR_COLORS: [BossBarColor; 7] = [
    BossBarColor::Pink,
    BossBarColor::Blue,
    BossBarColor::Red,
    BossBarColor::Green,
    BossBarColor::Yellow,
    BossBarColor::Purple,
    BossBarColor::White,
];

impl BossBarColor {
    /// `values()`.
    pub fn values() -> &'static [BossBarColor] {
        &BOSS_BAR_COLORS
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            BossBarColor::Pink => "pink",
            BossBarColor::Blue => "blue",
            BossBarColor::Red => "red",
            BossBarColor::Green => "green",
            BossBarColor::Yellow => "yellow",
            BossBarColor::Purple => "purple",
            BossBarColor::White => "white",
        }
    }

    /// `getSerializedName` (`StringRepresentable`).
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    /// `getFormatting`.
    pub fn get_formatting(&self) -> ChatFormatting {
        match self {
            BossBarColor::Pink => ChatFormatting::Red,
            BossBarColor::Blue => ChatFormatting::Blue,
            BossBarColor::Red => ChatFormatting::DarkRed,
            BossBarColor::Green => ChatFormatting::Green,
            BossBarColor::Yellow => ChatFormatting::Yellow,
            BossBarColor::Purple => ChatFormatting::DarkBlue,
            BossBarColor::White => ChatFormatting::White,
        }
    }
}

/// `BossEvent.BossBarOverlay`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BossBarOverlay {
    /// `PROGRESS`.
    Progress,
    /// `NOTCHED_6`.
    Notched6,
    /// `NOTCHED_10`.
    Notched10,
    /// `NOTCHED_12`.
    Notched12,
    /// `NOTCHED_20`.
    Notched20,
}

/// Все значения (`values()`).
pub const BOSS_BAR_OVERLAYS: [BossBarOverlay; 5] = [
    BossBarOverlay::Progress,
    BossBarOverlay::Notched6,
    BossBarOverlay::Notched10,
    BossBarOverlay::Notched12,
    BossBarOverlay::Notched20,
];

impl BossBarOverlay {
    /// `values()`.
    pub fn values() -> &'static [BossBarOverlay] {
        &BOSS_BAR_OVERLAYS
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            BossBarOverlay::Progress => "progress",
            BossBarOverlay::Notched6 => "notched_6",
            BossBarOverlay::Notched10 => "notched_10",
            BossBarOverlay::Notched12 => "notched_12",
            BossBarOverlay::Notched20 => "notched_20",
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }
}

/// `BossEvent` — данные босс-бара.
///
/// Набор игроков (`players: Set<ServerPlayer>`) и listener'ы
/// (`BossEventListener`) относятся к серверной логике и будут
/// портированы вместе с `ServerPlayer`/`BossEvent` listeners.
#[derive(Clone, Debug)]
pub struct BossEvent {
    /// `id` — UUID события.
    pub id: Uuid,
    /// `name` — отображаемое имя.
    pub name: Component,
    /// `progress` — прогресс 0.0..=1.0.
    pub progress: f32,
    /// `color`.
    pub color: BossBarColor,
    /// `overlay`.
    pub overlay: BossBarOverlay,
    /// `darkenScreen`.
    pub darken_screen: bool,
    /// `playMusic`.
    pub play_music: bool,
    /// `createWorldFog`.
    pub create_world_fog: bool,
}

impl BossEvent {
    /// `BossEvent::new`.
    pub fn new(id: Uuid, name: Component, color: BossBarColor, overlay: BossBarOverlay) -> Self {
        Self {
            id,
            name,
            progress: 1.0,
            color,
            overlay,
            darken_screen: true,
            play_music: true,
            create_world_fog: true,
        }
    }

    /// `getId`.
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    /// `getName`.
    pub fn get_name(&self) -> &Component {
        &self.name
    }

    /// `setName`.
    pub fn set_name(&mut self, name: Component) {
        self.name = name;
    }

    /// `getProgress`.
    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    /// `setProgress` — клампится в 0.0..=1.0.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// `getColor`.
    pub fn get_color(&self) -> BossBarColor {
        self.color
    }

    /// `setColor`.
    pub fn set_color(&mut self, color: BossBarColor) {
        self.color = color;
    }

    /// `getOverlay`.
    pub fn get_overlay(&self) -> BossBarOverlay {
        self.overlay
    }

    /// `setOverlay`.
    pub fn set_overlay(&mut self, overlay: BossBarOverlay) {
        self.overlay = overlay;
    }

    /// `shouldDarkenScreen`.
    pub fn should_darken_screen(&self) -> bool {
        self.darken_screen
    }

    /// `setDarkenScreen`.
    pub fn set_darken_screen(&mut self, darken: bool) {
        self.darken_screen = darken;
    }

    /// `shouldPlayBossMusic`.
    pub fn should_play_boss_music(&self) -> bool {
        self.play_music
    }

    /// `setPlayBossMusic`.
    pub fn set_play_boss_music(&mut self, play: bool) {
        self.play_music = play;
    }

    /// `shouldCreateWorldFog`.
    pub fn should_create_world_fog(&self) -> bool {
        self.create_world_fog
    }

    /// `setCreateWorldFog`.
    pub fn set_create_world_fog(&mut self, create: bool) {
        self.create_world_fog = create;
    }
}
