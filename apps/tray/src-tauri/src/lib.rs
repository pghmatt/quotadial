//! QuotaDial's tray app: the meter in the menu bar or system tray. When no daemon is running it
//! runs one itself ([`feed`]), so the CLI and the MCP server read its numbers and nothing polls
//! twice. The tray shows the tightest window's share used, its menu and tooltip every account, and
//! clicking it opens a popover beside it with every window, each number with its reset and age.

mod alerts;
mod feed;
mod icon;
mod meter;
mod settings;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use chrono::Utc;
use quotadial_core::{StatusReport, text};
use quotadial_daemon::{AddError, DataDir, History};
use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, RunEvent, WebviewWindow, WindowEvent, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_notification::NotificationExt as _;
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::sync::watch;

use crate::alerts::{TELL_WITHIN, Told};
use crate::feed::{Role, Timing, Update};
use crate::meter::Meter;
use crate::settings::{Settings, account_key, carry_over};

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
    settings: Mutex<Settings>,
    /// The daemon's history, which keeps the alert rules and their firings, when it can be opened.
    history: Option<Arc<History>>,
    /// The newest firing told as a notification.
    told: Mutex<Told>,
    /// Where the settings and the record of what was told are kept, if the system names a folder
    /// for them.
    files: Option<Files>,
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

struct Files {
    settings: PathBuf,
    told: PathBuf,
    /// Where M3 kept the notifications it sent, which the alert rules' firings replace (D-093).
    sent_before: PathBuf,
}

#[derive(Default)]
struct Known {
    report: Option<StatusReport>,
    problem: Option<String>,
    role: Option<Role>,
}

impl AppState {
    fn meter(&self) -> Meter {
        let settings = self
            .settings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let known = self.known.lock().unwrap_or_else(PoisonError::into_inner);
        let shown = known.report.as_ref().map(|report| settings.shown(report));
        Meter::new(
            shown.as_ref(),
            known.problem.as_deref(),
            known.role,
            Utc::now(),
        )
        .with_aliases(&settings.aliases)
    }
}

/// The settings as the popover shows them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SettingsView {
    /// Whether QuotaDial opens when the user logs in, as the system says.
    launch_at_login: bool,
    /// The alert rules, oldest first. They change through their own commands, not `set_settings`.
    alerts: Vec<AlertChoice>,
    providers: Vec<ProviderChoice>,
    /// Every account QuotaDial knows now, with the name the user gave it.
    accounts: Vec<AccountChoice>,
}

/// An alert rule the user can turn on or off, or remove.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AlertChoice {
    id: i64,
    /// The words it was written in.
    written: String,
    /// What QuotaDial read it as, such as "when any window reaches 80%".
    read_as: String,
    enabled: bool,
}

/// An account the user can name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AccountChoice {
    /// As the settings name it ([`account_key`]).
    key: String,
    /// Such as "Claude user@example.com".
    label: String,
    /// The name given, or empty.
    alias: String,
}

/// A provider the user can show or hide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ProviderChoice {
    id: String,
    name: String,
    shown: bool,
}

/// The settings, for the popover's settings panel.
#[tauri::command]
fn settings(app: AppHandle, state: tauri::State<'_, AppState>) -> SettingsView {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    SettingsView {
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        alerts: state
            .history
            .iter()
            .flat_map(|history| history.rules().unwrap_or_default())
            .map(|rule| AlertChoice {
                id: rule.id,
                read_as: rule.rule.to_string(),
                written: rule.written,
                enabled: rule.enabled,
            })
            .collect(),
        providers: quotadial_providers::all()
            .iter()
            .map(|provider| ProviderChoice {
                id: provider.id().to_owned(),
                name: text::provider_name(provider.id()).to_owned(),
                shown: settings.shows(provider.id()),
            })
            .collect(),
        accounts: state
            .known
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .report
            .iter()
            .flat_map(|report| &report.snapshots)
            .map(|snapshot| {
                let key = account_key(&snapshot.account);
                AccountChoice {
                    label: format!(
                        "{} {}",
                        text::provider_name(&snapshot.account.provider),
                        snapshot.account.display_name
                    ),
                    alias: settings.aliases.get(&key).cloned().unwrap_or_default(),
                    key,
                }
            })
            .collect(),
    }
}

/// Changes the settings from the popover, keeps them, and shows the result.
///
/// # Errors
///
/// What went wrong turning launch at login on or off, or keeping the settings, for the popover to
/// show.
#[tauri::command]
fn set_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    view: SettingsView,
) -> Result<SettingsView, String> {
    let autolaunch = app.autolaunch();
    if autolaunch.is_enabled().unwrap_or(false) != view.launch_at_login {
        let changed = if view.launch_at_login {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        changed.map_err(|error| format!("QuotaDial could not change opening at login: {error}"))?;
    }
    // Names of accounts not on show now are kept.
    let mut aliases = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .aliases
        .clone();
    for account in &view.accounts {
        match account.alias.trim() {
            "" => aliases.remove(&account.key),
            alias => aliases.insert(account.key.clone(), alias.to_owned()),
        };
    }
    let settings = Settings {
        hidden_providers: view
            .providers
            .iter()
            .filter(|provider| !provider.shown)
            .map(|provider| provider.id.clone())
            .collect(),
        aliases,
    };
    if let Some(files) = &state.files {
        settings
            .save(&files.settings)
            .map_err(|error| format!("QuotaDial could not keep its settings: {error}"))?;
    }
    *state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = settings;
    refresh(&app);
    Ok(self::settings(app.clone(), state))
}

/// Adds an alert rule written in plain words, and shows the settings with it.
///
/// # Errors
///
/// Why the rule was not added, such as the forms the grammar reads, for the popover to show.
#[tauri::command]
fn add_alert(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    rule: String,
) -> Result<SettingsView, String> {
    let history = state.history.as_ref().ok_or_else(no_history)?;
    history
        .add_rule(&rule, Utc::now())
        .map_err(|error| match error {
            AddError::Unread(message) => message,
            AddError::History(error) => format!("QuotaDial could not keep the rule: {error}."),
        })?;
    Ok(self::settings(app.clone(), state))
}

/// Turns an alert rule on or off, and shows the settings with it.
///
/// # Errors
///
/// When the rule cannot be changed, for the popover to show.
#[tauri::command]
fn set_alert(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: i64,
    enabled: bool,
) -> Result<SettingsView, String> {
    let history = state.history.as_ref().ok_or_else(no_history)?;
    history
        .set_rule_enabled(id, enabled)
        .map_err(|error| format!("QuotaDial could not change the rule: {error}."))?;
    Ok(self::settings(app.clone(), state))
}

/// Removes an alert rule, and shows the settings without it.
///
/// # Errors
///
/// When the rule cannot be removed, for the popover to show.
#[tauri::command]
fn remove_alert(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<SettingsView, String> {
    let history = state.history.as_ref().ok_or_else(no_history)?;
    history
        .remove_rule(id)
        .map_err(|error| format!("QuotaDial could not remove the rule: {error}."))?;
    Ok(self::settings(app.clone(), state))
}

fn no_history() -> String {
    "QuotaDial could not open its history, where the alert rules are kept.".to_owned()
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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            meter,
            fit,
            hide,
            quit,
            settings,
            set_settings,
            add_alert,
            set_alert,
            remove_alert
        ])
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
            let files = app.path().app_config_dir().ok().map(|folder| Files {
                settings: folder.join("settings.json"),
                told: folder.join("alerts.json"),
                sent_before: folder.join("notified.json"),
            });
            let settings = files
                .as_ref()
                .map(|files| Settings::load(&files.settings))
                .unwrap_or_default();
            let history = open_history();
            let mut told = Told::default();
            if let (Some(files), Some(history)) = (&files, &history) {
                if carry_over(&files.settings, history) {
                    // Saved without M3's switches, so they are carried over once.
                    let _ = settings.save(&files.settings);
                }
                // M3's record of what it told is no longer read: the rules' first check, quiet,
                // records what already holds.
                let _ = std::fs::remove_file(&files.sent_before);
                // On a first start nothing told before is told again.
                told = Told::load(&files.told).unwrap_or(Told {
                    last: history.last_firing().unwrap_or_default(),
                });
            }
            app.manage(AppState {
                known: Mutex::default(),
                quit,
                stopped,
                hidden_at: Mutex::default(),
                opened: Mutex::default(),
                settings: Mutex::new(settings),
                history,
                told: Mutex::new(told),
                files,
            });
            // What a rule told just before the app last stopped, and the app had not yet shown.
            tell(app.handle());
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

/// The daemon's history, opened alongside the daemon the app runs or follows, for the alert rules
/// and their firings.
fn open_history() -> Option<Arc<History>> {
    let dir = DataDir::from_env()?;
    dir.create().ok()?;
    History::open(&dir.history()).ok().map(Arc::new)
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
            Update::Fired => {
                drop(known);
                tell(app);
                return;
            }
        }
    }
    refresh(app);
}

/// Tells, as notifications, the firings the history holds that the app has not told yet, and keeps
/// the record of the newest one told.
fn tell(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Some(history) = &state.history else {
        return;
    };
    let mut told = state.told.lock().unwrap_or_else(PoisonError::into_inner);
    let Ok(firings) = history.firings(told.last, Utc::now() - TELL_WITHIN) else {
        return;
    };
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let (notices, now_told) = told.tell(&firings, &settings);
    for notice in notices {
        // Best effort: a notification the system refuses still leaves the number in the tray.
        let _ = app
            .notification()
            .builder()
            .title(notice.title)
            .body(notice.body)
            .show();
    }
    if now_told != *told {
        *told = now_told;
        if let Some(files) = &state.files {
            // A record that cannot be kept only means a notice may come again after a restart.
            let _ = told.save(&files.told);
        }
    }
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
