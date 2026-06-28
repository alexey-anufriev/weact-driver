//! Serial transport for `weact-display`.
//!
//! [`SerialTransport`] opens the device's USB CDC serial port
//! and implements `weact_display::Transport`, so the display driver
//! can send protocol bytes through it.

#![deny(warnings)]

use std::io::{self, Write};
use std::time::Duration;

use serialport::{FlowControl, SerialPort, SerialPortType, UsbPortInfo};
use weact_display::{DisplaySpec, SUPPORTED_SPECS, Transport, TransportError};

/// Serial-port transport for the WeAct USB CDC device.
pub struct SerialTransport {
    port: Box<dyn SerialPort>,
}

impl SerialTransport {
    /// Opens a serial port with the standard WeAct settings.
    ///
    /// Uses 115200 baud, a one-second timeout, and RTS/CTS flow control.
    pub fn open(path: &str) -> io::Result<Self> {
        Self::open_with_baud_rate(path, 115_200)
    }

    /// Opens a serial port with an explicit baud rate.
    ///
    /// Intended for experiments; normal hardware access should use [`SerialTransport::open`].
    pub fn open_with_baud_rate(path: &str, baud_rate: u32) -> io::Result<Self> {
        let port = serialport::new(path, baud_rate)
            .timeout(Duration::from_secs(1))
            .flow_control(FlowControl::Hardware)
            .open()?;
        Ok(Self { port })
    }
}

impl Transport for SerialTransport {
    /// Writes all bytes to the serial port.
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        self.port.write_all(bytes).map_err(TransportError::from)
    }

    /// Flushes the serial port.
    fn flush(&mut self) -> Result<(), TransportError> {
        self.port.flush().map_err(TransportError::from)
    }
}

/// A serial port matched to a supported display spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectedPort {
    /// Display spec inferred from USB metadata.
    pub spec: &'static DisplaySpec,
    /// Operating-system serial port path.
    pub port_name: String,
}

/// Finds serial ports that match any supported display spec.
pub fn find_display_ports() -> serialport::Result<Vec<DetectedPort>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .filter_map(|port| match port.port_type {
            SerialPortType::UsbPort(info) => Some((port.port_name, info)),
            _ => None,
        })
        .flat_map(|(port_name, info)| {
            SUPPORTED_SPECS
                .iter()
                .copied()
                .filter(move |spec| spec_matches_usb_info(spec, &info))
                .map(move |spec| DetectedPort {
                    spec,
                    port_name: port_name.clone(),
                })
        })
        .collect())
}

fn spec_matches_usb_info(spec: &DisplaySpec, info: &UsbPortInfo) -> bool {
    let product_name_hint = spec.product_name_hint.to_ascii_lowercase();
    let product_matches = info
        .product
        .as_ref()
        .is_some_and(|product| product.to_ascii_lowercase().contains(&product_name_hint));
    let serial_matches = info
        .serial_number
        .as_ref()
        .is_some_and(|serial| serial.starts_with(spec.serial_prefix));

    product_matches || serial_matches
}
