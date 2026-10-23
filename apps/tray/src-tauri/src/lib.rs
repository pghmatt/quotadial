//! QuotaDial's tray app: the meter in the menu bar or system tray. When no daemon is running it
//! runs one itself ([`feed`]), so the CLI and the MCP server read its numbers and nothing polls
//! twice. The tray shows the tightest window's share used, its menu and tooltip every account, and
//! clicking it opens a popover beside it with every window, each number with its reset and age.

mod feed;
mod icon;
mod meter;

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use chrono::Utc;
use quotadial_core::StatusReport;
use quotadial_daemon::DataDir;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, RunEvent, WebviewWindow, WindowEvent, Wry};
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::sync::watch;

use crate::feed::{Role, Timing, Update};
use crate::meter::Meter;

const TRAY: &str = "main";
const WINDOW: &str = "main";
/// How often relative times, such as "resets in 2 h", are brought up to date.
const REFRESH_EVERY: Duration = Duration::from_secs(30);
/// How long the app waits for its daemon to stop when the system quits it without asking.
const EXIT_WAIT: Duration = Duration::from_secs(3);
/// The popover's width; its height follows its content.
const POPOVER_WIDTH: f64 = 360.0;
/// A tray click this soon after the popover hid on losing focus is the click that hid it.
const CLICK_AFTER_HIDE: Duration = Duration::from_millis(300);
/// A focus loss this soon after the popover opens from a click on the icon is the system settling
/// which app is active, not the user clicking elsewhere, so the popover stays.
const FOCUS_SETTLE: Duration = Duration::from_millis(600);
/// How long a second launch has to exit before the first instance opens its popover.
const SECOND_LAUNCH_SETTLE: Duration = Duration::from_millis(400);

/// What the app knows, shared by the feed, the tray and the window.
struct AppState {
    known: Mutex<Known>,
    /// Turned true to stop the daemon before the app exits.
    quit: watch::Sender<bool>,
    /// True once the daemon has stopped and released its lock, when the app may exit.
    stopped: watch::Receiver<bool>,
    /// When the popover last hid because it lost focus.
    hidden_at: Mutex<Option<Instant>>,
    /// When the popover last opened, and how.
    opened: Mutex<Option<(Instant, Opened)>>,
}

/// How the popover was opened, which decides whether losing focus closes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opened {
    /// By a click on the tray icon or its menu: it closes when the user clicks elsewhere, as a
    /// menu does.
    ByClick,
    /// By reopening the app or launching it again. macOS may not let an app opened that way stay
    /// active, and takes the focus back on its own, so the popover stays until the icon is clicked
    /// again or Escape is pressed.
    Otherwise,
}

#[derive(Default)]
struct Known {
    report: Option<StatusReport>,
    problem: Option<String>,
    role: Option<Role>,
}

impl AppState {
    fn meter(&self) -> Meter {
        let known = self.known.lock().unwrap_or_else(PoisonError::into_inner);
        Meter::new(
            known.report.as_ref(),
            known.problem.as_deref(),
            known.role,
            Utc::now(),
        )
    }
}

/// The popover asks for the meter when it opens; after that it hears the `meter` event.
#[tauri::command]
fn meter(state: tauri::State<'_, AppState>) -> Meter {
    state.meter()
}

/// The popover's height follows its content: it reports its height, and the window fits it and
/// stays beside the tray.
#[tauri::command]
fn fit(window: WebviewWindow, height: f64) {
    let _ = window.set_size(LogicalSize::new(POPOVER_WIDTH, height.clamp(120.0, 640.0)));
    place(&window);
}

/// Escape hides the popover.
#[tauri::command]
fn hide(window: WebviewWindow) {
    let _ = window.hide();
}

/// The popover's Quit button: the same clean quit as the menu's.
#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
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
                // A second launch opens the first one's popover instead, once the second one has
                // gone and cannot take the focus back from it.
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(SECOND_LAUNCH_SETTLE).await;
                    show_popover(&app, Opened::Otherwise);
                });
            },
        ))
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![meter, fit, hide, quit])
        .on_window_event(|window, event| match event {
            // Closing the popover hides it; the app lives in the tray until it is quit.
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            // Like a menu, the popover goes away when you click elsewhere.
            WindowEvent::Focused(false) if window.is_visible().unwrap_or(false) => {
                let state = window.state::<AppState>();
                let closes = state
                    .opened
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .is_none_or(|(at, opened)| {
                        opened == Opened::ByClick && at.elapsed() >= FOCUS_SETTLE
                    });
                if closes {
                    let _ = window.hide();
                    *state
                        .hidden_at
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
                }
            }
            _ => {}
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
                hidden_at: Mutex::default(),
                opened: Mutex::default(),
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
        // Opening QuotaDial again on macOS, from Finder or Spotlight, opens the popover.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => show_popover(app, Opened::Otherwise),
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
            Update::Role(role) => known.role = Some(role),
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
            // Remembers where the tray icon is, so the popover can open beside it.
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_popover(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_popover(app, Opened::ByClick),
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
    let lines = meter.lines();
    for line in &lines {
        menu.append(&MenuItem::new(app, line, false, None::<&str>)?)?;
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

/// Opens the popover beside the tray icon.
fn show_popover(app: &AppHandle, how: Opened) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        *app.state::<AppState>()
            .opened
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some((Instant::now(), how));
        place(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_popover(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    let just_hidden = app
        .state::<AppState>()
        .hidden_at
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some_and(|hidden_at| hidden_at.elapsed() < CLICK_AFTER_HIDE);
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else if !just_hidden {
        show_popover(app, Opened::ByClick);
    }
}

/// Puts the popover beside the tray icon: below it under a menu bar at the top of the screen,
/// above it over a taskbar at the bottom. Before the tray has reported where it is, the popover
/// opens in the middle of the screen.
fn place(window: &WebviewWindow) {
    if window.move_window(Position::TrayCenter).is_err() {
        let _ = window.move_window(Position::Center);
    }
}
