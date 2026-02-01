//! Tauri command handlers for GRBL controller operations.

use std::sync::Arc;
use tauri::State;

use crate::grbl::{
    ConnectionState, Controller, ControllerError, ControllerSnapshot, MachineStatus,
    OverrideAdjust, PortInfo, RapidOverride,
};
use crate::grbl::protocol::SUPPORTED_BAUD_RATES;

/// Application state holding the controller
pub struct AppState {
    pub controller: Arc<Controller>,
}

/// Error type for Tauri commands with structured error info
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    /// Human-readable error message
    pub message: String,
    /// Error code for programmatic handling (e.g., "TIMEOUT", "GRBL_ERROR", "ALARM")
    pub code: String,
    /// Additional details (e.g., GRBL error number, retry count)
    pub details: Option<String>,
}

impl From<ControllerError> for CommandError {
    fn from(e: ControllerError) -> Self {
        let (code, details) = match &e {
            ControllerError::NotConnected => ("NOT_CONNECTED".into(), None),
            ControllerError::AlreadyConnected => ("ALREADY_CONNECTED".into(), None),
            ControllerError::Timeout(attempts) => {
                ("TIMEOUT".into(), Some(format!("{} attempts", attempts)))
            }
            ControllerError::GrblError(code) => ("GRBL_ERROR".into(), Some(format!("code {}", code))),
            ControllerError::Alarm(code) => ("ALARM".into(), Some(format!("code {}", code))),
            ControllerError::InvalidState(_) => ("INVALID_STATE".into(), None),
            ControllerError::Serial(_) => ("SERIAL_ERROR".into(), None),
            ControllerError::Internal(_) => ("INTERNAL_ERROR".into(), None),
        };

        Self {
            message: e.to_string(),
            code,
            details,
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

/// List available serial ports
#[tauri::command]
pub fn list_serial_ports(state: State<AppState>) -> CommandResult<Vec<PortInfo>> {
    state
        .controller
        .list_ports()
        .map_err(CommandError::from)
}

/// Get supported baud rates
#[tauri::command]
pub fn get_baud_rates() -> Vec<u32> {
    SUPPORTED_BAUD_RATES.to_vec()
}

/// Connect to a GRBL device
#[tauri::command]
pub fn connect(state: State<AppState>, port: String, baud_rate: u32) -> CommandResult<()> {
    state
        .controller
        .connect(&port, baud_rate)
        .map_err(CommandError::from)
}

/// Disconnect from the device
#[tauri::command]
pub fn disconnect(state: State<AppState>) -> CommandResult<()> {
    state.controller.disconnect().map_err(CommandError::from)
}

/// Get current connection state
#[tauri::command]
pub fn get_connection_state(state: State<AppState>) -> ConnectionState {
    state.controller.connection_state()
}

/// Check if connected
#[tauri::command]
pub fn is_connected(state: State<AppState>) -> bool {
    state.controller.is_connected()
}

/// Poll machine status (queries device and returns latest status)
#[tauri::command]
pub fn poll_status(state: State<AppState>) -> CommandResult<MachineStatus> {
    state.controller.poll_status().map_err(CommandError::from)
}

/// Get cached status without polling
#[tauri::command]
pub fn get_status(state: State<AppState>) -> MachineStatus {
    state.controller.status()
}

/// Get full controller snapshot (connection state + status + messages)
#[tauri::command]
pub fn get_controller_snapshot(state: State<AppState>) -> ControllerSnapshot {
    state.controller.snapshot()
}

/// Send home command
#[tauri::command]
pub fn home(state: State<AppState>) -> CommandResult<()> {
    state.controller.home().map_err(CommandError::from)
}

/// Send unlock command
#[tauri::command]
pub fn unlock(state: State<AppState>) -> CommandResult<()> {
    state.controller.unlock().map_err(CommandError::from)
}

/// Send jog command
#[tauri::command]
pub fn jog(
    state: State<AppState>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    feed: f64,
    incremental: bool,
) -> CommandResult<()> {
    state
        .controller
        .jog(x, y, z, feed, incremental)
        .map_err(CommandError::from)
}

/// Cancel active jog
#[tauri::command]
pub fn jog_cancel(state: State<AppState>) -> CommandResult<()> {
    state.controller.jog_cancel().map_err(CommandError::from)
}

/// Send feed hold (pause)
#[tauri::command]
pub fn feed_hold(state: State<AppState>) -> CommandResult<()> {
    state.controller.feed_hold().map_err(CommandError::from)
}

/// Send cycle start (resume)
#[tauri::command]
pub fn cycle_start(state: State<AppState>) -> CommandResult<()> {
    state.controller.cycle_start().map_err(CommandError::from)
}

/// Send soft reset
#[tauri::command]
pub fn soft_reset(state: State<AppState>) -> CommandResult<()> {
    state.controller.soft_reset().map_err(CommandError::from)
}

/// Adjust feed rate override
#[tauri::command]
pub fn feed_override(state: State<AppState>, adjust: OverrideAdjust) -> CommandResult<()> {
    state
        .controller
        .feed_override(adjust)
        .map_err(CommandError::from)
}

/// Set rapid override preset
#[tauri::command]
pub fn rapid_override(state: State<AppState>, preset: RapidOverride) -> CommandResult<()> {
    state
        .controller
        .rapid_override(preset)
        .map_err(CommandError::from)
}

/// Adjust spindle/laser power override
#[tauri::command]
pub fn spindle_override(state: State<AppState>, adjust: OverrideAdjust) -> CommandResult<()> {
    state
        .controller
        .spindle_override(adjust)
        .map_err(CommandError::from)
}

/// Run a frame/boundary trace
#[tauri::command]
pub fn run_frame(
    state: State<AppState>,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    feed: f64,
    power: u32,
) -> CommandResult<()> {
    state
        .controller
        .run_frame(x_min, x_max, y_min, y_max, feed, power)
        .map_err(CommandError::from)
}
