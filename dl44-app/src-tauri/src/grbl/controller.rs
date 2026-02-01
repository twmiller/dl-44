//! High-level GRBL controller coordinating serial worker and state.
//!
//! The controller maintains machine state and delegates all serial I/O
//! to a dedicated worker thread. Command handlers block waiting for
//! the worker response, but serial I/O is isolated, preventing port
//! access issues and providing centralized timeout handling.

use parking_lot::Mutex;
use std::sync::Arc;
use thiserror::Error;

use super::protocol;
use super::serial::PortInfo;
use super::status::{MachineState, MachineStatus};
use super::worker::{WorkerError, WorkerHandle, HOMING_TIMEOUT_MS};

/// Controller errors (UI-facing)
#[derive(Error, Debug, Clone, serde::Serialize)]
pub enum ControllerError {
    #[error("Serial error: {0}")]
    Serial(String),

    #[error("Not connected to device")]
    NotConnected,

    #[error("Already connected")]
    AlreadyConnected,

    #[error("Command timeout after {0} attempts")]
    Timeout(u32),

    #[error("GRBL error code {0}")]
    GrblError(u32),

    #[error("Device in alarm state (code {0})")]
    Alarm(u32),

    #[error("Invalid state for operation: {0}")]
    InvalidState(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<WorkerError> for ControllerError {
    fn from(e: WorkerError) -> Self {
        match e {
            WorkerError::OpenFailed(msg) => ControllerError::Serial(msg),
            WorkerError::Io(msg) => ControllerError::Serial(msg),
            WorkerError::NotConnected => ControllerError::NotConnected,
            WorkerError::Timeout { attempts } => ControllerError::Timeout(attempts),
            WorkerError::GrblError(code) => ControllerError::GrblError(code),
            WorkerError::Alarm(code) => ControllerError::Alarm(code),
            WorkerError::WorkerDead => {
                ControllerError::Internal("Worker thread not responding".into())
            }
        }
    }
}

/// Connection state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { port: String, baud: u32 },
    Error(String),
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Shared controller state (protected by mutex)
#[derive(Debug, Default)]
struct ControllerState {
    connection: ConnectionState,
    status: MachineStatus,
    last_error: Option<String>,
    welcome_message: Option<String>,
    /// Alarm code if device entered alarm during polling (with unique ID for dedup)
    pending_alarm: Option<(u32, u64)>, // (alarm_code, alarm_id)
    /// Counter for generating unique alarm IDs
    alarm_id_counter: u64,
    /// Whether the last status poll got a fresh response
    status_is_fresh: bool,
}

/// GRBL controller instance.
///
/// Thread-safe controller that delegates serial I/O to a worker thread.
/// Command handlers block waiting for the worker response (with timeout),
/// but serial I/O is isolated in the worker thread.
pub struct Controller {
    worker: WorkerHandle,
    state: Mutex<ControllerState>,
}

impl Controller {
    /// Create a new controller with its worker thread.
    pub fn new() -> Self {
        Self {
            worker: WorkerHandle::spawn(),
            state: Mutex::new(ControllerState::default()),
        }
    }

    /// Create a new controller wrapped in Arc for sharing.
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// List available serial ports.
    ///
    /// Note: This doesn't use the worker since port enumeration is fast.
    pub fn list_ports(&self) -> Result<Vec<PortInfo>, ControllerError> {
        super::serial::list_ports().map_err(|e| ControllerError::Serial(e.to_string()))
    }

    /// Connect to a GRBL device.
    pub fn connect(&self, port: &str, baud_rate: u32) -> Result<(), ControllerError> {
        // Check if already connected
        {
            let state = self.state.lock();
            if matches!(state.connection, ConnectionState::Connected { .. }) {
                return Err(ControllerError::AlreadyConnected);
            }
        }

        // Update state to connecting
        {
            let mut state = self.state.lock();
            state.connection = ConnectionState::Connecting;
            state.last_error = None;
            state.pending_alarm = None;
        }

        // Attempt connection via worker
        match self.worker.connect(port, baud_rate) {
            Ok(welcome_msg) => {
                let mut state = self.state.lock();
                state.connection = ConnectionState::Connected {
                    port: port.to_string(),
                    baud: baud_rate,
                };
                if !welcome_msg.is_empty() {
                    state.welcome_message = Some(welcome_msg);
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                let mut state = self.state.lock();
                state.connection = ConnectionState::Error(error_msg.clone());
                state.last_error = Some(error_msg);
                Err(e.into())
            }
        }
    }

    /// Disconnect from the device.
    pub fn disconnect(&self) -> Result<(), ControllerError> {
        // Check if connected
        {
            let state = self.state.lock();
            if !matches!(state.connection, ConnectionState::Connected { .. }) {
                return Err(ControllerError::NotConnected);
            }
        }

        self.worker.disconnect()?;

        let mut state = self.state.lock();
        state.connection = ConnectionState::Disconnected;
        state.status = MachineStatus::default();
        state.welcome_message = None;
        state.pending_alarm = None;
        state.status_is_fresh = false;

        Ok(())
    }

    /// Get current connection state.
    pub fn connection_state(&self) -> ConnectionState {
        self.state.lock().connection.clone()
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        matches!(
            self.state.lock().connection,
            ConnectionState::Connected { .. }
        )
    }

    /// Query and update machine status.
    ///
    /// Waits for a status report from the device (with timeout).
    /// Also captures any alarm/error seen during polling.
    pub fn poll_status(&self) -> Result<MachineStatus, ControllerError> {
        if !self.is_connected() {
            return Err(ControllerError::NotConnected);
        }

        match self.worker.query_status() {
            Ok(result) => {
                let mut state = self.state.lock();

                // Update freshness indicator
                state.status_is_fresh = result.is_fresh;

                // Update status if we got one
                if let Some(status) = result.status {
                    state.status = status;
                    // Clear stale alarm if we have a fresh, non-alarm state
                    if result.is_fresh && state.status.state != MachineState::Alarm {
                        state.pending_alarm = None;
                    }
                }

                // Record alarm if we saw a NEW one during polling
                // Only set pending_alarm if it's a different alarm than we already have
                if let Some(alarm_code) = result.alarm {
                    let should_set = match state.pending_alarm {
                        Some((existing_code, _)) => existing_code != alarm_code,
                        None => true,
                    };
                    if should_set {
                        state.alarm_id_counter += 1;
                        state.pending_alarm = Some((alarm_code, state.alarm_id_counter));
                        state.last_error = Some(format!("ALARM:{}", alarm_code));
                    }
                }

                // Record error if we saw one
                if let Some(error_code) = result.error {
                    state.last_error = Some(format!("error:{}", error_code));
                }

                Ok(state.status.clone())
            }
            Err(e) => {
                let mut state = self.state.lock();
                state.last_error = Some(e.to_string());
                state.status_is_fresh = false;
                Err(e.into())
            }
        }
    }

    /// Get cached machine status (without polling).
    pub fn status(&self) -> MachineStatus {
        self.state.lock().status.clone()
    }

    /// Send home command.
    ///
    /// Uses a longer timeout since homing can take 30+ seconds on large machines.
    pub fn home(&self) -> Result<(), ControllerError> {
        if !self.is_connected() {
            return Err(ControllerError::NotConnected);
        }

        // Homing: no retries (it either works or alarms), long timeout
        self.worker
            .send_command_with_policy(protocol::system::HOME, 0, HOMING_TIMEOUT_MS)
            .map_err(|e| {
                let mut state = self.state.lock();
                state.last_error = Some(e.to_string());
                e.into()
            })
    }

    /// Send unlock command.
    pub fn unlock(&self) -> Result<(), ControllerError> {
        // Clear pending alarm on unlock attempt
        self.state.lock().pending_alarm = None;
        self.send_command(protocol::system::UNLOCK)
    }

    /// Send jog command.
    pub fn jog(
        &self,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        feed: f64,
        incremental: bool,
    ) -> Result<(), ControllerError> {
        // Validate state - can only jog when idle or already jogging
        {
            let state = self.state.lock();
            match state.status.state {
                MachineState::Idle | MachineState::Jog => {}
                other => {
                    return Err(ControllerError::InvalidState(format!(
                        "Cannot jog in {:?} state",
                        other
                    )));
                }
            }
        }

        let cmd = protocol::build_jog_command(x, y, z, feed, incremental);
        self.send_command(&cmd)
    }

    /// Cancel active jog.
    pub fn jog_cancel(&self) -> Result<(), ControllerError> {
        self.send_realtime(protocol::JOG_CANCEL)
    }

    /// Send feed hold (pause).
    pub fn feed_hold(&self) -> Result<(), ControllerError> {
        self.send_realtime(protocol::realtime::FEED_HOLD)
    }

    /// Send cycle start (resume).
    pub fn cycle_start(&self) -> Result<(), ControllerError> {
        self.send_realtime(protocol::realtime::CYCLE_START)
    }

    /// Send soft reset.
    pub fn soft_reset(&self) -> Result<(), ControllerError> {
        let result = self.send_realtime(protocol::realtime::SOFT_RESET);

        // Reset cached state on soft reset
        if result.is_ok() {
            let mut state = self.state.lock();
            state.status = MachineStatus::default();
            state.pending_alarm = None;
            state.status_is_fresh = false;
        }

        result
    }

    /// Send a command with default retry/timeout policy.
    fn send_command(&self, cmd: &str) -> Result<(), ControllerError> {
        if !self.is_connected() {
            return Err(ControllerError::NotConnected);
        }

        self.worker.send_command(cmd).map_err(|e| {
            let mut state = self.state.lock();
            state.last_error = Some(e.to_string());
            e.into()
        })
    }

    /// Send a real-time command.
    fn send_realtime(&self, cmd: u8) -> Result<(), ControllerError> {
        if !self.is_connected() {
            return Err(ControllerError::NotConnected);
        }

        self.worker.send_realtime(cmd).map_err(|e| {
            let mut state = self.state.lock();
            state.last_error = Some(e.to_string());
            e.into()
        })
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Override adjustment type
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum OverrideAdjust {
    /// Reset to 100%
    Reset,
    /// Increase by 10%
    CoarsePlus,
    /// Decrease by 10%
    CoarseMinus,
    /// Increase by 1%
    FinePlus,
    /// Decrease by 1%
    FineMinus,
}

/// Rapid override preset
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RapidOverride {
    /// 100%
    Full,
    /// 50%
    Half,
    /// 25%
    Quarter,
}

impl Controller {
    /// Adjust feed rate override.
    pub fn feed_override(&self, adjust: OverrideAdjust) -> Result<(), ControllerError> {
        let cmd = match adjust {
            OverrideAdjust::Reset => protocol::realtime::FEED_OVR_RESET,
            OverrideAdjust::CoarsePlus => protocol::realtime::FEED_OVR_COARSE_PLUS,
            OverrideAdjust::CoarseMinus => protocol::realtime::FEED_OVR_COARSE_MINUS,
            OverrideAdjust::FinePlus => protocol::realtime::FEED_OVR_FINE_PLUS,
            OverrideAdjust::FineMinus => protocol::realtime::FEED_OVR_FINE_MINUS,
        };
        self.send_realtime(cmd)
    }

    /// Set rapid override preset.
    pub fn rapid_override(&self, preset: RapidOverride) -> Result<(), ControllerError> {
        let cmd = match preset {
            RapidOverride::Full => protocol::realtime::RAPID_OVR_RESET,
            RapidOverride::Half => protocol::realtime::RAPID_OVR_HALF,
            RapidOverride::Quarter => protocol::realtime::RAPID_OVR_QUARTER,
        };
        self.send_realtime(cmd)
    }

    /// Adjust spindle/laser power override.
    pub fn spindle_override(&self, adjust: OverrideAdjust) -> Result<(), ControllerError> {
        let cmd = match adjust {
            OverrideAdjust::Reset => protocol::realtime::SPINDLE_OVR_RESET,
            OverrideAdjust::CoarsePlus => protocol::realtime::SPINDLE_OVR_COARSE_PLUS,
            OverrideAdjust::CoarseMinus => protocol::realtime::SPINDLE_OVR_COARSE_MINUS,
            OverrideAdjust::FinePlus => protocol::realtime::SPINDLE_OVR_FINE_PLUS,
            OverrideAdjust::FineMinus => protocol::realtime::SPINDLE_OVR_FINE_MINUS,
        };
        self.send_realtime(cmd)
    }

    /// Run a frame/boundary trace at low power.
    ///
    /// Traces a rectangle from (x_min, y_min) to (x_max, y_max) at the
    /// specified feed rate and laser power (S value). Uses G1 moves with
    /// laser mode (M4) so the laser only fires during motion.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Not connected
    /// - Machine not in Idle state
    /// - Frame has zero area (x_min == x_max or y_min == y_max)
    pub fn run_frame(
        &self,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        feed: f64,
        power: u32,
        units: protocol::Units,
    ) -> Result<(), ControllerError> {
        if !self.is_connected() {
            return Err(ControllerError::NotConnected);
        }

        // Validate bounds - must have non-zero area
        // Note: inverted bounds (min > max) are normalized in build_frame_gcode
        let width = (x_max - x_min).abs();
        let height = (y_max - y_min).abs();
        if width < f64::EPSILON || height < f64::EPSILON {
            return Err(ControllerError::InvalidState(
                "Frame must have non-zero width and height".into(),
            ));
        }

        // Validate state - can only frame when idle
        {
            let state = self.state.lock();
            if state.status.state != MachineState::Idle {
                return Err(ControllerError::InvalidState(format!(
                    "Cannot run frame in {:?} state",
                    state.status.state
                )));
            }
        }

        let gcode = protocol::build_frame_gcode(x_min, x_max, y_min, y_max, feed, power, units);

        // Send each line of the frame GCode
        for line in gcode.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            self.send_command(&format!("{}\n", line))?;
        }

        Ok(())
    }
}

/// Serializable snapshot of controller state for the UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerSnapshot {
    pub connection: ConnectionState,
    pub status: MachineStatus,
    pub welcome_message: Option<String>,
    pub last_error: Option<String>,
    /// Pending alarm: (alarm_code, unique_id) - ID for deduplication
    pub pending_alarm: Option<(u32, u64)>,
    /// Whether the last status poll got a fresh response (false = stale/timeout)
    pub status_is_fresh: bool,
}

impl Controller {
    /// Get a serializable snapshot of controller state.
    pub fn snapshot(&self) -> ControllerSnapshot {
        let state = self.state.lock();
        ControllerSnapshot {
            connection: state.connection.clone(),
            status: state.status.clone(),
            welcome_message: state.welcome_message.clone(),
            last_error: state.last_error.clone(),
            pending_alarm: state.pending_alarm,
            status_is_fresh: state.status_is_fresh,
        }
    }
}
