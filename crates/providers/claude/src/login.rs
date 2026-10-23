//! Finds the Claude Code sign-in this shell would use, without reading any secret.
//!
//! The order is Claude Code's own: `CLAUDE_CODE_OAUTH_TOKEN`, then on macOS the Keychain item for
//! the config directory, then `.credentials.json` (Claude Code 2.1.286 reads the Keychain first
//! and falls back to the file). One sign-in per shell is the M1 scope (decision D-023).

use std::io;
use std::path::PathBuf;

use chrono::{DateTime, SecondsFormat, Utc};
use quotadial_core::{Account, Error, Result};

use crate::ID;
use crate::environment::{Environment, sha256_hex};
use crate::identity::{self, Identity};
use crate::keychain::Keychain;
use crate::secret::Secret;

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
    /// Where the token was read, as shown to the user, so they can see exactly what was touched.
    pub(crate) fn location(&self) -> String {
        match self {
            Self::Environment(_) => "the CLAUDE_CODE_OAUTH_TOKEN environment variable".to_owned(),
            Self::Keychain { service, .. } => format!("macOS Keychain item \"{service}\""),
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
}

impl Login {
    /// The account this sign-in belongs to, as far as is known without asking Anthropic.
    pub(crate) fn account(&self) -> Account {
        match &self.identity {
            Some(identity) => account_for(identity),
            None => {
                let (account_id, display_name) = self.source.stand_in();
                Account {
                    provider: ID.to_owned(),
                    account_id,
                    display_name,
                    plan: None,
                    org: None,
                }
            }
        }
    }
}

pub(crate) fn account_for(identity: &Identity) -> Account {
    Account {
        provider: ID.to_owned(),
        account_id: identity.account_uuid.clone(),
        display_name: identity
            .email
            .clone()
            .unwrap_or_else(|| identity.account_uuid.clone()),
        plan: identity.plan.clone(),
        org: identity.org_uuid.clone(),
    }
}

/// Finds the sign-in Claude Code would use with this environment, or `None` when it is signed out.
pub(crate) fn locate(env: &Environment, keychain: &dyn Keychain) -> Result<Option<Login>> {
    if let Some(token) = &env.oauth_token {
        return Ok(Some(Login {
            source: LoginSource::Environment(token.clone()),
            generation: None,
            identity: None,
        }));
    }
    let identity = || {
        env.account_config_file()
            .and_then(|file| identity::from_account_config(&file))
    };

    let service = env.keychain_service();
    let account = env.keychain_account();
    match keychain.find(&service, account.as_deref()) {
        Ok(Some(item)) => {
            return Ok(Some(Login {
                source: LoginSource::Keychain { service, account },
                generation: item.modified,
                identity: identity(),
            }));
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
        Ok(metadata) if metadata.is_file() => Ok(Some(Login {
            generation: metadata.modified().ok().map(|modified| {
                DateTime::<Utc>::from(modified).to_rfc3339_opts(SecondsFormat::Nanos, true)
            }),
            source: LoginSource::File(file),
            identity: identity(),
        })),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::Credentials {
            provider: ID,
            location: file.display().to_string(),
            reason: error.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::keychain::{ItemInfo, NoKeychain};

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
                account_id: "00000000-0000-0000-0000-000000000001".into(),
                display_name: "user@example.com".into(),
                plan: Some("max".into()),
                org: Some("00000000-0000-0000-0000-000000000002".into()),
            }
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

    #[test]
    fn an_unreadable_keychain_is_an_error_that_names_the_item() {
        let home = tempfile::tempdir().expect("temp dir");

        let error = locate(&env_in(home.path()), &BrokenKeychain).expect_err("fails");

        assert_eq!(
            error.to_string(),
            "claude: could not read credentials from macOS Keychain item \
             \"Claude Code-credentials\": /usr/bin/security exited with status 36"
        );
    }
}
