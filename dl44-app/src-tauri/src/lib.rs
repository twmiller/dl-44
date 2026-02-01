//! DL-44 Laser Control Application
//!
//! Tauri backend providing GRBL device communication and control.

mod commands;
mod grbl;
mod workspace;
mod workspace_commands;

use commands::AppState;
use grbl::Controller;
use workspace_commands::WorkspaceState;
use std::sync::Arc;

pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Create shared controller
    let controller = Arc::new(Controller::new());

    // Create workspace state
    let workspace = Arc::new(WorkspaceState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            controller: controller.clone(),
        })
        .manage(workspace)
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
            // Workspace commands
            workspace_commands::get_workspace,
            workspace_commands::get_workspace_settings,
            workspace_commands::update_workspace_settings,
            workspace_commands::get_documents,
            workspace_commands::get_workspace_bounds,
            workspace_commands::import_document,
            workspace_commands::import_document_bytes,
            workspace_commands::remove_document,
            workspace_commands::update_document_transform,
            workspace_commands::update_document_visibility,
            workspace_commands::reorder_document,
            workspace_commands::clear_workspace,
            workspace_commands::save_workspace_to_file,
            workspace_commands::load_workspace_from_file,
            workspace_commands::get_workspace_file_path,
            workspace_commands::new_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
