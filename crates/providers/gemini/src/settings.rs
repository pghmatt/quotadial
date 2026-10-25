//! The one thing QuotaDial reads in Gemini CLI's settings: how it signs in, the string at
//! `security.auth.selectedType` (`packages/cli/src/config/settingsSchema.ts` in Gemini CLI's source,
//! with its values in `AuthType`, `packages/core/src/core/contentGenerator.ts`).
//!
//! The file can hold other tools' settings and secrets, such as an MCP server's environment, so
//! nothing else in it is kept, logged or shown: the reader walks to that one field, skips everything
//! else without copying it, wipes the file's bytes when it is done, and gives back only which way
//! Gemini CLI signs in. A file it cannot read names no way, and no words of it are ever repeated.
//! Gemini CLI reads the file with comments allowed (`strip-json-comments`, in
//! `packages/cli/src/config/settings.ts`), so this reader allows them too, and refuses what Gemini
//! CLI refuses, such as a trailing comma.
//!
//! CodexBar reads the same field (`Providers/Gemini/GeminiStatusProbe.swift`, `currentAuthType`)
//! before it reads the sign-in and asks Google's servers, which QuotaDial never does (D-118).

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use zeroize::Zeroizing;

/// Where the field is: `security.auth.selectedType`.
const FIELD: [&str; 3] = ["security", "auth", "selectedType"];

/// How Gemini CLI's settings say it signs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Method {
    /// `oauth-personal`: "Sign in with Google".
    Google,
    /// `gemini-api-key`: "Use Gemini API Key".
    ApiKey,
    /// `vertex-ai`: "Vertex AI".
    VertexAi,
    /// Any other value, such as `compute-default-credentials`, `cloud-shell` or `gateway`.
    Other,
}

/// The most of the file QuotaDial reads. Gemini CLI's settings run to a few kilobytes; a larger file
/// is taken as one QuotaDial cannot read.
const MOST: u64 = 1024 * 1024;

impl Method {
    /// The way `value` names, as Gemini CLI compares it: exactly, with an empty one naming none.
    fn named(value: &str) -> Option<Self> {
        match value {
            "" => None,
            "oauth-personal" => Some(Self::Google),
            "gemini-api-key" => Some(Self::ApiKey),
            "vertex-ai" => Some(Self::VertexAi),
            _ => Some(Self::Other),
        }
    }
}

/// How Gemini CLI's settings at `path` say it signs in, or `None` when the file is missing, cannot
/// be read, or names no way. The file's bytes are wiped from memory once read.
pub(crate) fn read(path: &Path) -> Option<Method> {
    // Only a plain file is opened: opening a pipe left in its place would wait for a writer forever.
    if !std::fs::metadata(path).ok()?.is_file() {
        return None;
    }
    let file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    if length > MOST {
        return None;
    }
    // Sized to the file, so reading it never moves the bytes and leaves a copy behind.
    let mut bytes = Zeroizing::new(Vec::with_capacity(
        usize::try_from(length).ok()?.saturating_add(1),
    ));
    file.take(MOST + 1).read_to_end(&mut bytes).ok()?;
    if u64::try_from(bytes.len()).ok()? > MOST {
        return None;
    }
    blank_comments(&mut bytes);
    method(&bytes)
}

/// How `settings`, with its comments already blanked, says Gemini CLI signs in: `None` unless it is
/// one JSON object, and nothing more, whose field holds a string.
pub(crate) fn method(settings: &[u8]) -> Option<Method> {
    let mut deserializer = serde_json::Deserializer::from_slice(settings);
    let method = Field { path: &FIELD }.deserialize(&mut deserializer).ok()?;
    deserializer.end().ok()?;
    method
}

/// Blanks out the comments in `text` with spaces, keeping line breaks and every other byte where it
/// was, as `strip-json-comments` does for Gemini CLI: `//` to the end of the line and `/*` to `*/`,
/// outside strings, and an unclosed `/*` to the end.
pub(crate) fn blank_comments(text: &mut [u8]) {
    let blank = |byte: &mut u8| {
        if *byte != b'\n' {
            *byte = b' ';
        }
    };
    let mut at = 0;
    let mut in_string = false;
    while at < text.len() {
        match (in_string, text[at], text.get(at + 1)) {
            // An escaped byte, such as `\"`, never ends a string.
            (true, b'\\', _) => at += 2,
            (true, b'"', _) | (false, b'"', _) => {
                in_string = !in_string;
                at += 1;
            }
            (false, b'/', Some(b'/')) => {
                while at < text.len() && text[at] != b'\n' {
                    blank(&mut text[at]);
                    at += 1;
                }
            }
            (false, b'/', Some(b'*')) => {
                let end = text[at + 2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(text.len(), |found| at + 2 + found + 2);
                text[at..end].iter_mut().for_each(blank);
                at = end;
            }
            _ => at += 1,
        }
    }
}

/// Walks to the field at `path` and gives the way it names, skipping everything else unread. A
/// value that is not an object on the way, or not a string at the end, names no way.
struct Field<'a> {
    path: &'a [&'a str],
}

impl<'de> DeserializeSeed<'de> for Field<'_> {
    type Value = Option<Method>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Field<'_> {
    type Value = Option<Method>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Gemini CLI's settings")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut found = None;
        while let Some(wanted) = map.next_key_seed(KeyIs(self.path.first().copied()))? {
            if wanted {
                // As in JavaScript, the last of a repeated key wins, even when it names nothing.
                found = map.next_value_seed(Field {
                    path: &self.path[1..],
                })?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(found)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(None)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(if self.path.is_empty() {
            Method::named(value)
        } else {
            None
        })
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}

/// Whether a key is the one wanted, read without keeping it. `None` wants no key: the path ended
/// at an object, which names no way.
struct KeyIs<'a>(Option<&'a str>);

impl<'de> DeserializeSeed<'de> for KeyIs<'_> {
    type Value = bool;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyIs<'_> {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a key")
    }

    fn visit_str<E>(self, key: &str) -> Result<Self::Value, E> {
        Ok(self.0 == Some(key))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use quotadial_core::testing::json;
    use serde_json::json;

    use super::*;

    /// What `text` says, read as a file would be.
    fn method_in(text: &str) -> Option<Method> {
        let mut bytes = text.as_bytes().to_vec();
        blank_comments(&mut bytes);
        method(&bytes)
    }

    fn naming(value: &str) -> String {
        json!({ "security": { "auth": { "selectedType": value } } }).to_string()
    }

    #[test]
    fn each_way_gemini_cli_signs_in_is_read_from_its_one_field() {
        for (value, expected) in [
            ("oauth-personal", Some(Method::Google)),
            ("gemini-api-key", Some(Method::ApiKey)),
            ("vertex-ai", Some(Method::VertexAi)),
            ("compute-default-credentials", Some(Method::Other)),
            ("cloud-shell", Some(Method::Other)),
            ("gateway", Some(Method::Other)),
            // CodexBar's older name, which Gemini CLI no longer has.
            ("api-key", Some(Method::Other)),
            ("OAUTH-PERSONAL", Some(Method::Other)),
            // Gemini CLI takes an empty choice as none.
            ("", None),
        ] {
            assert_eq!(method_in(&naming(value)), expected, "{value:?}");
        }
    }

    #[test]
    fn settings_that_name_no_way_read_as_none() {
        for text in [
            "{}",
            r#"{ "security": {} }"#,
            r#"{ "security": { "auth": {} } }"#,
            r#"{ "security": { "auth": { "selectedType": null } } }"#,
            r#"{ "security": { "auth": { "selectedType": 7 } } }"#,
            r#"{ "security": { "auth": { "selectedType": true } } }"#,
            r#"{ "security": { "auth": { "selectedType": ["oauth-personal"] } } }"#,
            r#"{ "security": { "auth": { "selectedType": { "type": "oauth-personal" } } } }"#,
            r#"{ "security": { "auth": "oauth-personal" } }"#,
            r#"{ "security": "oauth-personal" }"#,
            r#"{ "selectedType": "oauth-personal", "auth": { "selectedType": "vertex-ai" } }"#,
            // The name older settings used, which Gemini CLI no longer reads.
            r#"{ "selectedAuthType": "oauth-personal" }"#,
            // Not an object: Gemini CLI takes the file as holding no settings.
            r#"[{ "security": { "auth": { "selectedType": "oauth-personal" } } }]"#,
            r#""oauth-personal""#,
        ] {
            assert_eq!(method_in(text), None, "{text}");
        }
    }

    #[test]
    fn comments_are_allowed_where_gemini_cli_allows_them() {
        let text = r#"// How Gemini CLI signs in.
            {
              /* Chosen with /auth. */
              "security": { "auth": { "selectedType": "vertex-ai" } }, // after a value
              "note": "a // in a string and a /* in one */ are not comments",
              "path": "C:\\Users\\me\\" /* an escaped backslash ends the string */
            }
            // The end."#;

        assert_eq!(method_in(text), Some(Method::VertexAi));
    }

    #[test]
    fn the_last_of_a_repeated_key_wins_as_gemini_cli_reads_it() {
        assert_eq!(
            method_in(
                r#"{ "security": { "auth": { "selectedType": "gemini-api-key",
                     "selectedType": "oauth-personal" } } }"#
            ),
            Some(Method::Google)
        );
        assert_eq!(
            method_in(
                r#"{ "security": { "auth": { "selectedType": "vertex-ai" } }, "security": {} }"#
            ),
            None
        );
    }

    #[test]
    fn a_key_written_with_escapes_is_the_same_key() {
        assert_eq!(
            method_in(r#"{ "security": { "auth": { "selected\u0054ype": "vertex-\u0061i" } } }"#),
            Some(Method::VertexAi)
        );
    }

    #[test]
    fn a_file_gemini_cli_could_not_read_names_no_way() {
        let whole = naming("oauth-personal");
        for text in [
            String::new(),
            "not JSON".to_owned(),
            whole[..whole.len() - 1].to_owned(),
            whole.replace("}}}", "},}}"),
            format!("{whole} and more"),
            format!("\u{feff}{whole}"),
            format!("/* never closed {whole}"),
        ] {
            assert_eq!(method_in(&text), None, "{text:?}");
        }
    }

    #[test]
    fn blanking_comments_keeps_every_other_byte_where_it_was() {
        let mut text = b"{\"a\": \"//x\"} // gone\n/* gone\ntoo */ 1 /* \xc3\xa9 */".to_vec();

        blank_comments(&mut text);

        let expected = format!(
            "{{\"a\": \"//x\"}} {}\n{}\n{} 1 {}",
            " ".repeat(7),
            " ".repeat(7),
            " ".repeat(6),
            " ".repeat(8)
        );
        assert_eq!(String::from_utf8(text).expect("UTF-8"), expected);
    }

    #[test]
    fn blanking_stops_at_the_end_of_the_file_whatever_is_left_open() {
        for (text, blanked) in [
            ("\"\"\"\\", "\"\"\"\\"),
            ("\"\\", "\"\\"),
            ("/*\n", "  \n"),
            ("/*", "  "),
            ("/", "/"),
            ("//", "  "),
            ("a // b", "a     "),
        ] {
            let mut bytes = text.as_bytes().to_vec();
            blank_comments(&mut bytes);
            assert_eq!(
                String::from_utf8(bytes).expect("UTF-8"),
                blanked,
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_missing_file_or_a_folder_names_no_way() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("settings.json");

        assert_eq!(read(&file), None, "missing");
        std::fs::write(&file, naming("gemini-api-key")).expect("writes");
        assert_eq!(read(&file), Some(Method::ApiKey));
        assert_eq!(read(temp.path()), None, "a folder");
    }

    #[cfg(unix)]
    #[test]
    fn a_pipe_in_place_of_the_file_is_never_opened() {
        let temp = tempfile::tempdir().expect("temp dir");
        let pipe = temp.path().join("settings.json");
        let made = std::process::Command::new("mkfifo")
            .arg(&pipe)
            .status()
            .expect("runs mkfifo");
        assert!(made.success(), "mkfifo {made}");
        let (sent, read_back) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sent.send(read(&pipe));
        });

        assert_eq!(
            read_back.recv_timeout(std::time::Duration::from_secs(5)),
            Ok(None),
            "reading waited on a pipe that nothing writes to"
        );
    }

    #[test]
    fn a_file_larger_than_settings_ever_are_names_no_way() {
        let temp = tempfile::tempdir().expect("temp dir");
        let file = temp.path().join("settings.json");
        let padding = " ".repeat(usize::try_from(MOST).expect("fits"));
        std::fs::write(&file, format!("{}{padding}", naming("vertex-ai"))).expect("writes");

        assert_eq!(read(&file), None);
    }

    /// Ways Gemini CLI signs in, and others.
    const VALUES: &[&str] = &[
        "oauth-personal",
        "gemini-api-key",
        "vertex-ai",
        "compute-default-credentials",
        "cloud-shell",
        "gateway",
        "",
    ];

    /// How a file can end mid-way: in a string, after an escape, or at the start or inside of a
    /// comment.
    const ENDINGS: &[&str] = &["\"", "\"\\", "\"a\\", "/", "/*", "/* a *", "//", "*/", "\\"];

    /// A settings file: any bytes, JSON-like punctuation with comments and escapes, the same cut off
    /// mid-way, any JSON, or any JSON with any value at the field and comments around it.
    fn settings() -> impl Strategy<Value = Vec<u8>> {
        prop_oneof![
            any::<Vec<u8>>(),
            "[{}\\[\\]\":,/*\\\\ \\na-zA-Z0-9_-]{0,160}".prop_map(String::into_bytes),
            (
                "[{}\\[\\]\":,/*\\\\ \\na-z]{0,40}",
                prop::sample::select(ENDINGS)
            )
                .prop_map(|(start, ending)| format!("{start}{ending}").into_bytes()),
            json().prop_map(|value| value.to_string().into_bytes()),
            (json(), json(), json()).prop_map(|(at, beside, around)| {
                json!({ "security": { "auth": { "selectedType": at }, "beside": beside },
                        "around": around })
                .to_string()
                .into_bytes()
            }),
            (prop::sample::select(VALUES), ".{0,20}", json()).prop_map(
                |(value, comment, around)| {
                    let field = naming(value);
                    let field = &field[1..field.len() - 1];
                    format!("// {comment}\n/* {comment} */ {{ \"around\": {around}, {field} }}")
                        .into_bytes()
                }
            ),
        ]
    }

    proptest! {
        #[test]
        fn no_settings_make_the_reader_panic(settings in settings()) {
            let mut blanked = settings.clone();
            blank_comments(&mut blanked);
            prop_assert_eq!(blanked.len(), settings.len());
            for (before, after) in settings.iter().zip(&blanked) {
                prop_assert!(after == before || (*after == b' ' && *before != b'\n'));
            }
            let _ = method(&blanked);
        }
    }
}
