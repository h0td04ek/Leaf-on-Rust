//! Аргументы Minecraft, не требующие доступа к конкретному миру.

use crate::core::block_pos::BlockPos;
use crate::resources::Identifier;
use crate::util::{mth, uuid::Uuid};
use crate::world::level::game_type::GameType;
use crate::world::phys::vec3::Vec3;

use crate::command::brigadier_argument::{ArgumentType, ArgumentValue};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCoordinate {
    pub relative: bool,
    pub value: f64,
}

impl WorldCoordinate {
    const RELATIVE_ZERO: Self = Self {
        relative: true,
        value: 0.0,
    };

    pub fn resolve(self, base: f64) -> f64 {
        if self.relative {
            self.value + base
        } else {
            self.value
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coordinates {
    World([WorldCoordinate; 3]),
    Local { left: f64, up: f64, forward: f64 },
}

impl Coordinates {
    /// Положение источника для '~' и положение его anchor для '^'.
    pub fn resolve(self, origin: Vec3, anchor: Vec3, yaw: f32, pitch: f32) -> Vec3 {
        match self {
            Self::World([x, y, z]) => Vec3::new(
                x.resolve(origin.x),
                y.resolve(origin.y),
                z.resolve(origin.z),
            ),
            Self::Local { left, up, forward } => {
                let yaw = (yaw + 90.0).to_radians();
                let pitch = -pitch;
                let (yaw_sin, yaw_cos) = (mth::sin(f64::from(yaw)), mth::cos(f64::from(yaw)));
                let make_axis = |angle: f32| {
                    let radians = f64::from(angle.to_radians());
                    let (sin, cos) = (mth::sin(radians), mth::cos(radians));
                    Vec3::new(
                        f64::from(yaw_cos * cos),
                        f64::from(sin),
                        f64::from(yaw_sin * cos),
                    )
                };
                let forward_axis = make_axis(pitch);
                let up_axis = make_axis(pitch + 90.0);
                let left_axis = forward_axis.cross(&up_axis).scale(-1.0);
                anchor
                    .add_vec(&forward_axis.scale(forward))
                    .add_vec(&up_axis.scale(up))
                    .add_vec(&left_axis.scale(left))
            }
        }
    }

    pub fn block_pos(self, origin: Vec3, anchor: Vec3, yaw: f32, pitch: f32) -> BlockPos {
        BlockPos::containing_vec3(&self.resolve(origin, anchor, yaw, pitch))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandRotation {
    pub yaw: WorldCoordinate,
    pub pitch: WorldCoordinate,
}

impl CommandRotation {
    pub fn resolve(self, yaw: f32, pitch: f32) -> (f32, f32) {
        (
            self.yaw.resolve(f64::from(yaw)) as f32,
            self.pitch.resolve(f64::from(pitch)) as f32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandAngle {
    pub relative: bool,
    pub value: f32,
}

impl CommandAngle {
    pub fn resolve(self, yaw: f32) -> f32 {
        mth::wrap_degrees_f32(if self.relative {
            yaw + self.value
        } else {
            self.value
        })
    }
}

pub(crate) fn parse(ty: ArgumentType, input: &str) -> Option<(ArgumentValue, usize)> {
    match ty {
        ArgumentType::Identifier | ArgumentType::Dimension => {
            let end = input
                .bytes()
                .take_while(|&b| {
                    b.is_ascii_lowercase()
                        || b.is_ascii_digit()
                        || matches!(b, b'_' | b':' | b'/' | b'.' | b'-')
                })
                .count();
            if end == 0 {
                return None;
            };
            let id = Identifier::parse(&input[..end]).ok()?;
            Some((ArgumentValue::Identifier(id), end))
        }
        ArgumentType::Uuid => {
            let end = input
                .bytes()
                .take_while(|&b| b.is_ascii_hexdigit() || b == b'-')
                .count();
            let id = Uuid::from_string(&input[..end])?;
            Some((ArgumentValue::Uuid(id), end))
        }
        ArgumentType::GameMode => {
            let token = input.split(' ').next()?;
            let mode = GameType::values()
                .iter()
                .copied()
                .find(|mode| mode.get_name() == token)?;
            Some((ArgumentValue::GameMode(mode), token.len()))
        }
        ArgumentType::Time { min } => {
            let end = number_end(input);
            let value = input[..end].parse::<f32>().ok()?;
            let unit = input[end..].split(' ').next()?;
            let multiplier = match unit {
                "d" => 24_000.0,
                "s" => 20.0,
                "" | "t" => 1.0,
                _ => return None,
            };
            let ticks = (f64::from(value * multiplier) + 0.5).floor() as i32;
            (ticks >= min).then_some((ArgumentValue::Integer(ticks), end + unit.len()))
        }
        ArgumentType::Angle => {
            let (coordinate, end) = world_coordinate(input, false, false)?;
            let value = coordinate.value as f32;
            if !value.is_finite() {
                return None;
            };
            Some((
                ArgumentValue::Angle(CommandAngle {
                    relative: coordinate.relative,
                    value,
                }),
                end,
            ))
        }
        ArgumentType::Rotation => {
            let (yaw, first) = world_coordinate(input, false, false)?;
            let (pitch, end) = world_coordinate(input[first..].strip_prefix(' ')?, false, false)?;
            Some((
                ArgumentValue::Rotation(CommandRotation { yaw, pitch }),
                first + 1 + end,
            ))
        }
        ArgumentType::BlockPos | ArgumentType::Vec3 => parse_position(ty, input),
        ArgumentType::ColumnPos | ArgumentType::Vec2 => parse_column(ty, input),
        _ => None,
    }
}

fn parse_position(ty: ArgumentType, input: &str) -> Option<(ArgumentValue, usize)> {
    let (pos, end) = if input.starts_with('^') {
        let (left, x) = local_coordinate(input)?;
        let (up, y) = local_coordinate(input[x..].strip_prefix(' ')?)?;
        let cursor = x + 1 + y;
        let (forward, z) = local_coordinate(input[cursor..].strip_prefix(' ')?)?;
        (Coordinates::Local { left, up, forward }, cursor + 1 + z)
    } else {
        let integers = ty == ArgumentType::BlockPos;
        let (x, x_end) = world_coordinate(input, integers, !integers)?;
        let (y, y_end) = world_coordinate(input[x_end..].strip_prefix(' ')?, integers, false)?;
        let cursor = x_end + 1 + y_end;
        let (z, z_end) = world_coordinate(input[cursor..].strip_prefix(' ')?, integers, !integers)?;
        (Coordinates::World([x, y, z]), cursor + 1 + z_end)
    };
    Some((ArgumentValue::Coordinates(pos), end))
}

fn parse_column(ty: ArgumentType, input: &str) -> Option<(ArgumentValue, usize)> {
    let integers = ty == ArgumentType::ColumnPos;
    let (x, first) = world_coordinate(input, integers, !integers)?;
    let (z, end) = world_coordinate(input[first..].strip_prefix(' ')?, integers, !integers)?;
    let pos = Coordinates::World([x, WorldCoordinate::RELATIVE_ZERO, z]);
    Some((ArgumentValue::Coordinates(pos), first + 1 + end))
}

fn world_coordinate(input: &str, integer: bool, center: bool) -> Option<(WorldCoordinate, usize)> {
    if input.is_empty() || input.starts_with('^') {
        return None;
    }
    let relative = input.starts_with('~');
    let marker = usize::from(relative);
    let remaining = &input[marker..];
    let end = number_end(remaining);
    let token = &remaining[..end];
    let mut value = if remaining.is_empty() || remaining.starts_with(' ') {
        0.0
    } else if integer && !relative {
        f64::from(token.parse::<i32>().ok()?)
    } else {
        token.parse::<f64>().ok()?
    };
    if center && !relative && !token.contains('.') {
        value += 0.5;
    }
    value
        .is_finite()
        .then_some((WorldCoordinate { relative, value }, marker + end))
}

fn local_coordinate(input: &str) -> Option<(f64, usize)> {
    let remaining = input.strip_prefix('^')?;
    let end = number_end(remaining);
    let value = if remaining.is_empty() || remaining.starts_with(' ') {
        0.0
    } else {
        remaining[..end].parse::<f64>().ok()?
    };
    value.is_finite().then_some((value, end + 1))
}

pub(crate) fn number_end(input: &str) -> usize {
    input
        .bytes()
        .take_while(|&c| c.is_ascii_digit() || matches!(c, b'.' | b'-'))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier::{CommandDispatcher, CommandNode};
    use crate::network::friendly_byte_buf::FriendlyByteBuf;

    #[test]
    fn resolves_world_relative_and_local_coordinates() {
        let origin = Vec3::new(4.0, 60.0, -3.0);
        let anchor = Vec3::new(4.0, 62.0, -3.0);
        let (ArgumentValue::Coordinates(world), end) =
            ArgumentType::Vec3.parse("1 ~1 -2 next", 0).unwrap()
        else {
            panic!("expected world coordinates")
        };
        assert_eq!(end, 7);
        assert_eq!(
            world.resolve(origin, anchor, 0.0, 0.0),
            Vec3::new(1.5, 61.0, -1.5)
        );
        assert!(ArgumentType::BlockPos.parse("0.5 0 0", 0).is_none());
        assert!(ArgumentType::BlockPos.parse("^ ~ ^", 0).is_none());

        let (ArgumentValue::Coordinates(local), _) =
            ArgumentType::Vec3.parse("^1 ^2 ^3", 0).unwrap()
        else {
            panic!("expected local coordinates")
        };
        let result = local.resolve(origin, anchor, 0.0, 0.0);
        assert!((result.x - 5.0).abs() < 1e-5);
        assert!((result.y - 64.0).abs() < 1e-5);
        assert!(result.z.abs() < 1e-5);

        let (ArgumentValue::Rotation(rotation), _) =
            ArgumentType::Rotation.parse("~-10 25", 0).unwrap()
        else {
            panic!("expected rotation")
        };
        assert_eq!(rotation.resolve(180.0, 30.0), (170.0, 25.0));
    }

    #[test]
    fn validates_time_and_names_and_writes_12111_argument_templates() {
        assert_eq!(
            ArgumentType::Time { min: 1 }.parse("1.25s", 0),
            Some((ArgumentValue::Integer(25), 5))
        );
        assert!(ArgumentType::Time { min: 1 }.parse("0.01s", 0).is_none());
        assert!(ArgumentType::Time { min: 1 }.parse("25m", 0).is_none());
        assert!(ArgumentType::GameMode.parse("sandbox", 0).is_none());
        assert_eq!(
            ArgumentType::Identifier.parse("fixture:some/path", 0),
            Some((
                ArgumentValue::Identifier(Identifier::parse("fixture:some/path").unwrap()),
                17
            )),
        );
        let cases = [
            (8, ArgumentType::BlockPos),
            (10, ArgumentType::Vec3),
            (36, ArgumentType::Identifier),
            (42, ArgumentType::GameMode),
            (43, ArgumentType::Time { min: 25 }),
            (56, ArgumentType::Uuid),
        ];
        for (id, ty) in cases {
            assert_eq!(ty.registry_id(), id);
            let mut buffer = FriendlyByteBuf::new();
            ty.write_template(&mut buffer);
            assert_eq!(ArgumentType::read_template(id, &mut buffer), Some(ty));
        }
        let mut buffer = FriendlyByteBuf::new();
        ArgumentType::Time { min: 25 }.write_template(&mut buffer);
        assert_eq!(buffer.readable_bytes_vec(), [0, 0, 0, 25]);

        let mut dispatcher = CommandDispatcher::<()>::new();
        let parent = dispatcher
            .add_node(dispatcher.root(), CommandNode::literal("time"))
            .unwrap();
        dispatcher
            .add_node(
                parent,
                CommandNode::argument("ticks", ArgumentType::Time { min: 0 }),
            )
            .unwrap();
        let completed = dispatcher.complete("/time 22", 8, &());
        assert_eq!(completed.range, 8..8);
        assert_eq!(
            completed
                .entries
                .iter()
                .map(|entry| entry.text.as_str())
                .collect::<Vec<_>>(),
            ["d", "s", "t"]
        );
    }
}
