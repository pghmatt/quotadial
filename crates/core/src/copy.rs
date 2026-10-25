//! Which copy of QuotaDial this is. A release is always QuotaDial itself. The example build (D-117),
//! made with the `fake` feature for pictures of the app, shows made-up accounts and keeps its own
//! data folder, cache folder and API port, read from its own variable, so it can run beside a real
//! copy on the same machine without ever taking that copy's lock, reading its token, or sharing its
//! history or cache. `quotadial-providers`' `fake` feature turns this on; no release does.

/// Whether this is the example build.
pub const EXAMPLE: bool = cfg!(feature = "fake");

/// QuotaDial's folder in the operating system's data and cache directories: `quotadial`, or
/// `quotadial-fake` in the example build.
pub const FOLDER: &str = if EXAMPLE {
    "quotadial-fake"
} else {
    "quotadial"
};

/// The environment variable that puts the data folder elsewhere: `QUOTADIAL_DATA_DIR`, or
/// `QUOTADIAL_FAKE_DATA_DIR` in the example build, so a variable set for a real copy can never point
/// the example build at that copy's data.
pub const DATA_DIR_VARIABLE: &str = if EXAMPLE {
    "QUOTADIAL_FAKE_DATA_DIR"
} else {
    "QUOTADIAL_DATA_DIR"
};

#[cfg(test)]
mod tests {
    use super::*;

    /// What a release has always used, which a release must keep, or a person's history, token and
    /// lock would be left behind in a folder it no longer reads.
    const RELEASE_FOLDER: &str = "quotadial";
    const RELEASE_VARIABLE: &str = "QUOTADIAL_DATA_DIR";

    #[test]
    fn a_release_keeps_its_folder_and_variable_and_the_example_build_shares_neither() {
        assert_eq!(EXAMPLE, cfg!(feature = "fake"));
        if EXAMPLE {
            assert_eq!(FOLDER, "quotadial-fake");
            assert_eq!(DATA_DIR_VARIABLE, "QUOTADIAL_FAKE_DATA_DIR");
            assert_ne!(FOLDER, RELEASE_FOLDER);
            assert_ne!(DATA_DIR_VARIABLE, RELEASE_VARIABLE);
        } else {
            assert_eq!(FOLDER, RELEASE_FOLDER);
            assert_eq!(DATA_DIR_VARIABLE, RELEASE_VARIABLE);
        }
    }
}
