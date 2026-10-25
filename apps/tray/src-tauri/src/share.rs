//! The picture of an account's history to share (D-115), a QuotaDial Pro feature. The window draws
//! it from the History view's own numbers, since it already draws those charts, and sends the PNG
//! here as the request's raw body. This module checks it and saves it in the Downloads folder, as
//! Export CSV saves its file, under a name chosen here and never by the window. QuotaDial sends the
//! picture nowhere.

use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use quotadial_core::ProFeature;
use quotadial_daemon::Licensing;
use tauri::http::HeaderMap;
use tauri::ipc::InvokeBody;

/// The picture's width in pixels: 1200 CSS pixels drawn at twice that, in the 16:9 shape social
/// feeds show whole.
pub const WIDTH: u32 = 2400;
/// The picture's height in pixels: 675 CSS pixels drawn at twice that.
pub const HEIGHT: u32 = 1350;
/// The largest picture saved. The window's are a few hundred kilobytes, so anything this large is
/// not one of them.
pub const MOST_BYTES: usize = 4 * 1024 * 1024;
/// The request header naming the account a picture is of: its settings key, written as
/// `encodeURIComponent` writes it, since a header carries only plain ASCII.
pub const ACCOUNT_HEADER: &str = "quotadial-account";

/// Every PNG's first eight bytes.
const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
/// The chunk every PNG ends with: no data, the type `IEND`, and its checksum.
const END: [u8; 12] = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82];
/// The signature, the header chunk (length, type, 13 bytes of data, checksum) and the end chunk:
/// the least a PNG can be.
const SMALLEST: usize = SIGNATURE.len() + 25 + END.len();
/// The most bytes of a label a file name keeps: well inside the 255 every file system allows,
/// whatever the script.
const LABEL_BYTES: usize = 80;
/// The marks a label keeps in a file name besides letters, figures and spaces: none of them means
/// anything to a file system.
const MARKS: [char; 8] = ['-', '_', '(', ')', '+', '&', '\'', ','];
/// The most pictures of one account on one day the Downloads folder takes before QuotaDial asks
/// for room.
const MOST_COPIES: u32 = 999;

/// What a refusal of what came ends with: the picture was not saved, and what to do.
const NOTHING_SAVED: &str = "so nothing was saved; if pressing Share image again does the \
                             same, choose Send feedback in Help";

/// The picture in `body`, when the copy `licensing` speaks for may save one: on QuotaDial Pro, a
/// whole PNG of exactly [`WIDTH`] by [`HEIGHT`] pixels and at most [`MOST_BYTES`]. Its signature
/// and header are read by hand, its header's checksum checked, and it must end with the end chunk.
///
/// # Errors
///
/// On QuotaDial Free, the feature's line ([`ProFeature::line`]), whatever came; otherwise what is
/// wrong with what came, naming the fix.
pub fn picture<'a>(licensing: &Licensing, body: &'a InvokeBody) -> Result<&'a [u8], String> {
    if !licensing.allows(ProFeature::ShareImage) {
        return Err(ProFeature::ShareImage.line());
    }
    let InvokeBody::Raw(bytes) = body else {
        return Err(format!(
            "QuotaDial was sent the picture in a form it cannot read, {NOTHING_SAVED}."
        ));
    };
    if bytes.len() > MOST_BYTES {
        return Err(format!(
            "The picture is over {} MB, larger than any QuotaDial draws, {NOTHING_SAVED}.",
            MOST_BYTES / (1024 * 1024)
        ));
    }
    let (width, height) = size(bytes).ok_or_else(|| {
        format!("QuotaDial was sent something other than a whole PNG picture, {NOTHING_SAVED}.")
    })?;
    if (width, height) != (WIDTH, HEIGHT) {
        return Err(format!(
            "The picture is {width} by {height} pixels, not {WIDTH} by {HEIGHT}, {NOTHING_SAVED}."
        ));
    }
    Ok(bytes)
}

/// A whole PNG's width and height, from its header: `None` for anything else.
fn size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < SMALLEST || !bytes.starts_with(&SIGNATURE) || !bytes.ends_with(&END) {
        return None;
    }
    // The first chunk is the header: 13 bytes of data, then its checksum over type and data.
    let header = &bytes[SIGNATURE.len()..SIGNATURE.len() + 25];
    let (length, rest) = header.split_at(4);
    let (typed, checksum) = rest.split_at(17);
    if length != 13_u32.to_be_bytes() || !typed.starts_with(b"IHDR") {
        return None;
    }
    if checksum != crc32(typed).to_be_bytes() {
        return None;
    }
    let number = |at: usize| {
        <[u8; 4]>::try_from(&typed[at..at + 4])
            .ok()
            .map(u32::from_be_bytes)
    };
    Some((number(4)?, number(8)?))
}

/// What the picture of an account's history, and its file, call the account: its provider and the
/// name given it, or its provider and plan, or its provider alone. Never its address: a name or
/// plan with an `@` in it is passed over, since it may be one. A name that already starts with the
/// provider is not given it twice, and control characters are spaces.
#[must_use]
pub fn label(provider: &str, alias: Option<&str>, plan: Option<&str>) -> String {
    let words = |text: Option<&str>| {
        text.map(|text| {
            text.split(|c: char| c.is_whitespace() || c.is_control())
                .filter(|word| !word.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|text| !text.is_empty() && !text.contains('@'))
    };
    match (words(alias), words(plan)) {
        (Some(alias), _) if starts_with_word(&alias, provider) => alias,
        (Some(alias), _) => format!("{provider} {alias}"),
        (None, Some(plan)) => format!("{provider} {}", capitalised(&plan)),
        (None, None) => provider.to_owned(),
    }
}

/// Whether `text` begins with the word `word`, in any case.
fn starts_with_word(text: &str, word: &str) -> bool {
    let lower = text.to_lowercase();
    lower
        .strip_prefix(&word.to_lowercase())
        .is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| !next.is_alphanumeric())
        })
}

/// `text` with its first letter a capital, as a plan's id becomes its name: `max` is `Max`.
fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The account a picture is of, by its settings key, from the request's [`ACCOUNT_HEADER`].
///
/// # Errors
///
/// When the header is missing or is not written as `encodeURIComponent` writes text.
pub fn account(headers: &HeaderMap) -> Result<String, String> {
    headers
        .get(ACCOUNT_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(percent_decoded)
        .ok_or_else(|| format!("The picture came without the account it is of, {NOTHING_SAVED}."))
}

/// `text` with each `%XX` turned back into its byte, as UTF-8.
fn percent_decoded(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text.bytes();
    while let Some(byte) = rest.next() {
        if byte == b'%' {
            let digit = |byte: Option<u8>| char::from(byte?).to_digit(16);
            let high = digit(rest.next())?;
            let low = digit(rest.next())?;
            bytes.push(u8::try_from(high << 4 | low).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

/// What to say when the account a picture is of is not one the history knows.
#[must_use]
pub fn unknown_account() -> String {
    "QuotaDial no longer knows the account this picture is of, so nothing was saved; open History \
     again and press Share image."
        .to_owned()
}

/// The name of the picture of the account called `label`, saved on `day`, as the `copy`th file of
/// that name: `QuotaDial Claude Work 2026-10-25.png`, then `... 2026-10-25 2.png`. The label keeps
/// only letters, figures, spaces and [`MARKS`], at most [`LABEL_BYTES`] of it, so the name is one
/// every file system takes and never a path.
#[must_use]
pub fn file_name(label: &str, day: NaiveDate, copy: u32) -> String {
    let kept: String = label
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || MARKS.contains(&c) {
                c
            } else {
                ' '
            }
        })
        .collect();
    let mut safe = String::new();
    for word in kept.split_whitespace() {
        let room = LABEL_BYTES.saturating_sub(safe.len() + usize::from(!safe.is_empty()));
        if room == 0 {
            break;
        }
        if !safe.is_empty() {
            safe.push(' ');
        }
        let mut end = word.len().min(room);
        while !word.is_char_boundary(end) {
            end -= 1;
        }
        safe.push_str(&word[..end]);
    }
    let safe = safe.trim_end();
    let stem = if safe.is_empty() {
        format!("QuotaDial {day}")
    } else {
        format!("QuotaDial {safe} {day}")
    };
    match copy {
        0 | 1 => format!("{stem}.png"),
        copy => format!("{stem} {copy}.png"),
    }
}

/// Saves `bytes` in `folder` as the picture of the account called `label` on `day`, under
/// [`file_name`], and returns where. A name already taken gets the next number rather than being
/// overwritten: each file is created only if nothing has that name, so not even another program
/// saving at the same moment loses a file.
///
/// # Errors
///
/// When the file cannot be written, naming the fix.
pub fn save(folder: &Path, label: &str, day: NaiveDate, bytes: &[u8]) -> Result<PathBuf, String> {
    let failed = |error: io::Error| {
        format!(
            "QuotaDial could not save the picture in the Downloads folder ({error}); check that \
             the folder is there and can be written to, then press Share image again."
        )
    };
    for copy in 1..=MOST_COPIES {
        let path = folder.join(file_name(label, day, copy));
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(failed(error)),
        };
        if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
            drop(file);
            // Half a picture is no picture.
            let _ = fs::remove_file(&path);
            return Err(failed(error));
        }
        return Ok(path);
    }
    Err(format!(
        "The Downloads folder already holds {MOST_COPIES} pictures of this account from today; \
         move some of them, then press Share image again."
    ))
}

/// The CRC-32 a PNG chunk ends with, over its type and data (ISO 3309, as the PNG specification
/// gives it), worked out bit by bit: a header is 17 bytes, so a table would buy nothing.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use quotadial_core::licence::TEST_SEED;
    use quotadial_core::{Licence, Plan};
    use quotadial_daemon::{DataDir, licence};
    use tauri::http::HeaderValue;

    use super::*;

    const DAY: &str = "2026-10-25";

    fn day() -> NaiveDate {
        DAY.parse().expect("a date")
    }

    /// A data folder whose licence makes it QuotaDial Pro, or Free with no key.
    fn copy(pro: bool) -> (tempfile::TempDir, Licensing) {
        let temp = tempfile::tempdir().expect("temp dir");
        let dir = DataDir::new(temp.path().join("quotadial"));
        if pro {
            let key = Licence {
                id: "share-1".into(),
                plan: Plan::Pro,
                issued: "2026-10-24".parse().expect("a date"),
                until: None,
            }
            .sign(&TEST_SEED);
            licence::add(&dir, &key, day()).expect("adds the key");
        }
        let licensing = Licensing::new(Some(&dir));
        (temp, licensing)
    }

    /// A PNG of `width` by `height`, as the window's canvas makes one: the signature, a header,
    /// some data and the end.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = SIGNATURE.to_vec();
        let chunk = |bytes: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]| {
            let length = u32::try_from(data.len()).expect("a small chunk");
            bytes.extend(length.to_be_bytes());
            let mut typed = kind.to_vec();
            typed.extend(data);
            bytes.extend(&typed);
            bytes.extend(crc32(&typed).to_be_bytes());
        };
        let mut header = width.to_be_bytes().to_vec();
        header.extend(height.to_be_bytes());
        // 8 bits a channel, RGBA, deflate, adaptive filters, not interlaced.
        header.extend([8, 6, 0, 0, 0]);
        chunk(&mut bytes, b"IHDR", &header);
        chunk(
            &mut bytes,
            b"IDAT",
            &[0x78, 0x9c, 0x63, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01],
        );
        chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    #[test]
    fn the_checksum_is_png_s_own() {
        // The CRC-32 check value, and the end chunk every PNG closes with.
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b"IEND"), 0xae42_6082);
    }

    #[test]
    fn on_pro_a_whole_png_of_the_right_size_is_taken() {
        let (_temp, pro) = copy(true);
        let bytes = png(WIDTH, HEIGHT);

        assert_eq!(
            picture(&pro, &InvokeBody::Raw(bytes.clone())),
            Ok(&bytes[..])
        );
    }

    #[test]
    fn free_is_refused_with_the_features_line() {
        let (_temp, free) = copy(false);

        assert_eq!(
            picture(&free, &InvokeBody::Raw(png(WIDTH, HEIGHT))),
            Err(ProFeature::ShareImage.line())
        );
        assert_eq!(
            picture(&free, &InvokeBody::Raw(Vec::new())),
            Err(ProFeature::ShareImage.line()),
            "the tier is asked first, whatever came"
        );
    }

    #[test]
    fn what_is_not_a_png_is_refused() {
        let (_temp, pro) = copy(true);
        let whole = png(WIDTH, HEIGHT);
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe0];
        jpeg.extend(&whole[4..]);
        let mut broken_header = whole.clone();
        broken_header[29] ^= 0xff;
        let cut_short = whole[..whole.len() - 12].to_vec();

        for (what, bytes) in [
            ("nothing", Vec::new()),
            ("text", b"not a picture at all".to_vec()),
            ("a JPEG", jpeg),
            ("a header whose checksum is wrong", broken_header),
            ("a PNG cut short", cut_short),
            ("only the signature", SIGNATURE.to_vec()),
        ] {
            let body = InvokeBody::Raw(bytes);
            let reason = picture(&pro, &body).expect_err(what);
            assert!(
                reason.starts_with("QuotaDial was sent something other than a whole PNG"),
                "{what}: {reason}"
            );
        }
        let json = InvokeBody::Json(serde_json::json!([137, 80, 78, 71]));
        assert!(
            picture(&pro, &json)
                .expect_err("JSON")
                .contains("form it cannot read"),
            "a JSON array of numbers is not the raw body"
        );
    }

    #[test]
    fn a_png_of_another_size_is_refused() {
        let (_temp, pro) = copy(true);

        for (width, height) in [(1200, 675), (2400, 1351), (1350, 2400), (0, 0)] {
            let reason = picture(&pro, &InvokeBody::Raw(png(width, height))).expect_err("refused");
            assert!(
                reason.starts_with(&format!(
                    "The picture is {width} by {height} pixels, not 2400 by 1350"
                )),
                "{reason}"
            );
        }
    }

    #[test]
    fn an_oversized_body_is_refused() {
        let (_temp, pro) = copy(true);
        let mut large = png(WIDTH, HEIGHT);
        let end = large.split_off(large.len() - 12);
        large.resize(MOST_BYTES + 1 - end.len(), 0);
        large.extend(end);

        let reason = picture(&pro, &InvokeBody::Raw(large)).expect_err("refused");
        assert!(reason.starts_with("The picture is over 4 MB"), "{reason}");
    }

    #[test]
    fn saving_twice_keeps_both_pictures() {
        let folder = tempfile::tempdir().expect("temp dir");
        let first = png(WIDTH, HEIGHT);
        let mut second = png(WIDTH, HEIGHT);
        second.push(0);

        let one = save(folder.path(), "Claude Work", day(), &first).expect("saves");
        let two = save(folder.path(), "Claude Work", day(), &second).expect("saves again");
        let three = save(folder.path(), "Claude Work", day(), &first).expect("saves a third");

        let name = |path: &Path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        };
        assert_eq!(
            name(&one).as_deref(),
            Some("QuotaDial Claude Work 2026-10-25.png")
        );
        assert_eq!(
            name(&two).as_deref(),
            Some("QuotaDial Claude Work 2026-10-25 2.png")
        );
        assert_eq!(
            name(&three).as_deref(),
            Some("QuotaDial Claude Work 2026-10-25 3.png")
        );
        assert_eq!(fs::read(&one).expect("reads"), first, "never overwritten");
        assert_eq!(fs::read(&two).expect("reads"), second);
        assert_eq!(fs::read_dir(folder.path()).expect("lists").count(), 3);
    }

    #[test]
    fn a_hostile_label_makes_a_safe_name() {
        let long = "A".repeat(300);
        for label in [
            "../../etc/passwd",
            "C:\\Windows\\System32",
            "Work: the \"big\" one? <yes> | no *",
            "..",
            ".hidden.",
            "Tab\there\nnew line\u{0}nul\u{7}bell\u{1b}escape",
            "Right\u{202e}gnp.exe",
            long.as_str(),
            "",
            "///",
        ] {
            let name = file_name(label, day(), 1);
            assert!(name.starts_with("QuotaDial "), "{name:?}");
            assert!(name.ends_with(" 2026-10-25.png"), "{name:?}");
            let stem = name.strip_suffix(".png").expect("a png");
            assert!(
                !stem.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|', '.']),
                "{name:?} keeps a character some file system refuses or reads as a path"
            );
            assert!(
                !name.chars().any(|c| c.is_control() || c == '\u{202e}'),
                "{name:?} keeps a control or direction character"
            );
            assert!(!name.contains("  "), "{name:?} has a run of spaces");
            assert!(name.len() <= 120, "{name:?} is {} bytes", name.len());
        }
        assert_eq!(
            file_name("../../etc/passwd", day(), 1),
            "QuotaDial etc passwd 2026-10-25.png"
        );
        assert_eq!(file_name("///", day(), 1), "QuotaDial 2026-10-25.png");
        assert_eq!(
            file_name("Café Ünïcode 1º", day(), 2),
            "QuotaDial Café Ünïcode 1º 2026-10-25 2.png",
            "letters and figures of every script stay"
        );
        let folder = tempfile::tempdir().expect("temp dir");
        let saved = save(folder.path(), "../../escape", day(), &png(WIDTH, HEIGHT)).expect("saves");
        assert_eq!(
            saved.parent(),
            Some(folder.path()),
            "the picture stays in the folder"
        );
    }

    #[test]
    fn the_label_is_the_name_given_or_the_plan_never_an_address() {
        assert_eq!(label("Claude", Some("Work"), Some("max")), "Claude Work");
        assert_eq!(label("Claude", None, Some("max")), "Claude Max");
        assert_eq!(label("Claude", None, Some("Max 5x")), "Claude Max 5x");
        assert_eq!(label("Codex", None, None), "Codex");
        assert_eq!(
            label("Codex", Some("  "), None),
            "Codex",
            "a blank name is no name"
        );
        assert_eq!(
            label("Claude", Some("dev@example.com"), Some("pro")),
            "Claude Pro",
            "a name with an @ may be an address, so it is passed over"
        );
        assert_eq!(label("Claude", Some("a@b"), Some("x@y")), "Claude");
        assert_eq!(
            label("Claude", Some("Claude personal"), None),
            "Claude personal",
            "a name that starts with the provider is not doubled"
        );
        assert_eq!(label("Claude", Some("Claudette"), None), "Claude Claudette");
        assert_eq!(
            label("Claude", Some("Work\nlate\tnights"), None),
            "Claude Work late nights",
            "control characters are spaces"
        );
    }

    #[test]
    fn the_account_is_read_as_encode_uri_component_writes_it() {
        let mut headers = HeaderMap::new();
        assert!(account(&headers).is_err(), "no header");
        headers.insert(
            ACCOUNT_HEADER,
            HeaderValue::from_static("claude%20user-1%20-"),
        );
        assert_eq!(account(&headers).as_deref(), Ok("claude user-1 -"));
        headers.insert(
            ACCOUNT_HEADER,
            HeaderValue::from_static("codex%20acct%C3%A9%20org-1"),
        );
        assert_eq!(account(&headers).as_deref(), Ok("codex acct\u{e9} org-1"));
        for broken in ["claude%2", "claude%zz", "%FF%FE"] {
            headers.insert(ACCOUNT_HEADER, HeaderValue::from_static(broken));
            assert!(account(&headers).is_err(), "{broken}");
        }
    }
}
