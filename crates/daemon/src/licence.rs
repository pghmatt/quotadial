//! The licence key this copy keeps (D-099): `licence` in the data folder, readable only by the
//! user, added and removed by `quotadial licence` and the app's Settings. The key is checked each
//! time it is read, so a licence that ends turns the copy back into QuotaDial Free by itself.

use std::fmt;
use std::fs;
use std::io;

use chrono::NaiveDate;
use quotadial_core::licence::canonical;
use quotadial_core::{KeyError, Licence, Standing, Tier};

use crate::{DataDir, write_private};

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
                "That licence ended on {until}, so it can't add QuotaDial Pro."
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
            "That licence ended on 2026-10-31, so it can't add QuotaDial Pro."
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
        assert!(after.text.contains("ended on 2026-11-30"), "{}", after.text);
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
