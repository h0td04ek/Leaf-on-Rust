//! Перенос jigsaw-части `JigsawBlock` — `getFrontFacing`/
//! `getTopFacing`/`canAttach` для генерации структур.
//!
//! `ORIENTATION`/`JOINT` читаются из `StateHolder` по имени
//! (как `BlockStateProperties.ORIENTATION`/`JigsawBlock.JOINT`).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::front_and_top::FrontAndTop;
use crate::world::level::block::entity::jigsaw_block_entity::JigsawJointType;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::levelgen::structure::templatesystem::structure_template::JigsawBlockInfo;
use crate::world::level::block::state::property::StateValue;

/// `JigsawBlock`-хелперы (статические методы Java-класса).
pub struct JigsawBlock;

impl JigsawBlock {
    /// `orientation(state)` → `FrontAndTop` (default NORTH_UP
    /// при отсутствии свойства — как у незарегистрированных
    /// jigsaw-состояний в шаблонах).
    fn orientation(state: &BlockState) -> FrontAndTop {
        match state.values.get_by_name("orientation") {
            Some(StateValue::Str(name)) => FrontAndTop::by_name(&name)
                .unwrap_or(FrontAndTop::NorthUp),
            _ => FrontAndTop::NorthUp,
        }
    }

    /// `jointType(state, nbt)` — `JointType` из state-свойства
    /// `joint`, fallback — nbt `joint`, затем ALIGNED.
    pub fn joint_type(info: &JigsawBlockInfo) -> JigsawJointType {
        if let Some(StateValue::Str(name)) =
            info.info.state.values.get_by_name("joint")
        {
            return JigsawJointType::by_name_or_default(&name);
        }
        if let Some(nbt) = &info.info.nbt {
            if let Some(joint) = nbt.get_string("joint") {
                return JigsawJointType::by_name_or_default(joint);
            }
        }
        JigsawJointType::ALIGNED
    }

    /// `getFrontFacing(state)`.
    pub fn get_front_facing(state: &BlockState) -> Direction {
        Self::orientation(state).front()
    }

    /// `getTopFacing(state)`.
    pub fn get_top_facing(state: &BlockState) -> Direction {
        Self::orientation(state).top()
    }

    /// `canAttach(parent, child)`.
    pub fn can_attach(parent: &JigsawBlockInfo, child: &JigsawBlockInfo) -> bool {
        let front_facing = Self::get_front_facing(&parent.info.state);
        let front_facing1 = Self::get_front_facing(&child.info.state);
        let top_facing = Self::get_top_facing(&parent.info.state);
        let top_facing1 = Self::get_top_facing(&child.info.state);
        let joint_type = Self::joint_type(parent);
        let flag = joint_type == JigsawJointType::ROLLABLE;
        front_facing == front_facing1.get_opposite()
            && (flag || top_facing == top_facing1)
            && parent.target == child.name
    }

    /// `getAttachmentFacing` — направление attachment-точки
    /// (front facing родителя).
    pub fn attachment_facing(state: &BlockState) -> Direction {
        Self::get_front_facing(state)
    }

    /// `BlockPos.relative(frontFacing)` — позиция за jigsaw-блоком.
    pub fn attachment_pos(jigsaw_pos: BlockPos, state: &BlockState) -> BlockPos {
        jigsaw_pos.relative(Self::get_front_facing(state))
    }
}
