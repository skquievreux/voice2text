mod audio;
mod transcribe;
mod text_injection;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, Emitter,
};
use cpal::traits::{DeviceTrait, HostTrait};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub fn write_to_log(_app: &AppHandle, message: &str) {
    let log_msg = format!("[{}] {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    
    // Hardcoded path request: %APPDATA%/Voice2Text/logs/app.log
    if let Ok(appdata) = std::env::var("APPDATA") {
        let log_dir = std::path::Path::new(&appdata).join("Voice2Text").join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("app.log");
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file) {
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
    unsafe {
        winapi::um::utilapiset::Beep(frequency, duration);
    }
}

struct AppState {
    recorder: Mutex<audio::AudioRecorder>,
    is_recording: Mutex<bool>,
    version: String,
}

#[tauri::command]
async fn toggle_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut is_recording_guard = state.is_recording.lock().unwrap();
    let is_recording = *is_recording_guard;
    
    if !is_recording {
        // Start recording
        state.recorder.lock().unwrap().start()?;
        *is_recording_guard = true;
        log_info!(&app, "Recording status: STARTED");
        play_feedback_sound(1500, 200); // Higher, clear start "Bling"
        Ok("started".to_string())
    } else {
        // Stop and transcribe
        *is_recording_guard = false;
        log_info!(&app, "Recording status: STOPPING...");
        
        let _app_sound = app.clone();
        tauri::async_runtime::spawn(async move {
            play_feedback_sound(1200, 150);
        });

        log_info!(&app, "Calling recorder.stop()...");
        let (wav_data, max_amp) = state.recorder.lock().unwrap().stop()?;
        log_info!(&app, "Recorder stopped. Data size: {} bytes, Peak: {:.4}", wav_data.len(), max_amp);
        
        // Run transcription in background
        let app_handle = app.clone();
        log_info!(&app_handle, "Spawning transcription task...");
        tauri::async_runtime::spawn(async move {
            match transcribe::send_to_api(&app_handle, wav_data).await {
                Ok(text) => {
                     log_info!(&app_handle, "Transcription received: \"{}\"", text);
                     let _ = app_handle.emit("transcription-result", text.clone());
                     if let Err(e) = text_injection::inject_text(&text) {
                         log_info!(&app_handle, "Injection failure: {}", e);
                     }
                 }
                 Err(e) => {
                     let _ = app_handle.emit("transcription-error", e.clone());
                     log_info!(&app_handle, "Backend error: {}", e);
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
        if cfg!(target_os = "windows") {
             std::process::Command::new("explorer")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn handle_shortcut(app: &AppHandle, label: &str) {
    let app_handle = app.clone();
    log_info!(&app_handle, "Global shortcut triggered: {}", label);
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
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Panic Hook
            std::panic::set_hook(Box::new(move |info| {
                let msg = format!("PANIC: {:?}", info);
                log_info!(&app_handle, "{}", msg);
            }));
            
            let app_handle = app.handle();
            let devices = audio::AudioRecorder::list_devices();
            log_info!(app_handle, "Available Audio Devices: {:?}", devices);
            if let Some(host_device) = cpal::default_host().default_input_device() {
                if let Ok(name) = host_device.name() {
                    log_info!(app_handle, "Active Default Device: {}", name);
                }
            }

            // Register global shortcuts
            let ctrl_f12 = Shortcut::new(Some(Modifiers::CONTROL), Code::F12);
            let f8 = Shortcut::new(None, Code::F8);
            
            match app.global_shortcut().register(ctrl_f12) {
                Ok(_) => log_info!(app_handle, "Shortcut Ctrl+F12 registered successfully"),
                Err(e) => log_info!(app_handle, "Failed to register Ctrl+F12: {}", e),
            }
            match app.global_shortcut().register(f8) {
                Ok(_) => log_info!(app_handle, "Shortcut F8 registered successfully"),
                Err(e) => log_info!(app_handle, "Failed to register F8: {}", e),
            }

            let _ = app.global_shortcut().on_shortcut(ctrl_f12, move |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    handle_shortcut(app, "Ctrl+F12");
                }
            });

            let _ = app.global_shortcut().on_shortcut(f8, move |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    handle_shortcut(app, "F8");
                }
            });

            // Tray Menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            // Tray Icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
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
