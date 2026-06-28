//! Command-line tool for exercising the WeAct display driver on real hardware.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use weact_display::{DisplaySpec, Framebuffer, Orientation, Rgb565, WeActDisplay};
use weact_display_serial::{DetectedPort, SerialTransport, find_display_ports};

/// Test CLI for official WeAct Display FS models.
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
    /// Find serial ports for supported display models.
    FindPort,

    /// Fill the display with one named color.
    Fill {
        /// Serial device path, such as `/dev/ttyACM0` on Linux.
        #[arg(long)]
        port: Option<String>,

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
        Command::FindPort => find_port(),
        Command::Fill {
            port,
            color,
            orientation,
            brightness,
        } => fill(port, color, orientation.into(), brightness),
    }
}

/// Runs the `find-port` subcommand.
fn find_port() -> Result<()> {
    let ports = find_display_ports()?;
    if ports.is_empty() {
        bail!("no supported WeAct Display FS serial ports found");
    }

    for detected in ports {
        println!(
            "{}\t{}\t{}",
            detected.spec.revision, detected.spec.name, detected.port_name
        );
    }

    Ok(())
}

/// Runs the `fill` subcommand.
fn fill(
    port: Option<String>,
    color: NamedColor,
    orientation: Orientation,
    brightness: Option<u8>,
) -> Result<()> {
    if let Some(brightness) = brightness
        && brightness > 100
    {
        bail!("brightness must be between 0 and 100");
    }

    let detected = resolve_display(port)?;
    let spec = detected.spec;
    let port = detected.port_name;
    let transport = SerialTransport::open(&port)
        .with_context(|| format!("failed to open serial port {port}"))?;
    let mut display = WeActDisplay::new(transport, spec, orientation);
    display.init()?;

    if let Some(brightness) = brightness {
        display.set_brightness(brightness)?;
    }

    let mut framebuffer = Framebuffer::new_for_display(spec, orientation);
    framebuffer.clear(color.rgb565());
    display.draw_framebuffer(&framebuffer)?;
    Ok(())
}

fn resolve_display(port: Option<String>) -> Result<DetectedPort> {
    let detected_ports = find_display_ports()?;

    match port {
        Some(port) => resolve_explicit_display_port(&detected_ports, &port),
        None => resolve_single_detected_display(&detected_ports),
    }
}

fn resolve_single_detected_display(detected_ports: &[DetectedPort]) -> Result<DetectedPort> {
    match detected_ports {
        [] => bail!("no supported WeAct Display FS serial ports found"),
        [detected] => Ok(detected.clone()),
        _ => bail!(
            "multiple supported WeAct Display FS serial ports found: {}. Pass one explicitly with --port",
            format_detected_ports(detected_ports)
        ),
    }
}

fn resolve_explicit_display_port(
    detected_ports: &[DetectedPort],
    port: &str,
) -> Result<DetectedPort> {
    let matches: Vec<_> = detected_ports
        .iter()
        .filter(|detected| detected.port_name == port)
        .cloned()
        .collect();

    match matches.as_slice() {
        [] => bail!("serial port {port} was not recognized as a supported WeAct Display FS device"),
        [detected] => Ok(detected.clone()),
        _ => bail!(
            "serial port {port} matched multiple display specs: {}",
            format_detected_ports(&matches)
        ),
    }
}

fn format_detected_ports(ports: &[DetectedPort]) -> String {
    ports
        .iter()
        .map(|detected| format_detected_port(detected.spec, &detected.port_name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_detected_port(spec: &DisplaySpec, port_name: &str) -> String {
    format!("{} {} on {}", spec.revision, spec.name, port_name)
}
