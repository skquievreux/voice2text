mod audio;
mod transcribe;
mod text_injection;
mod auth;
mod keyboard_hook;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton},
    AppHandle, Manager, State, Emitter,
};


use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub fn write_to_log(_app: &AppHandle, message: &str) {
    let log_msg = format!("[{}] {}\n", Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
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

use rodio::{OutputStream, Sink, Source};
use std::time::Duration;

pub fn play_feedback_sound(frequency: f32, duration_ms: u64) {
    std::thread::spawn(move || {
        // Rodio 0.17: OutputStream::try_default returns (stream, handle)
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let source = rodio::source::SineWave::new(frequency)
                    .take_duration(Duration::from_millis(duration_ms))
                    .amplify(0.80);
                sink.append(source);
                sink.sleep_until_end();
            }
        }
    });
}

struct AppState {
    recorder: Mutex<audio::AudioRecorder>,
    is_recording: Mutex<bool>,
    version: String,
    client_status: Mutex<Option<auth::ClientStatus>>,
    selected_mic: Mutex<Option<String>>,
}

#[tauri::command]
fn get_input_devices() -> Vec<String> {
    audio::AudioRecorder::list_devices()
}

#[tauri::command]
fn set_input_device(name: String, state: State<'_, AppState>) {
    let mut mic_guard = state.selected_mic.lock().unwrap();
    if name == "Default" {
        *mic_guard = None;
    } else {
        *mic_guard = Some(name);
    }
}

#[tauri::command]
fn get_input_device(state: State<'_, AppState>) -> Option<String> {
    state.selected_mic.lock().unwrap().clone()
}

#[tauri::command]
async fn start_recording(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut is_recording_guard = state.is_recording.lock().unwrap();
    if *is_recording_guard { return Ok(()); }

    // Check validity
    let _token = {
        let status_guard = state.client_status.lock().unwrap();
        if let Some(ref s) = *status_guard {
            if s.status == "banned" { return Err("Device is banned.".to_string()); }
            s.token.clone()
        } else {
            return Err("Registering... please wait.".to_string());
        }
    };

    let device_name = state.selected_mic.lock().unwrap().clone();
    let log_dev = device_name.clone().unwrap_or("Default".to_string());
    
    log_info!(&app, "Attempting to start recording with device: {}", log_dev);
    state.recorder.lock().unwrap().start(device_name)?;
    *is_recording_guard = true;
    log_info!(&app, "Recording status: STARTED");
    play_feedback_sound(440.0, 150); // A4 (Start)
    let _ = app.emit("recording-state", true);
    Ok(())
}

#[tauri::command]
async fn stop_recording(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut is_recording_guard = state.is_recording.lock().unwrap();
    if !*is_recording_guard { return Ok(()); }

    let token = {
        let status_guard = state.client_status.lock().unwrap();
        if let Some(ref s) = *status_guard {
            s.token.clone()
        } else {
            return Err("Registering... please wait.".to_string());
        }
    };

    *is_recording_guard = false;
    log_info!(&app, "Recording status: STOPPING...");
    play_feedback_sound(300.0, 100); // Lower tone (Stop)
    let _ = app.emit("recording-state", false);

    let (wav_data, _) = state.recorder.lock().unwrap().stop()?;
    
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match transcribe::send_to_api(&app_handle, wav_data, &token).await {
            Ok(text) => {
                 log_info!(&app_handle, "Transcription success.");
                 let _ = app_handle.emit("transcription-result", text.clone());
                 crate::play_feedback_sound(880.0, 100); // High ping (Success) - User requested feedback
                 match text_injection::inject_text(&text) {
                     Ok(_) => log_info!(&app_handle, "Text Injection: SUCCESS"),
                     Err(e) => log_info!(&app_handle, "Text Injection ERROR: {}", e),
                 }
             }
             Err(e) => {
                 log_info!(&app_handle, "Backend error: {}", e);
                 let _ = app_handle.emit("transcription-error", e);
             }
         }
     });
     
     Ok(())
}

#[tauri::command]
async fn toggle_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let is_recording = *state.is_recording.lock().unwrap();
    if !is_recording {
        start_recording(app, state).await?;
        Ok("started".to_string())
    } else {
        stop_recording(app, state).await?;
        Ok("stopped".to_string())
    }
}

#[tauri::command]
fn get_version(state: State<'_, AppState>) -> String {
    state.version.clone()
}

#[tauri::command]
async fn open_data_folder(_app: AppHandle) -> Result<(), String> {
    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        let log_dir = std::path::Path::new(&appdata).join("Voice2Text").join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        std::process::Command::new("explorer").arg(&log_dir).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_client_status(state: State<'_, AppState>) -> Option<auth::ClientStatus> {
    state.client_status.lock().unwrap().clone()
}

#[tauri::command]
async fn refresh_client_status(app: AppHandle, state: State<'_, AppState>) -> Result<auth::ClientStatus, String> {
    let app_handle = app.clone();
    let res = auth::check_status().await;
    match res {
        Ok(status) => {
             *state.client_status.lock().unwrap() = Some(status.clone());
             log_info!(&app_handle, "Status refreshed: {}", status.status);
             Ok(status)
        },
        Err(e) => {
            log_info!(&app_handle, "Status refresh failed: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
fn get_hw_id() -> String {
    auth::get_hw_id()
}

fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use std::ptr;
        use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        
        unsafe {
            let mut token = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0 {
                let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
                let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
                if GetTokenInformation(token, TokenElevation, &mut elevation as *mut _ as *mut _, size, &mut size) != 0 {
                    return elevation.TokenIsElevated != 0;
                }
            }
        }
    }
    false
}

fn handle_shortcut_toggle(app: &AppHandle, label: &str) {
    let app_handle = app.clone();
    let label_s = label.to_string();
    tauri::async_runtime::spawn(async move {
        let state: State<AppState> = app_handle.state();
        match toggle_recording(app_handle.clone(), state).await {
            Ok(res) => log_info!(&app_handle, "Shortcut {} triggered toggle. New state: {}", label_s, res),
            Err(e) => log_info!(&app_handle, "Shortcut {} toggle error: {}", label_s, e),
        }
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
            selected_mic: Mutex::new(None),
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Registration
            let reg_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                match auth::check_status().await {
                    Ok(status_resp) => {
                        let state: State<AppState> = reg_handle.state();
                        *state.client_status.lock().unwrap() = Some(status_resp.clone());
                        log_info!(&reg_handle, "Client registered. Status: {}", status_resp.status);
                    }
                    Err(e) => log_info!(&reg_handle, "Registration failed: {}", e),
                }
            });

            // Registration Info
            log_info!(&app_handle, "Startup: Is Admin? {}", is_admin());

            // Shortcuts
            let alt_f8 = Shortcut::new(Some(Modifiers::ALT), Code::F8);
            let ctrl_shift_alt_end = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT), Code::End);
            
            match app.global_shortcut().register(alt_f8) {
                Ok(_) => log_info!(&app_handle, "Registered Alt+F8"),
                Err(e) => log_info!(&app_handle, "Failed to register Alt+F8: {}", e),
            }
            
            match app.global_shortcut().register(ctrl_shift_alt_end) {
                Ok(_) => log_info!(&app_handle, "Registered Diagnostic-Key (Ctrl+Shift+Alt+End)"),
                Err(e) => log_info!(&app_handle, "Failed to register Diagnostic-Key: {}", e),
            }

            let _ = app.global_shortcut().on_shortcut(alt_f8, move |app, _, event| {
                if event.state() == ShortcutState::Pressed { 
                    unsafe { winapi::um::utilapiset::Beep(1000, 50); }
                    handle_shortcut_toggle(app, "Alt+F8"); 
                }
            });

            let _ = app.global_shortcut().on_shortcut(ctrl_shift_alt_end, move |app, _, event| {
                if event.state() == ShortcutState::Pressed { 
                    unsafe { winapi::um::utilapiset::Beep(2000, 100); }
                    log_info!(app, "DIAGNOSTIC SHORTCUT TRIGGERED!");
                }
            });

            // Install Low-Level Keyboard Hook (Fallback for blocked hotkeys)
            if let Err(e) = keyboard_hook::install_keyboard_hook() {
                log_info!(&app_handle, "Keyboard hook installation failed: {}", e);
            } else {
                log_info!(&app_handle, "Keyboard hook active (Alt+F8 will work even if blocked)");
                
                // Polling thread to check recording state
                let poll_handle = app_handle.clone();
                std::thread::spawn(move || {
                    let mut was_recording = false;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let is_recording = keyboard_hook::is_recording();
                        
                        if is_recording != was_recording {
                            was_recording = is_recording;
                            let app_clone = poll_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let state: State<AppState> = app_clone.state();
                                if is_recording {
                                    if let Err(e) = start_recording(app_clone.clone(), state).await {
                                        crate::write_to_log(&app_clone, &format!("ERROR: Failed to start recording: {}", e));
                                        unsafe { winapi::um::utilapiset::Beep(200, 300); } // Error Beep
                                    }
                                } else {
                                    let _ = stop_recording(app_clone.clone(), state).await;
                                }
                            });
                        }
                    }
                });
            }

            // Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let logs_i = MenuItem::with_id(app, "logs", "Open Logs", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&logs_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "logs" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = open_data_folder(app_handle).await;
                        });
                    },
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click { button: MouseButton::Left, .. } 
                        | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            toggle_recording, 
            get_version, 
            open_data_folder, 
            auth::fetch_campaigns,
            get_client_status,
            refresh_client_status,
            get_input_devices,
            set_input_device,
            get_input_device,
            get_hw_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
