//! GRBL protocol and controller module.
//!
//! This module provides the core GRBL communication layer:
//! - Protocol constants and command builders
//! - Serial port enumeration
//! - Status parsing and machine state
//! - Worker thread for non-blocking serial I/O
//! - High-level controller for coordinating operations

pub mod controller;
pub mod protocol;
pub mod serial;
pub mod status;
pub mod worker;

pub use controller::{ConnectionState, Controller, ControllerError, ControllerSnapshot};
pub use serial::PortInfo;
pub use status::MachineStatus;
