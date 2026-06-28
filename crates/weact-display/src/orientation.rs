//! Orientation values shared by official WeAct Display FS models.

use crate::model::DisplaySpec;

/// Display orientation values understood by the firmware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    /// Native portrait orientation, protocol value `0`.
    Portrait,

    /// Portrait rotated 180 degrees, protocol value `1`.
    PortraitFlipped,

    /// Landscape orientation, protocol value `2`.
    Landscape,

    /// Landscape rotated 180 degrees, protocol value `3`.
    LandscapeFlipped,
}

impl Orientation {
    /// Numeric value sent in the orientation command.
    pub const fn protocol_value(self) -> u8 {
        match self {
            Self::Portrait => 0,
            Self::PortraitFlipped => 1,
            Self::Landscape => 2,
            Self::LandscapeFlipped => 3,
        }
    }

    /// Logical width and height for `spec` in this orientation.
    pub const fn dimensions(self, spec: &DisplaySpec) -> (u16, u16) {
        match self {
            Self::Portrait | Self::PortraitFlipped => (spec.native_width, spec.native_height),
            Self::Landscape | Self::LandscapeFlipped => (spec.native_height, spec.native_width),
        }
    }
}
