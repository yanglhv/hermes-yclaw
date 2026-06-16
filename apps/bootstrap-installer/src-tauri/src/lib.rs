//! Hermes Setup — Tauri entrypoint.
//!
//! Spawns a single window pointed at the React frontend (apps/bootstrap-installer/src/).
//! All install-time work lives in `bootstrap.rs` and is invoked through the Tauri
//! commands registered at the bottom of `run()`.
//!
//! The Windows-subsystem strip lives on the binary crate (src/main.rs), not
//! here — a crate-level attribute on a lib doesn't propagate to the linker
//! flags of the executable that consumes it.

mod app;
mod bootstrap;
mod events;
mod install_script;
mod launcher;
mod powershell;
mod paths;
mod update;

use std::sync::Arc;
use tokio::sync::Mutex;

/// How the installer was invoked. Resolved once from the process args in
/// `run()` and exposed to the frontend via `get_mode` so it can route to the
/// install flow (first-run onboarding) or the update flow (driven by the
/// desktop app handing off via `Hermes-Setup.exe --update`).
///
/// Bare launch (double-click, first-run) => Install.
/// `--update` (spawned by the desktop's "Update" button) => Update.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Install,
    Update,
}

impl AppMode {
    /// Resolve the mode from an argument iterator. Anything containing the
    /// `--update` flag selects Update; otherwise Install. Kept arg-iterator
    /// generic (not reading `std::env` directly) so it's unit-testable.
    pub fn from_args<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for a in args {
            if a.as_ref() == "--update" {
                return AppMode::Update;
            }
        }
        AppMode::Install
    }
}

/// Returns true when the args request a forced installer UI (repair/reinstall)
/// via `--reinstall` or `--repair`, which overrides the macOS launcher
/// fast-path so a broken install can be repaired. Arg-iterator generic so it's
/// unit-testable, mirroring `AppMode::from_args`. Independent of mode selection:
/// these flags never flip Install<->Update.
pub fn force_setup_from_args<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|a| a.as_ref() == "--reinstall" || a.as_ref() == "--repair")
}

/// Process-wide install state, shared across Tauri commands.
///
/// The bootstrap is a one-shot, single-tenant process — we only need one
/// of these per window. `Arc<Mutex<...>>` lets command handlers grab it
/// without lifetime gymnastics.
pub struct AppState {
    pub bootstrap: Mutex<Option<bootstrap::BootstrapHandle>>,
    /// How this process was launched (install vs update). Immutable for the
    /// lifetime of the process; read by the `get_mode` command.
    pub mode: AppMode,
}

impl AppState {
    fn new(mode: AppMode) -> Self {
        Self {
            bootstrap: Mutex::new(None),
            mode,
        }
    }
}

pub fn launch_args_from_args<I, S>(args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    while let Some(a) = iter.next() {
        if a.as_ref() == "--launch" {
            return iter.next().map(|s| s.as_ref().to_string());
        }
    }
    None
}

/// Frontend → Rust: which flow should the UI render?
#[tauri::command]
fn get_mode(state: tauri::State<'_, Arc<AppState>>) -> AppMode {
    state.mode
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tracing → bootstrap-installer.log under HERMES_HOME/logs/ so install
    // failures leave a trail for support. Console output also goes here in
    // debug builds.
    let _guard = paths::init_logging();

    let mode = AppMode::from_args(std::env::args().skip(1));
    // Escape hatch: `--reinstall`/`--repair` forces the installer UI even when
    // Hermes is already installed, so users can re-run setup to repair a broken
    // install instead of the launcher fast path silently relaunching the app.
    let force_setup = force_setup_from_args(std::env::args().skip(1));
    tracing::info!(?mode, force_setup, "Hermes installer starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(AppState::new(mode)))
        .manage(launcher::commands::load_initial_state())
        .setup(move |app| {
            use tauri::Manager;
            let args: Vec<String> = std::env::args().skip(1).collect();
            let launch_target = launch_args_from_args(args.iter().map(String::as_str));
            let settings_requested = args.iter().any(|a| a == "--settings");

            let state: tauri::State<std::sync::Arc<launcher::commands::LauncherStateHandle>> =
                app.state();
            let launcher_state = state.0.lock().unwrap().clone();
            let kind = launcher::silent::classify_launch_state(
                &launcher_state,
                args.iter().map(String::as_str),
            );

            // CLI dispatch: non-UI paths that short-circuit before window creation
            match kind {
                launcher::silent::LaunchKind::FirstInstall => {
                    tracing::info!("CLI dispatch: FirstInstall → run_silent_default");
                }
                launcher::silent::LaunchKind::Silent => {
                    tracing::info!("CLI dispatch: Silent → run_silent_default");
                }
                launcher::silent::LaunchKind::Launch => {
                    tracing::info!(?launch_target, "CLI dispatch: Launch");
                    if let Some(id) = &launch_target {
                        if let Some(inst) = launcher_state.installed.get(id) {
                            let descriptor = crate::app::AppDescriptor::literal_hermes();
                            if launcher::launch::launch_and_exit(&descriptor, inst).is_ok() {
                                std::thread::sleep(std::time::Duration::from_millis(150));
                                app.handle().exit(0);
                                return Ok(());
                            }
                        }
                    }
                }
                launcher::silent::LaunchKind::Update => {
                    tracing::info!("CLI dispatch: Update → fall through to UI");
                }
                launcher::silent::LaunchKind::Settings => {
                    tracing::info!("CLI dispatch: Settings → fall through to UI");
                }
            }

            // Portable silent default: a bare ("Install") launch when Hermes is
            // already installed should silently launch the app and exit.
            if mode == AppMode::Install
                && !force_setup
                && launch_target.is_none()
                && !settings_requested
            {
                let id = launcher_state
                    .default_app_id
                    .clone()
                    .unwrap_or_else(|| "hermes".into());
                if let Some(inst) = launcher_state.installed.get(&id) {
                    let descriptor = crate::app::AppDescriptor::literal_hermes();
                    if launcher::launch::launch_and_exit(&descriptor, inst).is_ok() {
                        std::thread::sleep(std::time::Duration::from_millis(150));
                        app.handle().exit(0);
                        return Ok(());
                    }
                }
            }

            // macOS fast path: double-click on Hermes.app launches installed app
            // without showing the installer window.
            if cfg!(target_os = "macos") && mode == AppMode::Install && !force_setup {
                let install_root = paths::hermes_home().join("hermes-agent");
                if bootstrap::hermes_is_installed(&install_root) {
                    match bootstrap::spawn_installed_desktop(&install_root) {
                        Ok(()) => {
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            tracing::info!(
                                "hermes already installed — relaunched desktop; exiting installer"
                            );
                            app.handle().exit(0);
                            return Ok(());
                        }
                        Err(err) => {
                            tracing::warn!(
                                ?err,
                                "relaunch of installed desktop failed; showing installer UI"
                            );
                        }
                    }
                }
            }
            // First run / repair install, or Update mode: reveal the UI.
            match app.get_webview_window("main") {
                Some(win) => {
                    if let Err(err) = win.show() {
                        tracing::error!(?err, "failed to show main installer window");
                    }
                }
                None => {
                    tracing::error!("main installer window not found; installer UI will not appear");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Mode (install vs update)
            get_mode,
            // Bootstrap lifecycle
            bootstrap::start_bootstrap,
            bootstrap::cancel_bootstrap,
            bootstrap::get_bootstrap_status,
            // Update lifecycle
            update::start_update,
            // Hand-off
            bootstrap::launch_hermes_desktop,
            // Diagnostics
            paths::get_log_path,
            paths::get_hermes_home,
            paths::open_log_dir,
            // Launcher commands
            launcher::commands::list_available_apps,
            launcher::commands::get_app,
            launcher::commands::get_launcher_state,
            launcher::commands::get_launcher_config,
            launcher::commands::save_launcher_state,
            launcher::commands::save_launcher_config,
            launcher::commands::list_catalog_apps,
            launcher::commands::probe_network,
            launcher::commands::set_default_app,
            launcher::commands::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hermes Setup");
}

#[cfg(test)]
mod tests {
    use super::{force_setup_from_args, AppMode, launch_args_from_args};

    #[test]
    fn bare_args_are_install() {
        assert_eq!(AppMode::from_args(Vec::<String>::new()), AppMode::Install);
        assert_eq!(AppMode::from_args(["--foo", "bar"]), AppMode::Install);
    }

    #[test]
    fn update_flag_selects_update() {
        assert_eq!(AppMode::from_args(["--update"]), AppMode::Update);
        assert_eq!(
            AppMode::from_args(["--something", "--update", "--else"]),
            AppMode::Update
        );
    }

    #[test]
    fn reinstall_and_repair_flags_force_setup() {
        assert!(force_setup_from_args(["--reinstall"]));
        assert!(force_setup_from_args(["--repair"]));
        assert!(force_setup_from_args(["--foo", "--repair", "--bar"]));
    }

    #[test]
    fn bare_or_unrelated_args_do_not_force_setup() {
        assert!(!force_setup_from_args(Vec::<String>::new()));
        assert!(!force_setup_from_args(["--foo", "bar"]));
        // --update must not be mistaken for a force-setup flag.
        assert!(!force_setup_from_args(["--update"]));
    }

    #[test]
    fn force_setup_flags_do_not_affect_mode_selection() {
        // The repair flags must never flip Install<->Update.
        assert_eq!(AppMode::from_args(["--reinstall"]), AppMode::Install);
        assert_eq!(AppMode::from_args(["--repair"]), AppMode::Install);
        assert_eq!(
            AppMode::from_args(["--update", "--reinstall"]),
            AppMode::Update
        );
    }

    #[test]
    fn cli_mode_is_decoded_from_args() {
        let args = vec!["hermes-bootstrap".to_string(), "--launch".to_string()];
        let state = crate::app::LauncherState::default();
        let kind = crate::launcher::silent::classify_launch_state(&state, args);
        assert!(matches!(kind, crate::launcher::silent::LaunchKind::Launch));
    }

    #[test]
    fn parse_launch_args_returns_some_for_dash_launch() {
        assert_eq!(
            super::launch_args_from_args(["--launch", "myapp"]),
            Some("myapp".to_string())
        );
        assert_eq!(super::launch_args_from_args(Vec::<String>::new()), None);
        assert_eq!(super::launch_args_from_args(["--settings"]), None);
    }

    #[test]
    fn get_launcher_state_returns_result() {
        let result = crate::launcher::state::load_launcher_state();
        assert!(result.is_ok());
    }
}
