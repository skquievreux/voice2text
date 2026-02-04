mod audio;
mod transcribe;
mod text_injection;
mod auth;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, Emitter,
};


use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub fn write_to_log(_app: &AppHandle, message: &str) {
    let log_msg = format!("[{}] {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    if let Ok(appdata) = std::env::var("APPDATA") {
        let log_dir = std::path::Path::new(&appdata).join("Voice2Text").join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("app.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
            let _ = file.write_all(log_msg.as_bytes());
        }
    }
}

macro_rules! log_info {
    ($app:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("INFO: {}", msg);
        write_to_log($app, &msg)
    }}
}

#[cfg(windows)]
fn play_feedback_sound(frequency: u32, duration: u32) {
    unsafe { winapi::um::utilapiset::Beep(frequency, duration); }
}

struct AppState {
    recorder: Mutex<audio::AudioRecorder>,
    is_recording: Mutex<bool>,
    version: String,
    client_status: Mutex<Option<auth::ClientStatus>>,
}

#[tauri::command]
async fn toggle_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut is_recording_guard = state.is_recording.lock().unwrap();
    let is_recording = *is_recording_guard;
    
    // Check validity
    let token = {
        let status_guard = state.client_status.lock().unwrap();
        if let Some(ref s) = *status_guard {
            if s.status == "banned" {
                return Err("Device is banned.".to_string());
            }
            s.token.clone()
        } else {
            return Err("Registering... please wait.".to_string());
        }
    };

    if !is_recording {
        state.recorder.lock().unwrap().start()?;
        *is_recording_guard = true;
        log_info!(&app, "Recording status: STARTED");
        play_feedback_sound(1500, 100);
        Ok("started".to_string())
    } else {
        *is_recording_guard = false;
        log_info!(&app, "Recording status: STOPPING...");
        play_feedback_sound(1200, 80);

        let (wav_data, _) = state.recorder.lock().unwrap().stop()?;
        
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            match transcribe::send_to_api(&app_handle, wav_data, &token).await {
                Ok(text) => {
                     log_info!(&app_handle, "Transcription success.");
                     let _ = app_handle.emit("transcription-result", text.clone());
                     let _ = text_injection::inject_text(&text);
                 }
                 Err(e) => {
                     log_info!(&app_handle, "Backend error: {}", e);
                     let _ = app_handle.emit("transcription-error", e);
                 }
             }
         });
         
         Ok("stopped".to_string())
     }
}

#[tauri::command]
fn get_version(state: State<'_, AppState>) -> String {
    state.version.clone()
}

#[tauri::command]
async fn open_data_folder(_app: AppHandle) -> Result<(), String> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let log_dir = std::path::Path::new(&appdata).join("Voice2Text").join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        std::process::Command::new("explorer").arg(&log_dir).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn handle_shortcut(app: &AppHandle, _label: &str) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let state: State<AppState> = app_handle.state();
        let _ = toggle_recording(app_handle.clone(), state).await;
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(AppState {
            recorder: Mutex::new(audio::AudioRecorder::new()),
            is_recording: Mutex::new(false),
            version: env!("CARGO_PKG_VERSION").to_string(),
            client_status: Mutex::new(None),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Registration
            tauri::async_runtime::spawn(async move {
                match auth::check_status().await {
                    Ok(status_resp) => {
                        let state: State<AppState> = app_handle.state();
                        *state.client_status.lock().unwrap() = Some(status_resp.clone());
                        log_info!(&app_handle, "Client registered. Status: {}", status_resp.status);
                    }
                    Err(e) => log_info!(&app_handle, "Registration failed: {}", e),
                }
            });

            // Shortcuts
            let ctrl_f12 = Shortcut::new(Some(Modifiers::CONTROL), Code::F12);
            let f8 = Shortcut::new(None, Code::F8);
            let _ = app.global_shortcut().register(ctrl_f12);
            let _ = app.global_shortcut().register(f8);
            let _ = app.global_shortcut().on_shortcut(ctrl_f12, move |app, _, event| {
                if event.state() == ShortcutState::Pressed { handle_shortcut(app, "Ctrl+F12"); }
            });
            let _ = app.global_shortcut().on_shortcut(f8, move |app, _, event| {
                if event.state() == ShortcutState::Pressed { handle_shortcut(app, "F8"); }
            });

            // Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let logs_i = MenuItem::with_id(app, "logs", "Open Logs", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&logs_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "logs" => { let _ = open_data_folder(app.clone()); },
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![toggle_recording, get_version, open_data_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
