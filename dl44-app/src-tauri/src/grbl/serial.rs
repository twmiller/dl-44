//! Serial port enumeration.
//!
//! Note: Actual serial I/O is handled by the worker thread (see worker.rs).
//! This module only provides port listing functionality.

use thiserror::Error;

/// Serial port errors
#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Failed to enumerate ports: {0}")]
    EnumerationFailed(#[from] serialport::Error),
}

/// Information about an available serial port
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortInfo {
    /// Port path (e.g., "/dev/ttyUSB0" or "COM3")
    pub path: String,
    /// Port type description
    pub port_type: String,
    /// Manufacturer if available
    pub manufacturer: Option<String>,
    /// Product name if available
    pub product: Option<String>,
    /// Serial number if available
    pub serial_number: Option<String>,
}

/// List available serial ports.
pub fn list_ports() -> Result<Vec<PortInfo>, SerialError> {
    let ports = serialport::available_ports()?;

    Ok(ports
        .into_iter()
        .map(|p| {
            let (port_type, manufacturer, product, serial_number) = match p.port_type {
                serialport::SerialPortType::UsbPort(info) => (
                    "USB".to_string(),
                    info.manufacturer,
                    info.product,
                    info.serial_number,
                ),
                serialport::SerialPortType::PciPort => ("PCI".to_string(), None, None, None),
                serialport::SerialPortType::BluetoothPort => {
                    ("Bluetooth".to_string(), None, None, None)
                }
                serialport::SerialPortType::Unknown => ("Unknown".to_string(), None, None, None),
            };

            PortInfo {
                path: p.port_name,
                port_type,
                manufacturer,
                product,
                serial_number,
            }
        })
        .collect())
}
