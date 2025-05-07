// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Builder;
mod ws_server;

fn main() {
    Builder::default()
    .invoke_handler(
        tauri::generate_handler![
            ws_server::start_ws_server,
            ws_server::broadcast_message,
            ws_server::greet
        ]
    )
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
    // ws_pc_lib::run()
}
