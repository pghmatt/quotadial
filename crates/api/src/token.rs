//! The local API's bearer token: 32 random bytes, hex-encoded, generated the first time the daemon
//! starts and kept in the data directory, readable only by the user. Every request must present it,
//! so another user on the machine, or a web page that reaches `127.0.0.1`, reads nothing.

use std::fs;
use std::io;

use quotadial_core::Secret;
use quotadial_daemon::{DataDir, write_private};

/// Hex characters in a token.
const LENGTH: usize = 64;

/// The API's bearer token.
#[derive(Debug, Clone)]
pub struct Token(Secret);

impl Token {
    /// The token kept in `dir`, created the first time.
    ///
    /// # Errors
    ///
    /// When the token file cannot be read or written, or the system has no random source.
    pub fn load_or_create(dir: &DataDir) -> io::Result<Self> {
        if let Some(token) = Self::load(dir) {
            return Ok(token);
        }
        let mut bytes = [0_u8; LENGTH / 2];
        getrandom::fill(&mut bytes).map_err(io::Error::other)?;
        let token: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        // Written privately through a temporary file, so no reader sees half a token.
        write_private(&dir.token(), token.as_bytes())?;
        Ok(Self(Secret::new(token)))
    }

    /// The token kept in `dir`, if there is a valid one.
    #[must_use]
    pub fn load(dir: &DataDir) -> Option<Self> {
        let text = Secret::new(fs::read_to_string(dir.token()).ok()?);
        let token = text.expose().trim();
        (token.len() == LENGTH && token.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| Self(Secret::new(token)))
    }

    /// The value, for the `Authorization` header.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose()
    }

    /// Whether `presented` is this token, compared in constant time.
    #[must_use]
    pub fn matches(&self, presented: &str) -> bool {
        let (expected, presented) = (self.0.expose().as_bytes(), presented.as_bytes());
        expected.len() == presented.len()
            && expected
                .iter()
                .zip(presented)
                .fold(0, |difference, (a, b)| difference | (a ^ b))
                == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> (tempfile::TempDir, DataDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        (temp, dir)
    }

    #[test]
    fn a_token_is_made_once_and_kept_private() {
        let (_temp, dir) = dir();

        let first = Token::load_or_create(&dir).expect("creates");
        let again = Token::load_or_create(&dir).expect("loads");

        assert_eq!(first.expose().len(), 64);
        assert!(first.expose().bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(first.expose(), again.expose());
        assert!(!format!("{first:?}").contains(first.expose()));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.token())
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn a_damaged_token_is_replaced() {
        let (_temp, dir) = dir();
        dir.create().expect("creates");
        fs::write(dir.token(), "not a token").expect("writes");

        assert!(Token::load(&dir).is_none());
        let token = Token::load_or_create(&dir).expect("creates");
        assert_eq!(token.expose().len(), 64);
    }

    #[test]
    fn only_the_exact_token_matches() {
        let (_temp, dir) = dir();
        let token = Token::load_or_create(&dir).expect("creates");

        let mut wrong = token.expose().to_owned();
        let last = wrong.pop().expect("a character");
        wrong.push(if last == '0' { '1' } else { '0' });

        assert!(token.matches(token.expose()));
        assert!(!token.matches(&wrong));
        assert!(!token.matches(&token.expose()[1..]));
        assert!(!token.matches(""));
    }
}
