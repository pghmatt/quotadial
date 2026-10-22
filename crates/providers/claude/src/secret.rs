//! A credential that cannot end up in a log line, an error message or `Debug` output by accident.

use std::fmt;

/// An OAuth token. `Debug` prints `Secret(redacted)` and there is deliberately no `Display`, so
/// formatting a struct that holds one never reveals it. The value comes out only through
/// [`Secret::expose`], at the one place it is sent: the `Authorization` header.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Secret(String);

impl Secret {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(redacted)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_never_shows_the_value() {
        let secret = Secret::new("sk-ant-oat01-not-a-real-token");

        assert_eq!(format!("{secret:?}"), "Secret(redacted)");
        assert_eq!(
            format!("{:?}", Some(secret.clone())),
            "Some(Secret(redacted))"
        );
        assert_eq!(secret.expose(), "sk-ant-oat01-not-a-real-token");
    }
}
