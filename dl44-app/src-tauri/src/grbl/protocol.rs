//! GRBL protocol constants, commands, and response parsing.
//!
//! Reference: https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands

/// Default baud rate for GRBL controllers
pub const DEFAULT_BAUD_RATE: u32 = 115200;

/// Common baud rates supported by GRBL devices
pub const SUPPORTED_BAUD_RATES: &[u32] = &[9600, 19200, 38400, 57600, 115200, 230400];

/// Real-time commands (single byte, no newline needed)
pub mod realtime {
    /// Status report query
    pub const STATUS_QUERY: u8 = b'?';
    /// Feed hold (pause)
    pub const FEED_HOLD: u8 = b'!';
    /// Cycle start / resume
    pub const CYCLE_START: u8 = b'~';
    /// Soft reset
    pub const SOFT_RESET: u8 = 0x18;
    /// Safety door
    pub const SAFETY_DOOR: u8 = 0x84;

    /// Feed override: set to 100%
    pub const FEED_OVR_RESET: u8 = 0x90;
    /// Feed override: +10%
    pub const FEED_OVR_COARSE_PLUS: u8 = 0x91;
    /// Feed override: -10%
    pub const FEED_OVR_COARSE_MINUS: u8 = 0x92;
    /// Feed override: +1%
    pub const FEED_OVR_FINE_PLUS: u8 = 0x93;
    /// Feed override: -1%
    pub const FEED_OVR_FINE_MINUS: u8 = 0x94;

    /// Rapid override: set to 100%
    pub const RAPID_OVR_RESET: u8 = 0x95;
    /// Rapid override: set to 50%
    pub const RAPID_OVR_HALF: u8 = 0x96;
    /// Rapid override: set to 25%
    pub const RAPID_OVR_QUARTER: u8 = 0x97;

    /// Spindle override: set to 100%
    pub const SPINDLE_OVR_RESET: u8 = 0x99;
    /// Spindle override: +10%
    pub const SPINDLE_OVR_COARSE_PLUS: u8 = 0x9A;
    /// Spindle override: -10%
    pub const SPINDLE_OVR_COARSE_MINUS: u8 = 0x9B;
    /// Spindle override: +1%
    pub const SPINDLE_OVR_FINE_PLUS: u8 = 0x9C;
    /// Spindle override: -1%
    pub const SPINDLE_OVR_FINE_MINUS: u8 = 0x9D;

    /// Toggle spindle stop
    pub const SPINDLE_STOP_TOGGLE: u8 = 0x9E;
    /// Toggle flood coolant
    pub const COOLANT_FLOOD_TOGGLE: u8 = 0xA0;
    /// Toggle mist coolant
    pub const COOLANT_MIST_TOGGLE: u8 = 0xA1;
}

/// System commands ($ prefix)
pub mod system {
    /// Homing cycle
    pub const HOME: &str = "$H";
    /// Unlock after alarm
    pub const UNLOCK: &str = "$X";
    /// View GRBL settings
    pub const VIEW_SETTINGS: &str = "$$";
    /// View G-code parser state
    pub const VIEW_GCODE_STATE: &str = "$G";
    /// View build info
    pub const VIEW_BUILD_INFO: &str = "$I";
    /// View startup blocks
    pub const VIEW_STARTUP_BLOCKS: &str = "$N";
    /// Check G-code mode (dry run)
    pub const CHECK_MODE: &str = "$C";
}

/// Build a jog command.
///
/// # Arguments
/// * `x`, `y`, `z` - Optional axis distances (in current units)
/// * `feed` - Feed rate in units/min
/// * `incremental` - If true, use G91 (relative); if false, use G90 (absolute)
///
/// # Example
/// ```ignore
/// let cmd = build_jog_command(Some(10.0), None, None, 1000.0, true);
/// assert_eq!(cmd, "$J=G91 X10.000 F1000.000\n");
/// ```
pub fn build_jog_command(
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    feed: f64,
    incremental: bool,
) -> String {
    let mut cmd = String::from("$J=");

    // Motion mode
    cmd.push_str(if incremental { "G91" } else { "G90" });

    // Axis moves
    if let Some(x) = x {
        cmd.push_str(&format!(" X{:.3}", x));
    }
    if let Some(y) = y {
        cmd.push_str(&format!(" Y{:.3}", y));
    }
    if let Some(z) = z {
        cmd.push_str(&format!(" Z{:.3}", z));
    }

    // Feed rate
    cmd.push_str(&format!(" F{:.3}", feed));
    cmd.push('\n');

    cmd
}

/// Jog cancel command (real-time)
pub const JOG_CANCEL: u8 = 0x85;

/// Response types from GRBL
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Command accepted
    Ok,
    /// Error with code
    Error(u32),
    /// Alarm with code
    Alarm(u32),
    /// Status report
    Status(String),
    /// Feedback message [MSG:...]
    Message(String),
    /// Welcome message (Grbl X.Xx ['$' for help])
    Welcome(String),
    /// Settings value ($N=value)
    Setting(u32, String),
    /// Unknown/other line
    Other(String),
}

/// Parse a single line response from GRBL.
pub fn parse_response(line: &str) -> Response {
    let line = line.trim();

    if line == "ok" {
        return Response::Ok;
    }

    if let Some(rest) = line.strip_prefix("error:") {
        if let Ok(code) = rest.parse::<u32>() {
            return Response::Error(code);
        }
    }

    if let Some(rest) = line.strip_prefix("ALARM:") {
        if let Ok(code) = rest.parse::<u32>() {
            return Response::Alarm(code);
        }
    }

    if line.starts_with('<') && line.ends_with('>') {
        return Response::Status(line.to_string());
    }

    if let Some(msg) = line.strip_prefix("[MSG:") {
        if let Some(msg) = msg.strip_suffix(']') {
            return Response::Message(msg.to_string());
        }
    }

    if line.starts_with("Grbl ") {
        return Response::Welcome(line.to_string());
    }

    if let Some(rest) = line.strip_prefix('$') {
        if let Some((num, val)) = rest.split_once('=') {
            if let Ok(n) = num.parse::<u32>() {
                return Response::Setting(n, val.to_string());
            }
        }
    }

    Response::Other(line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jog_command() {
        let cmd = build_jog_command(Some(10.0), None, None, 1000.0, true);
        assert_eq!(cmd, "$J=G91 X10.000 F1000.000\n");

        let cmd = build_jog_command(Some(-5.0), Some(5.0), None, 500.0, false);
        assert_eq!(cmd, "$J=G90 X-5.000 Y5.000 F500.000\n");
    }

    #[test]
    fn test_parse_response() {
        assert_eq!(parse_response("ok"), Response::Ok);
        assert_eq!(parse_response("error:20"), Response::Error(20));
        assert_eq!(parse_response("ALARM:1"), Response::Alarm(1));
        assert!(matches!(
            parse_response("<Idle|MPos:0.000,0.000,0.000>"),
            Response::Status(_)
        ));
    }
}
