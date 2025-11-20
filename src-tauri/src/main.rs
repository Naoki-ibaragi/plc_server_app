// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// モジュール宣言
mod types;
mod state;
mod config;
mod plc_commands;
mod tray;
mod data_handler;
mod regist_data_to_db;

use tauri::{
    menu::{Menu,MenuItem,Submenu},
    Manager,
};
use tauri_plugin_single_instance::init as single_instance;

// モジュールからのインポート
use config::{init_socket, add_plc, delete_plc};
use plc_commands::{connect_plc, disconnect_plc};
use state::init_connection_state;
use data_handler::init_database;

fn main() {
    let connection_state = init_connection_state();

    // データベースを初期化し、チャネルの送信側を取得
    let db_channel = match init_database() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(connection_state)
        .manage(db_channel) // DB チャネルを状態として管理
        .invoke_handler(tauri::generate_handler![init_socket, connect_plc, disconnect_plc, add_plc, delete_plc])
        .plugin(single_instance(|app, _args, _cwd| {
            // 既にインスタンスが起動している場合、ウィンドウを表示
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // トレイアイコンをセットアップ
            tray::setup_tray_icon(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
