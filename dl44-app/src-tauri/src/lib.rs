//! DL-44 Laser Control Application
//!
//! Tauri backend providing GRBL device communication and control.

mod commands;
mod grbl;

use commands::AppState;
use grbl::Controller;
use std::sync::Arc;

pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Create shared controller
    let controller = Arc::new(Controller::new());

    tauri::Builder::default()
        .manage(AppState {
            controller: controller.clone(),
        })
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            commands::list_serial_ports,
            commands::get_baud_rates,
            commands::connect,
            commands::disconnect,
            commands::get_connection_state,
            commands::is_connected,
            // Status commands
            commands::poll_status,
            commands::get_status,
            commands::get_controller_snapshot,
            // Control commands
            commands::home,
            commands::unlock,
            commands::jog,
            commands::jog_cancel,
            commands::feed_hold,
            commands::cycle_start,
            commands::soft_reset,
            // Override commands
            commands::feed_override,
            commands::rapid_override,
            commands::spindle_override,
            // Frame command
            commands::run_frame,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
