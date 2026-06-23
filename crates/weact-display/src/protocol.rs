//! Command encoding for the WeAct display protocol.
//!
//! Each function returns the exact byte sequence expected by the display,
//! which keeps protocol details easy to test without hardware.

use crate::Rgb565;

/// Native portrait width of the 0.96-inch device.
pub const NATIVE_WIDTH: u16 = 80;

/// Native portrait height of the 0.96-inch device.
pub const NATIVE_HEIGHT: u16 = 160;

/// Changes orientation.
pub const CMD_SET_ORIENTATION: u8 = 0x02;

/// Changes backlight brightness.
pub const CMD_SET_BRIGHTNESS: u8 = 0x03;

/// Fills a rectangular area with one color.
pub const CMD_FULL: u8 = 0x04;

/// Uploads an uncompressed bitmap.
pub const CMD_SET_BITMAP: u8 = 0x05;

/// Uploads a FastLZ-compressed bitmap.
/// TODO: use this after the transport can read firmware capability responses
///       and the driver can encode FastLZ chunks.
pub const CMD_SET_BITMAP_WITH_FASTLZ: u8 = 0x15;

/// Tells the device the host is done using the display.
pub const CMD_FREE: u8 = 0x07;

/// Reads the device firmware version.
pub const CMD_SYSTEM_VERSION: u8 = 0x42;

/// Marks a command as a read request.
pub const CMD_READ: u8 = 0x80;

/// End marker appended to command headers.
pub const CMD_END: u8 = 0x0a;

/// Display orientation values understood by the firmware.
///
/// The 0.96-inch panel is natively `80x160`;
/// landscape orientations expose it as `160x80`.
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

    /// Logical width and height for this orientation.
    pub const fn dimensions(self) -> (u16, u16) {
        match self {
            Self::Portrait | Self::PortraitFlipped => (NATIVE_WIDTH, NATIVE_HEIGHT),
            Self::Landscape | Self::LandscapeFlipped => (NATIVE_HEIGHT, NATIVE_WIDTH),
        }
    }
}

/// Asks the device for its firmware version.
///
/// The host app uses the response to check FastLZ support.
///
/// TODO: call this during initialization after [`crate::Transport`] supports
///       reading response bytes.
pub fn system_version_request() -> [u8; 2] {
    [CMD_SYSTEM_VERSION | CMD_READ, CMD_END]
}

/// Builds a set-orientation command.
pub fn set_orientation(orientation: Orientation) -> [u8; 3] {
    [CMD_SET_ORIENTATION, orientation.protocol_value(), CMD_END]
}

/// Builds a brightness command.
///
/// The public API uses `0..=100` percent.
/// The protocol expects `0..=255` plus a 1000 ms transition time.
pub fn set_brightness(percent: u8) -> [u8; 5] {
    let percent = percent.min(100);
    let converted = ((percent as u16 * 255) / 100) as u8;
    let transition_ms = 1000_u16.to_le_bytes();
    [
        CMD_SET_BRIGHTNESS, converted, transition_ms[0], transition_ms[1], CMD_END,
    ]
}

/// Builds a solid-fill command for a rectangular area.
///
/// Protocol coordinates are inclusive, so a `10x10` rectangle at `(5, 2)` ends at `(14, 11)`.
pub fn fill_rect(x: u16, y: u16, width: u16, height: u16, color: Rgb565) -> [u8; 12] {
    let x0 = x.to_le_bytes();
    let y0 = y.to_le_bytes();
    let x1 = x.saturating_add(width).saturating_sub(1).to_le_bytes();
    let y1 = y.saturating_add(height).saturating_sub(1).to_le_bytes();
    let c = color.to_le_bytes();
    [
        CMD_FULL, x0[0], x0[1], y0[0], y0[1], x1[0], x1[1], y1[0], y1[1], c[0], c[1], CMD_END,
    ]
}

/// Starts an uncompressed bitmap upload for a rectangular area.
///
/// Pixel data follows this header as `width * height * 2` little-endian RGB565
/// bytes.
pub fn set_bitmap_header(x: u16, y: u16, width: u16, height: u16) -> [u8; 10] {
    let x0 = x.to_le_bytes();
    let y0 = y.to_le_bytes();
    let x1 = x.saturating_add(width).saturating_sub(1).to_le_bytes();
    let y1 = y.saturating_add(height).saturating_sub(1).to_le_bytes();
    [
        CMD_SET_BITMAP, x0[0], x0[1], y0[0], y0[1], x1[0], x1[1], y1[0], y1[1], CMD_END,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_set_bitmap_header() {
        assert_eq!(
            set_bitmap_header(0, 0, 160, 80),
            [0x05, 0, 0, 0, 0, 159, 0, 79, 0, 0x0a]
        );
    }

    #[test]
    fn encodes_brightness_command() {
        assert_eq!(set_brightness(100), [0x03, 255, 0xe8, 0x03, 0x0a]);
        assert_eq!(set_brightness(40), [0x03, 102, 0xe8, 0x03, 0x0a]);
    }

    #[test]
    fn encodes_full_area_fill_rect_command() {
        assert_eq!(
            fill_rect(0, 0, 160, 80, Rgb565::RED),
            [0x04, 0, 0, 0, 0, 159, 0, 79, 0, 0x00, 0xf8, 0x0a]
        );
    }

    #[test]
    fn encodes_fill_rect_command() {
        assert_eq!(
            fill_rect(5, 2, 10, 10, Rgb565::BLUE),
            [0x04, 5, 0, 2, 0, 14, 0, 11, 0, 0x1f, 0x00, 0x0a]
        );
    }
}
