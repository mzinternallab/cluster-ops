pub mod commands;
pub mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        // Tauri commands registered here in later phases
        // .invoke_handler(tauri::generate_handler![
        //     commands::kubeconfig::get_kubeconfig_contexts,
        //     commands::kubeconfig::set_active_context,
        //     commands::pods::list_pods,
        //     commands::kubectl::describe_pod,
        //     commands::logs::get_pod_logs,
        //     commands::ai::analyze_with_ai,
        // ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
