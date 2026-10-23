//! QuotaDial's tray app: the meter in the menu bar or system tray. When no daemon is running it
//! runs one itself ([`feed`]), so the CLI and the MCP server read its numbers and nothing polls
//! twice. The tray shows the tightest window's share used, its menu and tooltip every account, and
//! its window the same with room to read.

mod feed;
mod icon;
mod meter;

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use chrono::Utc;
use quotadial_core::StatusReport;
use quotadial_daemon::DataDir;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent, Wry};
use tokio::sync::watch;

use crate::feed::{Timing, Update};
use crate::meter::Meter;

const TRAY: &str = "main";
const WINDOW: &str = "main";
/// How often relative times, such as "resets in 2 h", are brought up to date.
const REFRESH_EVERY: Duration = Duration::from_secs(30);
/// How long the app waits for its daemon to stop when the system quits it without asking.
const EXIT_WAIT: Duration = Duration::from_secs(3);

/// What the app knows, shared by the feed, the tray and the window.
struct AppState {
    known: Mutex<Known>,
    /// Turned true to stop the daemon before the app exits.
    quit: watch::Sender<bool>,
    /// True once the daemon has stopped and released its lock, when the app may exit.
    stopped: watch::Receiver<bool>,
}

#[derive(Default)]
struct Known {
    report: Option<StatusReport>,
    problem: Option<String>,
}

impl AppState {
    fn meter(&self) -> Meter {
        let known = self.known.lock().unwrap_or_else(PoisonError::into_inner);
        Meter::new(known.report.as_ref(), known.problem.as_deref(), Utc::now())
    }
}

/// The window asks for the meter when it opens; after that it hears the `meter` event.
#[tauri::command]
fn meter(state: tauri::State<'_, AppState>) -> Meter {
    state.meter()
}

/// Runs the app until it is quit.
///
/// # Panics
///
/// When the app cannot start at all, as when its window cannot be made.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            |app, _arguments, _cwd| {
                // A second launch shows the first one's window instead.
                show_window(app);
            },
        ))
        .invoke_handler(tauri::generate_handler![meter])
        .on_window_event(|window, event| {
            // Closing the window hides it; the app lives in the tray until it is quit.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let (quit, quitting) = watch::channel(false);
            let (stopped_tx, stopped) = watch::channel(false);
            app.manage(AppState {
                known: Mutex::default(),
                quit,
                stopped,
            });
            build_tray(app.handle())?;
            start_feed(app.handle().clone(), quitting, stopped_tx);
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(REFRESH_EVERY).await;
                    refresh(&handle);
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("QuotaDial could not start its tray app");
    app.run(|app, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            let state = app.state::<AppState>();
            if !*state.stopped.borrow() {
                // Stop the daemon first, so its lock and `daemon.json` go before the app does;
                // the feed exits the app once they have.
                api.prevent_exit();
                let _ = state.quit.send(true);
            }
        }
        // macOS quits an app without asking first when the user logs out or another app tells it
        // to, as Windows does at sign-out. The daemon still stops before the process ends.
        RunEvent::Exit => stop_before_exit(app),
        _ => {}
    });
}

/// Stops the daemon, if it still runs, waiting up to [`EXIT_WAIT`]. If it takes longer, the
/// operating system releases the lock when the process ends, and consumers fetch for themselves.
fn stop_before_exit(app: &AppHandle) {
    let state = app.state::<AppState>();
    if *state.stopped.borrow() {
        return;
    }
    let _ = state.quit.send(true);
    let mut stopped = state.stopped.clone();
    tauri::async_runtime::block_on(async {
        let _ = tokio::time::timeout(EXIT_WAIT, stopped.wait_for(|stopped| *stopped)).await;
    });
}

/// Starts the feed, which runs or follows the daemon until the app quits, then exits the app.
fn start_feed(app: AppHandle, quitting: watch::Receiver<bool>, stopped: watch::Sender<bool>) {
    let Some(dir) = DataDir::from_env() else {
        receive(
            &app,
            Update::Problem(
                "The operating system names no data folder for QuotaDial; set \
                 QUOTADIAL_DATA_DIR."
                    .to_owned(),
            ),
        );
        let _ = stopped.send(true);
        return;
    };
    tauri::async_runtime::spawn(async move {
        let sender = app.clone();
        feed::run(
            dir,
            quotadial_providers::all,
            move |update| receive(&sender, update),
            Timing::default(),
            quitting,
        )
        .await;
        let _ = stopped.send(true);
        app.exit(0);
    });
}

/// Keeps an update and shows it.
fn receive(app: &AppHandle, update: Update) {
    {
        let state = app.state::<AppState>();
        let mut known = state.known.lock().unwrap_or_else(PoisonError::into_inner);
        match update {
            Update::Report(report) => {
                known.report = Some(report);
                known.problem = None;
            }
            // The last numbers stay on show, with what is wrong.
            Update::Problem(problem) => known.problem = Some(problem),
        }
    }
    refresh(app);
}

/// Brings the tray and the window up to date.
fn refresh(app: &AppHandle) {
    let meter = app.state::<AppState>().meter();
    if let Some(tray) = app.tray_by_id(TRAY) {
        show(app, &tray, &meter);
    }
    // The window may not be listening yet; it asks for the meter when it opens.
    let _ = app.emit("meter", &meter);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let meter = app.state::<AppState>().meter();
    let tray = TrayIconBuilder::with_id(TRAY)
        .icon(image(tray_icon(&meter)))
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    show(app, &tray, &meter);
    Ok(())
}

/// Puts `meter` on the tray. The tray is best effort: a failed update shows the last one, and the
/// next refresh tries again.
fn show(app: &AppHandle, tray: &TrayIcon, meter: &Meter) {
    let icon = tray_icon(meter);
    let template = icon.template;
    let _ = tray.set_icon(Some(image(icon)));
    let _ = tray.set_icon_as_template(template);
    // Windows cannot show text beside a tray icon, so there the number is drawn into the icon.
    if !cfg!(windows) {
        let _ = tray.set_title(Some(meter.title()));
    }
    let _ = tray.set_tooltip(Some(meter.tooltip()));
    if let Ok(menu) = tray_menu(app, meter) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// The icon for this platform (D-066).
fn tray_icon(meter: &Meter) -> icon::Icon {
    if cfg!(windows) {
        icon::number(32, meter.percent, meter.level)
    } else if cfg!(target_os = "macos") {
        // macOS tints a template to match the menu bar; amber and red show through as colour.
        let template = matches!(meter.level, meter::Level::Unknown | meter::Level::Normal);
        icon::gauge(36, meter.percent, meter.level, template)
    } else {
        icon::gauge(32, meter.percent, meter.level, false)
    }
}

fn image(icon: icon::Icon) -> Image<'static> {
    Image::new_owned(icon.rgba, icon.size, icon.size)
}

/// Every account and problem, then Open and Quit.
fn tray_menu(app: &AppHandle, meter: &Meter) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    let lines: Vec<&String> = meter.accounts.iter().chain(&meter.problems).collect();
    for line in &lines {
        menu.append(&MenuItem::new(app, line.as_str(), false, None::<&str>)?)?;
    }
    if !lines.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }
    menu.append(&MenuItem::with_id(
        app,
        "open",
        "Open QuotaDial",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "quit",
        "Quit QuotaDial",
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_window(app);
        }
    }
}
