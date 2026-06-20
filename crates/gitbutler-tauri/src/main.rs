#![cfg_attr(
    all(windows, not(test), not(debug_assertions)),
    windows_subsystem = "windows"
)]
// FIXME(qix-): Stuff we want to fix but don't have a lot of time for.
// FIXME(qix-): PRs welcome!
#![allow(
    clippy::used_underscore_binding,
    clippy::module_name_repetitions,
    clippy::struct_field_names,
    clippy::too_many_lines
)]

use std::sync::Arc;

use anyhow::{Context, bail};
#[cfg(feature = "irc")]
use but_irc::IrcManager;
use but_settings::AppSettingsWithDiskSync;
#[cfg(feature = "irc")]
use gitbutler_tauri::irc;
use gitbutler_tauri::{WindowState, broadcaster::Broadcaster, csp::csp_with_extras, logs, menu};
use tauri::{Emitter, Manager, generate_context};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::Mutex;

#[cfg(feature = "irc")]
/// Return a copy of `irc` with `connection.enabled` forced to `false` when
/// the IRC feature flag is off. This lets the existing reconciliation logic
/// treat "flag turned off" the same as "user disabled the connection".
fn effective_irc(
    irc: &but_settings::app_settings::IrcSettings,
    feature_enabled: bool,
) -> but_settings::app_settings::IrcSettings {
    if feature_enabled {
        irc.clone()
    } else {
        let mut copy = irc.clone();
        copy.connection.enabled = false;
        copy
    }
}

fn main() -> anyhow::Result<()> {
    but_api::panic_capture::install_panic_hook();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    #[cfg(feature = "builtin-but")]
    {
        if but::is_executed_as_but()? {
            but_askpass::disable();
            return runtime.block_on(but::handle_args(std::env::args_os()));
        }
    }
    let performance_logging = std::env::var_os("GITBUTLER_PERFORMANCE_LOG").is_some();
    let tauri_debug_logging = std::env::var_os("GITBUTLER_TAURI_DEBUG_LOG").is_some();

    let mut tauri_context = generate_context!();
    but_secret::secret::set_application_namespace(&tauri_context.config().identifier);

    // Set the macOS notification bundle ID so notifications appear as GitButler.
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = notify_rust::set_application(&tauri_context.config().identifier) {
            tracing::warn!(error = %e, "Failed to set notification application");
        }
    }

    let config_dir = but_path::app_config_dir().expect("missing config dir");
    std::fs::create_dir_all(&config_dir).expect("failed to create config dir");
    let custom_settings = cfg!(feature = "packaged-but-distribution")
        .then(but_settings::customization::packaged_but_binary);
    // While it serves a function, this behavior is sub-optimal. The proper solution is to decouple:
    // - Checking for updates from
    // - Performing an update
    // This way people can be informed that there is an update even if self-updating is not possible (i.e. installed via package manager).
    let custom_settings = if cfg!(feature = "disable-auto-updates") {
        but_settings::customization::merge_two(
            but_settings::customization::disable_auto_update_checks(),
            custom_settings,
        )
        .into()
    } else {
        custom_settings
    };
    let mut app_settings =
        AppSettingsWithDiskSync::new_with_customization(config_dir.clone(), custom_settings)
            .expect("failed to create app settings");

    if let Ok(updated_csp) = csp_with_extras(
        tauri_context.config().app.security.csp.as_ref().cloned(),
        &app_settings,
    ) {
        tauri_context.config_mut().app.security.csp = updated_csp;
    };

    if let Some(project_to_open) =
        std::env::var_os("GITBUTLER_PROJECT_DIR").map(std::path::PathBuf::from)
    {
        bail!(
            "GUI says: how do we tell the frontend to open: {}? \
               We could figure out the project-ID while that's important, and pass it along somehow",
            project_to_open.display()
        );
    }
    let (app_data_dir, app_cache_dir, app_log_dir) = (
        but_path::app_data_dir()?,
        but_path::app_cache_dir()?,
        but_path::app_log_dir()?,
    );
    std::fs::create_dir_all(&app_data_dir).context("failed to create app data dir")?;
    std::fs::create_dir_all(&app_cache_dir).context("failed to create app cache dir")?;
    std::fs::create_dir_all(&app_log_dir).context("failed to create app log dir")?;

    let tokio_debug = matches!(std::env::var("GITBUTLER_TOKIO_DEBUG").as_deref(), Ok("1"));
    let app_settings_for_menu = app_settings.clone();
    runtime.block_on(async {
        tauri::async_runtime::set(tokio::runtime::Handle::current());

        let log = tauri_plugin_log::Builder::default()
            .target(Target::new(TargetKind::LogDir {
                file_name: Some("ui-logs".to_string()),
            }))
            .level(if tauri_debug_logging {
                tauri_plugin_log::log::LevelFilter::Debug
            } else {
                tauri_plugin_log::log::LevelFilter::Error
            });

        let builder = tauri::Builder::default()
            .setup(move |tauri_app| {
                let window = gitbutler_tauri::window::create(
                    tauri_app.handle(),
                    "main",
                    "index.html".into(),
                )
                .expect("Failed to create window");

                // TODO(mtsgrd): Is there a better way to disable devtools in E2E tests?
                #[cfg(debug_assertions)]
                if tauri_app.config().product_name != Some("GitButler Test".to_string()) {
                    window.open_devtools();
                }

                let app_handle = tauri_app.handle();

                logs::init(app_handle, &app_log_dir, performance_logging, tokio_debug);

                but_action::cli::auto_fix_broken_but_cli_symlink();
                inherit_interactive_login_shell_environment_if_not_launched_from_terminal();
                migrate_projects().ok();

                tracing::info!(
                    "system git executable for fetch/push: {git:?}",
                    git = gix::path::env::exe_invocation(),
                );
                if cfg!(windows) {
                    tracing::info!("system git bash: {bash:?}", bash = gix::path::env::shell());
                } else {
                    tracing::info!("SHELL env: {var:?}", var = std::env::var_os("SHELL"));
                }

                but_askpass::init({
                    let handle = app_handle.clone();
                    move |event| {
                        handle
                            .emit("git_prompt", event)
                            .expect("tauri event emission doesn't fail in practice")
                    }
                });

                tracing::info!(version = %app_handle.package_info().version,
                                   name = %app_handle.package_info().name, "starting app");

                app_handle.manage(WindowState::new(app_handle.clone()));
                #[cfg(feature = "irc")]
                {
                    let irc_manager = IrcManager::new();
                    app_handle.manage(but_irc::WorkingFilesBroadcast::new(irc_manager.clone()));
                    app_handle.manage(irc_manager);
                }

                // Track previous effective IRC settings for diffing on changes.
                // "Effective" means connection.enabled is forced false when the
                // feature flag is off, so toggling the flag also disconnects.
                #[cfg(feature = "irc")]
                let prev_irc_settings = std::sync::Mutex::new(
                    app_settings
                        .get()
                        .ok()
                        .map(|s| effective_irc(&s.irc, s.feature_flags.irc)),
                );

                app_settings.watch_in_background({
                    let app_handle = app_handle.clone();
                    move |app_settings| {
                        #[cfg(feature = "irc")]
                        {
                            let new_irc =
                                effective_irc(&app_settings.irc, app_settings.feature_flags.irc);
                            if let Ok(mut prev) = prev_irc_settings.lock() {
                                if let Some(old_irc) = prev.as_ref()
                                    && old_irc != &new_irc
                                {
                                    gitbutler_tauri::irc_lifecycle::on_settings_changed(
                                        &app_handle,
                                        old_irc,
                                        &new_irc,
                                    );
                                }
                                *prev = Some(new_irc);
                            }
                        }

                        gitbutler_tauri::ChangeForFrontend::from(app_settings).send(&app_handle)
                    }
                })?;

                let broadcaster = Arc::new(Mutex::new(Broadcaster::new()));

                let (send, mut recv) = tokio::sync::mpsc::unbounded_channel();
                let broadcaster2 = broadcaster.clone();
                tokio::spawn(async move {
                    broadcaster2
                        .lock()
                        .await
                        .register_sender(&uuid::Uuid::new_v4(), send)
                });

                let window2 = window.clone();
                std::thread::spawn(move || {
                    while let Some(message) = recv.blocking_recv() {
                        window2.emit(&message.name, message.payload).unwrap();
                    }
                });

                let archival = but_feedback::Archival {
                    cache_dir: app_cache_dir.clone(),
                    logs_dir: app_log_dir.clone(),
                };
                app_handle.manage(archival);
                app_handle.manage(app_settings);

                // Auto-connect IRC connections based on settings (only when feature flag is on).
                #[cfg(feature = "irc")]
                if let Ok(settings) = app_handle.state::<AppSettingsWithDiskSync>().get() {
                    let irc = effective_irc(&settings.irc, settings.feature_flags.irc);
                    gitbutler_tauri::irc_lifecycle::auto_connect_on_startup(app_handle, &irc);
                }
                tauri_app.on_menu_event(move |handle, event| {
                    let target_window = handle
                        .webview_windows()
                        .into_values()
                        .find(|webview| webview.is_focused().unwrap_or(false))
                        .or_else(|| handle.get_webview_window("main"));

                    if let Some(webview) = target_window {
                        menu::handle_event(&webview, &event);
                    } else {
                        tracing::warn!(
                            menu_event = %event.id().as_ref(),
                            "no webview window available to handle menu event"
                        );
                    }
                });

                let app_handle_for_deep_link = app_handle.clone();
                app_handle.deep_link().on_open_url(move |_| {
                    // Get main window
                    if let Some(window) = app_handle_for_deep_link.get_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                });
                Ok(())
            })
            .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_os::init())
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_deep_link::init())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init())
            .plugin(tauri_plugin_clipboard_manager::init())
            .plugin(tauri_plugin_store::Builder::default().build())
            .plugin(log.build())
            .invoke_handler(gitbutler_tauri::invoke_handler())
            .menu(move |handle| menu::build(handle, &app_settings_for_menu))
            .on_window_event(|window, event| match event {
                #[cfg(target_os = "macos")]
                tauri::WindowEvent::CloseRequested { .. } => {
                    let app_handle = window.app_handle();
                    if app_handle.windows().len() == 1 {
                        app_handle.cleanup_before_exit();
                        app_handle.exit(0);
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    window
                        .app_handle()
                        .state::<WindowState>()
                        .remove(window.label());
                }
                tauri::WindowEvent::Focused(focused) if *focused => {
                    window
                        .app_handle()
                        .state::<WindowState>()
                        .flush(window.label())
                        .ok();
                }
                _ => {}
            });

        #[cfg(not(target_os = "linux"))]
        let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());

        builder
            .build(tauri_context)
            .expect("Failed to build tauri app")
            .run(
                |#[cfg_attr(not(feature = "irc"), allow(unused_variables))] app_handle,
                 #[cfg_attr(not(feature = "irc"), allow(unused_variables))] event| {
                    #[cfg(feature = "irc")]
                    if let tauri::RunEvent::Exit = event {
                        let irc_manager = app_handle.state::<IrcManager>();
                        // Note that we can't use `tauri::async_runtime::block_on`  during shutdown as it panics.
                        irc_manager.shutdown_now();
                    }
                },
            );
    });
    Ok(())
}

/// read all objects, migrate them, and write them back if there was a migration.
fn migrate_projects() -> anyhow::Result<()> {
    for mut project in gitbutler_project::dangerously_list_projects_without_migration()? {
        if let Ok(true) = project.migrate() {
            let (title, git_dir) = (project.title.clone(), project.git_dir().to_owned());
            if let Err(err) = gitbutler_project::update(project.into()) {
                tracing::warn!(
                    "Failed to store migrated project {} at {}: {err}",
                    title,
                    git_dir.display()
                );
            } else {
                tracing::info!("Migrated project {} at {}", title, git_dir.display());
            }
        }
    }
    Ok(())
}

/// Launch a shell as interactive login shell, similar to what a login terminal would do if we are not already in a terminal.
///
/// That way, each process launched by the backend will act similar to what users would get in their terminal,
/// something vital to act more similar to Git, which is also launched from an interactive shell most of the time.
fn inherit_interactive_login_shell_environment_if_not_launched_from_terminal() {
    if std::env::var_os("TERM").is_some() {
        tracing::info!(
            "TERM is set - assuming the app is run from a terminal with suitable environment variables"
        );
        return;
    }

    fn doit() {
        if let Some(terminal_vars) = but_core::cmd::extract_interactive_login_shell_environment() {
            tracing::info!(
                "Inheriting static interactive shell environment, valid for the entire runtime of the application"
            );
            for (key, value) in terminal_vars {
                unsafe {
                    std::env::set_var(key, value);
                }
            }
        } else {
            tracing::info!(
                "SHELL variable isn't set - launching with default GUI application environment "
            );
        }
    }
    if cfg!(windows) {
        // This can be slow on Windows IF it runs, so background it.
        // This could also trigger a race, so let's only do it when we must, and hope that this works
        // in the few occasions where it may run.
        std::thread::spawn(doit);
    } else {
        doit();
    }
}
