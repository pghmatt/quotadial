//! QuotaDial's tray app: the meter in the menu bar or system tray. When no daemon is running it
//! runs one itself ([`feed`]), so the CLI and the MCP server read its numbers and nothing polls
//! twice. The tray shows the tightest window's share used, its menu and tooltip every account, and
//! clicking it opens a popover beside it with every window, each number with its reset and age.
//! It keeps itself up to date from QuotaDial's GitHub releases when the person chooses
//! ([`updates`]).

mod alerts;
mod feed;
mod help;
mod history;
mod icon;
mod meter;
mod settings;
mod updates;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use chrono::Utc;
use quotadial_core::{ProFeature, StatusReport, text};
use quotadial_daemon::{AddError, Config, DataDir, History, Licensing, licence, needs_pro};
use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, RunEvent, WebviewWindow, WindowEvent, Wry};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_notification::NotificationExt as _;
use tauri_plugin_opener::OpenerExt as _;
use tauri_plugin_positioner::{Position, WindowExt};
use tokio::sync::{Notify, watch};

use crate::alerts::{TELL_WITHIN, Told};
use crate::feed::{Role, Timing, Update};
use crate::meter::Meter;
use crate::settings::{LicenceView, Settings, account_key, carry_over, licence_view};
use crate::updates::{Record, Trigger, UpdateView, Updates};

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
    /// The daemon's own history, which keeps the alert rules and their firings, while the app runs
    /// the daemon. Read through [`AppState::history`].
    history: Mutex<Option<Arc<History>>>,
    /// The newest firing told as a notification.
    told: Mutex<Told>,
    /// Where the settings and the record of what was told are kept, if the system names a folder
    /// for them.
    files: Option<Files>,
    /// Notified when Settings adds or removes a licence key, so the daemon the app runs acts on
    /// it at once.
    licence_changed: Arc<Notify>,
    /// Whether there is a newer version, and the one on offer.
    updates: Mutex<Updates>,
    /// What the app does once its daemon has stopped.
    after_stop: Mutex<AfterStop>,
}

/// What the app does once its daemon has stopped.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum AfterStop {
    /// Exits, as when the person quits.
    #[default]
    Exit,
    /// Starts again, as the new version an update installed.
    Restart,
    /// Stays without it, while Windows' installer takes over and closes the app.
    Stay,
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
    /// When updates were last checked for, and what was offered and told (D-103).
    updates: PathBuf,
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
    /// The daemon's history: its own while the app runs it, or else opened for this use only.
    /// The app never holds the file open itself, so a daemon it follows can move a damaged one
    /// aside, which Windows refuses while the file is open, and the app reads the new one at its
    /// next use rather than the moved file (D-108).
    fn history(&self) -> Option<Arc<History>> {
        self.history
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .or_else(open_history)
    }

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
    /// The hosts of the webhooks alerts are also sent to, from `config.json` (D-094).
    alert_targets: Vec<String>,
    /// What `config.json` could not be used for, each naming the fix.
    alert_problems: Vec<String>,
    providers: Vec<ProviderChoice>,
    /// Every account QuotaDial knows now, with the name the user gave it.
    accounts: Vec<AccountChoice>,
    /// Whether the copy is QuotaDial Free or Pro, from its licence key. It changes through its own
    /// commands, not `set_settings`.
    licence: LicenceView,
    /// Whether the app asks GitHub once a day for a newer version (D-103).
    check_updates: bool,
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
    /// Whether it waits for QuotaDial Pro: a kind of rule only Pro checks, on QuotaDial Free.
    waits: bool,
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
    let dir = DataDir::from_env();
    let config = dir.as_ref().map(Config::load).unwrap_or_default();
    let tier = Licensing::new(dir.as_ref()).tier();
    // On QuotaDial Free alerts go only to the system's notifications, which Settings says where
    // it would name the webhooks.
    let sends = tier.allows(ProFeature::AlertDelivery);
    let mut alert_problems = config.warnings;
    if !sends && !config.webhooks.is_empty() {
        alert_problems.push(ProFeature::AlertDelivery.line());
    }
    SettingsView {
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        check_updates: settings.check_updates,
        licence: licence_view(dir.as_ref(), quotadial_core::licence::today()),
        alert_targets: config
            .webhooks
            .iter()
            .filter(|_| sends)
            .map(|webhook| webhook.host().to_owned())
            .collect(),
        alert_problems,
        alerts: state
            .history()
            .iter()
            .flat_map(|history| history.rules().unwrap_or_default())
            .map(|rule| AlertChoice {
                id: rule.id,
                read_as: rule.rule.to_string(),
                waits: needs_pro(&rule.rule.condition) && !tier.allows(ProFeature::AlertRules),
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
        check_updates: view.check_updates,
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
    // Help says whether the daily check is on.
    updates::emit(&app);
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
    let history = state.history().ok_or_else(no_history)?;
    let tier = Licensing::new(DataDir::from_env().as_ref()).tier();
    history
        .add_rule(&rule, Utc::now(), tier)
        .map_err(|error| match error {
            AddError::Unread(message) => message,
            AddError::Pro => ProFeature::AlertRules.line(),
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
    let history = state.history().ok_or_else(no_history)?;
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
    let history = state.history().ok_or_else(no_history)?;
    history
        .remove_rule(id)
        .map_err(|error| format!("QuotaDial could not remove the rule: {error}."))?;
    Ok(self::settings(app.clone(), state))
}

/// Adds a licence key for QuotaDial Pro, and shows the settings with what the copy is now.
///
/// # Errors
///
/// Why the key was not added, such as part of it missing, for the popover to show.
#[tauri::command]
fn add_licence(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<SettingsView, String> {
    let dir = DataDir::from_env().ok_or_else(no_data_folder)?;
    licence::add(&dir, &key, quotadial_core::licence::today())
        .map_err(|error| error.to_string())?;
    state.licence_changed.notify_one();
    Ok(self::settings(app.clone(), state))
}

/// Removes the licence key, and shows the settings with the copy as QuotaDial Free.
///
/// # Errors
///
/// When the key cannot be removed, for the popover to show.
#[tauri::command]
fn remove_licence(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SettingsView, String> {
    let dir = DataDir::from_env().ok_or_else(no_data_folder)?;
    licence::remove(&dir).map_err(|error| {
        format!(
            "QuotaDial could not remove the licence key ({error}). Check that its data folder can \
             be written, then try again."
        )
    })?;
    state.licence_changed.notify_one();
    Ok(self::settings(app.clone(), state))
}

/// What the app's Help shows: what is new in this version, what the copy is, the support address
/// and the credits.
#[tauri::command]
fn help() -> help::HelpView {
    let version = env!("CARGO_PKG_VERSION");
    help::HelpView {
        version: version.to_owned(),
        whats_new: help::whats_new(help::CHANGELOG, version),
        licence: licence_view(
            DataDir::from_env().as_ref(),
            quotadial_core::licence::today(),
        )
        .text,
        support: quotadial_api::diagnostics::SUPPORT_EMAIL.to_owned(),
        credits: help::CREDITS
            .iter()
            .map(|credit| (*credit).to_owned())
            .collect(),
        privacy: help::blocks(quotadial_core::privacy::PRIVACY),
    }
}

/// Where updates stand: the popover asks when it opens; after that it hears the `update` event.
#[tauri::command]
fn updates(app: AppHandle) -> UpdateView {
    updates::view(&app)
}

/// Asks GitHub now whether there is a newer version, whether or not the daily check is on.
#[tauri::command]
async fn check_for_updates(app: AppHandle) -> UpdateView {
    updates::check(&app, Trigger::ByHand).await;
    updates::view(&app)
}

/// Downloads and installs the version on offer, then starts the app again as it. How it goes is
/// told through the `update` event.
#[tauri::command]
async fn install_update(app: AppHandle) {
    updates::install(&app).await;
}

/// Opens a new GitHub issue in the person's browser with what `quotadial doctor` prints filled in,
/// from the numbers the app has now, so they see all of it before anything is sent.
///
/// # Errors
///
/// When the browser cannot be opened, for the popover to show.
#[tauri::command]
fn send_feedback(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (report, numbers_from) = {
        let known = state.known.lock().unwrap_or_else(PoisonError::into_inner);
        let numbers_from = match known.role {
            Some(Role::Running { .. }) => "the daemon the app runs",
            Some(Role::Following) => "the daemon `quotadial daemon` runs",
            None => "nowhere yet: the app is starting",
        };
        (known.report.clone().unwrap_or_default(), numbers_from)
    };
    let now = Utc::now();
    let text =
        quotadial_api::diagnostics::gather(DataDir::from_env().as_ref(), report, numbers_from, now)
            .text(now, std::env::home_dir().as_deref());
    open(&app, &quotadial_api::diagnostics::feedback_url(&text))
}

/// Opens one of Help's links: `changelog`, `support` or `site`. The window names a link, never an
/// address, so it cannot open anything else.
///
/// # Errors
///
/// For a link Help does not have, or when it cannot be opened.
#[tauri::command]
fn open_link(app: AppHandle, link: &str) -> Result<(), String> {
    let address = match link {
        "changelog" => quotadial_api::diagnostics::CHANGELOG_URL.to_owned(),
        "support" => format!(
            "mailto:{}?subject=QuotaDial%20{}",
            quotadial_api::diagnostics::SUPPORT_EMAIL,
            env!("CARGO_PKG_VERSION")
        ),
        "site" => "https://quotadial.com".to_owned(),
        other => return Err(format!("Help has no link called {other}.")),
    };
    open(&app, &address)
}

/// Says, as a notification, why something chosen from the tray's menu could not be done, since
/// the menu has nowhere of its own to say it.
fn tell_problem(app: &AppHandle, problem: &str) {
    // Best effort: with notifications refused too, there is nowhere left to say it.
    let _ = app
        .notification()
        .builder()
        .title("QuotaDial")
        .body(problem)
        .show();
}

fn open(app: &AppHandle, address: &str) -> Result<(), String> {
    app.opener()
        .open_url(address, None::<&str>)
        .map_err(|error| format!("QuotaDial could not open {address}: {error}"))
}

/// The history view of the last `days`: 7 or 30.
///
/// # Errors
///
/// When the history cannot be read, for the popover to show.
#[tauri::command]
fn history(state: tauri::State<'_, AppState>, days: u32) -> Result<history::HistoryView, String> {
    let history = state.history().ok_or_else(no_history)?;
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    history::view(
        &history,
        &settings,
        days.clamp(1, 365),
        Utc::now(),
        &chrono::Local,
    )
}

/// Saves every reading of the last `days` as CSV in the Downloads folder, and says where.
///
/// # Errors
///
/// When there is no Downloads folder, or the file cannot be written, for the popover to show.
#[tauri::command]
fn export_history(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    days: u32,
) -> Result<String, String> {
    let history = state.history().ok_or_else(no_history)?;
    let folder = app
        .path()
        .download_dir()
        .map_err(|_| "The system names no Downloads folder to save the file in.".to_owned())?;
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let path = history::export(&history, &settings, days.clamp(1, 365), Utc::now(), &folder)?;
    Ok(format!(
        "Saved to {}.",
        text::tilde(&path, app.path().home_dir().ok().as_deref())
    ))
}

fn no_history() -> String {
    "QuotaDial could not open its history, where the alert rules are kept.".to_owned()
}

fn no_data_folder() -> String {
    "This system names no folder for QuotaDial's data, so the licence key has nowhere to be kept; \
     set QUOTADIAL_DATA_DIR to one."
        .to_owned()
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            remove_alert,
            add_licence,
            remove_licence,
            help,
            send_feedback,
            open_link,
            history,
            export_history,
            updates,
            check_for_updates,
            install_update
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
                updates: folder.join("updates.json"),
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
            // Closed before the daemon starts, which may need to move a damaged history aside.
            drop(history);
            let licence_changed = Arc::new(Notify::new());
            let record = files
                .as_ref()
                .map(|files| Record::load(&files.updates))
                .unwrap_or_default();
            app.manage(AppState {
                known: Mutex::default(),
                quit,
                stopped,
                hidden_at: Mutex::default(),
                opened: Mutex::default(),
                settings: Mutex::new(settings),
                history: Mutex::default(),
                told: Mutex::new(told),
                files,
                licence_changed: Arc::clone(&licence_changed),
                updates: Mutex::new(Updates::new(record)),
                after_stop: Mutex::default(),
            });
            // What a rule told just before the app last stopped, and the app had not yet shown.
            tell(app.handle());
            build_tray(app.handle())?;
            start_feed(app.handle().clone(), licence_changed, quitting, stopped_tx);
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(REFRESH_EVERY).await;
                    refresh(&handle);
                }
            });
            tauri::async_runtime::spawn(updates::schedule(app.handle().clone()));
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

/// Stops the daemon the app runs or follows, then does `after` (D-103): an update restarts the app
/// as its new version, and on Windows hands over to the installer. If the daemon takes longer than
/// [`EXIT_WAIT`] to stop, a restart goes ahead and the operating system releases the lock.
async fn stop_daemon(app: &AppHandle, after: AfterStop) {
    let state = app.state::<AppState>();
    *state
        .after_stop
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = after;
    let mut stopped = state.stopped.clone();
    if *stopped.borrow() {
        // No daemon was running, so nothing will act on `after` but this.
        if after == AfterStop::Restart {
            app.request_restart();
        }
        return;
    }
    let _ = state.quit.send(true);
    let in_time = tokio::time::timeout(EXIT_WAIT, stopped.wait_for(|stopped| *stopped))
        .await
        .is_ok();
    // In time, the feed acted on `after` itself.
    if !in_time && after == AfterStop::Restart {
        app.request_restart();
    }
}

/// The daemon's history, opened alongside the daemon the app runs or follows, for the alert rules
/// and their firings.
fn open_history() -> Option<Arc<History>> {
    let dir = DataDir::from_env()?;
    dir.create().ok()?;
    History::open(&dir.history()).ok().map(Arc::new)
}

/// Starts the feed, which runs or follows the daemon until the app quits, then exits the app.
fn start_feed(
    app: AppHandle,
    licence_changed: Arc<Notify>,
    quitting: watch::Receiver<bool>,
    stopped: watch::Sender<bool>,
) {
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
            licence_changed,
            quitting,
        )
        .await;
        let _ = stopped.send(true);
        let after = *app
            .state::<AppState>()
            .after_stop
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        match after {
            AfterStop::Exit => app.exit(0),
            AfterStop::Restart => app.request_restart(),
            AfterStop::Stay => {}
        }
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
            Update::Role(role) => {
                if role == Role::Following {
                    // The daemon is another's now: its history is read afresh at each use.
                    *state.history.lock().unwrap_or_else(PoisonError::into_inner) = None;
                }
                known.role = Some(role);
            }
            Update::Fired => {
                drop(known);
                tell(app);
                return;
            }
            Update::History(kept) => {
                drop(known);
                *state.history.lock().unwrap_or_else(PoisonError::into_inner) = Some(kept.0);
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
    let Some(history) = state.history() else {
        return;
    };
    let mut told = state.told.lock().unwrap_or_else(PoisonError::into_inner);
    // A history started again in place of a damaged one numbers its firings from 1, so what was
    // told of the old one says nothing about it: what it holds already is not told.
    if let Ok(newest) = history.last_firing()
        && newest < told.last
    {
        told.last = newest;
    }
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
            "help" => {
                show_popover(app, Opened::ByClick);
                let _ = app.emit("panel", "help");
            }
            "feedback" => {
                if let Err(problem) = send_feedback(app.clone(), app.state::<AppState>()) {
                    tell_problem(app, &problem);
                }
            }
            "support" => {
                if let Err(problem) = open_link(app.clone(), "support") {
                    tell_problem(app, &problem);
                }
            }
            "updates" => {
                show_popover(app, Opened::ByClick);
                let _ = app.emit("panel", "help");
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    updates::check(&app, Trigger::ByHand).await;
                });
            }
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

/// Every account and problem, then Open, Help and Quit.
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
    menu.append(&Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &MenuItem::with_id(app, "help", "What's New and About", true, None::<&str>)?,
            &MenuItem::with_id(app, "feedback", "Send Feedback…", true, None::<&str>)?,
            &MenuItem::with_id(app, "support", "Write to Support", true, None::<&str>)?,
            &MenuItem::with_id(app, "updates", "Check for Updates…", true, None::<&str>)?,
        ],
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
