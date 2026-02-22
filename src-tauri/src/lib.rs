pub mod commands;
pub mod models;

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Holds the kubectl proxy child process so we can kill it on exit.
/// Arc lets us clone out of the tauri State borrow for the RunEvent::Exit handler.
struct KubectlProxy(Arc<Mutex<Option<Child>>>);

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

            // Spawn `kubectl proxy --port=8001`.
            // All cluster auth is delegated to kubectl; kube-rs talks to localhost.
            let child: Option<Child> = Command::new("kubectl")
                .args(["proxy", "--port=8001", "--keepalive=30s"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();
            app.manage(KubectlProxy(Arc::new(Mutex::new(child))));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::kubeconfig::get_kubeconfig_contexts,
            commands::kubeconfig::set_active_context,
            commands::kubeconfig::check_cluster_health,
            commands::pods::list_pods,
            commands::pods::list_namespaces,
            commands::kubectl::describe_pod,
            // commands::kubectl::run_kubectl,      — Step 13
            commands::logs::get_pod_logs,
            // commands::ai::analyze_with_ai,       — Step 11
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Clone the Arc out of the State borrow so the borrow is dropped
                // before we lock and kill — avoids lifetime conflicts.
                let arc = app_handle.state::<KubectlProxy>().0.clone();
                if let Ok(mut guard) = arc.lock() {
                    if let Some(mut child) = guard.take() {
                        let _ = child.kill();
                    }
                };
            }
        });
}
