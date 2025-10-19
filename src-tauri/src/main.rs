// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            audio::get_audio_state_command,
            audio::set_volume_command,
            audio::set_mute_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
