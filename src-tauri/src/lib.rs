mod command;
mod error;
mod model;
mod crawl;
mod database;
mod utils;
mod indicator;
mod task;

use std::path::PathBuf;
use tauri::{Listener, Manager};
use tauri::path::BaseDirectory;
use crate::command::setup_global_monitor;
use crate::command::kline::{ sync_kline_data, un_sync_kline_data,query_kline_hist};
use crate::command::stock::{query_stock_list,fuzzy_query};
use crate::model::{AppState, APP_HANDLE};
use crate::model::c2s::{C2S_START_GLOBAL_EVENT, InitializePayload};
use crate::command::call_my_sidecar;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .manage(AppState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new()
            .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
            .level(log::LevelFilter::Debug)
            .level_for("sqlx::query", log::LevelFilter::Info)
            .build())
        .setup(|app|{
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }


            let resource_dir = app.path().resolve("", BaseDirectory::Resource)?;
            #[cfg(dev)]
            let resource_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            app.manage(AppState::new(resource_dir));
            //全局使用
            APP_HANDLE.set(app.handle().to_owned()).unwrap();
            
            {
                let handler_clone = app.handle().to_owned();
                app.once(C2S_START_GLOBAL_EVENT, move |event| {
                    let payload = event.payload();
                    let parsed_payload: InitializePayload = serde_json::from_str(payload)
                        .expect("Could not parse  payload");
                    tauri::async_runtime::spawn(async move {
                        let _result = setup_global_monitor(&handler_clone, parsed_payload).await;
                    });
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, sync_kline_data, un_sync_kline_data,
            fuzzy_query, query_stock_list, call_my_sidecar,
            query_kline_hist,
          ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}




