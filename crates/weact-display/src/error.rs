use crate::TransportError;
use thiserror::Error;

/// Errors returned by the display driver.
///
/// Transport-specific failures are converted into [`TransportError`]
/// by the active transport and then wrapped as [`Error::Transport`].
#[derive(Debug, Error)]
pub enum Error {
    /// The framebuffer size does not match the current display size.
    ///
    /// `draw_framebuffer` currently uploads full-screen images only.
    /// For example, a `160x80` landscape display expects a `160x80` framebuffer.
    #[error(
        "framebuffer size {actual_width}x{actual_height} does not match display size {display_width}x{display_height}"
    )]
    FramebufferSizeMismatch {
        /// Framebuffer width supplied by the caller.
        actual_width: u16,
        /// Framebuffer height supplied by the caller.
        actual_height: u16,
        /// Width expected for the current orientation.
        display_width: u16,
        /// Height expected for the current orientation.
        display_height: u16,
    },

    /// A rectangle does not fit inside the current display area.
    #[error(
        "rectangle at {x},{y} with size {width}x{height} does not fit display size {display_width}x{display_height}"
    )]
    RectOutOfBounds {
        /// Rectangle start x coordinate.
        x: u16,
        /// Rectangle start y coordinate.
        y: u16,
        /// Rectangle width.
        width: u16,
        /// Rectangle height.
        height: u16,
        /// Current display width.
        display_width: u16,
        /// Current display height.
        display_height: u16,
    },

    /// The transport failed while writing or flushing bytes.
    ///
    /// `#[from]` lets transport errors flow through `?` as driver errors.
    #[error(transparent)]
    Transport(#[from] TransportError),
}
