//! GRBL status report parsing and machine state types.
//!
//! Parses status reports in the format:
//! `<State|MPos:x,y,z|FS:feed,spindle|WCO:x,y,z|Ov:f,r,s|...>`

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Machine operating state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MachineState {
    /// Machine is stationary and ready
    #[default]
    Idle,
    /// Running a G-code program
    Run,
    /// Feed hold active (paused)
    Hold,
    /// Jogging in progress
    Jog,
    /// Alarm state - requires unlock or reset
    Alarm,
    /// Door open / safety interlock
    Door,
    /// Checking G-code (dry run mode)
    Check,
    /// Homing cycle in progress
    Home,
    /// Sleeping
    Sleep,
    /// Unknown state
    Unknown,
}

impl FromStr for MachineState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle states that may have sub-states (e.g., "Hold:0", "Door:1")
        let base = s.split(':').next().unwrap_or(s);

        Ok(match base {
            "Idle" => MachineState::Idle,
            "Run" => MachineState::Run,
            "Hold" => MachineState::Hold,
            "Jog" => MachineState::Jog,
            "Alarm" => MachineState::Alarm,
            "Door" => MachineState::Door,
            "Check" => MachineState::Check,
            "Home" => MachineState::Home,
            "Sleep" => MachineState::Sleep,
            _ => MachineState::Unknown,
        })
    }
}

/// 3D position (X, Y, Z)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Parse from comma-separated values: "x,y,z"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() >= 3 {
            Some(Self {
                x: parts[0].parse().ok()?,
                y: parts[1].parse().ok()?,
                z: parts[2].parse().ok()?,
            })
        } else {
            None
        }
    }
}

/// Override percentages
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Overrides {
    /// Feed rate override (percentage, 100 = normal)
    pub feed: u32,
    /// Rapid rate override (percentage)
    pub rapid: u32,
    /// Spindle/laser power override (percentage)
    pub spindle: u32,
}

/// Accessory state flags
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Accessories {
    pub spindle_cw: bool,
    pub spindle_ccw: bool,
    pub flood_coolant: bool,
    pub mist_coolant: bool,
}

/// Complete machine status from a status report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MachineStatus {
    /// Current machine state
    pub state: MachineState,
    /// Machine position (absolute)
    pub machine_pos: Position,
    /// Work position (with offsets applied)
    pub work_pos: Option<Position>,
    /// Work coordinate offset
    pub work_offset: Option<Position>,
    /// Current feed rate (units/min)
    pub feed_rate: Option<f64>,
    /// Current spindle speed (RPM or S-value)
    pub spindle_speed: Option<f64>,
    /// Override percentages
    pub overrides: Option<Overrides>,
    /// Input pin states
    pub input_pins: Option<String>,
    /// Accessory states
    pub accessories: Option<Accessories>,
    /// Buffer state: (planner blocks available, rx chars available)
    pub buffer: Option<(u32, u32)>,
    /// Line number being executed
    pub line_number: Option<u32>,
}

impl MachineStatus {
    /// Parse a GRBL status report string.
    ///
    /// Format: `<State|MPos:x,y,z|WPos:x,y,z|FS:f,s|Ov:f,r,s|...>`
    pub fn parse(report: &str) -> Option<Self> {
        // Strip angle brackets
        let inner = report.strip_prefix('<')?.strip_suffix('>')?;

        let mut status = MachineStatus::default();
        let mut parts = inner.split('|');

        // First part is always the state
        if let Some(state_str) = parts.next() {
            status.state = state_str.parse().unwrap_or(MachineState::Unknown);
        }

        // Parse remaining fields
        for part in parts {
            if let Some((key, value)) = part.split_once(':') {
                match key {
                    "MPos" => {
                        status.machine_pos = Position::parse(value).unwrap_or_default();
                    }
                    "WPos" => {
                        status.work_pos = Position::parse(value);
                    }
                    "WCO" => {
                        status.work_offset = Position::parse(value);
                    }
                    "FS" => {
                        let vals: Vec<&str> = value.split(',').collect();
                        if !vals.is_empty() {
                            status.feed_rate = vals[0].parse().ok();
                        }
                        if vals.len() > 1 {
                            status.spindle_speed = vals[1].parse().ok();
                        }
                    }
                    "F" => {
                        status.feed_rate = value.parse().ok();
                    }
                    "Ov" => {
                        let vals: Vec<&str> = value.split(',').collect();
                        if vals.len() >= 3 {
                            status.overrides = Some(Overrides {
                                feed: vals[0].parse().unwrap_or(100),
                                rapid: vals[1].parse().unwrap_or(100),
                                spindle: vals[2].parse().unwrap_or(100),
                            });
                        }
                    }
                    "Pn" => {
                        status.input_pins = Some(value.to_string());
                    }
                    "A" => {
                        status.accessories = Some(parse_accessories(value));
                    }
                    "Bf" => {
                        let vals: Vec<&str> = value.split(',').collect();
                        if vals.len() >= 2 {
                            if let (Ok(a), Ok(b)) = (vals[0].parse(), vals[1].parse()) {
                                status.buffer = Some((a, b));
                            }
                        }
                    }
                    "Ln" => {
                        status.line_number = value.parse().ok();
                    }
                    _ => {}
                }
            }
        }

        // Calculate work position from machine position and offset if needed
        if status.work_pos.is_none() {
            if let Some(wco) = status.work_offset {
                status.work_pos = Some(Position {
                    x: status.machine_pos.x - wco.x,
                    y: status.machine_pos.y - wco.y,
                    z: status.machine_pos.z - wco.z,
                });
            }
        }

        Some(status)
    }
}

fn parse_accessories(s: &str) -> Accessories {
    Accessories {
        spindle_cw: s.contains('S'),
        spindle_ccw: s.contains('C'),
        flood_coolant: s.contains('F'),
        mist_coolant: s.contains('M'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_idle_status() {
        let status = MachineStatus::parse("<Idle|MPos:0.000,0.000,0.000|FS:0,0>").unwrap();
        assert_eq!(status.state, MachineState::Idle);
        assert_eq!(status.machine_pos.x, 0.0);
        assert_eq!(status.feed_rate, Some(0.0));
    }

    #[test]
    fn test_parse_run_status() {
        let status =
            MachineStatus::parse("<Run|MPos:10.500,20.300,0.000|FS:1000,255|Ov:100,100,100>")
                .unwrap();
        assert_eq!(status.state, MachineState::Run);
        assert_eq!(status.machine_pos.x, 10.5);
        assert_eq!(status.feed_rate, Some(1000.0));
        assert_eq!(status.spindle_speed, Some(255.0));
        assert!(status.overrides.is_some());
    }

    #[test]
    fn test_parse_with_wco() {
        let status =
            MachineStatus::parse("<Idle|MPos:100.000,50.000,0.000|WCO:10.000,5.000,0.000>").unwrap();
        let work = status.work_pos.unwrap();
        assert_eq!(work.x, 90.0);
        assert_eq!(work.y, 45.0);
    }

    #[test]
    fn test_machine_state_parsing() {
        assert_eq!("Idle".parse::<MachineState>().unwrap(), MachineState::Idle);
        assert_eq!(
            "Hold:0".parse::<MachineState>().unwrap(),
            MachineState::Hold
        );
        assert_eq!(
            "Door:1".parse::<MachineState>().unwrap(),
            MachineState::Door
        );
    }
}
