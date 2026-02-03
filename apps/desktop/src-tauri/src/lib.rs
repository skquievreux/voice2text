mod audio;
mod transcribe;
mod text_injection;

use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, State,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

struct AppState {
    recorder: Mutex<audio::AudioRecorder>,
    is_recording: Mutex<bool>,
}

#[tauri::command]
async fn toggle_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut is_recording = state.is_recording.lock().unwrap();
    
    if !*is_recording {
        // Start recording
        state.recorder.lock().unwrap().start()?;
        *is_recording = true;
        println!("Recording started...");
        Ok("started".to_string())
    } else {
        // Stop and transcribe
        *is_recording = false;
        println!("Recording stopped, transcribing...");
        let wav_data = state.recorder.lock().unwrap().stop()?;
        
        // Run transcription in background
        let app_handle = app.clone();
        tauri::async_runtime::spawn(async move {
            match transcribe::send_to_api(wav_data).await {
                Ok(text) => {
                    println!("Transcription: {}", text);
                    if let Err(e) = text_injection::inject_text(&text) {
                        eprintln!("Injection error: {}", e);
                    }
                }
                Err(e) => eprintln!("Transcription error: {}", e),
            }
        });
        
        Ok("stopped".to_string())
    }
}

fn handle_shortcut<R: Runtime>(app: &AppHandle<R>) {
    let state: State<AppState> = app.state();
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = toggle_recording(app_handle, state).await;
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
        })
        .setup(|app| {
            // Register global shortcut Ctrl+Shift+V
            let ctrl_shift_v = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
            app.global_shortcut().register(ctrl_shift_v)?;

            app.on_shortcut(ctrl_shift_v, move |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    handle_shortcut(app);
                }
            })?;

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
                .on_tray_icon_event(|window, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(window) = window.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![toggle_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
