// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio;
mod audio_observer;

use audio_observer::{VolumeObserverState, start_volume_observer_manual};
use tauri::{Manager, Listener};

fn main() {
    tauri::Builder::default()
        .manage(VolumeObserverState::default())
        .setup(|app| {
            let state = app.state::<VolumeObserverState>();
            start_volume_observer_manual(app.handle().clone(), state)?;

            //TODO：Надо как-то отслеживать завершение работы приложения
            // let stop_state = app.state::<VolumeObserverState>();
            // let app_handle = app.handle();
            // app_handle.listen_global("tauri://close-requested", move |_event| {
            //     if let Some(tx) = stop_state.stop_tx.lock().unwrap().take() {
            //         let _ = tx.send(());
            //         println!("Volume observer stopped before closing");
            //     }
            // });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::get_audio_state_command,
            audio::set_volume_command,
            audio::set_mute_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
