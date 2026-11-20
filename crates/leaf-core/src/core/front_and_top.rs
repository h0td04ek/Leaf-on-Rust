//! Перенос `net.minecraft.core.FrontAndTop` — направление "front"
//! (куда смотрит) + "top" (верх) для jigsaw/лестниц.

use crate::core::direction::Direction;

/// `FrontAndTop` — 12 комбинаций front/top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrontAndTop {
    /// `DOWN_EAST`.
    DownEast,
    /// `DOWN_NORTH`.
    DownNorth,
    /// `DOWN_SOUTH`.
    DownSouth,
    /// `DOWN_WEST`.
    DownWest,
    /// `UP_EAST`.
    UpEast,
    /// `UP_NORTH`.
    UpNorth,
    /// `UP_SOUTH`.
    UpSouth,
    /// `UP_WEST`.
    UpWest,
    /// `WEST_UP`.
    WestUp,
    /// `EAST_UP`.
    EastUp,
    /// `NORTH_UP`.
    NorthUp,
    /// `SOUTH_UP`.
    SouthUp,
}

impl FrontAndTop {
    /// Все значения (порядок объявления в Java).
    pub const VALUES: [FrontAndTop; 12] = [
        FrontAndTop::DownEast,
        FrontAndTop::DownNorth,
        FrontAndTop::DownSouth,
        FrontAndTop::DownWest,
        FrontAndTop::UpEast,
        FrontAndTop::UpNorth,
        FrontAndTop::UpSouth,
        FrontAndTop::UpWest,
        FrontAndTop::WestUp,
        FrontAndTop::EastUp,
        FrontAndTop::NorthUp,
        FrontAndTop::SouthUp,
    ];

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            FrontAndTop::DownEast => "down_east",
            FrontAndTop::DownNorth => "down_north",
            FrontAndTop::DownSouth => "down_south",
            FrontAndTop::DownWest => "down_west",
            FrontAndTop::UpEast => "up_east",
            FrontAndTop::UpNorth => "up_north",
            FrontAndTop::UpSouth => "up_south",
            FrontAndTop::UpWest => "up_west",
            FrontAndTop::WestUp => "west_up",
            FrontAndTop::EastUp => "east_up",
            FrontAndTop::NorthUp => "north_up",
            FrontAndTop::SouthUp => "south_up",
        }
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        Self::VALUES
            .iter()
            .copied()
            .find(|v| v.get_serialized_name() == name)
    }

    /// `fromFrontAndTop(front, top)` — lookup по таблице.
    pub fn from_front_and_top(front: Direction, top: Direction) -> Self {
        // Таблица BY_TOP_FRONT: индекс = front*6 + top.
        // Валидные комбинации — только 12 из 36; для невалидных
        // Java вернул бы null → здесь паникуем как ArrayIndexOutOfBounds.
        match (front, top) {
            (Direction::DOWN, Direction::EAST) => FrontAndTop::DownEast,
            (Direction::DOWN, Direction::NORTH) => FrontAndTop::DownNorth,
            (Direction::DOWN, Direction::SOUTH) => FrontAndTop::DownSouth,
            (Direction::DOWN, Direction::WEST) => FrontAndTop::DownWest,
            (Direction::UP, Direction::EAST) => FrontAndTop::UpEast,
            (Direction::UP, Direction::NORTH) => FrontAndTop::UpNorth,
            (Direction::UP, Direction::SOUTH) => FrontAndTop::UpSouth,
            (Direction::UP, Direction::WEST) => FrontAndTop::UpWest,
            (Direction::WEST, Direction::UP) => FrontAndTop::WestUp,
            (Direction::EAST, Direction::UP) => FrontAndTop::EastUp,
            (Direction::NORTH, Direction::UP) => FrontAndTop::NorthUp,
            (Direction::SOUTH, Direction::UP) => FrontAndTop::SouthUp,
            _ => panic!("Invalid front/top combination: {:?}/{:?}", front, top),
        }
    }

    /// `front()`.
    pub fn front(&self) -> Direction {
        match self {
            FrontAndTop::DownEast | FrontAndTop::DownNorth | FrontAndTop::DownSouth
            | FrontAndTop::DownWest => Direction::DOWN,
            FrontAndTop::UpEast | FrontAndTop::UpNorth | FrontAndTop::UpSouth
            | FrontAndTop::UpWest => Direction::UP,
            FrontAndTop::WestUp => Direction::WEST,
            FrontAndTop::EastUp => Direction::EAST,
            FrontAndTop::NorthUp => Direction::NORTH,
            FrontAndTop::SouthUp => Direction::SOUTH,
        }
    }

    /// `top()`.
    pub fn top(&self) -> Direction {
        match self {
            FrontAndTop::DownEast => Direction::EAST,
            FrontAndTop::DownNorth => Direction::NORTH,
            FrontAndTop::DownSouth => Direction::SOUTH,
            FrontAndTop::DownWest => Direction::WEST,
            FrontAndTop::UpEast => Direction::EAST,
            FrontAndTop::UpNorth => Direction::NORTH,
            FrontAndTop::UpSouth => Direction::SOUTH,
            FrontAndTop::UpWest => Direction::WEST,
            FrontAndTop::WestUp | FrontAndTop::EastUp | FrontAndTop::NorthUp
            | FrontAndTop::SouthUp => Direction::UP,
        }
    }
}

impl crate::world::level::block::state::property::PropertyValue for FrontAndTop {
    fn parse(s: &str) -> Option<Self> {
        Self::by_name(s)
    }

    fn to_state_value(&self) -> crate::world::level::block::state::property::StateValue {
        crate::world::level::block::state::property::StateValue::Str(
            self.get_serialized_name().to_string(),
        )
    }
}
