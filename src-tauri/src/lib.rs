// 模块声明

mod commands;
#[cfg(target_os = "windows")]
mod constants;
#[cfg(target_os = "windows")]
mod extractor;
#[cfg(target_os = "windows")]
mod extractors;
mod path;
#[cfg(target_os = "windows")]
mod shortcut;
mod sources;
mod types;

use commands::*;
use tauri::{AppHandle, Manager};

// Windows 入口，注册所有Windows专用命令
#[cfg(target_os = "windows")]
pub fn run() {
    // 在 Windows 启动时预初始化 COM，防止任务栏图标因 COM 引用计数归零而消失
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_single_instance::init(|app, _, _cwd| {
          show_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_desktop_icons,
            get_desktop_icons_with_method,
            get_available_icon_methods,
            // 软件来源命令
            get_icons_from_source,
            get_all_source_icons,
            get_icons_from_multiple_sources,
            get_available_sources
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// macOS 入口，只注册跨平台与macOS相关命令
#[cfg(target_os = "macos")]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_single_instance::init(|app, _, _cwd| {
          show_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_available_icon_methods,
            // 软件来源命令（macOS）
            get_icons_from_source,
            get_all_source_icons,
            get_icons_from_multiple_sources,
            get_available_sources,
            reveal_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window(app: &AppHandle) {
  let windows = app.webview_windows();

  let webview_window = windows.values().next().expect("Sorry, no window found");
  webview_window
    .unminimize()
    .expect("Can't Bring Window to unminimize");
  webview_window.show().expect("Can't Bring Window to show");
  webview_window
    .set_focus()
    .expect("Can't Bring Window to Focus");
}
