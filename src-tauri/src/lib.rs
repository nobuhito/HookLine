pub mod logic;

use tauri::{AppHandle, Manager, Emitter};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
            let _ = window.set_skip_taskbar(true); // 隠すときはタスクバーからも消す
        } else {
            // [意図] 表示処理の安定化。表示・フォーカス・タスクバー設定を順序立てて実行する。
            // また、ウィンドウサイズを強制することで透明な巨大ウィンドウで覆われる現象を回避。
            let _ = window.set_size(tauri::LogicalSize::new(800.0, 280.0));
            let _ = window.set_skip_taskbar(false); // 表示するときはタスクバーに出す
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.set_always_on_top(true);

            // [意図] OS側のフォーカス処理が遅延するケースがあるため、少し後のタイミングで再度フォーカスを叩く
            let win = window.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                let _ = win.set_focus();
            });
        }
    }
}

#[tauri::command]
fn start_dragging(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn append_to_file(app: AppHandle, path: &str, text: &str) -> Result<(), String> {
    logic::append_to_file(path, text).map_err(|e| {
        let _ = app.emit("backend-error", format!("File append failed: {}", e));
        e
    })
}

#[tauri::command]
fn execute_command(app: AppHandle, cmd_template: &str, text: &str) -> Result<String, String> {
    logic::execute_command(cmd_template, text).map_err(|e| {
        let _ = app.emit("backend-error", format!("Command failed: {}", e));
        e
    })
}

#[tauri::command]
fn save_settings(app: AppHandle, data: &str, config_dir: &str) -> Result<(), String> {
    logic::save_settings(data, config_dir).map_err(|e| {
        let _ = app.emit("backend-error", format!("Settings save failed: {}", e));
        e
    })
}

#[tauri::command]
fn load_settings(app: AppHandle, config_dir: &str) -> Result<String, String> {
    logic::load_settings(config_dir).map_err(|e| {
        let _ = app.emit("backend-error", format!("Settings load failed: {}", e));
        e
    })
}

#[tauri::command]
fn save_history(app: AppHandle, history: &str, config_dir: &str) -> Result<(), String> {
    logic::save_history(history, config_dir).map_err(|e| {
        let _ = app.emit("backend-error", format!("History save failed: {}", e));
        e
    })
}

#[tauri::command]
fn load_history(app: AppHandle, config_dir: &str) -> Result<String, String> {
    logic::load_history(config_dir).map_err(|e| {
        let _ = app.emit("backend-error", format!("History load failed: {}", e));
        e
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init()) 
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![start_dragging, append_to_file, execute_command, save_history, load_history, save_settings, load_settings])
        .setup(|app| {
            // トレイアイコンの設定
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Alt + Space の登録
            let alt_space = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            
            if let Err(e) = app.global_shortcut().on_shortcut(alt_space, move |app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    toggle_window(app);
                }
            }) {
                eprintln!("Failed to register global shortcut: {}", e);
            }

            // プラットフォーム固有のウィンドウ調整
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    // [意図] Windowsでは透明ウィンドウにおいてOS標準の影が境界線として描画されるバグを回避するため、
                    // OSの影を無効化し、CSS側で描画する設計としている。
                    let _ = window.set_shadow(false); // Windowsでは枠線回避のためオフ
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // [意図] macOS/Linuxではネイティブの影描画が適切に機能するため、これを活かす。
                    let _ = window.set_shadow(true); // macOS/Linuxではネイティブの影を活かす
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}