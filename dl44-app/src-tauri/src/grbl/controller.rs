//! High-level GRBL controller coordinating serial worker and state.
//!
//! The controller maintains machine state and delegates all serial I/O
//! to a dedicated worker thread. While command handlers block waiting for
//! the worker response, the actual serial I/O is isolated, preventing
//! issues with port access and providing centralized timeout handling.

use parking_lot::Mutex;
use std::sync::Arc;
use thiserror::Error;

use super::protocol;
use super::serial::PortInfo;
use super::status::{MachineState, MachineStatus};
use super::worker::{WorkerError, WorkerHandle};

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
            WorkerError::WorkerDead => ControllerError::Internal("Worker thread not responding".into()),
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
    /// Alarm code if device entered alarm during polling
    pending_alarm: Option<u32>,
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

                // Update status if we got one
                if let Some(status) = result.status {
                    state.status = status;
                }

                // Record alarm if we saw one during polling
                if let Some(alarm_code) = result.alarm {
                    state.pending_alarm = Some(alarm_code);
                    state.last_error = Some(format!("ALARM:{}", alarm_code));
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
                Err(e.into())
            }
        }
    }

    /// Get cached machine status (without polling).
    pub fn status(&self) -> MachineStatus {
        self.state.lock().status.clone()
    }

    /// Check and clear pending alarm (returns alarm code if one was detected).
    pub fn take_pending_alarm(&self) -> Option<u32> {
        self.state.lock().pending_alarm.take()
    }

    /// Send home command.
    pub fn home(&self) -> Result<(), ControllerError> {
        self.send_command(protocol::system::HOME)
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
        }

        result
    }

    /// Send a command with retry/timeout policy.
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

/// Serializable snapshot of controller state for the UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ControllerSnapshot {
    pub connection: ConnectionState,
    pub status: MachineStatus,
    pub welcome_message: Option<String>,
    pub last_error: Option<String>,
    /// Pending alarm code (if alarm detected during polling)
    pub pending_alarm: Option<u32>,
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
        }
    }
}
