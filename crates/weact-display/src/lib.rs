//! Driver building blocks for official WeAct Display FS models.
//!
//! Protocol details are taken from the official
//! [`WeActStudio.SystemMonitor`](https://github.com/WeActStudio/WeActStudio.SystemMonitor)
//! repository, especially `library/lcd/lcd_comm_weact_a.py` and
//! `library/lcd/lcd_comm_weact_b.py` at commit
//! 2420db509aa4dd5b205147806243f2e002bc2f33.
//!
//! The crate is organized around three pieces:
//!
//! - [`framebuffer`] owns pixels and simple drawing operations.
//! - [`protocol`] turns typed values into WeAct command bytes.
//! - [`transport`] abstracts the byte stream used by hardware.
//!
//! [`WeActDisplay`] ties those pieces together.

#![deny(warnings)]

/// RGB565 colors.
pub mod color;
/// Display driver and orientation handling.
pub mod display;
/// Operational errors.
pub mod error;
/// Framebuffer and drawing helpers.
pub mod framebuffer;
/// Official display model definitions.
pub mod model;
/// Orientation values.
pub mod orientation;
/// Protocol command encoders.
pub mod protocol;
/// Byte transport abstraction.
pub mod transport;

pub use color::Rgb565;
pub use display::WeActDisplay;
pub use error::Error;
pub use framebuffer::Framebuffer;
pub use model::{
    DisplayModel, DisplaySpec, SUPPORTED_SPECS, WEACT_FS_096_80X160, WEACT_FS_V1_320X480,
};
pub use orientation::Orientation;
pub use transport::{Transport, TransportError};

pub type Result<T> = std::result::Result<T, Error>;
