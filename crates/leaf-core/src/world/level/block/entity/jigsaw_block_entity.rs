//! Перенос `net.minecraft.world.level.block.entity.JigsawBlockEntity.JointType`.

/// `JigsawBlockEntity.JointType`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JigsawJointType {
    /// `ROLLABLE`.
    ROLLABLE,
    /// `ALIGNED`.
    ALIGNED,
}

impl JigsawJointType {
    /// `values()`.
    pub const VALUES: [JigsawJointType; 2] = [JigsawJointType::ROLLABLE, JigsawJointType::ALIGNED];

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            JigsawJointType::ROLLABLE => "rollable",
            JigsawJointType::ALIGNED => "aligned",
        }
    }

    /// `CODEC.byName(name, ALIGNED)` — default при неизвестном имени.
    pub fn by_name_or_default(name: &str) -> JigsawJointType {
        Self::VALUES
            .iter()
            .copied()
            .find(|v| v.get_serialized_name() == name)
            .unwrap_or(JigsawJointType::ALIGNED)
    }
}

/// Маркер `JigsawBlockEntity`.
#[derive(Debug)]
pub struct JigsawBlockEntity;
