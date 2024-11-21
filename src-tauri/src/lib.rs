mod commands;
use commands::*;
use tauri::{Builder, Listener};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            app.once("front-ready", move |_| {
                let handle_ = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = setup_db(&handle_).await;
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_todo,
            get_todos,
            update_todo,
            delete_todo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
