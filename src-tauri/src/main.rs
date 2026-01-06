#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_client;
mod core;
mod platform_windows;
mod storage;
mod ui_bridge;

use crate::core::{AppState, TranslatorCore};
use crate::ui_bridge::{AppCommands, UiBridge};
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tracing_subscriber::EnvFilter;

fn build_tray() -> SystemTray {
    let toggle = CustomMenuItem::new("toggle_detection".to_string(), "开启/暂停划词检测");
    let screenshot = CustomMenuItem::new("screenshot_translate".to_string(), "截图翻译");
    let settings = CustomMenuItem::new("open_settings".to_string(), "打开设置");
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let menu = SystemTrayMenu::new()
        .add_item(toggle)
        .add_item(screenshot)
        .add_item(settings)
        .add_item(quit);
    SystemTray::new().with_menu(menu)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let core = TranslatorCore::new();

    tauri::Builder::default()
        .manage(AppState::new(core))
        .system_tray(build_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let handle = app.handle();
                match id.as_str() {
                    "toggle_detection" => {
                        UiBridge::toggle_detection(&handle);
                    }
                    "screenshot_translate" => {
                        UiBridge::trigger_screenshot(&handle);
                    }
                    "open_settings" => {
                        UiBridge::open_settings(&handle);
                    }
                    "quit" => {
                        handle.exit(0);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            AppCommands::translate,
            AppCommands::save_settings,
            AppCommands::load_settings,
            AppCommands::set_api_key,
            AppCommands::read_api_key,
        ])
        .setup(|app| {
            let app_handle = app.handle();
            UiBridge::start_background(&app_handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
