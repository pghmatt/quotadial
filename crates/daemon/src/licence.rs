//! The licence key this copy keeps (D-099), and the one switch every Pro feature asks (D-100):
//! `licence` in the data folder, readable only by the user, added and removed by `quotadial
//! licence` and the app's Settings. The key is checked each time it is read, so a licence added,
//! removed or ended counts at the next ask, without a restart.

use std::fmt;
use std::fs;
use std::io;

use chrono::NaiveDate;
use quotadial_core::licence::canonical;
use quotadial_core::{
    AccountLimit, KeyError, LeftOut, Licence, ProFeature, Provider, Standing, Tier,
};

use crate::{DataDir, write_private};

/// What the copy keeping its data in a data folder may do: the tier of the licence key kept
/// there, read afresh at each ask. With no data folder there is no key, so QuotaDial Free.
#[derive(Debug, Clone, Default)]
pub struct Licensing {
    dir: Option<DataDir>,
}

impl Licensing {
    /// The licensing of the copy keeping its data in `dir`.
    #[must_use]
    pub fn new(dir: Option<&DataDir>) -> Self {
        Self { dir: dir.cloned() }
    }

    /// What the copy is now. Reading the key and checking its signature takes well under a
    /// millisecond, so nothing is cached and a change counts at once.
    #[must_use]
    pub fn standing(&self) -> Standing {
        let today = quotadial_core::licence::today();
        match &self.dir {
            Some(dir) => standing(dir, today),
            None => Standing::of(None, today),
        }
    }

    /// The copy's tier now: the one switch.
    #[must_use]
    pub fn tier(&self) -> Tier {
        self.standing().tier
    }

    /// Whether the copy has `feature` now.
    #[must_use]
    pub fn allows(&self, feature: ProFeature) -> bool {
        self.tier().allows(feature)
    }

    /// The most accounts each provider shows, asked at each discovery: every account on QuotaDial
    /// Pro, up to the `max_accounts` setting, and one, this shell's own, on QuotaDial Free.
    #[must_use]
    pub fn account_limit(&self, setting: Option<usize>) -> AccountLimit {
        let licensing = self.clone();
        AccountLimit::new(move || {
            if licensing.allows(ProFeature::Accounts) {
                setting
            } else {
                Some(1)
            }
        })
    }
}

/// Whether the cost scan has logs of `provider`'s to read, as [`Provider::log_folders`] says: what
/// the week's cost is offered on. A provider that keeps none, such as Gemini CLI or the example
/// build's, leaves nothing for the cost to be read from (D-125).
#[must_use]
pub fn keeps_logs(provider: &dyn Provider) -> bool {
    !provider.log_folders().is_empty()
}

/// What a copy with `tier` leaves out of its report, each part asked of [`Tier::allows`] for its
/// own feature: the accounts each provider held back at its last discovery, and, when `costs` says
/// the daemon would read logs for an account (one of a provider that [`keeps_logs`]), the week's
/// cost. Nothing on QuotaDial Pro.
#[must_use]
pub fn left_out<P: AsRef<dyn Provider>>(tier: Tier, providers: &[P], costs: bool) -> Vec<LeftOut> {
    let mut left_out = Vec::new();
    if !tier.allows(ProFeature::Accounts) {
        left_out.extend(
            providers
                .iter()
                .map(AsRef::as_ref)
                .filter(|provider| provider.held_back() > 0)
                .map(|provider| {
                    LeftOut::accounts(provider.id(), provider.name(), provider.held_back())
                }),
        );
    }
    if costs && !tier.allows(ProFeature::CostScan) {
        left_out.push(LeftOut::feature(ProFeature::CostScan));
    }
    left_out
}

/// Why a key was not added.
#[derive(Debug)]
pub enum LicenceError {
    /// The key holds no licence, and why.
    Key(KeyError),
    /// The licence the key holds ended on this day.
    Ended(NaiveDate),
    /// The key could not be kept.
    Io(io::Error),
}

impl fmt::Display for LicenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(error) => write!(formatter, "{error}"),
            Self::Ended(until) => write!(
                formatter,
                "That licence ended on {}, so it can't add QuotaDial Pro.",
                quotadial_core::text::date(*until)
            ),
            Self::Io(error) => write!(
                formatter,
                "The licence key could not be kept ({error}). Check that the data folder can be \
                 written, then add the key again."
            ),
        }
    }
}

impl std::error::Error for LicenceError {}

/// What this copy is on `today`, from the key kept in `dir`. An empty file is no key.
#[must_use]
pub fn standing(dir: &DataDir, today: NaiveDate) -> Standing {
    match fs::read_to_string(dir.licence()) {
        Ok(key) if key.trim().is_empty() => Standing::of(None, today),
        Ok(key) => Standing::of(Some(&key), today),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Standing::of(None, today),
        Err(error) => Standing {
            tier: Tier::Free,
            licence: None,
            text: format!(
                "QuotaDial Free: its licence key can't be read ({error}). Adding the key again \
                 replaces it."
            ),
        },
    }
}

/// Keeps `key` in `dir` when it holds a licence that holds on `today`, on one line, and returns
/// what the copy is then. A key that gives nothing is not kept, and the key kept before stays.
///
/// # Errors
///
/// [`LicenceError`], which says why in a sentence.
pub fn add(dir: &DataDir, key: &str, today: NaiveDate) -> Result<Standing, LicenceError> {
    let licence = Licence::verify(key).map_err(LicenceError::Key)?;
    if let Some(until) = licence.until.filter(|_| !licence.holds_on(today)) {
        return Err(LicenceError::Ended(until));
    }
    write_private(&dir.licence(), format!("{}\n", canonical(key)).as_bytes())
        .map_err(LicenceError::Io)?;
    Ok(standing(dir, today))
}

/// Removes the key kept in `dir`. Returns whether there was one.
///
/// # Errors
///
/// When the file is there but cannot be removed.
pub fn remove(dir: &DataDir) -> io::Result<bool> {
    match fs::remove_file(dir.licence()) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use quotadial_core::licence::TEST_SEED;
    use quotadial_core::{Plan, Tier};

    use super::*;

    fn day(text: &str) -> NaiveDate {
        text.parse().expect("a date")
    }

    fn key(id: &str, until: Option<&str>) -> String {
        Licence {
            id: id.into(),
            plan: Plan::Pro,
            issued: day("2026-10-24"),
            until: until.map(day),
        }
        .sign(&TEST_SEED)
    }

    fn dir() -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        (temp, dir)
    }

    #[test]
    fn a_key_is_kept_privately_on_one_line_and_gives_pro() {
        let (_temp, dir) = dir();
        let today = day("2026-11-01");
        let pro = key("order-1", None);

        assert_eq!(standing(&dir, today).tier, Tier::Free);
        let added = add(&dir, &format!("{}\n  {}\n", &pro[..50], &pro[50..]), today).expect("adds");

        assert_eq!(added.tier, Tier::Pro);
        assert_eq!(added.text, "QuotaDial Pro, licence order-1.");
        assert_eq!(
            fs::read_to_string(dir.licence()).expect("kept"),
            format!("{pro}\n")
        );
        assert_eq!(standing(&dir, today), added);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.licence())
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn a_key_that_gives_nothing_is_not_kept_and_the_last_one_stays() {
        let (_temp, dir) = dir();
        let today = day("2026-11-01");
        add(&dir, &key("order-1", None), today).expect("adds");

        let refused = [
            add(&dir, "hello", today),
            add(&dir, "QD1.abc.def", today),
            add(&dir, &key("order-2", Some("2026-10-31")), today),
        ];

        assert!(matches!(
            refused[0],
            Err(LicenceError::Key(KeyError::NotAKey))
        ));
        assert!(matches!(
            refused[1],
            Err(LicenceError::Key(KeyError::DoesNotCheckOut))
        ));
        let Err(ended) = &refused[2] else {
            panic!("an ended licence is refused");
        };
        assert_eq!(
            ended.to_string(),
            "That licence ended on 31 Oct 2026, so it can't add QuotaDial Pro."
        );
        assert_eq!(
            standing(&dir, today).licence.map(|licence| licence.id),
            Some("order-1".into())
        );
    }

    #[test]
    fn a_licence_that_ends_turns_the_copy_back_to_free_by_itself() {
        let (_temp, dir) = dir();
        add(&dir, &key("order-3", Some("2026-11-30")), day("2026-11-01")).expect("adds");

        assert_eq!(standing(&dir, day("2026-11-30")).tier, Tier::Pro);
        let after = standing(&dir, day("2026-12-01"));
        assert_eq!(after.tier, Tier::Free);
        assert!(
            after.text.contains("ended on 30 Nov 2026"),
            "{}",
            after.text
        );
    }

    #[test]
    fn licensing_reads_the_key_at_each_ask() {
        let (_temp, dir) = dir();
        let licensing = Licensing::new(Some(&dir));

        assert_eq!(licensing.tier(), Tier::Free);
        assert!(!licensing.allows(ProFeature::CostScan));
        add(
            &dir,
            &key("order-1", None),
            quotadial_core::licence::today(),
        )
        .expect("adds");
        assert_eq!(licensing.tier(), Tier::Pro);
        assert!(licensing.clone().allows(ProFeature::CostScan));
        remove(&dir).expect("removes");
        assert_eq!(licensing.tier(), Tier::Free);
        assert_eq!(Licensing::new(None).tier(), Tier::Free);
    }

    #[test]
    fn free_shows_one_account_per_provider_and_pro_the_setting() {
        let (_temp, dir) = dir();
        let licensing = Licensing::new(Some(&dir));

        assert_eq!(licensing.account_limit(None).get(), Some(1));
        assert_eq!(licensing.account_limit(Some(3)).get(), Some(1));
        add(
            &dir,
            &key("order-1", None),
            quotadial_core::licence::today(),
        )
        .expect("adds");
        assert_eq!(licensing.account_limit(None).get(), None);
        assert_eq!(licensing.account_limit(Some(3)).get(), Some(3));
    }

    #[test]
    fn removing_the_key_makes_the_copy_free() {
        let (_temp, dir) = dir();
        let today = day("2026-11-01");
        add(&dir, &key("order-1", None), today).expect("adds");

        assert!(remove(&dir).expect("removes"));
        assert!(!remove(&dir).expect("nothing to remove"));
        assert_eq!(standing(&dir, today), Standing::of(None, today));
    }

    #[test]
    fn an_empty_or_damaged_file_says_what_it_is() {
        let (_temp, dir) = dir();
        let today = day("2026-11-01");
        dir.create().expect("creates");

        fs::write(dir.licence(), "\n").expect("writes");
        assert_eq!(standing(&dir, today), Standing::of(None, today));

        fs::write(dir.licence(), "QD1.damaged.key\n").expect("writes");
        let damaged = standing(&dir, today);
        assert_eq!(damaged.tier, Tier::Free);
        assert!(
            damaged.text.contains("doesn't check out"),
            "{}",
            damaged.text
        );
    }
}
