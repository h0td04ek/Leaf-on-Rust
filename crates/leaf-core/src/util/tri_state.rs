//! Перенос `net.minecraft.util.TriState`.
//! DFU-кодек (`CODEC`) появится вместе с портом DataFixerUpper.

/// `TriState` — true/false/default для опциональных булевых настроек.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TriState {
    TRUE,
    FALSE,
    #[default]
    DEFAULT,
}

impl TriState {
    pub fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }

    pub fn to_boolean(self, default_value: bool) -> bool {
        match self {
            Self::TRUE => true,
            Self::FALSE => false,
            Self::DEFAULT => default_value,
        }
    }

    /// `getSerializedName`
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::TRUE => "true",
            Self::FALSE => "false",
            Self::DEFAULT => "default",
        }
    }
}

impl crate::util::string_representable::StringRepresentable for TriState {
    fn get_serialized_name(&self) -> String {
        self.serialized_name().to_string()
    }
}

impl std::fmt::Display for TriState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.serialized_name())
    }
}
