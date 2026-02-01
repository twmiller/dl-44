//! Serial worker thread for GRBL communication.
//!
//! This module provides a dedicated worker thread that handles all serial I/O.
//! While Tauri command handlers still block waiting for responses, the actual
//! serial I/O is isolated in the worker thread, preventing issues with serial
//! port access from multiple threads and providing centralized timeout handling.
//!
//! Architecture:
//! - Main thread sends requests via mpsc channel
//! - Worker thread processes requests and sends responses via oneshot channels
//! - Worker handles retries, timeouts, and buffer management internally
//! - Response channel has a timeout to prevent indefinite blocking

use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use thiserror::Error;

use super::protocol::{self, Response};
use super::status::MachineStatus;

/// Retry/timeout configuration
pub const DEFAULT_RETRIES: u32 = 2;
pub const DEFAULT_TIMEOUT_MS: u64 = 500;
pub const STATUS_TIMEOUT_MS: u64 = 300;

/// Timeout for waiting on worker response (prevents indefinite blocking)
const RESPONSE_CHANNEL_TIMEOUT_MS: u64 = 5000;

/// Worker errors
#[derive(Error, Debug, Clone)]
pub enum WorkerError {
    #[error("Failed to open port: {0}")]
    OpenFailed(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Command timeout after {attempts} attempts")]
    Timeout { attempts: u32 },

    #[error("GRBL error code {0}")]
    GrblError(u32),

    #[error("GRBL alarm code {0}")]
    Alarm(u32),

    #[error("Worker thread not responding")]
    WorkerDead,
}

/// Response channel type
pub type ResponseTx<T> = std::sync::mpsc::Sender<Result<T, WorkerError>>;

/// Request types sent to the worker
pub enum WorkerRequest {
    /// Connect to a serial port
    Connect {
        port: String,
        baud_rate: u32,
        response_tx: ResponseTx<String>, // Returns welcome message if any
    },

    /// Disconnect from current port
    Disconnect { response_tx: ResponseTx<()> },

    /// Send a command and wait for ok/error (with retries)
    SendCommand {
        command: String,
        retries: u32,
        timeout_ms: u64,
        response_tx: ResponseTx<()>,
    },

    /// Send a real-time command (single byte, no response expected)
    SendRealtime {
        byte: u8,
        response_tx: ResponseTx<()>,
    },

    /// Query status and wait for status report
    QueryStatus {
        timeout_ms: u64,
        response_tx: ResponseTx<StatusQueryResult>,
    },

    /// Shutdown the worker thread
    Shutdown,
}

/// Result of a status query - may include alarm/error seen during polling
#[derive(Debug, Clone)]
pub struct StatusQueryResult {
    pub status: Option<MachineStatus>,
    pub alarm: Option<u32>,
    pub error: Option<u32>,
}

/// Handle to communicate with the serial worker
pub struct WorkerHandle {
    request_tx: Sender<WorkerRequest>,
    thread_handle: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    /// Spawn a new serial worker thread
    pub fn spawn() -> Self {
        let (request_tx, request_rx) = mpsc::channel();

        let thread_handle = thread::Builder::new()
            .name("grbl-serial-worker".into())
            .spawn(move || {
                let mut worker = SerialWorker::new(request_rx);
                worker.run();
            })
            .expect("Failed to spawn serial worker thread");

        Self {
            request_tx,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a request to the worker and wait for response (with timeout).
    ///
    /// Note: This blocks the calling thread until the worker responds or times out.
    /// The benefit is that serial I/O is isolated in the worker thread.
    fn send_request<T, F>(&self, make_request: F) -> Result<T, WorkerError>
    where
        F: FnOnce(ResponseTx<T>) -> WorkerRequest,
    {
        let (response_tx, response_rx) = mpsc::channel();
        let request = make_request(response_tx);

        self.request_tx
            .send(request)
            .map_err(|_| WorkerError::WorkerDead)?;

        // Use recv_timeout to prevent indefinite blocking if worker wedges
        let timeout = Duration::from_millis(RESPONSE_CHANNEL_TIMEOUT_MS);
        match response_rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                log::error!("Worker response timeout - worker may be wedged");
                Err(WorkerError::WorkerDead)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(WorkerError::WorkerDead),
        }
    }

    /// Connect to a serial port
    pub fn connect(&self, port: &str, baud_rate: u32) -> Result<String, WorkerError> {
        self.send_request(|response_tx| WorkerRequest::Connect {
            port: port.to_string(),
            baud_rate,
            response_tx,
        })
    }

    /// Disconnect from current port
    pub fn disconnect(&self) -> Result<(), WorkerError> {
        self.send_request(|response_tx| WorkerRequest::Disconnect { response_tx })
    }

    /// Send a command with default retry/timeout policy
    pub fn send_command(&self, command: &str) -> Result<(), WorkerError> {
        self.send_command_with_policy(command, DEFAULT_RETRIES, DEFAULT_TIMEOUT_MS)
    }

    /// Send a command with custom retry/timeout policy
    pub fn send_command_with_policy(
        &self,
        command: &str,
        retries: u32,
        timeout_ms: u64,
    ) -> Result<(), WorkerError> {
        self.send_request(|response_tx| WorkerRequest::SendCommand {
            command: command.to_string(),
            retries,
            timeout_ms,
            response_tx,
        })
    }

    /// Send a real-time command
    pub fn send_realtime(&self, byte: u8) -> Result<(), WorkerError> {
        self.send_request(|response_tx| WorkerRequest::SendRealtime { byte, response_tx })
    }

    /// Query status (waits for status report or timeout)
    pub fn query_status(&self) -> Result<StatusQueryResult, WorkerError> {
        self.query_status_with_timeout(STATUS_TIMEOUT_MS)
    }

    /// Query status with custom timeout
    pub fn query_status_with_timeout(
        &self,
        timeout_ms: u64,
    ) -> Result<StatusQueryResult, WorkerError> {
        self.send_request(|response_tx| WorkerRequest::QueryStatus {
            timeout_ms,
            response_tx,
        })
    }

    /// Shutdown the worker (called on drop)
    pub fn shutdown(&self) {
        let _ = self.request_tx.send(WorkerRequest::Shutdown);
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Internal worker state
struct SerialWorker {
    request_rx: Receiver<WorkerRequest>,
    connection: Option<SerialConnection>,
}

/// Internal serial connection wrapper
struct SerialConnection {
    port: Box<dyn SerialPort>,
    reader: BufReader<Box<dyn SerialPort>>,
}

impl SerialConnection {
    fn open(path: &str, baud_rate: u32) -> Result<Self, WorkerError> {
        let port = serialport::new(path, baud_rate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(50)) // Short timeout for non-blocking reads
            .open()
            .map_err(|e| WorkerError::OpenFailed(e.to_string()))?;

        let reader_port = port
            .try_clone()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        let reader = BufReader::new(reader_port);

        Ok(Self { port, reader })
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), WorkerError> {
        self.port
            .write_all(data)
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        self.port
            .flush()
            .map_err(|e| WorkerError::Io(e.to_string()))?;
        Ok(())
    }

    fn send_command(&mut self, cmd: &str) -> Result<(), WorkerError> {
        let cmd = if cmd.ends_with('\n') {
            cmd.to_string()
        } else {
            format!("{}\n", cmd)
        };
        self.write_bytes(cmd.as_bytes())
    }

    fn read_line(&mut self) -> Result<Option<String>, WorkerError> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(None),
            Ok(_) => Ok(Some(line.trim().to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(WorkerError::Io(e.to_string())),
        }
    }

    /// Drain all pending input from the serial buffer.
    /// This prevents stale responses from being consumed by subsequent commands.
    fn drain_input(&mut self) -> Vec<Response> {
        let mut responses = Vec::new();
        loop {
            match self.read_line() {
                Ok(Some(line)) if !line.is_empty() => {
                    let resp = protocol::parse_response(&line);
                    log::trace!("Drained: {:?}", resp);
                    responses.push(resp);
                }
                _ => break,
            }
        }
        responses
    }

    fn clear_buffers(&mut self) -> Result<(), WorkerError> {
        self.port
            .clear(serialport::ClearBuffer::All)
            .map_err(|e| WorkerError::Io(e.to_string()))
    }
}

impl SerialWorker {
    fn new(request_rx: Receiver<WorkerRequest>) -> Self {
        Self {
            request_rx,
            connection: None,
        }
    }

    fn run(&mut self) {
        log::info!("Serial worker started");

        loop {
            match self.request_rx.recv() {
                Ok(WorkerRequest::Shutdown) => {
                    log::info!("Serial worker shutting down");
                    break;
                }
                Ok(request) => self.handle_request(request),
                Err(_) => {
                    log::warn!("Request channel closed, worker exiting");
                    break;
                }
            }
        }

        // Clean up connection on exit
        self.connection = None;
        log::info!("Serial worker stopped");
    }

    fn handle_request(&mut self, request: WorkerRequest) {
        match request {
            WorkerRequest::Connect {
                port,
                baud_rate,
                response_tx,
            } => {
                let result = self.handle_connect(&port, baud_rate);
                let _ = response_tx.send(result);
            }

            WorkerRequest::Disconnect { response_tx } => {
                let result = self.handle_disconnect();
                let _ = response_tx.send(result);
            }

            WorkerRequest::SendCommand {
                command,
                retries,
                timeout_ms,
                response_tx,
            } => {
                let result = self.handle_send_command(&command, retries, timeout_ms);
                let _ = response_tx.send(result);
            }

            WorkerRequest::SendRealtime { byte, response_tx } => {
                let result = self.handle_send_realtime(byte);
                let _ = response_tx.send(result);
            }

            WorkerRequest::QueryStatus {
                timeout_ms,
                response_tx,
            } => {
                let result = self.handle_query_status(timeout_ms);
                let _ = response_tx.send(result);
            }

            WorkerRequest::Shutdown => unreachable!(),
        }
    }

    fn handle_connect(&mut self, port: &str, baud_rate: u32) -> Result<String, WorkerError> {
        // Disconnect if already connected
        self.connection = None;

        log::info!("Connecting to {} at {} baud", port, baud_rate);

        let mut conn = SerialConnection::open(port, baud_rate)?;

        // Clear buffers and send soft reset
        let _ = conn.clear_buffers();
        conn.write_bytes(&[protocol::realtime::SOFT_RESET])?;

        // Wait for and collect welcome message
        let start = Instant::now();
        let timeout = Duration::from_millis(1000);
        let mut welcome_message = String::new();

        while start.elapsed() < timeout {
            if let Ok(Some(line)) = conn.read_line() {
                let response = protocol::parse_response(&line);
                if let Response::Welcome(msg) = response {
                    welcome_message = msg;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        self.connection = Some(conn);
        log::info!("Connected successfully");

        Ok(welcome_message)
    }

    fn handle_disconnect(&mut self) -> Result<(), WorkerError> {
        if self.connection.is_none() {
            return Err(WorkerError::NotConnected);
        }

        self.connection = None;
        log::info!("Disconnected");
        Ok(())
    }

    fn handle_send_command(
        &mut self,
        command: &str,
        max_retries: u32,
        timeout_ms: u64,
    ) -> Result<(), WorkerError> {
        let conn = self.connection.as_mut().ok_or(WorkerError::NotConnected)?;

        let timeout = Duration::from_millis(timeout_ms);
        let mut attempts = 0;

        loop {
            attempts += 1;

            // IMPORTANT: Drain any stale responses before each attempt.
            // This prevents late ok/error from a previous attempt being
            // consumed by a retry.
            let stale = conn.drain_input();
            if !stale.is_empty() {
                log::debug!(
                    "Drained {} stale response(s) before attempt {}",
                    stale.len(),
                    attempts
                );
            }

            log::debug!("Sending command (attempt {}): {}", attempts, command.trim());

            // Send the command
            conn.send_command(command)?;

            // Wait for ok/error response
            let start = Instant::now();
            while start.elapsed() < timeout {
                if let Ok(Some(line)) = conn.read_line() {
                    let response = protocol::parse_response(&line);
                    match response {
                        Response::Ok => {
                            log::debug!("Command ok");
                            return Ok(());
                        }
                        Response::Error(code) => {
                            log::warn!("GRBL error {}", code);
                            return Err(WorkerError::GrblError(code));
                        }
                        Response::Alarm(code) => {
                            log::warn!("GRBL alarm {}", code);
                            return Err(WorkerError::Alarm(code));
                        }
                        _ => {
                            // Log but continue waiting (status reports, messages, etc.)
                            log::trace!("Ignored during command wait: {:?}", response);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(5));
            }

            // Timeout - retry if we have attempts left
            if attempts > max_retries {
                log::warn!(
                    "Command timeout after {} attempts: {}",
                    attempts,
                    command.trim()
                );
                return Err(WorkerError::Timeout { attempts });
            }

            log::debug!("Command timeout, retrying...");
        }
    }

    fn handle_send_realtime(&mut self, byte: u8) -> Result<(), WorkerError> {
        let conn = self.connection.as_mut().ok_or(WorkerError::NotConnected)?;
        conn.write_bytes(&[byte])?;
        log::debug!("Sent realtime command: 0x{:02X}", byte);
        Ok(())
    }

    fn handle_query_status(&mut self, timeout_ms: u64) -> Result<StatusQueryResult, WorkerError> {
        let conn = self.connection.as_mut().ok_or(WorkerError::NotConnected)?;

        // Send status query
        conn.write_bytes(&[protocol::realtime::STATUS_QUERY])?;

        // Wait for status report, but also capture any alarm/error we see
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        let mut result = StatusQueryResult {
            status: None,
            alarm: None,
            error: None,
        };

        while start.elapsed() < timeout {
            if let Ok(Some(line)) = conn.read_line() {
                let response = protocol::parse_response(&line);
                match response {
                    Response::Status(report) => {
                        if let Some(status) = MachineStatus::parse(&report) {
                            result.status = Some(status);
                            // Got status, return immediately
                            return Ok(result);
                        }
                    }
                    Response::Alarm(code) => {
                        log::warn!("Alarm {} during status query", code);
                        result.alarm = Some(code);
                        // Continue waiting for status, but record alarm
                    }
                    Response::Error(code) => {
                        log::warn!("Error {} during status query", code);
                        result.error = Some(code);
                        // Continue waiting for status, but record error
                    }
                    _ => {
                        log::trace!("Ignored during status query: {:?}", response);
                    }
                }
            }
            thread::sleep(Duration::from_millis(5));
        }

        // Timeout - return what we have (may include alarm/error but no status)
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_spawn_and_shutdown() {
        let handle = WorkerHandle::spawn();
        // Worker should shutdown cleanly when handle is dropped
        drop(handle);
    }
}
