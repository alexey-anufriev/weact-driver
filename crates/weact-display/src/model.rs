//! Official WeAct Display FS model definitions.
//!
//! The supported hardware revisions are:
//!
//! - `A_320x480` for WeAct Studio Display FS V1;
//! - `B_80x160` for WeAct Studio Display FS 0.96 Inch.

/// Official display model identifiers supported by the current library.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayModel {
    /// WeAct Studio Display FS V1, native portrait size 320x480.
    WeActFsV1_320x480,

    /// WeAct Studio Display FS 0.96 Inch, native portrait size 80x160.
    WeActFs096_80x160,
}

/// Static model facts used by the driver and serial discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplaySpec {
    /// Stable model identifier.
    pub model: DisplayModel,
    /// Human-readable model name.
    pub name: &'static str,
    /// Hardware revision string.
    pub revision: &'static str,
    /// Native portrait width.
    pub native_width: u16,
    /// Native portrait height.
    pub native_height: u16,
    /// Serial-number prefix used by the vendor auto-detect behavior.
    pub serial_prefix: &'static str,
    /// USB product substring used by the Rust serial discovery helper.
    pub product_name_hint: &'static str,
    /// Whether the official protocol implementation exposes humiture reports.
    pub supports_humiture: bool,
}

/// WeAct Studio Display FS V1, official revision `A_320x480`.
pub const WEACT_FS_V1_320X480: DisplaySpec = DisplaySpec {
    model: DisplayModel::WeActFsV1_320x480,
    name: "WeAct Studio Display FS V1",
    revision: "A_320x480",
    native_width: 320,
    native_height: 480,
    serial_prefix: "AB",
    product_name_hint: "Display FS V1",
    supports_humiture: true,
};

/// WeAct Studio Display FS 0.96 Inch, official revision `B_80x160`.
pub const WEACT_FS_096_80X160: DisplaySpec = DisplaySpec {
    model: DisplayModel::WeActFs096_80x160,
    name: "WeAct Studio Display FS 0.96 Inch",
    revision: "B_80x160",
    native_width: 80,
    native_height: 160,
    serial_prefix: "AD",
    product_name_hint: "Display FS 0.96 Inch",
    supports_humiture: false,
};

/// All display specs supported by this crate.
pub const SUPPORTED_SPECS: [&DisplaySpec; 2] = [&WEACT_FS_096_80X160, &WEACT_FS_V1_320X480];

impl DisplayModel {
    /// Returns static facts for this model.
    pub const fn spec(self) -> &'static DisplaySpec {
        match self {
            Self::WeActFsV1_320x480 => &WEACT_FS_V1_320X480,
            Self::WeActFs096_80x160 => &WEACT_FS_096_80X160,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayModel, WEACT_FS_096_80X160, WEACT_FS_V1_320X480};

    #[test]
    fn specs_match_official_native_sizes() {
        assert_eq!(
            (
                WEACT_FS_096_80X160.native_width,
                WEACT_FS_096_80X160.native_height
            ),
            (80, 160)
        );
        assert_eq!(
            (
                WEACT_FS_V1_320X480.native_width,
                WEACT_FS_V1_320X480.native_height
            ),
            (320, 480)
        );
    }

    #[test]
    fn model_returns_its_spec() {
        assert_eq!(DisplayModel::WeActFs096_80x160.spec(), &WEACT_FS_096_80X160);
        assert_eq!(DisplayModel::WeActFsV1_320x480.spec(), &WEACT_FS_V1_320X480);
    }
}
