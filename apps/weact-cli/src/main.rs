//! Command-line tool for exercising the WeAct display driver on real hardware.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use weact_display::{Framebuffer, Orientation, Rgb565, WeActDisplay};
use weact_display_serial::{SerialTransport, find_ports_by_name};

/// Parsed command-line arguments.
///
/// Clap derives the parser from this struct and the nested enums below.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Subcommand to run.
    #[command(subcommand)]
    command: Command,
}

/// CLI subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Find serial ports by USB product name.
    FindPort {
        /// USB product name to search for.
        #[arg(long, default_value = "Display FS 0.96 Inch")]
        name: String,
    },

    /// Fill the display with one named color.
    Fill {
        /// Serial device path, such as `/dev/ttyACM0` on Linux.
        #[arg(long)]
        port: Option<String>,

        /// USB product name to use when `--port` is omitted.
        #[arg(long, default_value = "Display FS 0.96 Inch")]
        name: String,

        /// Color to draw.
        #[arg(long, value_enum, default_value_t = NamedColor::Red)]
        color: NamedColor,

        /// Display orientation.
        #[arg(long, value_enum, default_value_t = CliOrientation::Landscape)]
        orientation: CliOrientation,

        /// Optional brightness percentage, `0..=100`.
        #[arg(long)]
        brightness: Option<u8>,
    },
}

/// Named colors.
///
/// `ValueEnum` lets Clap parse values like `--color red`.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum NamedColor {
    Red,
    Green,
    Blue,
    Black,
    White,
}

impl NamedColor {
    /// Converts the CLI color into RGB565.
    const fn rgb565(self) -> Rgb565 {
        match self {
            Self::Red => Rgb565::RED,
            Self::Green => Rgb565::GREEN,
            Self::Blue => Rgb565::BLUE,
            Self::Black => Rgb565::BLACK,
            Self::White => Rgb565::WHITE,
        }
    }
}

/// CLI orientation names.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliOrientation {
    Portrait,
    Landscape,
    PortraitFlipped,
    LandscapeFlipped,
}

impl From<CliOrientation> for Orientation {
    fn from(value: CliOrientation) -> Self {
        match value {
            CliOrientation::Portrait => Self::Portrait,
            CliOrientation::Landscape => Self::Landscape,
            CliOrientation::PortraitFlipped => Self::PortraitFlipped,
            CliOrientation::LandscapeFlipped => Self::LandscapeFlipped,
        }
    }
}

/// Program entry point.
fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::FindPort { name } => find_port(&name),
        Command::Fill {
            port,
            name,
            color,
            orientation,
            brightness,
        } => fill(port, &name, color, orientation.into(), brightness),
    }
}

/// Runs the `find-port` subcommand.
fn find_port(name: &str) -> Result<()> {
    let ports = find_ports_by_name(name)?;
    if ports.is_empty() {
        bail!("no serial ports found with USB product name containing `{name}`");
    }

    for port in ports {
        println!("{port}");
    }
    Ok(())
}

/// Runs the `fill` subcommand.
fn fill(
    port: Option<String>,
    name: &str,
    color: NamedColor,
    orientation: Orientation,
    brightness: Option<u8>,
) -> Result<()> {
    if let Some(brightness) = brightness
        && brightness > 100
    {
        bail!("brightness must be between 0 and 100");
    }

    let port = resolve_port(port, name)?;
    let transport = SerialTransport::open(&port)
        .with_context(|| format!("failed to open serial port {port}"))?;
    let mut display = WeActDisplay::new(transport, orientation);
    display.init()?;

    if let Some(brightness) = brightness {
        display.set_brightness(brightness)?;
    }

    let mut framebuffer = match orientation {
        Orientation::Portrait | Orientation::PortraitFlipped => Framebuffer::new_portrait(),
        Orientation::Landscape | Orientation::LandscapeFlipped => Framebuffer::new_landscape(),
    };
    framebuffer.clear(color.rgb565());
    display.draw_framebuffer(&framebuffer)?;
    Ok(())
}

fn resolve_port(port: Option<String>, name: &str) -> Result<String> {
    if let Some(port) = port {
        return Ok(port);
    }

    let ports = find_ports_by_name(name)?;
    match ports.as_slice() {
        [] => bail!("no serial ports found with USB product name containing `{name}`"),
        [port] => Ok(port.clone()),
        _ => bail!(
            "multiple serial ports found for `{name}`: {}. Pass one explicitly with --port",
            ports.join(", ")
        ),
    }
}
