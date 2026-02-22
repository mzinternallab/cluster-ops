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
        .invoke_handler(tauri::generate_handler![
            commands::kubeconfig::get_kubeconfig_contexts,
            commands::kubeconfig::set_active_context,
            commands::kubeconfig::check_cluster_health,
            // commands::pods::list_pods,          — Step 6
            // commands::pods::list_namespaces,     — Step 6
            // commands::kubectl::describe_pod,     — Step 9
            // commands::kubectl::run_kubectl,      — Step 13
            // commands::logs::get_pod_logs,        — Step 9
            // commands::ai::analyze_with_ai,       — Step 11
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
