pub mod commands;
pub mod models;

use std::io::Write;
use std::process::Child;
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Holds the kubectl proxy child process so it can be killed on exit.
/// Arc lets us clone out of the tauri State borrow inside the RunEvent::Exit handler.
pub struct KubectlProxy(pub Arc<Mutex<Option<Child>>>);

/// A live exec session.  Both fields must be kept alive together:
/// dropping `child` kills the process; dropping `writer` closes stdin.
pub struct PtySession {
    pub writer: Box<dyn Write + Send>,
    pub child:  Box<dyn portable_pty::Child + Send + Sync>,
}

/// Holds the active exec session so `send_exec_input` can forward keystrokes.
/// Replaced (and previous session dropped) each time a new exec session starts.
pub struct PtyState(pub Mutex<Option<PtySession>>);

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

            // Proxy starts as None — the frontend calls start_kubectl_proxy on mount.
            app.manage(KubectlProxy(Arc::new(Mutex::new(None))));
            // PTY writer starts as None — populated when exec_into_pod is called.
            app.manage(PtyState(Mutex::new(None)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::kubeconfig::get_kubeconfig_contexts,
            commands::kubeconfig::set_active_context,
            commands::kubeconfig::check_cluster_health,
            commands::pods::list_pods,
            commands::pods::list_namespaces,
            commands::pods::delete_pod,
            commands::pods::exec_into_pod,
            commands::pods::send_exec_input,
            commands::kubectl::describe_pod,
            // commands::kubectl::run_kubectl,      — Step 13
            commands::logs::get_pod_logs,
            commands::proxy::start_kubectl_proxy,
            commands::proxy::stop_kubectl_proxy,
            // commands::ai::analyze_with_ai,       — Step 11
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Belt-and-suspenders: kill proxy even if the frontend didn't call stop.
                let arc = app_handle.state::<KubectlProxy>().0.clone();
                if let Ok(mut guard) = arc.lock() {
                    if let Some(mut child) = guard.take() {
                        let _ = child.kill();
                    }
                };
            }
        });
}
