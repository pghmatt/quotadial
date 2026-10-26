//! Finds Claude Code's sign-ins on this machine, without reading any secret.
//!
//! [`locate`] finds the one this shell would use, in Claude Code's own order:
//! `CLAUDE_CODE_OAUTH_TOKEN`, then on macOS the Keychain item for the config directory, then
//! `.credentials.json` (Claude Code 2.1.286 reads the Keychain first and falls back to the file).
//!
//! [`locate_all`] finds every one stored (#34): that one first (D-023), Claude Code's default
//! folder, the folders other tools keep sign-ins in, and any Keychain item no folder accounts for.
//! A subscription seen from two folders is one account (D-024).

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use quotadial_core::{Account, Error, Result, Secret};

use crate::environment::{Environment, KEYCHAIN_SERVICE, sha256_hex};
use crate::identity::{self, Identity};
use crate::keychain::{Item, Keychain, NoKeychain};
use crate::{ID, NAME};

/// Where a sign-in's token lives.
#[derive(Debug, Clone)]
pub(crate) enum LoginSource {
    /// The `CLAUDE_CODE_OAUTH_TOKEN` environment variable.
    Environment(Secret),
    /// A macOS Keychain item.
    Keychain {
        service: String,
        account: Option<String>,
    },
    /// A `.credentials.json` file.
    File(PathBuf),
}

impl LoginSource {
    /// Where the token was read, as shown to the user, so they can see exactly what was touched:
    /// what an error says it could not read the sign-in from.
    pub(crate) fn location(&self) -> String {
        match self {
            Self::Environment(_) => "the CLAUDE_CODE_OAUTH_TOKEN environment variable".to_owned(),
            Self::Keychain { service, .. } => format!("the macOS Keychain item \"{service}\""),
            Self::File(path) => path.display().to_string(),
        }
    }

    /// What stands in for the account until Anthropic names it: an id that keeps different
    /// sign-ins apart, and a name to show.
    ///
    /// An environment token's id is a one-way hash of the token, so two shells holding different
    /// tokens never share saved numbers. The hash cannot be turned back into the token.
    fn stand_in(&self) -> (String, String) {
        match self {
            Self::Environment(token) => (
                format!(
                    "env:{}",
                    &sha256_hex(&format!(
                        "quotadial:claude-env-token:v1\0{}",
                        token.expose()
                    ))[..16]
                ),
                "CLAUDE_CODE_OAUTH_TOKEN".to_owned(),
            ),
            Self::Keychain { service, .. } => (service.clone(), service.clone()),
            Self::File(path) => (path.display().to_string(), path.display().to_string()),
        }
    }
}

/// The Claude Code config folder a sign-in belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Folder {
    /// Claude Code's default folder, used when `CLAUDE_CONFIG_DIR` is not set.
    Default,
    /// The folder `CLAUDE_CONFIG_DIR` names.
    Named(PathBuf),
    /// A Keychain item whose folder QuotaDial could not find.
    Unknown,
}

/// A Claude Code sign-in found on this machine.
#[derive(Debug, Clone)]
pub(crate) struct Login {
    pub(crate) source: LoginSource,
    /// Changes whenever Claude Code writes a new token: the Keychain item's or the file's
    /// modification time. `None` for an environment token.
    pub(crate) generation: Option<String>,
    /// Who the sign-in belongs to, from Claude Code's account config. `None` for an environment
    /// token, whose account the config does not describe.
    pub(crate) identity: Option<Identity>,
    pub(crate) folder: Folder,
    /// Where the sign-in is, written for people, as [`Account::sign_in`] carries it (P-015).
    pub(crate) place: String,
    /// What Claude Code sees for this sign-in, so it can be looked up again on its own.
    env: Environment,
}

impl Login {
    pub(crate) fn new(
        env: &Environment,
        source: LoginSource,
        generation: Option<String>,
        identity: Option<Identity>,
        folder: Folder,
    ) -> Self {
        let place = match (&source, &folder) {
            (LoginSource::Environment(_), _) => source.location(),
            (LoginSource::Keychain { service, .. }, Folder::Named(dir)) => format!(
                "the macOS Keychain item \"{service}\" (CLAUDE_CONFIG_DIR {})",
                env.tilde(dir)
            ),
            (LoginSource::Keychain { service, .. }, _) => {
                format!("the macOS Keychain item \"{service}\"")
            }
            (LoginSource::File(path), _) => env.tilde(path),
        };
        Self {
            source,
            generation,
            identity,
            folder,
            place,
            env: env.clone(),
        }
    }

    /// This sign-in looked up again on its own, without reading its token: its current generation
    /// and who it belongs to now, or `None` once it is gone.
    ///
    /// # Errors
    ///
    /// When the Keychain or the credentials file cannot be read, which is not the same as gone.
    pub(crate) fn relocate(&self, keychain: &dyn Keychain) -> Result<Option<Self>> {
        let found = |generation| {
            Some(Self::new(
                &self.env,
                self.source.clone(),
                generation,
                self.identity_now(),
                self.folder.clone(),
            ))
        };
        match &self.source {
            LoginSource::Environment(_) => Ok(Some(self.clone())),
            LoginSource::Keychain { service, account } => {
                match keychain.find(service, account.as_deref()) {
                    Ok(Some(item)) => Ok(found(item.modified)),
                    Ok(None) => Ok(None),
                    Err(reason) => Err(Error::Credentials {
                        provider: ID,
                        location: self.source.location(),
                        reason,
                    }),
                }
            }
            LoginSource::File(path) => match std::fs::metadata(path) {
                Ok(metadata) if metadata.is_file() => Ok(found(modified(&metadata))),
                Ok(_) => Ok(None),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(Error::Credentials {
                    provider: ID,
                    location: path.display().to_string(),
                    reason: format!("{error}; check that you can read it"),
                }),
            },
        }
    }

    /// Who the sign-in belongs to now, from its folder's account config. An environment token's
    /// account is not in it, and an item whose folder is unknown has none to read.
    fn identity_now(&self) -> Option<Identity> {
        if self.folder == Folder::Unknown || matches!(self.source, LoginSource::Environment(_)) {
            return None;
        }
        self.env
            .account_config_file()
            .and_then(|file| identity::from_account_config(&file))
    }

    /// The account this sign-in belongs to, as far as is known without asking Anthropic.
    pub(crate) fn account(&self) -> Account {
        let account = match &self.identity {
            Some(identity) => account_for(identity),
            None => {
                let (account_id, display_name) = self.source.stand_in();
                Account {
                    provider: ID.to_owned(),
                    provider_name: NAME.to_owned(),
                    account_id,
                    display_name,
                    plan: None,
                    org: None,
                    sign_in: None,
                    sign_in_use: sign_in_use(None),
                }
            }
        };
        Account {
            sign_in: Some(self.place.clone()),
            sign_in_use: sign_in_use(Some(&self.place)),
            ..account
        }
    }

    /// The command that signs this sign-in's account in again, such as
    /// `` `CLAUDE_CONFIG_DIR=~/.claude-work claude` ``.
    pub(crate) fn sign_in_command(&self) -> String {
        match (&self.folder, &self.source) {
            (Folder::Named(dir), _) => {
                format!("`{}`", claude_in(dir, &self.env.tilde(dir), cfg!(windows)))
            }
            (Folder::Unknown, LoginSource::Keychain { service, .. }) => format!(
                "`claude` with the CLAUDE_CONFIG_DIR that made the Keychain item \"{service}\""
            ),
            _ => "`claude`".to_owned(),
        }
    }

    /// The Keychain service this sign-in is filed under, if it is in the Keychain.
    fn service(&self) -> Option<&str> {
        match &self.source {
            LoginSource::Keychain { service, .. } => Some(service),
            _ => None,
        }
    }
}

/// The command that runs `claude` with `CLAUDE_CONFIG_DIR` set to `dir`: for PowerShell on
/// Windows, and elsewhere for a POSIX shell, writing `shown`, the folder with `~`, when it needs no
/// quotes.
pub(crate) fn claude_in(dir: &Path, shown: &str, windows: bool) -> String {
    if windows {
        let dir = dir.display().to_string().replace('\'', "''");
        return format!("$env:CLAUDE_CONFIG_DIR = '{dir}'; claude");
    }
    let plain = shown.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '~' | '/' | '.' | '_' | '-' | '+' | '@' | ',' | ':')
    });
    if plain {
        format!("CLAUDE_CONFIG_DIR={shown} claude")
    } else {
        let dir = dir.display().to_string().replace('\'', r"'\''");
        format!("CLAUDE_CONFIG_DIR='{dir}' claude")
    }
}

/// What QuotaDial does with a sign-in found at `place`, in Claude's words (P-020): where it read
/// it, or, with no place known, every place Claude Code keeps one, as the popover used to say.
pub(crate) fn sign_in_use(place: Option<&str>) -> String {
    if let Some(place) = place {
        return quotadial_core::text::sign_in_read_from(place);
    }
    let keychain = if cfg!(target_os = "macos") {
        "then the macOS Keychain item \"Claude Code-credentials\", "
    } else {
        ""
    };
    format!(
        "QuotaDial reads the sign-in Claude Code uses: the CLAUDE_CODE_OAUTH_TOKEN environment \
         variable when it is set, {keychain}then .credentials.json in ~/.claude or the folder \
         CLAUDE_CONFIG_DIR names. It only reads the sign-in and never changes it. If it stops \
         working, run claude once."
    )
}

pub(crate) fn account_for(identity: &Identity) -> Account {
    Account {
        provider: ID.to_owned(),
        provider_name: NAME.to_owned(),
        account_id: identity.account_uuid.clone(),
        display_name: identity
            .email
            .clone()
            .unwrap_or_else(|| identity.account_uuid.clone()),
        plan: identity.plan.clone(),
        org: identity.org_uuid.clone(),
        sign_in: None,
        sign_in_use: sign_in_use(None),
    }
}

/// The folder an environment's stored sign-in belongs to.
fn folder_of(env: &Environment) -> Folder {
    env.scoped_dir()
        .map_or(Folder::Default, |dir| Folder::Named(PathBuf::from(dir)))
}

/// Finds the sign-in Claude Code would use with this environment, or `None` when it is signed out.
pub(crate) fn locate(env: &Environment, keychain: &dyn Keychain) -> Result<Option<Login>> {
    if let Some(token) = &env.oauth_token {
        return Ok(Some(Login::new(
            env,
            LoginSource::Environment(token.clone()),
            None,
            None,
            Folder::Default,
        )));
    }
    let identity = || {
        env.account_config_file()
            .and_then(|file| identity::from_account_config(&file))
    };

    let service = env.keychain_service();
    let account = env.keychain_account();
    match keychain.find(&service, account.as_deref()) {
        Ok(Some(item)) => {
            return Ok(Some(Login::new(
                env,
                LoginSource::Keychain { service, account },
                item.modified,
                identity(),
                folder_of(env),
            )));
        }
        Ok(None) => {}
        Err(reason) => {
            return Err(Error::Credentials {
                provider: ID,
                location: LoginSource::Keychain { service, account }.location(),
                reason,
            });
        }
    }

    let Some(file) = env.credentials_file() else {
        return Ok(None);
    };
    match std::fs::metadata(&file) {
        Ok(metadata) if metadata.is_file() => Ok(Some(Login::new(
            env,
            LoginSource::File(file),
            modified(&metadata),
            identity(),
            folder_of(env),
        ))),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::Credentials {
            provider: ID,
            location: file.display().to_string(),
            reason: format!("{error}; check that you can read it"),
        }),
    }
}

/// A file's modification time, which changes whenever Claude Code writes a new token.
fn modified(metadata: &std::fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .map(|modified| DateTime::<Utc>::from(modified).to_rfc3339_opts(SecondsFormat::Nanos, true))
}

/// What [`locate_all`] found.
#[derive(Debug)]
pub(crate) struct Found {
    /// One sign-in per account.
    pub(crate) logins: Vec<Login>,
    /// Why the Keychain could not be listed, when it could not. Each folder's item was then looked
    /// up by its name, and an item whose folder is unknown could not be found.
    pub(crate) unlisted: Option<String>,
    /// `extra_dirs` entries left out because they are not full paths.
    pub(crate) skipped: Vec<PathBuf>,
}

/// Finds every Claude Code sign-in stored on this machine, one per account (#34): this shell's
/// own first (D-023), then the sign-in Claude Code keeps for this shell's folder when a token in
/// the environment comes first, then Claude Code's default folder, then the sign-ins kept for
/// other config folders, then any Keychain item no folder accounts for. `extra_dirs` adds
/// folders, and a folder ending in `*` adds every folder inside it.
///
/// The Keychain is listed once, by attributes, so every folder can be matched to its item without
/// reading a secret. If it cannot be listed, as when `security` takes too long on a large
/// keychain, each folder's item is looked up by its name instead, and the reason comes back in
/// [`Found::unlisted`].
///
/// # Errors
///
/// When this shell's own sign-in cannot be read. Other folders whose sign-in cannot be read are
/// skipped.
pub(crate) fn locate_all(
    env: &Environment,
    keychain: &dyn Keychain,
    extra_dirs: &[PathBuf],
) -> Result<Found> {
    let own = locate(env, keychain)?;
    let has_own = own.is_some();
    let mut logins: Vec<Login> = own.into_iter().collect();
    let (items, unlisted) = match keychain.list(KEYCHAIN_SERVICE) {
        Ok(items) => (Some(items), None),
        Err(reason) => (None, Some(reason)),
    };
    let mut scoped = Vec::new();
    // With a token in the environment, the sign-in Claude Code keeps for this shell's folder is
    // another account.
    if env.oauth_token.is_some() {
        scoped.push(env.without_token());
    }
    // With a folder chosen, so is the one in Claude Code's default folder.
    if env.scoped_dir().is_some() {
        scoped.push(env.default_only());
    }
    let own_dir = env.scoped_dir();
    let candidates = candidate_dirs(env, extra_dirs);
    let mut search = Search {
        items: items.as_deref(),
        keychain,
        keychain_answers: true,
    };
    for env in &scoped {
        logins.extend(search.folder(env));
    }
    for dir in candidates
        .dirs
        .iter()
        .filter(|dir| Some(*dir) != own_dir.as_ref())
    {
        logins.extend(search.candidate(env, dir));
    }
    for item in items.iter().flatten() {
        if !logins
            .iter()
            .any(|login| login.service() == Some(&item.service))
        {
            logins.push(Login::new(
                env,
                LoginSource::Keychain {
                    service: item.service.clone(),
                    account: item.account.clone(),
                },
                item.info.modified.clone(),
                None,
                Folder::Unknown,
            ));
        }
    }
    Ok(Found {
        logins: one_per_account(logins, has_own),
        unlisted,
        skipped: candidates.skipped,
    })
}

/// How folders are searched in one [`locate_all`]: through the Keychain's listing, or without it,
/// by each item's name.
struct Search<'a> {
    items: Option<&'a [Item]>,
    keychain: &'a dyn Keychain,
    /// Without the listing, a keychain that fails one lookup is not asked again, so a keychain
    /// that has stopped answering costs one lookup's wait, not one per folder.
    keychain_answers: bool,
}

impl Search<'_> {
    /// The sign-in Claude Code keeps for `env`'s folder.
    fn folder(&mut self, env: &Environment) -> Option<Login> {
        match self.items {
            Some(items) => stored(env, items),
            None if self.keychain_answers => locate(env, self.keychain).unwrap_or_else(|_| {
                self.keychain_answers = false;
                locate(env, &NoKeychain).ok().flatten()
            }),
            None => locate(env, &NoKeychain).ok().flatten(),
        }
    }

    /// The sign-in kept for the candidate folder `dir`. Claude Code names a folder's Keychain item
    /// after the exact text it was given, so with the listing each way of writing the folder is
    /// tried, and the one that matches is the one its fix names (#39).
    fn candidate(&mut self, env: &Environment, dir: &str) -> Option<Login> {
        let matched = self.items.and_then(|items| {
            spellings(dir)
                .into_iter()
                .map(|spelling| env.for_config_dir(&spelling))
                .find(|env| {
                    let service = env.keychain_service();
                    items.iter().any(|item| item.service == service)
                })
        });
        self.folder(&matched.unwrap_or_else(|| env.for_config_dir(dir)))
    }
}

/// The ways Claude Code may have been given `dir`: as written, with and without a trailing
/// separator, and with links resolved, as when the home folder is reached through one.
fn spellings(dir: &str) -> Vec<String> {
    let separator = std::path::MAIN_SEPARATOR;
    let mut spellings: Vec<String> = Vec::new();
    let real = std::fs::canonicalize(dir)
        .ok()
        .map(|real| real.display().to_string());
    for written in std::iter::once(dir.to_owned()).chain(real) {
        let bare = written.trim_end_matches(separator).to_owned();
        for spelling in [bare.clone(), format!("{bare}{separator}")] {
            if !spellings.contains(&spelling) {
                spellings.push(spelling);
            }
        }
    }
    spellings
}

/// The sign-in stored for `env`'s folder, found among the listed Keychain `items` or as its
/// credentials file.
fn stored(env: &Environment, items: &[Item]) -> Option<Login> {
    let identity = || {
        env.account_config_file()
            .and_then(|file| identity::from_account_config(&file))
    };
    let service = env.keychain_service();
    if let Some(item) = items.iter().find(|item| item.service == service) {
        return Some(Login::new(
            env,
            LoginSource::Keychain {
                service,
                account: item.account.clone(),
            },
            item.info.modified.clone(),
            identity(),
            folder_of(env),
        ));
    }
    let file = env.credentials_file()?;
    let metadata = std::fs::metadata(&file)
        .ok()
        .filter(|metadata| metadata.is_file())?;
    Some(Login::new(
        env,
        LoginSource::File(file),
        modified(&metadata),
        identity(),
        folder_of(env),
    ))
}

/// The folders [`candidate_dirs`] found.
#[derive(Debug, Default)]
pub(crate) struct Candidates {
    /// Full paths, sorted.
    pub(crate) dirs: Vec<String>,
    /// `extra_dirs` entries left out because they are not full paths.
    pub(crate) skipped: Vec<PathBuf>,
}

/// Folders that may hold a Claude Code sign-in besides the default one, as `CLAUDE_CONFIG_DIR`
/// would name them: `~/.claude-*` and other `~/.claude…` folders, the account folders Codus keeps
/// in `~/.codus/accounts`, and `extra_dirs`. An `extra_dirs` entry must be a full path or start
/// with `~`: a relative one would mean a different folder for every process, and could never match
/// the Keychain item Claude Code made for it (#39).
pub(crate) fn candidate_dirs(env: &Environment, extra_dirs: &[PathBuf]) -> Candidates {
    let mut dirs = BTreeSet::new();
    let mut skipped = Vec::new();
    if let Some(home) = &env.home {
        dirs.extend(folders_in(home, |name| {
            name.starts_with(".claude") && name != ".claude"
        }));
        // Joined one part at a time, so the path is written as Claude Code would be given it on
        // Windows too, and hashes to the same Keychain item.
        dirs.extend(folders_in(&home.join(".codus").join("accounts"), |_| true));
    }
    for entry in extra_dirs {
        let dir = match (entry.strip_prefix("~"), &env.home) {
            (Ok(rest), Some(home)) => home.join(rest),
            _ => entry.clone(),
        };
        if !dir.is_absolute() {
            skipped.push(entry.clone());
        } else if dir.file_name().is_some_and(|name| name == "*") {
            if let Some(parent) = dir.parent() {
                dirs.extend(folders_in(parent, |_| true));
            }
        } else if dir.is_dir() {
            dirs.insert(dir.display().to_string());
        }
    }
    Candidates {
        dirs: dirs.into_iter().collect(),
        skipped,
    }
}

/// The folders directly inside `parent` whose names pass `keep`, as full paths. A link to a folder
/// counts as a folder.
fn folders_in(parent: &Path, keep: impl Fn(&str) -> bool) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| keep(&entry.file_name().to_string_lossy()))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().display().to_string())
        .collect()
}

/// One sign-in per account (D-024): the first found, except that a newer token in the same
/// account replaces an older one from another folder, being less likely to have expired. When
/// `has_own`, the first is this shell's own sign-in, which always stays.
fn one_per_account(logins: Vec<Login>, has_own: bool) -> Vec<Login> {
    let mut kept: Vec<Login> = Vec::new();
    for login in logins {
        let account = login.account();
        match kept
            .iter_mut()
            .enumerate()
            .find(|(_, known)| known.account().same_as(&account))
        {
            Some((0, _)) if has_own => {}
            Some((_, known)) if written(&login) > written(known) => *known = login,
            Some(_) => {}
            None => kept.push(login),
        }
    }
    kept
}

/// When a sign-in's token was last written, read from its generation: a Keychain time such as
/// `20261022205736Z`, or a file's RFC 3339 time. Compared as times, so neither kind of sign-in wins
/// just because of how its time is written.
fn written(login: &Login) -> Option<DateTime<Utc>> {
    let generation = login.generation.as_deref()?;
    NaiveDateTime::parse_from_str(generation, "%Y%m%d%H%M%SZ")
        .map(|naive| naive.and_utc())
        .ok()
        .or_else(|| {
            DateTime::parse_from_rfc3339(generation)
                .ok()
                .map(|time| time.with_timezone(&Utc))
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::keychain::{Item, ItemInfo, NoKeychain};

    /// A keychain whose only item is the one a test puts there. Reading secrets is not allowed.
    struct OneItem {
        service: String,
    }

    impl Keychain for OneItem {
        fn find(
            &self,
            service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<ItemInfo>, String> {
            Ok((service == self.service).then(|| ItemInfo {
                modified: Some("20991231000000Z".to_owned()),
            }))
        }

        fn read(
            &self,
            _service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<Secret>, String> {
            panic!("locating a sign-in must never read a secret");
        }
    }

    struct BrokenKeychain;

    impl Keychain for BrokenKeychain {
        fn find(
            &self,
            _service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<ItemInfo>, String> {
            Err("/usr/bin/security exited with status 36".to_owned())
        }

        fn read(
            &self,
            _service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<Secret>, String> {
            panic!("locating a sign-in must never read a secret");
        }
    }

    fn env_in(home: &Path) -> Environment {
        Environment {
            home: Some(home.to_path_buf()),
            user: Some("example".into()),
            ..Environment::default()
        }
    }

    fn write_account_config(home: &Path) {
        std::fs::write(
            home.join(".claude.json"),
            json!({
                "oauthAccount": {
                    "accountUuid": "00000000-0000-0000-0000-000000000001",
                    "emailAddress": "user@example.com",
                    "organizationUuid": "00000000-0000-0000-0000-000000000002",
                    "organizationType": "claude_max"
                }
            })
            .to_string(),
        )
        .expect("writes account config");
    }

    #[test]
    fn nothing_signed_in_means_no_login() {
        let home = tempfile::tempdir().expect("temp dir");

        let login = locate(&env_in(home.path()), &NoKeychain).expect("locates");

        assert!(login.is_none());
    }

    #[test]
    fn the_environment_token_wins_and_names_no_account() {
        let home = tempfile::tempdir().expect("temp dir");
        write_account_config(home.path());
        let env = Environment {
            oauth_token: Some(Secret::new("sk-ant-oat01-QUOTADIAL-TEST-TOKEN")),
            ..env_in(home.path())
        };
        let keychain = OneItem {
            service: "Claude Code-credentials".into(),
        };

        let login = locate(&env, &keychain)
            .expect("locates")
            .expect("signed in");

        assert!(matches!(login.source, LoginSource::Environment(_)));
        assert_eq!(login.identity, None);
        let account = login.account();
        assert!(
            account.account_id.starts_with("env:"),
            "{}",
            account.account_id
        );
        assert_eq!(account.account_id.len(), "env:".len() + 16);
        assert!(!account.account_id.contains("QUOTADIAL-TEST-TOKEN"));
        assert_eq!(account.display_name, "CLAUDE_CODE_OAUTH_TOKEN");
        assert_eq!(
            login.source.location(),
            "the CLAUDE_CODE_OAUTH_TOKEN environment variable"
        );
    }

    #[test]
    fn different_environment_tokens_are_different_accounts() {
        let home = tempfile::tempdir().expect("temp dir");
        let with_token = |token: &str| Environment {
            oauth_token: Some(Secret::new(token)),
            ..env_in(home.path())
        };
        let account = |token: &str| {
            locate(&with_token(token), &NoKeychain)
                .expect("locates")
                .expect("signed in")
                .account()
        };

        assert_ne!(
            account("sk-ant-oat01-QUOTADIAL-TEST-TOKEN-A").account_id,
            account("sk-ant-oat01-QUOTADIAL-TEST-TOKEN-B").account_id
        );
        assert_eq!(
            account("sk-ant-oat01-QUOTADIAL-TEST-TOKEN-A"),
            account("sk-ant-oat01-QUOTADIAL-TEST-TOKEN-A")
        );
    }

    #[test]
    fn the_keychain_item_comes_before_the_credentials_file() {
        let home = tempfile::tempdir().expect("temp dir");
        write_account_config(home.path());
        std::fs::create_dir_all(home.path().join(".claude")).expect("config dir");
        std::fs::write(home.path().join(".claude").join(".credentials.json"), "{}")
            .expect("writes");
        let keychain = OneItem {
            service: "Claude Code-credentials".into(),
        };

        let login = locate(&env_in(home.path()), &keychain)
            .expect("locates")
            .expect("signed in");

        assert!(matches!(
            &login.source,
            LoginSource::Keychain { service, account }
                if service == "Claude Code-credentials" && account.as_deref() == Some("example")
        ));
        assert_eq!(login.generation.as_deref(), Some("20991231000000Z"));
        assert_eq!(
            login.account(),
            Account {
                provider: "claude".into(),
                provider_name: "Claude".into(),
                account_id: "00000000-0000-0000-0000-000000000001".into(),
                display_name: "user@example.com".into(),
                plan: Some("max".into()),
                org: Some("00000000-0000-0000-0000-000000000002".into()),
                sign_in: Some("the macOS Keychain item \"Claude Code-credentials\"".into()),
                sign_in_use: "QuotaDial reads this account's sign-in from the macOS Keychain \
                              item \"Claude Code-credentials\". It only reads it and never \
                              changes it."
                    .into(),
            }
        );
    }

    #[test]
    fn its_name_and_every_sentence_it_says_of_a_sign_in_keep_to_plain_words() {
        let home = Path::new("/Users/example");
        let env = env_in(home);
        let keychain = |folder| {
            Login::new(
                &env,
                LoginSource::Keychain {
                    service: "Claude Code-credentials-1df4ed49".into(),
                    account: Some("example".into()),
                },
                None,
                None,
                folder,
            )
        };
        let logins = [
            Login::new(
                &env,
                LoginSource::Environment(Secret::new("not a token")),
                None,
                None,
                Folder::Default,
            ),
            keychain(Folder::Default),
            keychain(Folder::Named(home.join(".claude-work"))),
            Login::new(
                &env,
                LoginSource::File(home.join(".claude").join(".credentials.json")),
                None,
                None,
                Folder::Default,
            ),
        ];
        let said = logins
            .iter()
            .map(|login| login.account().sign_in_use)
            .chain([sign_in_use(None), crate::NAME.to_owned()]);

        for words in said {
            // #52's plain words: never "quota", "auth" or "credential" outside a name.
            assert_eq!(
                quotadial_core::text::unplain_words(&words),
                Vec::<String>::new(),
                "{words}"
            );
        }
    }

    #[test]
    fn with_no_place_known_the_sentence_names_every_place_claude_code_keeps_a_sign_in() {
        let keychain = if cfg!(target_os = "macos") {
            "then the macOS Keychain item \"Claude Code-credentials\", "
        } else {
            ""
        };

        assert_eq!(
            sign_in_use(None),
            format!(
                "QuotaDial reads the sign-in Claude Code uses: the CLAUDE_CODE_OAUTH_TOKEN \
                 environment variable when it is set, {keychain}then .credentials.json in \
                 ~/.claude or the folder CLAUDE_CONFIG_DIR names. It only reads the sign-in and \
                 never changes it. If it stops working, run claude once."
            )
        );
    }

    #[test]
    fn the_credentials_file_is_used_when_there_is_no_keychain_item() {
        let home = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(home.path().join(".claude")).expect("config dir");
        // Built one component at a time, as Claude Code does, so the text matches on Windows too.
        let file = home.path().join(".claude").join(".credentials.json");
        std::fs::write(&file, "{}").expect("writes");

        let login = locate(&env_in(home.path()), &NoKeychain)
            .expect("locates")
            .expect("signed in");

        assert!(matches!(&login.source, LoginSource::File(path) if *path == file));
        assert!(login.generation.is_some());
        // No account config: the file stands in for the account until Anthropic names it.
        assert_eq!(login.account().account_id, file.display().to_string());
        assert_eq!(login.account().org, None);
    }

    /// A keychain whose listing fails, as `security dump-keychain` does when it takes too long,
    /// holding items by service. Lookups by name work until `lookups_answered` have been made,
    /// then fail, as a keychain that stops answering does.
    struct UnlistableKeychain {
        services: Vec<String>,
        lookups_answered: usize,
        lookups: std::sync::atomic::AtomicUsize,
    }

    impl UnlistableKeychain {
        fn holding(services: &[&str]) -> Self {
            Self {
                services: services
                    .iter()
                    .map(|service| (*service).to_owned())
                    .collect(),
                lookups_answered: usize::MAX,
                lookups: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn lookups(&self) -> usize {
            self.lookups.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl Keychain for UnlistableKeychain {
        fn find(
            &self,
            service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<ItemInfo>, String> {
            let made = self
                .lookups
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if made >= self.lookups_answered {
                return Err("/usr/bin/security did not answer within 5 seconds".to_owned());
            }
            Ok(self
                .services
                .iter()
                .any(|known| known == service)
                .then(|| ItemInfo {
                    modified: Some("20991231000000Z".to_owned()),
                }))
        }

        fn read(
            &self,
            _service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<Secret>, String> {
            panic!("locating a sign-in must never read a secret");
        }

        fn list(&self, _prefix: &str) -> std::result::Result<Vec<Item>, String> {
            Err("/usr/bin/security did not answer within 20 seconds".to_owned())
        }
    }

    /// A folder at `name` in `home` with a credentials file and an account config naming
    /// `account_uuid`, as Claude Code leaves it where there is no Keychain.
    fn signed_in_folder(home: &Path, name: &str, account_uuid: &str) -> PathBuf {
        let folder = home.join(name);
        std::fs::create_dir_all(&folder).expect("makes the folder");
        std::fs::write(folder.join(".credentials.json"), "{}").expect("writes");
        std::fs::write(
            folder.join(".claude.json"),
            json!({ "oauthAccount": {
                "accountUuid": account_uuid,
                "emailAddress": format!("{account_uuid}@example.com"),
                "organizationUuid": "00000000-0000-0000-0000-000000000002"
            } })
            .to_string(),
        )
        .expect("writes account config");
        folder
    }

    #[test]
    fn the_fix_command_suits_the_shell_and_the_folder() {
        let home_folder = Path::new("/Users/example/.claude-work");
        assert_eq!(
            claude_in(home_folder, "~/.claude-work", false),
            "CLAUDE_CONFIG_DIR=~/.claude-work claude"
        );
        assert_eq!(
            claude_in(Path::new("/Users/example/My Claude"), "~/My Claude", false),
            "CLAUDE_CONFIG_DIR='/Users/example/My Claude' claude",
            "quoted, with the full path, since quotes stop ~ from expanding"
        );
        assert_eq!(
            claude_in(Path::new("/Users/example/it's"), "~/it's", false),
            r"CLAUDE_CONFIG_DIR='/Users/example/it'\''s' claude"
        );
        assert_eq!(
            claude_in(
                Path::new(r"C:\Users\example\.claude-work"),
                r"~\.claude-work",
                true
            ),
            r"$env:CLAUDE_CONFIG_DIR = 'C:\Users\example\.claude-work'; claude",
            "PowerShell, which neither expands ~ nor takes VAR=value before a command"
        );
        assert_eq!(
            claude_in(Path::new(r"C:\Users\o'brien\.claude"), r"~\.claude", true),
            r"$env:CLAUDE_CONFIG_DIR = 'C:\Users\o''brien\.claude'; claude"
        );
    }

    #[test]
    fn with_a_token_in_the_environment_its_folders_own_sign_in_is_another_account() {
        let home = tempfile::tempdir().expect("temp dir");
        let work = signed_in_folder(
            home.path(),
            "work-claude",
            "00000000-0000-0000-0000-00000000000b",
        );
        let env = Environment {
            config_dir: Some(work.display().to_string()),
            oauth_token: Some(Secret::new("sk-ant-oat01-QUOTADIAL-TEST-TOKEN")),
            ..env_in(home.path())
        };

        let logins = locate_all(&env, &NoKeychain, &[]).expect("locates").logins;

        let accounts: Vec<String> = logins
            .iter()
            .map(|login| login.account().account_id)
            .collect();
        assert_eq!(accounts.len(), 2, "{accounts:?}");
        assert!(accounts[0].starts_with("env:"));
        assert_eq!(accounts[1], "00000000-0000-0000-0000-00000000000b");
        assert_eq!(logins[1].folder, Folder::Named(work.clone()));
    }

    #[test]
    fn a_newer_sign_in_of_the_same_account_wins_unless_the_first_is_this_shells_own() {
        let home = tempfile::tempdir().expect("temp dir");
        let env = env_in(home.path());
        let identity = Identity {
            account_uuid: "00000000-0000-0000-0000-000000000001".into(),
            email: None,
            org_uuid: None,
            plan: None,
        };
        let keychain = |generation: &str| {
            Login::new(
                &env,
                LoginSource::Keychain {
                    service: "Claude Code-credentials-0000000a".into(),
                    account: None,
                },
                Some(generation.into()),
                Some(identity.clone()),
                Folder::Unknown,
            )
        };
        let file = |generation: &str| {
            Login::new(
                &env,
                LoginSource::File(home.path().join(".credentials.json")),
                Some(generation.into()),
                Some(identity.clone()),
                Folder::Default,
            )
        };

        // A Keychain time and a file time, compared as times.
        let newer_file = one_per_account(
            vec![
                keychain("20991231000000Z"),
                file("2100-01-01T00:00:00.000000000Z"),
            ],
            false,
        );
        assert!(matches!(newer_file[0].source, LoginSource::File(_)));
        let older_file = one_per_account(
            vec![
                keychain("20991231000000Z"),
                file("2099-12-30T00:00:00.000000000Z"),
            ],
            false,
        );
        assert!(matches!(older_file[0].source, LoginSource::Keychain { .. }));
        // This shell's own sign-in stays, even when another is newer.
        let own = one_per_account(
            vec![
                keychain("20991231000000Z"),
                file("2100-01-01T00:00:00.000000000Z"),
            ],
            true,
        );
        assert!(matches!(own[0].source, LoginSource::Keychain { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_a_folder_is_searched_like_the_folder() {
        let home = tempfile::tempdir().expect("temp dir");
        let real = signed_in_folder(
            home.path(),
            "claude-accounts-real",
            "00000000-0000-0000-0000-00000000000c",
        );
        std::os::unix::fs::symlink(&real, home.path().join(".claude-linked")).expect("links");

        let dirs = candidate_dirs(&env_in(home.path()), &[]).dirs;

        assert_eq!(
            dirs,
            [home.path().join(".claude-linked").display().to_string()]
        );
    }

    #[test]
    fn a_keychain_that_cannot_be_listed_is_searched_folder_by_folder() {
        let home = tempfile::tempdir().expect("temp dir");
        write_account_config(home.path());
        let work = home.path().join(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        let env = env_in(home.path());
        let work_item = env
            .for_config_dir(&work.display().to_string())
            .keychain_service();
        let keychain = UnlistableKeychain::holding(&[
            "Claude Code-credentials",
            &work_item,
            // An item whose folder is gone, which only a listing can find.
            "Claude Code-credentials-deadbeef",
        ]);

        let found = locate_all(&env, &keychain, &[]).expect("still searches");

        let services: Vec<Option<&str>> = found.logins.iter().map(Login::service).collect();
        assert_eq!(
            services,
            [Some("Claude Code-credentials"), Some(work_item.as_str())],
            "this shell's own sign-in and the folder's, found by name"
        );
        assert_eq!(found.logins[1].folder, Folder::Named(work));
        assert_eq!(
            found.unlisted.as_deref(),
            Some("/usr/bin/security did not answer within 20 seconds")
        );
    }

    #[test]
    fn a_keychain_that_stops_answering_is_not_asked_again() {
        let home = tempfile::tempdir().expect("temp dir");
        for name in [".claude-one", ".claude-two", ".claude-three"] {
            signed_in_folder(home.path(), name, "00000000-0000-0000-0000-00000000000d");
        }
        let keychain = UnlistableKeychain {
            // This shell's own lookup is answered; the next one fails.
            lookups_answered: 1,
            ..UnlistableKeychain::holding(&[])
        };

        let found = locate_all(&env_in(home.path()), &keychain, &[]).expect("still searches");

        assert_eq!(keychain.lookups(), 2, "one failed lookup, then none");
        assert_eq!(
            found.logins.len(),
            1,
            "the folders' credentials files are still read: one subscription"
        );
        assert!(matches!(found.logins[0].source, LoginSource::File(_)));
    }

    /// A keychain listing exactly the items for `services`. Reading secrets is not allowed.
    struct Listed {
        services: Vec<String>,
    }

    impl Keychain for Listed {
        fn find(
            &self,
            service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<ItemInfo>, String> {
            Ok(self
                .services
                .iter()
                .any(|known| known == service)
                .then(|| ItemInfo {
                    modified: Some("20991231000000Z".to_owned()),
                }))
        }

        fn read(
            &self,
            _service: &str,
            _account: Option<&str>,
        ) -> std::result::Result<Option<Secret>, String> {
            panic!("locating a sign-in must never read a secret");
        }

        fn list(&self, _prefix: &str) -> std::result::Result<Vec<Item>, String> {
            Ok(self
                .services
                .iter()
                .map(|service| Item {
                    service: service.clone(),
                    account: Some("example".into()),
                    info: ItemInfo {
                        modified: Some("20991231000000Z".to_owned()),
                    },
                })
                .collect())
        }
    }

    #[test]
    fn a_folder_given_to_claude_code_with_a_trailing_separator_is_still_its_own() {
        let home = tempfile::tempdir().expect("temp dir");
        let work = home.path().join(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        let env = env_in(home.path());
        // As `CLAUDE_CONFIG_DIR=~/.claude-work/ claude` would name it.
        let written = format!("{}{}", work.display(), std::path::MAIN_SEPARATOR);
        let service = env.for_config_dir(&written).keychain_service();

        let found = locate_all(
            &env,
            &Listed {
                services: vec![service.clone()],
            },
            &[],
        )
        .expect("locates");

        assert_eq!(found.logins.len(), 1);
        assert_eq!(found.logins[0].service(), Some(service.as_str()));
        assert_eq!(
            found.logins[0].folder,
            Folder::Named(PathBuf::from(&written)),
            "named as Claude Code was given it, so its fix makes the same item again"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_reached_through_a_linked_home_matches_the_real_path() {
        let real = tempfile::tempdir().expect("temp dir");
        let work = real.path().join(".claude-work");
        std::fs::create_dir_all(&work).expect("makes the folder");
        let links = tempfile::tempdir().expect("temp dir");
        let home = links.path().join("home");
        std::os::unix::fs::symlink(real.path(), &home).expect("links");
        let env = env_in(&home);
        // Claude Code was given the real path.
        let real_work = std::fs::canonicalize(&work).expect("resolves");
        let service = env
            .for_config_dir(&real_work.display().to_string())
            .keychain_service();

        let found = locate_all(
            &env,
            &Listed {
                services: vec![service.clone()],
            },
            &[],
        )
        .expect("locates");

        assert_eq!(found.logins.len(), 1);
        assert_eq!(found.logins[0].folder, Folder::Named(real_work));
    }

    #[test]
    fn a_relative_extra_folder_is_left_out() {
        let home = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(home.path().join("elsewhere")).expect("makes the folder");

        let candidates = candidate_dirs(
            &env_in(home.path()),
            &[PathBuf::from("claude-work"), PathBuf::from("~/elsewhere")],
        );

        assert_eq!(candidates.skipped, [PathBuf::from("claude-work")]);
        assert_eq!(
            candidates.dirs,
            [home.path().join("elsewhere").display().to_string()]
        );
    }

    #[test]
    fn an_unreadable_keychain_is_an_error_that_names_the_item() {
        let home = tempfile::tempdir().expect("temp dir");

        let error = locate(&env_in(home.path()), &BrokenKeychain).expect_err("fails");

        assert_eq!(
            error.to_string(),
            "could not read the sign-in from the macOS Keychain item \"Claude Code-credentials\": \
             /usr/bin/security exited with status 36"
        );
    }
}
