//! The words QuotaDial uses for times, sources and statuses. The CLI, the tray app and its
//! notifications all say them the same way.

use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};

use crate::{Cost, ExtraUsage, ModelCost, Pace, Source, Status};

/// `path` with `~` for the home folder, as people write it, such as `~/.codex/auth.json`: `~`, the
/// system's own separator and the rest, so Windows writes `~\.codex\auth.json` (D-129).
#[must_use]
pub fn tilde(path: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| path.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~{}{}", std::path::MAIN_SEPARATOR, rest.display()),
        None => path.display().to_string(),
    }
}

/// `text` with every path in it that starts in the home folder `home` written as [`tilde`] writes
/// it, such as `~/.codex/auth.json`, so a sentence that names a place keeps the user's name out of
/// it (#52): `~`, the separator the path has after the home folder, which for a path the system
/// wrote is its own, and the rest as written. With no home folder, or one that is a whole disk, the
/// text is left as it is. Every message QuotaDial writes that can name a place, a provider's error
/// or one of its own, goes through it once where its text is made, and so does doctor's whole text
/// (D-128, D-129). Written again, a text is what it was written once.
///
/// A path starts where the text does or after a character no path holds, such as a space or a
/// quote, and starts in the home folder only where the folder's own name ends. The home folder is
/// matched as written, without a separator at its end. With `/Users/ann` as the home folder, a path
/// in `/Users/anna` or in `/Volumes/Backup/Users/ann` stays as it is, and so does one in a folder
/// whose name goes on with any character, such as `/Users/ann (old)`, `/Users/ann's` or
/// `/Users/ann;x` (D-133). Where a name ends is read in two ways:
///
/// - **A word in single quotes** is one place, read whole as the shell that wrote it reads it: a
///   POSIX shell's `'...'`, in which `'\''` is a quote, and PowerShell's, in which a single quotation
///   mark, curly ones too, is written twice. The text does not say which shell wrote it, so the
///   longer reading is taken. The word is the home folder where it starts a place, not going on
///   from a path as `'Library'` does after `~/`, and its text is the home folder's path or starts
///   with it and a separator. Otherwise it is left exactly as it is written, whatever it holds, a
///   space, a quote, the home folder's path again: so every place outside the home folder that
///   [`shell_word`] and [`powershell_string`] write comes through unchanged. A quote opens a word at
///   the start of the text or after a character that is not a letter, a digit, a quote or a
///   backslash, as in `Ann's` and `'\''`, and a word that never ends is read as the sentence it may
///   be.
/// - **Anything else is a sentence**, where text cannot always tell the home folder on its own from
///   the start of a longer name, so what follows it is read. The home folder is written `~` at a
///   separator, at the end of the text or of a line, and at the marks a sentence puts after a path
///   (`:`, `,`, `;`, `)`, a quote, a backtick, and a full stop before a space or the end) when
///   nothing but marks joins it to the next space, unless the next word on the line carries the
///   name on: a word that opens with a bracket, as `(old)` does, or has a separator after a name, as
///   `old/quotadial` does, but not one that is a place of its own, from a root, `~` or a drive.
///   Anything else joined to it is more of the name, as in `ann's` and `ann;x`. Where this must
///   guess, it leaves the name: a name shown is the lesser error, and a place named that is not the
///   one in the text is the greater. It still writes `~` for a name of several words that has no
///   separator before its last, such as `/Users/ann old stuff/x`, or that ends a place, such as a
///   folder `/Users/ann old`, and it leaves a name where the home folder itself is followed by a word
///   with a separator in it, such as `/Users/ann, see docs/CLI.md`.
#[must_use]
pub fn tilde_in(text: &str, home: Option<&Path>) -> String {
    let Some(home) = home.filter(|home| home.parent().is_some()) else {
        return text.to_owned();
    };
    let home = home.display().to_string();
    let home = home.trim_end_matches(std::path::is_separator);
    if home.is_empty() || !text.contains(home) {
        return text.to_owned();
    }
    let mut written = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(next) = rest.chars().next() {
        let before = written.chars().next_back();
        if let Some(word) = single_quoted(rest, before) {
            // One place, read whole: the home folder where its text says so, as `tilde` writes it
            // inside the quotes, and otherwise exactly as it was written, quotes and all. A word
            // that goes on from a path, as `'Library'` does after `~/`, is not a place of its own.
            let starts = before.is_none_or(|before| !in_path(before));
            let in_home = starts
                && rest[1..word.closes]
                    .strip_prefix(home)
                    .is_some_and(|after| {
                        after.is_empty() || after.starts_with(std::path::is_separator)
                    });
            if in_home {
                written.push_str("'~");
                written.push_str(&rest[1 + home.len()..word.end]);
            } else {
                written.push_str(&rest[..word.end]);
            }
            rest = &rest[word.end..];
            continue;
        }
        if let Some(after) = rest.strip_prefix(home)
            && before.is_none_or(|before| !in_path(before))
        {
            match after.chars().next() {
                // As `tilde` writes it: `~`, the separator, and the rest of the path as it was
                // written. The separator is the path's own, so a path written with `/` on Windows
                // keeps it.
                Some(separator) if std::path::is_separator(separator) => {
                    written.push('~');
                    written.push(separator);
                    rest = &after[separator.len_utf8()..];
                    continue;
                }
                _ if ends_path(after, home) => {
                    written.push('~');
                    rest = after;
                    continue;
                }
                _ => {}
            }
        }
        // Not the home folder here: keep this character and look again after it.
        written.push(next);
        rest = &rest[next.len_utf8()..];
    }
    written
}

/// Whether `character` can be part of a path where a sentence names one, so that a home folder
/// right after it is inside a longer path.
fn in_path(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '/' | '\\' | '.' | '_' | '-' | '~')
}

/// Whether a path that has run from the home folder's text ends there, `after` being what follows
/// it, rather than going on as a longer name (D-133). It ends where the text does. A mark a
/// sentence puts after a path ends it only if nothing but marks comes before the next space, so
/// `ann;x` and `ann's` are longer names and `ann;` is not; a full stop is a mark only before a space
/// or the end, so `ann.old` is a longer name. At a space the next word decides ([`name_goes_on`]).
fn ends_path(after: &str, home: &str) -> bool {
    let mut characters = after.char_indices();
    while let Some((at, character)) = characters.next() {
        if character.is_whitespace() {
            return !name_goes_on(&after[at..], home);
        }
        let mark = matches!(character, ':' | ',' | ';' | ')' | '"' | '\'' | '`')
            || (character == '.'
                && characters
                    .as_str()
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace));
        if !mark {
            return false;
        }
    }
    true
}

/// Whether the next word after `tail`'s spaces carries on a name that starts with the home folder's
/// text `home`, so the path did not end where `tail` starts (D-133): a word that opens with a
/// bracket, as a copy is named, `(old)` or `[1]`, or has a separator after a name, as `old/quotadial`
/// does. A word that is a place of its own does not, and neither does a line break, which no name
/// holds here. The home folder's own text inside the word is skipped, so that writing a second
/// mention of it `~` does not change the answer.
fn name_goes_on(tail: &str, home: &str) -> bool {
    let word = tail.trim_start_matches(|character: char| {
        character.is_whitespace() && !is_line_break(character)
    });
    if word.starts_with(['(', '[']) {
        return true;
    }
    if starts_a_place(word) {
        return false;
    }
    let mut named = false;
    let mut rest = word;
    while let Some(next) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix(home) {
            rest = after;
        } else if next.is_whitespace() {
            break;
        } else if std::path::is_separator(next) {
            return named;
        } else {
            named |= next.is_alphanumeric() || matches!(next, '_' | '-');
            rest = &rest[next.len_utf8()..];
        }
    }
    false
}

/// Whether `word` starts a place of its own: from a root, as `/usr/bin` does, from the home
/// folder, as `~/x` does, or from a drive, as `C:\x` does.
fn starts_a_place(word: &str) -> bool {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) if std::path::is_separator(first) || first == '~' => true,
        Some(drive) if drive.is_ascii_alphabetic() => {
            characters.next() == Some(':') && characters.next().is_some_and(std::path::is_separator)
        }
        _ => false,
    }
}

/// Whether `character` ends a line, which no folder's name holds in a sentence.
fn is_line_break(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{2028}' | '\u{2029}'
    )
}

/// A word in single quotes: where its last quote starts, and where the word ends after it, as byte
/// positions from its first quote.
struct Quoted {
    closes: usize,
    end: usize,
}

/// The word in single quotes that starts at `rest`'s first character, `before` being the character
/// written before it, read as the shell that wrote it reads it, if there is one and it ends (D-133):
/// a POSIX shell's, and PowerShell's, whichever reads the longer. Never one that starts after a
/// letter or a digit, as the apostrophe in `Ann's` does, or after a quote or a backslash, which make
/// the quote that follows part of a word.
fn single_quoted(rest: &str, before: Option<char>) -> Option<Quoted> {
    if !rest.starts_with('\'')
        || before.is_some_and(|before| before.is_alphanumeric() || matches!(before, '\'' | '\\'))
    {
        return None;
    }
    [posix_quoted(rest), powershell_quoted(rest)]
        .into_iter()
        .flatten()
        .max_by_key(|word| word.end)
}

/// `rest`'s word in single quotes as a POSIX shell reads it: up to the next quote, unless that is
/// `'\''`, which closes the quotes, escapes a quote and opens them again, so the word goes on.
fn posix_quoted(rest: &str) -> Option<Quoted> {
    const A_QUOTE: &str = r"'\''";
    let mut from = 1;
    loop {
        let closes = from + rest[from..].find('\'')?;
        if rest[closes..].starts_with(A_QUOTE) {
            from = closes + A_QUOTE.len();
        } else {
            return Some(Quoted {
                closes,
                end: closes + 1,
            });
        }
    }
}

/// `rest`'s word in single quotes as PowerShell reads it: up to the next single quotation mark,
/// straight or curly, unless another follows it at once, which is one in the string.
fn powershell_quoted(rest: &str) -> Option<Quoted> {
    let mut characters = rest.char_indices().skip(1).peekable();
    while let Some((at, quote)) = characters.next() {
        if POWERSHELL_SINGLE_QUOTES.contains(&quote)
            && characters
                .next_if(|&(_, next)| POWERSHELL_SINGLE_QUOTES.contains(&next))
                .is_none()
        {
            return Some(Quoted {
                closes: at,
                end: at + quote.len_utf8(),
            });
        }
    }
    None
}

/// `place` as a POSIX shell takes it, for a command a user pastes as written (D-130, D-131): in the
/// home folder `home`, `~/` left unquoted, so the shell expands it, and the rest in single quotes,
/// so a space does not split it, such as `~/'Library/Application Support/quotadial/config.json'`;
/// anywhere else, or with no home folder or one that is a whole disk, the whole place in single
/// quotes. A single quote in a name is written `'\''`: the quotes closed, a quote escaped, and the
/// quotes opened again. Quoted whatever the place holds, so no character a shell reads can be
/// missed. In the home folder by whole names, as [`tilde`] finds it, so `/Users/anna` is not in
/// `/Users/ann`. The word comes through [`tilde_in`] as it is written, whatever the place holds,
/// since that reads a word in single quotes whole (D-133). `~/` on every system, being for a POSIX
/// shell: neither command that uses it is shown on Windows, where `config.json`'s warning cannot
/// occur and a sign-in command is written for PowerShell, with [`powershell_string`].
#[must_use]
pub fn shell_word(place: &Path, home: Option<&Path>) -> String {
    let quoted = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', r"'\''"));
    match after_home(place, home) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", quoted(rest)),
        None => quoted(place),
    }
}

/// The characters PowerShell takes for `"`: the straight double quotation mark and the curly `“`,
/// `”` and `„` (the PowerShell Language Specification, 2.3.5.2, double-quote-character).
const POWERSHELL_DOUBLE_QUOTES: [char; 4] = ['"', '\u{201c}', '\u{201d}', '\u{201e}'];

/// The characters PowerShell takes for `'`: the straight single quotation mark and the curly `‘`,
/// `’`, `‚` and `‛` (the same section, single-quote-character).
const POWERSHELL_SINGLE_QUOTES: [char; 5] = ['\'', '\u{2018}', '\u{2019}', '\u{201a}', '\u{201b}'];

/// `place` as PowerShell takes it, for a command a user pastes as written (D-131). In the home folder
/// `home`, a double-quoted string that starts with `$HOME`, which PowerShell sets to the home folder,
/// from `USERPROFILE` on Windows and `HOME` elsewhere (about_Automatic_Variables), then the system's
/// own separator and the rest, such as `"$HOME\.claude-work"`, or `"$HOME"` for the home folder
/// itself. In double quotes PowerShell reads `$` as the start of a variable, the backtick as its
/// escape character, and `"`, `“`, `”` or `„` as the string's end, so each of those in the rest is
/// written after a backtick, which takes it as written (about_Quoting_Rules). Anywhere else, or with
/// no home folder or one that is a whole disk, the whole place in single quotes, in which PowerShell
/// reads nothing but the string's end, `'`, `‘`, `’`, `‚` or `‛`, so each of those is written twice.
/// In the home folder by whole names, as [`shell_word`] finds it; like that word, the string comes
/// through [`tilde_in`] as it is written (D-133): a place outside the home folder is in single
/// quotes, which that reads whole, and one in it starts `"$HOME`, so holds the home folder's path
/// only where a name holds it, which a name on Windows, where this string is written, cannot.
#[must_use]
pub fn powershell_string(place: &Path, home: Option<&Path>) -> String {
    let Some(rest) = after_home(place, home) else {
        let mut string = String::from('\'');
        for character in place.display().to_string().chars() {
            if POWERSHELL_SINGLE_QUOTES.contains(&character) {
                string.push(character);
            }
            string.push(character);
        }
        string.push('\'');
        return string;
    };
    let mut string = String::from("\"$HOME");
    if !rest.as_os_str().is_empty() {
        string.push(std::path::MAIN_SEPARATOR);
        for character in rest.display().to_string().chars() {
            if matches!(character, '`' | '$') || POWERSHELL_DOUBLE_QUOTES.contains(&character) {
                string.push('`');
            }
            string.push(character);
        }
    }
    string.push('"');
    string
}

/// What follows the home folder `home` in `place`, found by whole names, or `None` where `place` is
/// not in it, or there is no home folder or one that is a whole disk, which [`tilde_in`] leaves as
/// it is too: the place a shell word or a PowerShell string writes from the home folder.
fn after_home<'a>(place: &'a Path, home: Option<&Path>) -> Option<&'a Path> {
    home.filter(|home| home.parent().is_some())
        .and_then(|home| place.strip_prefix(home).ok())
}

/// What the user's name is written as where [`hide_user_in`] hides it.
const HIDDEN_USER: &str = "[user]";

/// The one character that stands for each name, and for each `[user]` already in the text, in the
/// text [`hide_user_in`] reads places from, so that hiding a name cannot change what is read after
/// it, whatever the name holds, such as the space in `Ann Smith` (D-134).
const STAND_IN: char = 'x';

/// `text` with the user's name, the last name of the home folder `home`, written `[user]` where it
/// names the user: as a name in a place outside the home folder, such as
/// `/Volumes/Backup/Users/ann` for the home folder `/Users/ann`, or
/// `/private/tmp/ann-501/-Users-ann--work`, a temporary folder named from the working folder (#52,
/// D-134). Doctor's text goes through it after [`tilde_in`] has written every place in the home
/// folder `~`. With no home folder, or one that is a whole disk, whose name is not text or has
/// fewer than 3 characters, which could be part of anything, the text is left as it is. Written
/// again, a text is what it was written once.
///
/// The name is hidden only where it is a whole name, bounded by anything but an ASCII letter or
/// digit, and only inside a place outside the home folder. A word that is only spelled as the name
/// stays, in a sentence, in a command, in a web address and in a place that starts at `~` or
/// `$HOME`, which is the user's own folder however it is written: for a user called `claude`, `run
/// claude once`, `CLAUDE_CONFIG_DIR=~/'.claude-work' claude` and `~/.claude` stay as they are, where
/// a plain replace of the whole word wrote `run [user] once`, `CLAUDE_CONFIG_DIR=~/'.[user]-work'
/// [user]` and `~/.[user]`. The words in a command that names a place in the home folder are never
/// hidden, whatever the user is called.
///
/// A place outside the home folder is read from the text as [`tilde_in`] reads a path, and so is a
/// word in single quotes, one place, read whole:
///
/// - **It starts** where a path can start, at the start of the text or after a character no path
///   holds, with a root: `/` or `\`, or a drive and a separator, such as `C:\` or `C:/`. Both
///   separators count on every system, so a place written on another system is found. A `//`
///   after `scheme:` starts a web address and not a place, except after `file:`, so
///   `http://127.0.0.1/v1/status` is left alone for a user called `status`. A word that starts with
///   `~`, `$` or `%` starts a place in the home folder, its variable and what follows it in one
///   piece, as in `%USERPROFILE%\x` and `${HOME}/x`, and a relative path has no root, so neither
///   is a place outside it.
/// - **It goes on** to the end of its word, and past a space into the next word when that holds a
///   separator, as in `/Volumes/My Passport/Users/ann`, and so on, up to a line break. A plain
///   word ends it, which is where a sentence goes on after a place.
/// - **A word in single quotes** that starts a place is read whole: a quoted place outside the home
///   folder, as [`shell_word`] and [`powershell_string`] write it, is one place whatever it holds,
///   and the words of the command around it are not in it. A word that goes on from a path, as
///   `'Library'` does after `~/`, is part of that path, and PowerShell's string from `$HOME`,
///   [`powershell_string`]'s form for a place in the home folder, is read whole too, so a command
///   for a place in the home folder is never changed.
///
/// Where this must guess, it hides: a name is hidden in whatever reads as a place outside the home
/// folder, so it also writes `[user]` for a component that is only spelled as the name, such as
/// `/usr/bin/security` for a user called `security`, and for a word that reads as a place, such as
/// `/model`, or one joined to a place by a separator after a space (D-134 lists them). It leaves a
/// name it cannot tell is in a place: after a plain word in a folder's name, as in
/// `/Volumes/Time Machine Backups/Users/ann`, and in a relative path, a sentence, or a place under
/// `~` that holds the user's encoded folder.
#[must_use]
pub fn hide_user_in(text: &str, home: Option<&Path>) -> String {
    let Some(name) = user_name(home).filter(|name| text.contains(*name)) else {
        return text.to_owned();
    };
    // The text places are read from: each name and each `[user]` as one character. Where each name
    // is in `text`, and where its character is in what is read.
    let mut read = String::with_capacity(text.len());
    let mut names = Vec::new();
    let mut at = 0;
    while let Some(next) = text[at..].chars().next() {
        let rest = &text[at..];
        if rest.starts_with(HIDDEN_USER) {
            read.push(STAND_IN);
            at += HIDDEN_USER.len();
        } else if rest.starts_with(name) && is_whole_name(text, at, at + name.len()) {
            names.push((at, read.len()));
            read.push(STAND_IN);
            at += name.len();
        } else {
            read.push(next);
            at += next.len_utf8();
        }
    }
    let outside = outside_the_home_folder(&read);
    let mut hidden = String::with_capacity(text.len());
    let mut copied = 0;
    for (at, stand_in) in names {
        if outside[stand_in] {
            hidden.push_str(&text[copied..at]);
            hidden.push_str(HIDDEN_USER);
            copied = at + name.len();
        }
    }
    hidden.push_str(&text[copied..]);
    hidden
}

/// The user's name: the last name of the home folder `home`, when there is one that is not a whole
/// disk, it is text, and it has 3 or more characters.
fn user_name(home: Option<&Path>) -> Option<&str> {
    home.filter(|home| home.parent().is_some())
        .and_then(Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .filter(|name| name.chars().count() >= 3)
}

/// Whether the name at `at..end` in `text` is a whole name: no ASCII letter or digit is beside it,
/// and no `[user]`, which is a name already hidden. A name that touches one is left, so hiding the
/// text again finds the same names.
fn is_whole_name(text: &str, at: usize, end: usize) -> bool {
    let (before, after) = (&text[..at], &text[end..]);
    !before.ends_with(HIDDEN_USER)
        && !after.starts_with(HIDDEN_USER)
        && before
            .chars()
            .next_back()
            .is_none_or(|character| !character.is_ascii_alphanumeric())
        && after
            .chars()
            .next()
            .is_none_or(|character| !character.is_ascii_alphanumeric())
}

/// Which bytes of `text` are in a place outside the home folder, read as [`hide_user_in`] says:
/// a place starts at a root where a path can start and goes on to the end of its word, and past a
/// space into the next word when that holds a separator, up to a line break; a word in single
/// quotes is one place, read whole, as [`tilde_in`] reads it, and PowerShell's string from `$HOME`
/// is one place in the home folder, read whole as [`powershell_string`] writes it (D-134).
fn outside_the_home_folder(text: &str) -> Vec<bool> {
    let mut outside = vec![false; text.len()];
    let mut inside = false;
    let mut at = 0;
    while let Some(next) = text[at..].chars().next() {
        let rest = &text[at..];
        let before = text[..at].chars().next_back();
        // What starts right after a path character goes on from that path, as `'Library'` does
        // after `~/`: it is in the place before it, whichever that is, and starts none of its own.
        let goes_on = before.is_some_and(continues_a_path);
        let after_a_space = before.is_some_and(char::is_whitespace);
        let width = if let Some(width) = home_string(rest).filter(|_| !goes_on) {
            inside = false;
            width
        } else if let Some(word) = single_quoted(rest, before) {
            if !goes_on {
                inside = starts_at_a_root(&rest[1..word.closes]);
            }
            word.end
        } else {
            if next.is_whitespace() {
                if is_line_break(next) {
                    inside = false;
                } else if inside && !after_a_space {
                    inside = place_goes_on(&rest[next.len_utf8()..]);
                }
            } else if !goes_on {
                if starts_at_a_root(rest) && !is_web_address(text, at) {
                    inside = true;
                } else if rest.starts_with(['~', '$', '%']) {
                    inside = false;
                }
            }
            next.len_utf8()
        };
        if inside {
            outside[at..at + width].fill(true);
        }
        at += width;
    }
    outside
}

/// Whether `character` is one a path goes on with where a root follows it: one a path holds, or the
/// end of a variable the home folder is written with, so that `%USERPROFILE%\x` and `${HOME}/x`
/// are the home folder's, each in one piece, and not a variable and then a place of its own.
fn continues_a_path(character: char) -> bool {
    in_path(character) || matches!(character, '%' | '}')
}

/// Whether `separator` is one a place is written with, on any system: a place written on another
/// system is a place too, so the name in it is hidden.
fn is_slash(separator: char) -> bool {
    matches!(separator, '/' | '\\')
}

/// Whether `rest` starts at a root: a separator, or a drive and a separator, as `C:\` and `C:/`.
fn starts_at_a_root(rest: &str) -> bool {
    let mut characters = rest.chars();
    match characters.next() {
        Some(first) if is_slash(first) => true,
        Some(drive) if drive.is_ascii_alphabetic() => {
            characters.next() == Some(':') && characters.next().is_some_and(is_slash)
        }
        _ => false,
    }
}

/// Whether the `/` at `at` in `text` starts the `//` of a web address, as in `http://127.0.0.1`,
/// which no place starts with: a scheme and a colon come before it, and the scheme is not `file`,
/// whose address names a place. A drive written with two separators, `D://Backup`, has started a
/// place at its letter already, whatever this says of the `//`.
fn is_web_address(text: &str, at: usize) -> bool {
    let Some(before) = text[..at].strip_suffix(':') else {
        return false;
    };
    let scheme = &before[before.trim_end_matches(is_scheme_character).len()..];
    text[at..].starts_with("//")
        && scheme.starts_with(|first: char| first.is_ascii_alphabetic())
        && !scheme.eq_ignore_ascii_case("file")
}

/// Whether `character` can be in the scheme of a web address.
fn is_scheme_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
}

/// How far PowerShell's string from `$HOME` that starts `rest` goes, as [`powershell_string`]
/// writes it for a place in the home folder: a backtick takes the character after it as written,
/// and a double quotation mark of any of PowerShell's kinds ends the string. `None` where `rest`
/// starts no such string, or it never ends.
fn home_string(rest: &str) -> Option<usize> {
    const START: &str = "\"$HOME";
    let mut characters = rest.strip_prefix(START)?.char_indices();
    while let Some((at, character)) = characters.next() {
        if character == '`' {
            characters.next();
        } else if POWERSHELL_DOUBLE_QUOTES.contains(&character) {
            return Some(START.len() + at + character.len_utf8());
        }
    }
    None
}

/// Whether the place outside the home folder that ends at a space goes on, `after` being what
/// follows the space: in the next word, after any more spaces, when that holds a separator, as
/// `Passport/Users/ann` does after `/Volumes/My`. Not in a plain word, which is a sentence going
/// on, nor at a line break. A word that starts a place in the home folder, with `~`, `$` or `%`,
/// ends the place when the scan reaches it, whatever this says.
fn place_goes_on(after: &str) -> bool {
    let word = after.trim_start_matches(|character: char| {
        character.is_whitespace() && !is_line_break(character)
    });
    word.chars()
        .take_while(|character| !character.is_whitespace())
        .any(is_slash)
}

/// For example `resets in 2 h 10 min`, or `reset 3 min ago` when the numbers predate a reset.
#[must_use]
pub fn resets(resets_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    match resets_at {
        None => "no reset time reported".to_owned(),
        Some(at) if at > now => format!("resets in {}", duration(at - now)),
        Some(at) => format!("reset {} ago", duration(now - at)),
    }
}

/// A window's pace, given when it resets: for example `at 8.5%/h, runs out in 1 h 30 min, 40 min
/// before it resets`, or `at 2.1%/h, lasts until it resets`.
#[must_use]
pub fn pace(pace: &Pace, resets_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> String {
    let rate = match pace.pct_per_hour {
        rate if rate >= 10.0 => format!("{rate:.0}%/h"),
        rate if rate >= 0.05 => format!("{rate:.1}%/h"),
        _ => "0%/h".to_owned(),
    };
    match (pace.runs_out_at, resets_at) {
        (Some(out), _) if out <= now => format!("at {rate}, runs out about now"),
        (Some(out), Some(reset)) => format!(
            "at {rate}, runs out in {}, {} before it resets",
            duration(out - now),
            duration(reset - out)
        ),
        (Some(out), None) => format!("at {rate}, runs out in {}", duration(out - now)),
        (None, Some(_)) => format!("at {rate}, lasts until it resets"),
        (None, None) => format!("at {rate}"),
    }
}

/// What QuotaDial does with a sign-in it reads from `place`, as a provider that reads one says it
/// (P-020): such as `QuotaDial reads this account's sign-in from ~/.codex/auth.json. It only reads
/// it and never changes it.`
#[must_use]
pub fn sign_in_read_from(place: &str) -> String {
    format!(
        "QuotaDial reads this account's sign-in from {place}. It only reads it and never changes it."
    )
}

/// A share as people write it: `80` for 80.0 and `80.5` for 80.5, to a tenth.
#[must_use]
pub fn percent(pct: f64) -> String {
    format!("{}", (pct * 10.0).round() / 10.0)
}

/// A whole number with commas between the thousands, such as `1,204`.
#[must_use]
pub fn thousands(count: u64) -> String {
    let digits = count.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// A count of something, such as `1 request` or `1,204 requests`.
#[must_use]
pub fn plural(count: u64, noun: &str) -> String {
    if count == 1 {
        format!("1 {noun}")
    } else {
        format!("{} {noun}s", thousands(count))
    }
}

/// An amount in US dollars as people write it: `$1,234` from $100 up, `$12.34` below that,
/// `under $0.01` for less than a cent, and `$0` for nothing.
#[must_use]
pub fn usd(amount: f64) -> String {
    if amount.is_nan() || amount <= 0.0 {
        "$0".to_owned()
    } else if amount < 0.01 {
        "under $0.01".to_owned()
    } else if amount < 100.0 {
        format!("${amount:.2}")
    } else {
        format!("${}", thousands(amount.round() as u64))
    }
}

/// A provider's cost, such as `$1,234 at API prices over the last 7 days, from 51,234 requests`,
/// or `no requests in the last 7 days`.
#[must_use]
pub fn cost(cost: &Cost) -> String {
    let days = (cost.until - cost.since).num_days();
    if cost.requests == 0 {
        return format!("no requests in the last {days} days");
    }
    format!(
        "{} at API prices over the last {days} days, from {}",
        usd(cost.usd),
        plural(cost.requests, "request")
    )
}

/// A model's part of a cost, such as `claude-opus-5-5: $1,000, 41,000 requests`, or `no price`
/// in place of the amount when the price table has none.
#[must_use]
pub fn model_cost(model: &ModelCost) -> String {
    format!(
        "{}: {}, {}",
        model.model,
        model.usd.map_or_else(|| "no price".to_owned(), usd),
        plural(model.requests, "request")
    )
}

/// For example `4 min ago`, or `just now`.
#[must_use]
pub fn ago(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    if now - at < TimeDelta::minutes(1) {
        return "just now".to_owned();
    }
    format!("{} ago", duration(now - at))
}

/// A moment as a sentence addressed to a person writes it (D-142): `26 Oct 2026 at 09:12 UTC`, the
/// day, the month in three letters and the year, then the time to the minute, in UTC. The seconds
/// are dropped, since nobody acts on them and the sentence is read, not sorted. UTC until the front
/// ends write a time in the reader's own zone from a field of its own; a time a program reads stays
/// RFC 3339, as the JSON writes it.
#[must_use]
pub fn date_and_time(at: DateTime<Utc>) -> String {
    at.format("%-d %b %Y at %H:%M UTC").to_string()
}

/// For example `under a minute`, `45 min`, `2 h 10 min` or `3 d 4 h`, rounded down.
#[must_use]
pub fn duration(length: TimeDelta) -> String {
    let minutes = length.num_minutes();
    let (days, hours) = (minutes / (24 * 60), minutes / 60 % 24);
    match (days, hours, minutes % 60) {
        _ if minutes < 1 => "under a minute".to_owned(),
        (0, 0, minutes) => format!("{minutes} min"),
        (0, hours, 0) => format!("{hours} h"),
        (0, hours, minutes) => format!("{hours} h {minutes} min"),
        (days, 0, _) => format!("{days} d"),
        (days, hours, _) => format!("{days} d {hours} h"),
    }
}

/// Where an account's extra usage stands, as the CLI and the app's window write it after "Extra
/// usage:": `off`; what is used of the limit when both amounts are known, such as `$12.50 of $50.00
/// used (25%)`; the share alone when only it is, such as `25% used`, in the words a window's share
/// is said in; or `on` when nothing is known of what is used. While it is on, credits left come
/// after, such as `on, 1056.44 credits left`. The two amounts take one form, to the cent with their
/// thousands grouped, as the limit is set: US dollars, the currency when none is given, with the
/// symbol before each, such as `$1,234.50 of $5,000.00`, and any other currency with its code after
/// them, such as `1,234.50 of 5,000.00 EUR used (25%)`. Not [`usd`], which writes a cost said on
/// its own, in whole dollars from $100, and would put two forms in one line, as `$99.50 of $150`.
#[must_use]
pub fn extra_usage(extra: &ExtraUsage) -> String {
    if !extra.enabled {
        return "off".to_owned();
    }
    let share = extra.used_pct.map(|used_pct| format!("{used_pct:.0}%"));
    let mut text = match (extra.used.zip(extra.limit), share) {
        (Some((used, limit)), share) => format!(
            "{} used{}",
            used_of_limit(used, limit, extra.currency.as_deref()),
            share.map(|share| format!(" ({share})")).unwrap_or_default()
        ),
        (None, Some(share)) => format!("{share} used"),
        (None, None) => "on".to_owned(),
    };
    if let Some(balance) = extra.balance {
        text.push_str(&format!(", {balance:.2} credits left"));
    }
    text
}

/// `used` of `limit`, both in `currency` and both [`to_the_cent`]: `$1,234.50 of $5,000.00` in US
/// dollars, which a currency left out or blank is too, and in any other currency the amounts with
/// its code after them, `1,234.50 of 5,000.00 EUR`.
fn used_of_limit(used: f64, limit: f64, currency: Option<&str>) -> String {
    let (used, limit) = (to_the_cent(used), to_the_cent(limit));
    match currency.map(str::trim).filter(|code| !code.is_empty()) {
        Some(code) if !code.eq_ignore_ascii_case("USD") => format!("{used} of {limit} {code}"),
        _ => format!("${used} of ${limit}"),
    }
}

/// An amount of money to the cent, its whole part grouped by [`thousands`], such as `1,234.50`;
/// `0.00` for nothing, and for a negative amount or one that is not a number, which [`usd`] takes
/// as nothing too.
fn to_the_cent(amount: f64) -> String {
    if amount.is_nan() || amount <= 0.0 {
        return "0.00".to_owned();
    }
    let cents = (amount * 100.0).round() as u64;
    format!("{}.{:02}", thousands(cents / 100), cents % 100)
}

/// For example `ok`, `stale` or `sign-in required`.
#[must_use]
pub fn status_label(status: Status) -> &'static str {
    match status {
        Status::Ok => "ok",
        Status::Stale => "stale",
        Status::AuthRequired => "sign-in required",
        Status::RateLimited => "rate limited",
        Status::Unsupported => "unsupported",
    }
}

/// Where a number came from: `the API`, `local logs`, `cache`, `the example build` for a number
/// made up for pictures of the app (D-117), or `the CLI's status line` for one the official CLI
/// handed over (P-022), whose account names which CLI.
#[must_use]
pub fn source_label(source: Source) -> &'static str {
    match source {
        Source::Api => "the API",
        Source::LocalLog => "local logs",
        Source::Cache => "cache",
        Source::Example => "the example build",
        Source::StatusLine => "the CLI's status line",
    }
}

/// `name`, to show beside a provider's label `label`, or none when it says nothing more: when it is
/// empty or blank, or the label itself, whatever the case. The one rule for every account and every
/// surface that writes a provider's label and an account's name side by side.
#[must_use]
pub fn name_beside<'a>(label: &str, name: &'a str) -> Option<&'a str> {
    let said = name.trim();
    (!said.is_empty() && said.to_lowercase() != label.trim().to_lowercase()).then_some(name)
}

/// The provider's label `label` with `name` beside it when it says more ([`name_beside`]):
/// "Claude user@example.com", or "Example" alone for an account called Example.
#[must_use]
pub fn labelled(label: &str, name: &str) -> String {
    match name_beside(label, name) {
        Some(name) => format!("{label} {name}"),
        None => label.to_owned(),
    }
}

/// The words #52 keeps out of anything a user reads, "quota", "auth" and "credential" with their
/// plurals, found in `text` outside a name, each lowercased: what tests hold every sentence people
/// read to. A name is written as it is: anything in backticks or double quotes, and a path, a file
/// or a variable, a word holding `/`, `\`, `~`, `_` or a `.` inside it, such as
/// `~/.codex/auth.json`, `.credentials.json` or `CLAUDE_CODE_OAUTH_TOKEN`.
#[must_use]
pub fn unplain_words(text: &str) -> Vec<String> {
    const KEPT_OUT: [&str; 5] = ["quota", "quotas", "auth", "credential", "credentials"];
    let mut prose = String::with_capacity(text.len());
    let mut inside = None;
    for char in text.chars() {
        match inside {
            Some(close) if char == close => inside = None,
            Some(_) => {}
            None if char == '`' || char == '"' => inside = Some(char),
            None => prose.push(char),
        }
    }
    prose
        .split_whitespace()
        .map(|word| word.trim_matches(|char: char| !char.is_alphanumeric()))
        .filter(|word| !word.contains(['/', '\\', '~', '_', '.']))
        .flat_map(|word| word.split(|char: char| !char.is_alphanumeric()))
        .map(str::to_lowercase)
        .filter(|word| KEPT_OUT.contains(&word.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, PathBuf};

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn paths_in_the_home_folder_start_with_a_tilde() {
        let home = PathBuf::from("/Users/example");
        // `~` and the system's own separator, then the rest as the path has it: on Windows
        // `~\.codex/auth.json` here (D-129).
        assert_eq!(
            tilde(&home.join(".codex/auth.json"), Some(&home)),
            Path::new("~")
                .join(".codex/auth.json")
                .display()
                .to_string()
        );
        assert_eq!(tilde(&home, Some(&home)), "~");
        assert_eq!(tilde(Path::new("/etc/x"), Some(&home)), "/etc/x");
        assert_eq!(tilde(Path::new("/etc/x"), None), "/etc/x");
    }

    /// A path from its parts, joined with this system's separator.
    fn path(parts: &[&str]) -> PathBuf {
        parts.iter().collect()
    }

    /// `parts` in the home folder, as [`tilde`] writes it: `~` and the system's own separator, so
    /// `~/.codex/auth.json` on macOS and Linux and `~\.codex\auth.json` on Windows.
    fn in_home(parts: &[&str]) -> String {
        Path::new("~").join(path(parts)).display().to_string()
    }

    /// A sentence naming `place`, as a provider's error does.
    fn naming(place: &Path) -> String {
        format!(
            "could not read the sign-in from {}: it is not valid JSON",
            place.display()
        )
    }

    #[test]
    fn a_place_in_the_home_folder_is_written_with_a_tilde_in_any_text() {
        let home = path(&["/", "Users", "ann"]);
        let place = home.join(".codex").join("auth.json");

        assert_eq!(
            tilde_in(&naming(&place), Some(&home)),
            format!(
                "could not read the sign-in from {}: it is not valid JSON",
                in_home(&[".codex", "auth.json"])
            )
        );
        assert_eq!(
            tilde_in(&place.display().to_string(), Some(&home)),
            tilde(&place, Some(&home)),
            "the same as a path on its own"
        );
        assert_eq!(
            tilde_in(
                &format!("{} and {}", place.display(), home.display()),
                Some(&home)
            ),
            format!("{} and ~", in_home(&[".codex", "auth.json"])),
            "every place in the text, and the home folder itself"
        );
        assert_eq!(
            tilde_in(
                &format!("nothing in {}. Sign in.", home.display()),
                Some(&home)
            ),
            "nothing in ~. Sign in.",
            "the home folder at the end of a sentence"
        );
        let old = home.with_file_name("ann.old");
        assert_eq!(
            tilde_in(&format!("nothing in {}.", old.display()), Some(&home)),
            format!("nothing in {}.", old.display()),
            "a full stop inside a name"
        );
    }

    #[test]
    fn a_place_outside_the_home_folder_stays_as_it_is() {
        let home = path(&["/", "Users", "ann"]);
        for place in [
            path(&["/", "opt", "codex", "auth.json"]),
            // Another user's folder, whose name starts as the home folder's does.
            path(&["/", "Users", "anna", ".codex", "auth.json"]),
            // The home folder's own words inside a longer path.
            path(&[
                "/",
                "Volumes",
                "Backup",
                "Users",
                "ann",
                ".codex",
                "auth.json",
            ]),
        ] {
            assert_eq!(
                tilde_in(&naming(&place), Some(&home)),
                naming(&place),
                "{}",
                place.display()
            );
            assert_eq!(
                tilde(&place, Some(&home)),
                place.display().to_string(),
                "as a path on its own"
            );
        }
        let place = home.join(".codex").join("auth.json");
        assert_eq!(tilde_in(&naming(&place), None), naming(&place), "no home");
        assert_eq!(
            tilde_in(&naming(&place), Some(&path(&["/"]))),
            naming(&place),
            "a home folder that is the whole disk would make every path start with ~"
        );
    }

    #[test]
    fn a_windows_home_folder_is_written_with_a_tilde_too() {
        // As Windows writes it, from its parts with this system's separator, so on Windows the
        // text is `C:\Users\ann\.codex\auth.json`.
        let home = path(&["C:\\", "Users", "ann"]);
        let place = home.join(".codex").join("auth.json");

        assert_eq!(
            tilde_in(&naming(&place), Some(&home)),
            format!(
                "could not read the sign-in from {}: it is not valid JSON",
                in_home(&[".codex", "auth.json"])
            )
        );
        let other = path(&["C:\\", "Users", "anna", ".codex", "auth.json"]);
        assert_eq!(tilde_in(&naming(&other), Some(&home)), naming(&other));
        // A home folder written with a separator at its end names the same folder.
        let ended = PathBuf::from(format!("{}{}", home.display(), std::path::MAIN_SEPARATOR));
        assert_eq!(
            tilde_in(&naming(&place), Some(&ended)),
            tilde_in(&naming(&place), Some(&home))
        );
        // A path that goes on with `/`, which Windows takes too, keeps it after `~` (D-129).
        assert_eq!(
            tilde_in(&format!("nothing in {}/x.", home.display()), Some(&home)),
            "nothing in ~/x."
        );
    }

    /// `shell_word`'s word for `place`, held to the warning's pass leaving it as it was written.
    fn word(place: &Path, home: Option<&Path>) -> String {
        let word = shell_word(place, home);
        assert_eq!(tilde_in(&word, home), word, "the warning's pass");
        word
    }

    #[test]
    fn a_place_in_the_home_folder_is_quoted_after_a_bare_tilde() {
        // macOS keeps the data folder in `~/Library/Application Support`, whose space split the
        // command, and a shell expands `~` only where it is not quoted (D-130).
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let rest: PathBuf = ["Library", "Application Support", "quotadial", "config.json"]
            .iter()
            .collect();
        assert_eq!(
            word(&home.join(&rest), Some(&home)),
            format!("~/'{}'", rest.display())
        );
        assert_eq!(word(&home, Some(&home)), "~", "the home folder itself");
    }

    #[test]
    fn a_place_outside_the_home_folder_is_quoted_whole() {
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let beside: PathBuf = [MAIN_SEPARATOR_STR, "Users", "anna", "config.json"]
            .iter()
            .collect();
        let elsewhere: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "Data", "config.json"]
            .iter()
            .collect();
        for place in [beside, elsewhere] {
            assert_eq!(
                word(&place, Some(&home)),
                format!("'{}'", place.display()),
                "a folder whose name only starts as the home folder's does, and another disk"
            );
        }
        let place = home.join("config.json");
        assert_eq!(
            word(&place, None),
            format!("'{}'", place.display()),
            "no home folder"
        );
        assert_eq!(
            word(&place, Some(Path::new(MAIN_SEPARATOR_STR))),
            format!("'{}'", place.display()),
            "a home folder that is a whole disk, which the warning's sentence leaves as it is too"
        );
    }

    #[test]
    fn a_single_quote_in_a_place_is_written_as_a_shell_reads_it() {
        // `'\''`: the quotes closed, a quote escaped, and the quotes opened again.
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        assert_eq!(
            word(&home.join("Ann's data").join("config.json"), Some(&home)),
            format!(r"~/'Ann'\''s data{MAIN_SEPARATOR}config.json'")
        );
        let elsewhere: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "O'Brien", "config.json"]
            .iter()
            .collect();
        assert_eq!(
            word(&elsewhere, Some(&home)),
            format!(
                r"'{MAIN_SEPARATOR}Volumes{MAIN_SEPARATOR}O'\''Brien{MAIN_SEPARATOR}config.json'"
            )
        );
    }

    /// `powershell_string`'s string for `place`, held to the pass a message goes through leaving it
    /// as it was written (D-131).
    fn powershell(place: &Path, home: Option<&Path>) -> String {
        let string = powershell_string(place, home);
        assert_eq!(tilde_in(&string, home), string, "the pass");
        string
    }

    #[test]
    fn a_place_in_the_home_folder_is_a_powershell_string_from_home() {
        // PowerShell sets `$HOME` to the home folder and reads it in double quotes, so a command
        // names no user and works with a space or a quote in a name (D-131).
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        for name in [".claude-work", "My Claude", "Ann's Claude"] {
            assert_eq!(
                powershell(&home.join(name), Some(&home)),
                format!("\"$HOME{MAIN_SEPARATOR}{name}\"")
            );
        }
        let rest: PathBuf = ["Claude", "work"].iter().collect();
        assert_eq!(
            powershell(&home.join(&rest), Some(&home)),
            format!("\"$HOME{MAIN_SEPARATOR}{}\"", rest.display()),
            "the rest with the system's own separator"
        );
        assert_eq!(
            powershell(&home, Some(&home)),
            "\"$HOME\"",
            "the home folder itself"
        );
    }

    #[test]
    fn what_a_double_quoted_powershell_string_reads_is_written_after_a_backtick() {
        // In double quotes PowerShell reads `$` as a variable, the backtick as its escape character,
        // and `"` and the curly `“`, `”` and `„` as the string's end; a backtick before each takes
        // it as written (about_Quoting_Rules). Single quotes, curly or not, are plain there.
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        assert_eq!(
            powershell(
                &home.join("$HOME `x` \"q\" “a” „b ‘c’ $(d) {e}"),
                Some(&home)
            ),
            format!("\"$HOME{MAIN_SEPARATOR}`$HOME ``x`` `\"q`\" `“a`” `„b ‘c’ `$(d) {{e}}\"")
        );
    }

    #[test]
    fn a_place_outside_the_home_folder_is_a_single_quoted_powershell_string() {
        // In single quotes PowerShell reads nothing but the string's end, which `'` and the curly
        // `‘`, `’`, `‚` and `‛` each mark, so each is written twice (about_Quoting_Rules).
        let home: PathBuf = [MAIN_SEPARATOR_STR, "Users", "ann"].iter().collect();
        let beside: PathBuf = [MAIN_SEPARATOR_STR, "Users", "anna", "My Claude"]
            .iter()
            .collect();
        let elsewhere: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "Data", "$HOME `x` \"q\""]
            .iter()
            .collect();
        for place in [beside, elsewhere] {
            assert_eq!(
                powershell(&place, Some(&home)),
                format!("'{}'", place.display()),
                "a folder whose name only starts as the home folder's does, and another disk"
            );
        }
        let quoted: PathBuf = [MAIN_SEPARATOR_STR, "Volumes", "O'Brien ‘x’ ‚y‛"]
            .iter()
            .collect();
        assert_eq!(
            powershell(&quoted, Some(&home)),
            format!("'{MAIN_SEPARATOR}Volumes{MAIN_SEPARATOR}O''Brien ‘‘x’’ ‚‚y‛‛'")
        );
        let place = home.join("My Claude");
        assert_eq!(
            powershell(&place, None),
            format!("'{}'", place.display()),
            "no home folder"
        );
        assert_eq!(
            powershell(&place, Some(Path::new(MAIN_SEPARATOR_STR))),
            format!("'{}'", place.display()),
            "a home folder that is a whole disk"
        );
    }

    /// The home folders the tests write places beside, each from its parts: one rooted at `/`, as
    /// macOS and Linux write it, and one on a drive, as Windows does, which on Windows is the
    /// system's own.
    fn home_folders() -> [PathBuf; 2] {
        [
            path(&["/", "Users", "ann"]),
            path(&["C:\\", "Users", "ann"]),
        ]
    }

    /// A folder beside the home folder `home` whose name starts as the home folder's own and goes
    /// on with `more`: the home folder's text and then `more`, one name, such as `/Users/ann (old)`.
    fn longer(home: &Path, more: &str) -> PathBuf {
        PathBuf::from(format!("{}{more}", home.display()))
    }

    /// What a folder's name can go on with after the home folder's own (D-133): a space and a
    /// bracket, as people name a copy, a space and a word, a tab, an apostrophe, straight and
    /// curly, each mark a sentence puts after a path when something follows it, and a quote alone.
    /// Every one is legal in a folder's name, and none ends the home folder's own.
    const GOES_ON: [&str; 13] = [
        " (old)", " old", "\told", "'s", "’s", ", old", ";x", ")", ":x", "\"x\"", "`x`", ". old",
        "'",
    ];

    #[test]
    fn a_folder_whose_name_goes_on_from_the_home_folders_is_left_as_written_in_a_sentence() {
        // D-130's leftover: `/Users/ann (old)/quotadial` beside the home folder `/Users/ann`,
        // which a sentence wrote `~ (old)/quotadial`, a place that does not exist (D-133).
        for home in home_folders() {
            for more in GOES_ON {
                let place = longer(&home, more).join("quotadial").join("history.db");
                let file = place.display();
                for sentence in [
                    format!("could not open the history at {file}: permission denied"),
                    format!(
                        "could not use {file}: permission denied; check that you can write to it"
                    ),
                    format!("{file} holds webhook addresses, which work like passwords"),
                    file.to_string(),
                ] {
                    assert_eq!(
                        tilde_in(&sentence, Some(&home)),
                        sentence,
                        "{more:?} beside {}",
                        home.display()
                    );
                }
            }
        }
    }

    #[test]
    fn a_folder_whose_name_goes_on_from_the_home_folders_is_quoted_whole_in_a_posix_command() {
        // The command `config.json`'s warning and Claude's sign-in write for such a folder, held to
        // the pass leaving it as written, whatever the name holds (D-133).
        for home in home_folders() {
            let config = |more: &str| longer(&home, more).join("quotadial").join("config.json");
            let place = config(" (old)");
            assert_eq!(
                word(&place, Some(&home)),
                format!("'{}'", place.display()),
                "`chmod 600 '/Users/ann (old)/quotadial/config.json'`"
            );
            assert_eq!(
                word(&config("'s"), Some(&home)),
                format!(
                    r"'{}'\''s{MAIN_SEPARATOR}quotadial{MAIN_SEPARATOR}config.json'",
                    home.display()
                ),
                "a folder named `ann's`, its quote written `'\\''`"
            );
            for more in GOES_ON {
                // The folder itself and a file in it, each in the commands as they are written.
                for place in [longer(&home, more), config(more)] {
                    let word = word(&place, Some(&home));
                    for command in [
                        format!("chmod 600 {word}"),
                        format!("CLAUDE_CONFIG_DIR={word} claude"),
                        format!("run `CLAUDE_CONFIG_DIR={word} claude` to sign in again."),
                    ] {
                        assert_eq!(tilde_in(&command, Some(&home)), command, "{more:?}");
                    }
                }
                // As `config.json`'s warning words it: the file as a sentence names it, then the
                // command.
                let file = config(more);
                let warning = format!(
                    "{} holds webhook addresses, which work like passwords, and other users can \
                     read it; run `chmod 600 {}`.",
                    file.display(),
                    word(&file, Some(&home))
                );
                assert_eq!(tilde_in(&warning, Some(&home)), warning, "{more:?}");
            }
        }
    }

    #[test]
    fn a_folder_whose_name_goes_on_from_the_home_folders_is_a_single_quoted_powershell_string() {
        // The same in PowerShell, which writes each single quotation mark twice, curly ones too, so
        // a name holding one is a string that goes on past it (D-133).
        for home in home_folders() {
            let config = |more: &str| longer(&home, more).join("quotadial").join("config.json");
            let rest = format!("{MAIN_SEPARATOR}quotadial{MAIN_SEPARATOR}config.json'");
            let place = config(" (old)");
            assert_eq!(
                powershell(&place, Some(&home)),
                format!("'{}'", place.display()),
                "`'C:\\Users\\ann (old)\\quotadial\\config.json'`"
            );
            assert_eq!(
                powershell(&config("'s"), Some(&home)),
                format!("'{}''s{rest}", home.display()),
                "a folder named `ann's`, its quote written twice"
            );
            assert_eq!(
                powershell(&config("’s"), Some(&home)),
                format!("'{}’’s{rest}", home.display()),
                "and its curly quote"
            );
            for more in GOES_ON {
                for place in [longer(&home, more), config(more)] {
                    let string = powershell(&place, Some(&home));
                    for command in [
                        format!("$env:CLAUDE_CONFIG_DIR = {string}; claude"),
                        format!("Set-Location {string}"),
                        format!(
                            "run `$env:CLAUDE_CONFIG_DIR = {string}; claude` to sign in again."
                        ),
                    ] {
                        assert_eq!(tilde_in(&command, Some(&home)), command, "{more:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn the_home_folder_on_its_own_is_told_from_a_longer_name_beside_it() {
        for home in home_folders() {
            let old = longer(&home, " (old)").join("quotadial");
            // Beside each other in one sentence: the longer name as written, the home folder `~`.
            assert_eq!(
                tilde_in(
                    &format!(
                        "could not use {}: permission denied; {} is not writable",
                        old.display(),
                        home.display()
                    ),
                    Some(&home)
                ),
                format!(
                    "could not use {}: permission denied; ~ is not writable",
                    old.display()
                )
            );
            // Every way the home folder on its own can end a place in a sentence: the end of the
            // text or of a line, a full stop, a colon, a comma, a bracket, or a space and a word.
            for ending in [
                "",
                ".",
                ":",
                ": permission denied",
                ". Sign in.",
                ", then try again",
                "; try again",
                ") and",
                " is not writable",
                " and ~/.codex/auth.json",
                " /usr/local/bin",
                " ./here/too",
                " ../up/there",
                " C:\\Windows\\System32",
                " C:/Windows/System32",
                "\nthe next line holds a/separator",
                "  \nsome/where else",
            ] {
                assert_eq!(
                    tilde_in(
                        &format!("nothing in {}{ending}", home.display()),
                        Some(&home)
                    ),
                    format!("nothing in ~{ending}"),
                    "{ending:?}"
                );
            }
            // The next word holds a second mention of the home folder, which is written `~` too.
            // The word carries no name on, whether the mention is written yet or not, since the
            // home folder's own separators are not a name's, so a second pass changes nothing.
            let twice = format!("{0} x({0}) y", home.display());
            let once = tilde_in(&twice, Some(&home));
            assert_eq!(once, "~ x(~) y");
            assert_eq!(tilde_in(&once, Some(&home)), once);
        }
    }

    #[test]
    fn a_word_in_single_quotes_is_read_whole() {
        for home in home_folders() {
            let at = home.display();
            // The home folder on its own or at the start of a place in it, as without the quotes.
            assert_eq!(tilde_in(&format!("'{at}'"), Some(&home)), "'~'");
            assert_eq!(
                tilde_in(
                    &format!("said '{at}', or '{at}/a b' (see '{at}'.)"),
                    Some(&home)
                ),
                "said '~', or '~/a b' (see '~'.)"
            );
            // A word that goes on from the home folder's text is another place, whatever follows
            // it and however it ends: as a POSIX shell reads it, `'\''` is a quote, and as
            // PowerShell reads it, two quotes in a row are one, curly ones too.
            for quoted in [
                format!("'{at} (old)'"),
                format!("'{at}, old'"),
                format!("'{at};x'"),
                format!("'{at}.'"),
                format!("'{at})'"),
                format!(r"'{at}'\''s'"),
                format!("'{at}''s'"),
                format!("'{at}’’s'"),
                format!("'{at}’s'"),
                format!("'{at}'\\'''"),
            ] {
                let said = format!("cd {quoted} now");
                assert_eq!(tilde_in(&said, Some(&home)), said);
            }
            // What is inside a word that does not start in the home folder is not a place of its
            // own, so nothing in it is written `~`.
            for quoted in [
                format!("'/Volumes/Backup {at}/x'"),
                format!("'a {at}'"),
                format!("'a'\\'' {at}/b'"),
                format!("'a''{at}/b'"),
            ] {
                let said = format!("cd {quoted} now");
                assert_eq!(tilde_in(&said, Some(&home)), said);
            }
            // A word that goes on from a path, as the rest of a place in the home folder does after
            // `~/`, is not a place of its own, so the home folder's text in it is left as written.
            let after_tilde = format!("cd ~/'{at}'");
            assert_eq!(tilde_in(&after_tilde, Some(&home)), after_tilde);
            // A quote that opens no word is read as a sentence reads it: an apostrophe in a word,
            // or a quote that never closes.
            assert_eq!(
                tilde_in(
                    &format!("can't open {at}/x: it's gone, or '{at} is open"),
                    Some(&home)
                ),
                "can't open ~/x: it's gone, or '~ is open"
            );
            assert_eq!(
                tilde_in(&format!("'{at} (old)/x is open"), Some(&home)),
                format!("'{at} (old)/x is open"),
                "an open quote and a longer name"
            );
        }
    }

    /// What a hidden name is written as, in the tests as in the code.
    const USER: &str = "[user]";

    /// `parts` joined with `/`, as a sentence of any system can write a place: `["", "Volumes"]` is
    /// `/Volumes`. Every place the tests of [`hide_user_in`] write is made from its parts, and the
    /// rule reads both separators on every system, so none depends on the one this runs on.
    fn slashed(parts: &[&str]) -> String {
        parts.join("/")
    }

    /// `parts` joined with `\`, as Windows writes a place: `["C:", "Users"]` is `C:\Users`.
    fn backslashed(parts: &[&str]) -> String {
        parts.join("\\")
    }

    /// `text` as doctor's pass leaves it: the home folder written `~`, then the user's name hidden.
    fn doctor_pass(text: &str, home: &Path) -> String {
        hide_user_in(&tilde_in(text, Some(home)), Some(home))
    }

    #[test]
    fn a_users_name_is_hidden_in_a_place_outside_the_home_folder() {
        // Another disk's copy of the home folder, the temporary folder Claude Code makes for a
        // working folder (its path with `/` written `-`, and the user id after the name), a drive
        // and a network share as Windows writes them, and a folder named after the name: the name
        // is a component of a place that is not the home folder, whoever the user is (D-134).
        for name in ["ann", "claude", "codex"] {
            let home = path(&["/", "Users", name]);
            let places = |who: &str| {
                [
                    slashed(&["", "Volumes", "Backup", "Users", who, ".keys", "cache.json"]),
                    slashed(&[
                        "",
                        "private",
                        "tmp",
                        &format!("{who}-501"),
                        &format!("-Users-{who}--work"),
                        "data",
                    ]),
                    backslashed(&["D:", "Backup", "Users", who, ".keys"]),
                    backslashed(&["", "", "server", "home", who]),
                    slashed(&["", "home", &format!("{who}.old"), "x"]),
                ]
            };
            for (place, hidden) in places(name).into_iter().zip(places(USER)) {
                // The name hidden in the place, and kept in the sentence around it.
                let said = |place: &str| {
                    [
                        format!("could not read the sign-in from {place}: it is not valid JSON"),
                        format!("Data folder: {place}\n"),
                        format!("settings: ({place}), {place}."),
                        format!("{name} says: could not read {place} for {name}"),
                        place.to_owned(),
                    ]
                };
                for (said, hidden) in said(&place).into_iter().zip(said(&hidden)) {
                    assert_eq!(doctor_pass(&said, &home), hidden, "{name}: {said}");
                }
            }
        }
    }

    #[test]
    fn a_word_that_is_only_spelled_as_the_users_name_stays_as_it_is() {
        // In a sentence, a setting, a provider's id and a web address, none of which names a place:
        // for a user called `claude`, the plain replace of the whole word wrote `run [user] once`
        // (D-134). A relative path has no root, so it is none of these either.
        for (name, said) in [
            ("claude", "If it stops working, run claude once."),
            ("claude", "The command 'claude' is not on the PATH."),
            (
                "claude",
                "Run `claude` with the CLAUDE_CONFIG_DIR it was made with.",
            ),
            ("copilot", "copilot: request failed: GitHub did not answer."),
            ("status", "  status_pages: on"),
            ("history", "  history_days: 30"),
            ("daemon", "Numbers: from the daemon"),
            (
                "quotadial",
                "The week's cost at API prices is part of QuotaDial Pro: quotadial.com/pro",
            ),
            ("ann", "see ann/x and Users/ann/y, then ann"),
        ] {
            let home = path(&["/", "Users", name]);
            assert_eq!(hide_user_in(said, Some(&home)), said, "{name}");
        }
        let home = path(&["/", "Users", "someone"]);
        assert_eq!(
            hide_user_in("/x/someones /y/someone_x /z/xsomeone", Some(&home)),
            "/x/someones /y/[user]_x /z/xsomeone",
            "a whole name only"
        );
    }

    #[test]
    fn a_place_in_the_home_folder_keeps_the_words_it_has_whoever_the_user_is() {
        // `~/.claude` for a user called `claude`, `~/.codex` for `codex`, and macOS's own folders
        // for a user called `Support` or `quotadial`: a place that starts at `~` is the user's own
        // however it is spelled, and the plain replace wrote `~/.[user]` (D-134).
        for name in ["claude", "codex", "gemini", "quotadial", "Support"] {
            let home = path(&["/", "Users", name]);
            let in_the_home = |parts: &[&str]| {
                parts
                    .iter()
                    .fold(home.clone(), |place, part| place.join(part))
            };
            for place in [
                in_the_home(&[".claude", ".credentials.json"]),
                in_the_home(&[".codex", "auth.json"]),
                in_the_home(&[".gemini", "settings.json"]),
                in_the_home(&["Library", "Application Support", "quotadial", "config.json"]),
                in_the_home(&[name, name]),
            ] {
                for said in [
                    naming(&place),
                    format!("Data folder: {}\n", place.display()),
                    format!("could not use {}: permission denied", place.display()),
                    place.display().to_string(),
                ] {
                    let written = tilde_in(&said, Some(&home));
                    assert!(written.contains('~'), "{name}: {written}");
                    assert_eq!(
                        hide_user_in(&written, Some(&home)),
                        written,
                        "{name}: {said}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_command_for_a_place_in_the_home_folder_comes_through_whole_for_any_user() {
        // The sign-in commands, in a POSIX shell and in PowerShell, and `config.json`'s `chmod 600`
        // come through doctor's whole pass as they were written, with the user's name among their
        // words and in the place (D-134): `CLAUDE_CONFIG_DIR=~/'.claude-work' claude` for a user
        // called `claude` was `CLAUDE_CONFIG_DIR=~/'.[user]-work' [user]`.
        for name in ["claude", "codex", "config", "quotadial", "work"] {
            let home = path(&["/", "Users", name]);
            // A folder whose name ends in a space beside one named like the user: in PowerShell's
            // string it is `"$HOME/a /claude"`, whose second word starts like a place.
            let rests = [
                ".claude-work".to_owned(),
                ".claude".to_owned(),
                "My Claude".to_owned(),
                "Ann's Claude".to_owned(),
                name.to_owned(),
                "$x `y` \"z\"".to_owned(),
                format!("a /{name}"),
            ];
            for rest in &rests {
                let dir = home.join(rest);
                let file = dir.join("config.json");
                let (posix, shell) = (word(&dir, Some(&home)), powershell(&dir, Some(&home)));
                let mut commands = vec![
                    format!("CLAUDE_CONFIG_DIR={posix} claude"),
                    format!("Run `CLAUDE_CONFIG_DIR={posix} claude` to sign in again."),
                    format!("$env:CLAUDE_CONFIG_DIR = {shell}; claude"),
                    format!("Run `$env:CLAUDE_CONFIG_DIR = {shell}; claude` to sign in again."),
                    format!("chmod 600 {}", word(&file, Some(&home))),
                ];
                // The warning names the file as a sentence writes it, which a folder name with a
                // quote in it, or one that ends in a space before a separator, makes a guess: the
                // commands are what is held to be exact.
                if !rest.contains('"') && !rest.contains(" /") {
                    commands.push(format!(
                        "{} holds webhook addresses, which work like passwords, and other users \
                         can read it; run `chmod 600 {}`.",
                        tilde(&file, Some(&home)),
                        word(&file, Some(&home))
                    ));
                }
                for command in commands {
                    assert_eq!(doctor_pass(&command, &home), command, "{name}: {command}");
                }
            }
        }
    }

    #[test]
    fn a_web_address_is_not_a_place() {
        // The daemon's API in a message, and the hosts a provider is asked, for a user whose name is
        // a word in one, such as `status`; only `file:` names a place (D-134).
        let said = |address: &str| {
            format!(
                "the QuotaDial daemon is running, but its API at {address} could not be reached"
            )
        };
        for (name, address) in [
            ("status", "http://127.0.0.1:47115/v1/status"),
            ("headroom", "http://127.0.0.1:47115/v1/headroom"),
            ("anthropic", "https://api.anthropic.com/api/oauth/usage"),
            ("api", "https://api.anthropic.com/api/oauth/usage"),
            ("wham", "https://chatgpt.com/backend-api/wham/usage"),
        ] {
            let home = path(&["/", "Users", name]);
            assert_eq!(
                hide_user_in(&said(address), Some(&home)),
                said(address),
                "{name}"
            );
        }
        let home = path(&["/", "Users", "ann"]);
        assert_eq!(
            hide_user_in(
                "file:///Users/ann/.claude and file://host/Users/ann/x",
                Some(&home)
            ),
            "file:///Users/[user]/.claude and file://host/Users/[user]/x"
        );
        // A drive written with two separators is a drive, not a scheme of one letter.
        assert_eq!(
            hide_user_in("D://Backup/Users/ann/x", Some(&home)),
            "D://Backup/Users/[user]/x"
        );
    }

    #[test]
    fn the_words_around_a_quoted_place_outside_the_home_folder_stay() {
        // `shell_word` and `powershell_string` write a place outside the home folder in single
        // quotes, one place whatever it holds. The name in it is hidden, and the variable's name
        // before it and the `claude` that runs after it are not in it (D-134).
        let home = path(&["/", "Users", "claude"]);
        for rest in ["My Claude", "O'Brien's data", "a b/c d"] {
            let outside = |who: &str| path(&["/", "Volumes", "Data", who]).join(rest);
            let (shown, hidden) = (outside("claude"), outside(USER));
            for (shown, hidden) in [
                (word(&shown, Some(&home)), word(&hidden, Some(&home))),
                (
                    powershell(&shown, Some(&home)),
                    powershell(&hidden, Some(&home)),
                ),
            ] {
                let said = |place: &str| {
                    [
                        format!("CLAUDE_CONFIG_DIR={place} claude"),
                        format!("Run `$env:CLAUDE_CONFIG_DIR = {place}; claude` to sign in."),
                        format!("chmod 600 {place}"),
                    ]
                };
                for (said, hidden) in said(&shown).into_iter().zip(said(&hidden)) {
                    assert_eq!(doctor_pass(&said, &home), hidden, "{rest}");
                }
            }
        }
        // A place on a drive, as PowerShell's single quotes hold it on Windows, where the quoted
        // word starts at the drive and no separator comes first.
        let on_a_drive = |who: &str| {
            format!(
                "$env:CLAUDE_CONFIG_DIR = '{}'; claude",
                backslashed(&["D:", "Data", who, "My Claude"])
            )
        };
        assert_eq!(doctor_pass(&on_a_drive("claude"), &home), on_a_drive(USER));
    }

    #[test]
    fn a_quoted_word_that_goes_on_from_a_path_is_part_of_that_path() {
        // As `tilde_in` reads it (D-133): after `~/` the word is the home folder's own, whatever
        // it starts with, and after a place outside the home folder it is that place (D-134).
        let home = path(&["/", "Users", "ann"]);
        for (said, hidden) in [
            ("cd ~/'/Volumes/ann'", "cd ~/'/Volumes/ann'"),
            ("cd ~/'a b/ann'", "cd ~/'a b/ann'"),
            (
                "cd /Volumes/Backup/'Users/ann'",
                "cd /Volumes/Backup/'Users/[user]'",
            ),
            (
                "cd /Volumes/Backup/'my disk'/ann",
                "cd /Volumes/Backup/'my disk'/[user]",
            ),
        ] {
            assert_eq!(hide_user_in(said, Some(&home)), hidden, "{said}");
        }
    }

    #[test]
    fn a_place_goes_on_past_a_space_only_into_a_word_with_a_separator() {
        let home = path(&["/", "Users", "ann"]);
        // A disk or a folder named with a space: the next word holds a separator, so it is the
        // same place, and the name in it is hidden wherever it is.
        for (said, hidden) in [
            (
                slashed(&["", "Volumes", "My Passport", "Users", "ann", "x"]),
                slashed(&["", "Volumes", "My Passport", "Users", USER, "x"]),
            ),
            (
                backslashed(&["D:", "My Backup", "Users", "ann"]),
                backslashed(&["D:", "My Backup", "Users", USER]),
            ),
            (
                format!(
                    "{} ~/ann/y",
                    slashed(&["", "Volumes", "Backup", "Users", "ann"])
                ),
                format!(
                    "{} ~/ann/y",
                    slashed(&["", "Volumes", "Backup", "Users", USER])
                ),
            ),
        ] {
            let sentence = |place: &str| format!("could not read {place}: it is not valid JSON");
            assert_eq!(
                hide_user_in(&sentence(&said), Some(&home)),
                sentence(&hidden)
            );
        }
        // A plain word is the sentence going on, and a line break ends a place, so the name after
        // them is a word of the sentence, not of the place.
        for said in [
            format!(
                "could not read {} for ann",
                slashed(&["", "opt", "x", "auth.json"])
            ),
            format!("{}: ann says no", slashed(&["", "opt", "x", "auth.json"])),
            format!("{}\nann/x and Users/ann", slashed(&["", "opt", "x"])),
        ] {
            assert_eq!(hide_user_in(&said, Some(&home)), said);
        }
    }

    #[test]
    fn a_place_outside_the_home_folder_ends_where_one_in_it_starts_in_the_same_word() {
        // Places joined by `:` or `=` in a word, as a list of folders is, or a command's variable
        // after a place: what comes after `~` or `$HOME` is the user's own, whatever it holds, and
        // the place before it is not (D-134).
        let home = path(&["/", "Users", "ann"]);
        for (said, hidden) in [
            (
                "/Volumes/Backup/Users/ann:~/ann/y",
                "/Volumes/Backup/Users/[user]:~/ann/y",
            ),
            (
                "/Volumes/Backup/Users/ann:$HOME/ann",
                "/Volumes/Backup/Users/[user]:$HOME/ann",
            ),
            (
                "/Volumes/Backup/Users/ann:%USERPROFILE%/ann",
                "/Volumes/Backup/Users/[user]:%USERPROFILE%/ann",
            ),
            (
                "/Volumes/Backup/Users/ann:%USERPROFILE%\\ann",
                "/Volumes/Backup/Users/[user]:%USERPROFILE%\\ann",
            ),
            (
                "/Volumes/Backup/Users/ann:${HOME}/ann",
                "/Volumes/Backup/Users/[user]:${HOME}/ann",
            ),
            (
                "/Volumes/Backup/Users/ann CLAUDE_CONFIG_DIR=~/ann/y claude",
                "/Volumes/Backup/Users/[user] CLAUDE_CONFIG_DIR=~/ann/y claude",
            ),
            (
                "/Volumes/Backup/Users/ann \"$HOME/ann\"",
                "/Volumes/Backup/Users/[user] \"$HOME/ann\"",
            ),
        ] {
            assert_eq!(hide_user_in(said, Some(&home)), hidden, "{said}");
        }
    }

    #[test]
    fn a_name_with_a_space_in_it_is_hidden_as_one_name() {
        // Windows names a profile folder after the person, `C:\Users\Ann Smith`, with a space in it:
        // the name is hidden whole, where it ends a place and where it starts one, and what comes
        // after it in the sentence is not taken for the place going on (D-134).
        let home = path(&["C:\\", "Users", "Ann Smith"]);
        for (said, hidden) in [
            (
                backslashed(&["D:", "Backup", "Ann Smith", "Documents"]),
                backslashed(&["D:", "Backup", USER, "Documents"]),
            ),
            (
                backslashed(&["D:", "Backup", "Ann Smith"]),
                backslashed(&["D:", "Backup", USER]),
            ),
            (
                backslashed(&["D:", "Backup", "Ann Smith (old)", "x"]),
                backslashed(&["D:", "Backup", &format!("{USER} (old)"), "x"]),
            ),
        ] {
            let sentence =
                |place: &str| format!("Ann Smith: could not read {place}: it is not valid JSON");
            assert_eq!(
                hide_user_in(&sentence(&said), Some(&home)),
                sentence(&hidden)
            );
        }
        // The name's own space does not end a place, and what is read after one name is read the
        // same once it is hidden: both names go in one pass, and the text is as it will be again.
        let said = backslashed(&["D:", "a b", "Ann Smith y", "Ann Smith", "z"]);
        let once = hide_user_in(&said, Some(&home));
        assert_eq!(once, backslashed(&["D:", "a b", "[user] y", "[user]", "z"]));
        assert_eq!(hide_user_in(&once, Some(&home)), once);
    }

    #[test]
    fn a_name_that_is_not_whole_does_not_hide_a_whole_one_that_overlaps_it() {
        // `a.a` in `xa.a.a`: the first is part of a longer name, and the second shares its last
        // letter with it (D-134).
        let home = path(&["/", "Users", "a.a"]);
        let once = hide_user_in("/x/xa.a.a", Some(&home));

        assert_eq!(once, "/x/xa.[user]");
        assert_eq!(hide_user_in(&once, Some(&home)), once);
    }

    #[test]
    fn a_name_that_is_a_word_of_the_marker_is_hidden_once() {
        // A user called `user` is hidden as `[user]`, which holds the name as a word: it reads as
        // hidden already, so hiding it again changes nothing (D-134).
        let home = path(&["/", "Users", "user"]);
        let once = hide_user_in("/Volumes/Backup/user and /opt/user/x", Some(&home));

        assert_eq!(once, "/Volumes/Backup/[user] and /opt/[user]/x");
        assert_eq!(hide_user_in(&once, Some(&home)), once);
    }

    #[test]
    fn nothing_is_hidden_without_a_name_of_three_characters_in_the_text() {
        let text = "/Volumes/Backup/Users/ann/x and /Volumes/Backup/Users/me/y";
        assert_eq!(hide_user_in(text, None), text, "no home folder");
        assert_eq!(
            hide_user_in(text, Some(Path::new(MAIN_SEPARATOR_STR))),
            text,
            "a home folder that is a whole disk"
        );
        assert_eq!(
            hide_user_in(text, Some(&path(&["/", "Users", "me"]))),
            text,
            "too short to be sure"
        );
        assert_eq!(
            hide_user_in(text, Some(&path(&["/", "Users", "bob"]))),
            text,
            "a name the text does not hold"
        );
    }

    fn ts(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .expect("valid timestamp")
            .with_timezone(&Utc)
    }

    /// Four minutes after the times the tests start from.
    fn now() -> DateTime<Utc> {
        ts("2099-01-01T00:04:00Z")
    }

    #[test]
    fn durations_read_naturally_and_round_down() {
        assert_eq!(duration(TimeDelta::seconds(59)), "under a minute");
        assert_eq!(duration(TimeDelta::minutes(45)), "45 min");
        assert_eq!(duration(TimeDelta::hours(2)), "2 h");
        assert_eq!(
            duration(TimeDelta::minutes(130) + TimeDelta::seconds(59)),
            "2 h 10 min"
        );
        assert_eq!(duration(TimeDelta::days(3)), "3 d");
        assert_eq!(
            duration(TimeDelta::days(3) + TimeDelta::hours(4) + TimeDelta::minutes(5)),
            "3 d 4 h"
        );
    }

    #[test]
    fn reset_times_and_ages_are_relative_to_now() {
        assert_eq!(
            resets(Some(ts("2099-01-01T05:00:00Z")), now()),
            "resets in 4 h 56 min"
        );
        assert_eq!(
            resets(Some(ts("2099-01-01T00:01:00Z")), now()),
            "reset 3 min ago",
            "numbers from before a reset say so"
        );
        assert_eq!(resets(None, now()), "no reset time reported");
        assert_eq!(ago(ts("2099-01-01T00:00:00Z"), now()), "4 min ago");
        assert_eq!(ago(ts("2099-01-01T00:03:30Z"), now()), "just now");
        assert_eq!(
            ago(ts("2099-01-01T00:09:00Z"), now()),
            "just now",
            "a clock that moved back"
        );
    }

    #[test]
    fn a_moment_in_a_sentence_is_written_as_a_person_reads_it() {
        assert_eq!(
            date_and_time(ts("2026-10-26T09:12:47Z")),
            "26 Oct 2026 at 09:12 UTC"
        );
        // The seconds are dropped, never rounded up into the next minute, day or year.
        assert_eq!(
            date_and_time(ts("2098-12-31T23:59:59.999Z")),
            "31 Dec 2098 at 23:59 UTC"
        );
        // A day has no nought before it; an hour and a minute have one.
        assert_eq!(
            date_and_time(ts("2026-01-05T07:05:00Z")),
            "5 Jan 2026 at 07:05 UTC"
        );
        assert_eq!(
            date_and_time(DateTime::<Utc>::UNIX_EPOCH),
            "1 Jan 1970 at 00:00 UTC"
        );
        // A moment from another zone is written in UTC.
        assert_eq!(
            date_and_time(ts("2026-10-26T10:12:00+01:00")),
            "26 Oct 2026 at 09:12 UTC"
        );
        // Each month in three letters.
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        for (month, name) in (1..).zip(months) {
            assert_eq!(
                date_and_time(ts(&format!("2026-{month:02}-01T12:00:00Z"))),
                format!("1 {name} 2026 at 12:00 UTC")
            );
        }
    }

    #[test]
    fn a_pace_says_whether_the_window_runs_out_before_it_resets() {
        let resets_at = Some(ts("2099-01-01T02:14:00Z"));
        let pace = |pct_per_hour, runs_out_at: Option<&str>| Pace {
            pct_per_hour,
            runs_out_at: runs_out_at.map(ts),
        };

        assert_eq!(
            super::pace(&pace(8.46, Some("2099-01-01T01:34:00Z")), resets_at, now()),
            "at 8.5%/h, runs out in 1 h 30 min, 40 min before it resets"
        );
        assert_eq!(
            super::pace(&pace(25.0, None), resets_at, now()),
            "at 25%/h, lasts until it resets"
        );
        assert_eq!(
            super::pace(&pace(0.01, None), resets_at, now()),
            "at 0%/h, lasts until it resets"
        );
        assert_eq!(
            super::pace(&pace(90.0, Some("2099-01-01T00:03:00Z")), resets_at, now()),
            "at 90%/h, runs out about now",
            "a run-out the clock has passed since the reading"
        );
        assert_eq!(
            super::pace(&pace(2.0, Some("2099-01-01T19:04:00Z")), None, now()),
            "at 2.0%/h, runs out in 19 h"
        );
        assert_eq!(super::pace(&pace(0.0, None), None, now()), "at 0%/h");
    }

    #[test]
    fn every_source_is_named_as_the_sentence_after_from_reads() {
        assert_eq!(source_label(Source::Api), "the API");
        assert_eq!(source_label(Source::LocalLog), "local logs");
        assert_eq!(source_label(Source::Cache), "cache");
        assert_eq!(
            source_label(Source::Example),
            "the example build",
            "so an example is never taken for a real account's numbers"
        );
        assert_eq!(
            source_label(Source::StatusLine),
            "the CLI's status line",
            "a number a CLI handed over, never one QuotaDial asked for (P-022)"
        );
    }

    #[test]
    fn a_sign_in_read_from_a_place_says_so_and_that_it_is_never_changed() {
        assert_eq!(
            sign_in_read_from("~/.codex/auth.json"),
            "QuotaDial reads this account's sign-in from ~/.codex/auth.json. It only reads it and \
             never changes it."
        );
    }

    #[test]
    fn shares_are_written_as_people_write_them() {
        assert_eq!(percent(80.0), "80");
        assert_eq!(percent(80.5), "80.5");
        assert_eq!(percent(33.333), "33.3");
    }

    /// Extra usage that is on, with nothing more known: what the tests below add to.
    fn extra_on() -> ExtraUsage {
        ExtraUsage {
            enabled: true,
            used_pct: None,
            used: None,
            limit: None,
            currency: None,
            balance: None,
        }
    }

    /// Extra usage that is on, with `used` of `limit` spent in `currency`, `used_pct` of it.
    fn spent(used: f64, limit: f64, currency: Option<&str>, used_pct: Option<f64>) -> ExtraUsage {
        ExtraUsage {
            used_pct,
            used: Some(used),
            limit: Some(limit),
            currency: currency.map(str::to_owned),
            ..extra_on()
        }
    }

    #[test]
    fn extra_usage_that_is_off_says_so() {
        let off = ExtraUsage {
            enabled: false,
            ..extra_on()
        };
        assert_eq!(extra_usage(&off), "off");
        assert_eq!(
            extra_usage(&ExtraUsage {
                enabled: false,
                ..spent(12.5, 50.0, Some("USD"), Some(25.0))
            }),
            "off",
            "whatever numbers come with it"
        );
    }

    #[test]
    fn extra_usage_that_is_on_with_nothing_more_known_says_on() {
        // Nothing is known but that it is on, so the word stays: without it the line says nothing.
        assert_eq!(extra_usage(&extra_on()), "on");
    }

    #[test]
    fn extra_usage_in_dollars_says_what_is_used_with_the_symbol_before_each_amount() {
        assert_eq!(
            extra_usage(&spent(12.5, 50.0, Some("USD"), Some(25.0))),
            "$12.50 of $50.00 used (25%)"
        );
        assert_eq!(
            extra_usage(&spent(12.5, 50.0, Some("USD"), None)),
            "$12.50 of $50.00 used",
            "with no share given"
        );
        // Both amounts in one form, to the cent with their thousands grouped, whatever their size.
        assert_eq!(
            extra_usage(&spent(0.0, 50.0, Some("USD"), Some(0.0))),
            "$0.00 of $50.00 used (0%)"
        );
        assert_eq!(
            extra_usage(&spent(99.5, 150.0, Some("USD"), Some(66.33))),
            "$99.50 of $150.00 used (66%)"
        );
        assert_eq!(
            extra_usage(&spent(1_234.5, 5_000.0, Some("USD"), Some(24.69))),
            "$1,234.50 of $5,000.00 used (25%)"
        );
        // A currency left out or blank is dollars, as the Claude provider reads one, and so is
        // USD in any case.
        for currency in [None, Some(""), Some(" "), Some("usd"), Some(" USD ")] {
            assert_eq!(
                extra_usage(&spent(1.5, 20.0, currency, Some(7.0))),
                "$1.50 of $20.00 used (7%)",
                "{currency:?}"
            );
        }
    }

    #[test]
    fn extra_usage_in_another_currency_keeps_its_code_after_the_amounts() {
        assert_eq!(
            extra_usage(&spent(12.5, 50.0, Some("EUR"), Some(25.0))),
            "12.50 of 50.00 EUR used (25%)"
        );
        assert_eq!(
            extra_usage(&spent(1_234.5, 5_000.0, Some("EUR"), Some(24.69))),
            "1,234.50 of 5,000.00 EUR used (25%)"
        );
    }

    #[test]
    fn money_people_set_is_written_to_the_cent_with_its_thousands_grouped() {
        assert_eq!(to_the_cent(12.5), "12.50");
        assert_eq!(to_the_cent(0.0), "0.00");
        assert_eq!(to_the_cent(0.004), "0.00", "under a cent");
        assert_eq!(
            to_the_cent(99.999),
            "100.00",
            "a cent carried into the whole part"
        );
        assert_eq!(to_the_cent(1_234.5), "1,234.50");
        assert_eq!(to_the_cent(1_000_000.0), "1,000,000.00");
        // Nothing, as `usd` takes them: a negative amount, negative zero and one that is not a
        // number.
        for nothing in [-1.5, -0.0, f64::NAN] {
            assert_eq!(to_the_cent(nothing), "0.00", "{nothing}");
        }
    }

    #[test]
    fn extra_usage_with_a_share_and_no_amounts_says_the_share_used() {
        let share = ExtraUsage {
            used_pct: Some(25.0),
            ..extra_on()
        };
        assert_eq!(extra_usage(&share), "25% used");
        // The amounts are written together or not at all, so one alone leaves the share, or "on".
        assert_eq!(
            extra_usage(&ExtraUsage {
                used: Some(12.5),
                currency: Some("USD".into()),
                ..share
            }),
            "25% used"
        );
        assert_eq!(
            extra_usage(&ExtraUsage {
                limit: Some(50.0),
                ..extra_on()
            }),
            "on"
        );
    }

    #[test]
    fn a_credit_balance_is_shown() {
        let balance = ExtraUsage {
            balance: Some(1_056.436_375),
            ..extra_on()
        };
        assert_eq!(extra_usage(&balance), "on, 1056.44 credits left");
        // After what is used, as it is after "on".
        assert_eq!(
            extra_usage(&ExtraUsage {
                balance: Some(12.5),
                ..spent(12.5, 50.0, Some("USD"), Some(25.0))
            }),
            "$12.50 of $50.00 used (25%), 12.50 credits left"
        );
        assert_eq!(
            extra_usage(&ExtraUsage {
                used_pct: Some(25.0),
                ..balance
            }),
            "25% used, 1056.44 credits left"
        );
    }

    #[test]
    fn counts_and_money_read_as_people_write_them() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_204), "1,204");
        assert_eq!(thousands(12_345_678), "12,345,678");
        assert_eq!(plural(1, "request"), "1 request");
        assert_eq!(plural(0, "request"), "0 requests");
        assert_eq!(plural(51_234, "request"), "51,234 requests");

        assert_eq!(usd(0.0), "$0");
        assert_eq!(usd(f64::NAN), "$0");
        assert_eq!(usd(0.004), "under $0.01");
        assert_eq!(usd(12.345), "$12.35");
        assert_eq!(usd(99.99), "$99.99");
        assert_eq!(usd(1_234.5), "$1,235");
    }

    #[test]
    fn a_cost_says_what_the_week_would_have_cost() {
        let until = DateTime::parse_from_rfc3339("2026-10-24T00:00:00Z")
            .expect("valid")
            .with_timezone(&Utc);
        let mut cost = Cost {
            provider: "claude".to_owned(),
            since: until - TimeDelta::days(7),
            until,
            usd: 1_234.4,
            requests: 51_234,
            tokens: crate::Tokens::default(),
            models: Vec::new(),
            prices_updated: "2026-10-23".to_owned(),
            notes: Vec::new(),
        };

        assert_eq!(
            super::cost(&cost),
            "$1,234 at API prices over the last 7 days, from 51,234 requests"
        );
        cost.requests = 0;
        cost.usd = 0.0;
        assert_eq!(super::cost(&cost), "no requests in the last 7 days");

        let mut model = ModelCost {
            model: "claude-opus-5-5".to_owned(),
            usd: Some(1_000.25),
            requests: 41_000,
            tokens: crate::Tokens::default(),
        };
        assert_eq!(
            model_cost(&model),
            "claude-opus-5-5: $1,000, 41,000 requests"
        );
        model.usd = None;
        model.requests = 1;
        assert_eq!(model_cost(&model), "claude-opus-5-5: no price, 1 request");
    }

    #[test]
    fn the_words_people_never_read_are_found_outside_names_only() {
        // #52's plain words: a window, a reset, a sign-in, never these.
        assert_eq!(
            unplain_words("Your quota resets soon; check your Auth and Credentials."),
            ["quota", "auth", "credentials"]
        );
        assert_eq!(
            unplain_words("A credential-free quotas page."),
            ["credential", "quotas"]
        );
        // A name is written as it is: in backticks or quotes, or a path, file or variable.
        for named in [
            "QuotaDial reads this account's sign-in from ~/.codex/auth.json. It only reads it.",
            "QuotaDial reads the sign-in Codex keeps in auth.json, in ~/.codex.",
            "the macOS Keychain item \"Claude Code-credentials\", then .credentials.json.",
            "the CLAUDE_CODE_OAUTH_TOKEN variable, or `quota` in a command.",
            "QuotaDial only checks that Gemini CLI's sign-in is at ~/.gemini/oauth_creds.json.",
            "C:\\Users\\someone\\.codex\\auth.json, as Windows writes it.",
        ] {
            assert_eq!(unplain_words(named), Vec::<String>::new(), "{named}");
        }
    }

    /// A home folder as a system names one: a root, `/`, or `C:\` as Windows writes it (on macOS
    /// and Linux only a name), then one to three names of letters, digits, `.`, `_`, `-` and spaces
    /// inside, such as `/Users/ann` or `C:\Users\Ann Smith`. None holds `~`.
    fn home_folder() -> impl Strategy<Value = PathBuf> {
        (
            prop_oneof![Just("/"), Just("C:\\")],
            proptest::collection::vec("[A-Za-z0-9_-][A-Za-z0-9 ._-]{0,8}[A-Za-z0-9]", 1..4),
        )
            .prop_map(|(root, names)| {
                std::iter::once(root)
                    .chain(names.iter().map(String::as_str))
                    .collect()
            })
    }

    /// The text `pieces` make, each `None` the home folder `home` as written, so the home folder
    /// comes up often, and beside anything.
    fn pieced(home: &str, pieces: &[Option<String>]) -> String {
        pieces
            .iter()
            .map(|piece| piece.as_deref().unwrap_or(home))
            .collect()
    }

    /// What a sentence can put right after the home folder's text, with whether the home folder is
    /// the home folder on its own there (D-133). On its own: the end of the text, or the marks a
    /// sentence puts after a path (a full stop only as the last) and then the end, a space and a
    /// plain word, or a line break. A longer name that goes on: in the same word, or after those
    /// marks and a space, in the next word, which opens with a bracket or holds a separator after a
    /// name.
    fn after_the_home_folder() -> impl Strategy<Value = (String, bool)> {
        const MARKS: &str = "[:,;)\"'`]{0,2}\\.?";
        const MARKS_WITHOUT_A_STOP: &str = "[:,;)\"'`]{0,2}";
        prop_oneof![
            MARKS.prop_map(|marks| (marks, true)),
            (MARKS, "[a-z]{1,8}", ".{0,10}")
                .prop_map(|(marks, word, more)| (format!("{marks} {word} {more}"), true)),
            (MARKS, ".{0,10}").prop_map(|(marks, more)| (format!("{marks}\n{more}"), true)),
            (MARKS_WITHOUT_A_STOP, "[A-Za-z0-9_-]", "[^ \t\n]{0,6}")
                .prop_map(|(marks, name, more)| (format!("{marks}{name}{more}"), false)),
            (MARKS, "[ \t]{1,3}", "[(\\[]", ".{0,6}").prop_map(|(marks, space, open, more)| (
                format!("{marks}{space}{open}{more}"),
                false
            )),
            (MARKS, "[ \t]{1,3}", "[A-Za-z0-9_-]{1,6}", ".{0,6}").prop_map(
                |(marks, space, name, more)| (format!("{marks}{space}{name}/{more}"), false)
            ),
        ]
    }

    /// The pieces of a text made of what a sentence or a command puts around the home folder and a
    /// longer name beside it, `None` being the home folder's own text: marks, quotes, brackets,
    /// spaces, line breaks and separators, any characters, words, words with a separator in them,
    /// and the shapes a longer name and a quoted word take.
    fn around_the_home_folder() -> impl Strategy<Value = Vec<Option<String>>> {
        proptest::collection::vec(
            proptest::option::weighted(
                0.6,
                prop_oneof![
                    "[ \t\n./\\\\:;,)(\\[\\]'’\"`~a-z]{0,3}",
                    ".{0,3}",
                    "[a-z]{1,4}/[a-z]{0,4}",
                    Just(" (old)".to_owned()),
                    Just("'\\''".to_owned()),
                    Just("''".to_owned()),
                ],
            ),
            0..16,
        )
    }

    // `tilde_in` takes text from outside, such as a server's words in a request's error, so no
    // text and no home folder may make it panic (the manager on #82, D-129). Any string, and short
    // ones of a path's characters, so a home folder of one or two characters, and a separator, a
    // stop, a space, a quote or a bracket beside it, come up often.
    proptest! {
        #[test]
        fn no_text_or_home_folder_makes_writing_the_home_folder_panic(
            home in proptest::option::of(prop_oneof![any::<String>(), "[a-z./\\\\~ '’(]{1,3}"]),
            pieces in proptest::collection::vec(
                proptest::option::of(prop_oneof![any::<String>(), "[ \n./\\\\:a-z~'’(\\[]{0,2}"]),
                0..8,
            ),
        ) {
            let home = home.map(PathBuf::from);
            let written_home = home.as_deref().map(|home| home.display().to_string());
            let text = pieced(written_home.as_deref().unwrap_or_default(), &pieces);

            let written = tilde_in(&text, home.as_deref());

            // A place is only ever shortened, to `~`, or `~`, its separator and the rest as it was.
            prop_assert!(written.len() <= text.len(), "{text:?} became {written:?}");
            prop_assert_eq!(tilde_in(&text, None), text.as_str(), "no home folder");
            let disk = Path::new(std::path::MAIN_SEPARATOR_STR);
            prop_assert_eq!(tilde_in(&text, Some(disk)), text.as_str(), "a whole disk");
        }

        // Written once, text is written (D-129): also what a longer name, a word in quotes, a
        // bracket and a line break make together with the home folder's own text, in any order,
        // which the look-ahead for a longer name reads forward through (D-133).
        #[test]
        fn writing_the_home_folder_again_changes_nothing(
            home in home_folder(),
            pieces in around_the_home_folder(),
        ) {
            let text = pieced(&home.display().to_string(), &pieces);

            let once = tilde_in(&text, Some(&home));

            prop_assert_eq!(tilde_in(&once, Some(&home)), once);
        }

        #[test]
        fn a_place_in_the_home_folder_after_a_space_is_written_as_tilde_writes_it(
            home in home_folder(),
            names in proptest::collection::vec("\\.?[A-Za-z0-9_-][A-Za-z0-9._-]{0,8}", 0..4),
            before in ".{0,24}",
        ) {
            let place = names.iter().fold(home.clone(), |place, name| place.join(name));

            prop_assert_eq!(
                tilde_in(&format!("{before} {}", place.display()), Some(&home)),
                format!("{} {}", tilde_in(&before, Some(&home)), tilde(&place, Some(&home)))
            );
        }

        #[test]
        fn a_place_beside_the_home_folder_or_around_its_name_stays_as_it_is(
            home in home_folder(),
            more in "[A-Za-z0-9_-][A-Za-z0-9._-]{0,3}",
            lead in "[A-Za-z0-9._-]{1,4}",
            names in proptest::collection::vec("[A-Za-z0-9_-]{1,8}", 0..3),
        ) {
            let name = home.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
            let inside = |folder: PathBuf| names.iter().fold(folder, |place, name| place.join(name));
            // Another folder whose name starts as the home folder's does, as `/Users/anna` beside
            // `/Users/ann`, and the home folder's own words inside a longer path, as in
            // `/Volumes/Backup/Users/ann`.
            for place in [
                inside(home.with_file_name(format!("{name}{more}"))),
                inside(PathBuf::from(format!("{lead}{}", home.display()))),
            ] {
                prop_assert_eq!(tilde_in(&naming(&place), Some(&home)), naming(&place));
            }
        }

        // D-133: a place outside the home folder, whose text starts with the home folder's path and
        // goes on with any character but a separator, and then with anything, the home folder's own
        // text and every quote among it, is left as written in both commands: in a word by
        // itself and in the sentences and commands that hold it.
        #[test]
        fn a_place_outside_the_home_folder_is_left_as_written_in_both_commands(
            home in home_folder(),
            first in "[^/\\\\]",
            pieces in proptest::collection::vec(
                proptest::option::of(prop_oneof![
                    "[ \t./\\\\:;,)(\\[\\]'‘’‚‛\"`~a-z]{0,3}",
                    ".{0,3}",
                ]),
                0..6,
            ),
        ) {
            let written = home.display().to_string();
            let place = PathBuf::from(format!("{written}{first}{}", pieced(&written, &pieces)));
            prop_assume!(place.strip_prefix(&home).is_err());

            for word in [
                shell_word(&place, Some(&home)),
                powershell_string(&place, Some(&home)),
            ] {
                for command in [
                    word.clone(),
                    format!("chmod 600 {word}"),
                    format!("CLAUDE_CONFIG_DIR={word} claude"),
                    format!("$env:CLAUDE_CONFIG_DIR = {word}; claude"),
                    format!("run `chmod 600 {word}`."),
                ] {
                    prop_assert_eq!(tilde_in(&command, Some(&home)), command);
                }
            }
        }

        // The same for a place in the home folder, which a POSIX shell gets as `~/` and the rest in
        // single quotes, whatever the rest holds.
        #[test]
        fn a_posix_command_for_a_place_in_the_home_folder_is_left_as_written(
            home in home_folder(),
            first in "[^/\\\\]",
            pieces in proptest::collection::vec(
                proptest::option::of(prop_oneof![
                    "[ \t./\\\\:;,)(\\[\\]'‘’‚‛\"`~a-z]{0,3}",
                    ".{0,3}",
                ]),
                0..6,
            ),
        ) {
            let written = home.display().to_string();
            let place = PathBuf::from(format!(
                "{written}{MAIN_SEPARATOR}{first}{}",
                pieced(&written, &pieces)
            ));
            prop_assume!(place.strip_prefix(&home).is_ok());
            let word = shell_word(&place, Some(&home));

            for command in [
                word.clone(),
                format!("chmod 600 {word}"),
                format!("CLAUDE_CONFIG_DIR={word} claude"),
            ] {
                prop_assert_eq!(tilde_in(&command, Some(&home)), command);
            }
        }

        // D-133: in a sentence, what follows the home folder's text tells the home folder on its
        // own, written `~`, from the start of a longer name, left as written.
        #[test]
        fn what_follows_the_home_folder_tells_it_from_a_longer_name(
            home in home_folder(),
            before in "([a-z]{1,6} ){0,4}",
            (after, alone) in after_the_home_folder(),
        ) {
            let written = home.display().to_string();
            let text = format!("{before}{written}{after}");
            let expected = if alone { format!("{before}~{after}") } else { text.clone() };

            prop_assert_eq!(tilde_in(&text, Some(&home)), expected);
        }

        // D-133: a word in single quotes is one place, so it is the home folder or a place in it
        // where its text says so, and anything else in or around it is left as written.
        #[test]
        fn a_word_in_single_quotes_is_read_whole_before_it_is_taken_for_the_home_folder(
            home in home_folder(),
            more in "[^/\\\\'][^']{0,8}",
            inside in "[^']{0,8}",
            ahead in "[A-Za-z0-9][^']{0,5}",
            before in "([a-z]{1,4} ){0,2}",
            after in "( [a-z]{1,4}){0,2}",
        ) {
            let written = home.display().to_string();
            let said = |word: &str| format!("{before}'{word}'{after}");

            // The home folder on its own, and a place in it, as without the quotes.
            prop_assert_eq!(tilde_in(&said(&written), Some(&home)), said("~"));
            prop_assert_eq!(
                tilde_in(&said(&format!("{written}/{inside}")), Some(&home)),
                said(&format!("~/{inside}"))
            );
            // A name that goes on from it, and a word that starts anywhere else, whatever else is
            // in it.
            for word in [format!("{written}{more}"), format!("{ahead}{written}{inside}")] {
                prop_assert_eq!(tilde_in(&said(&word), Some(&home)), said(&word));
            }
        }
    }

    /// A user's name as a system names one, of 3 to 11 characters: letters, digits, `.`, `_`, `-`
    /// and spaces inside, such as `ann` or `Ann Smith`.
    fn user_name() -> impl Strategy<Value = String> {
        "[A-Za-z0-9_-][A-Za-z0-9 ._-]{1,8}[A-Za-z0-9]"
    }

    /// The pieces of a text made of what a sentence, a command or a place puts around a user's
    /// name, `None` being the name itself: marks, quotes, brackets, spaces, line breaks and
    /// separators, any characters, words with a separator in them, roots and drives, web
    /// addresses, the home folder's own `~` and `$HOME`, a `[user]` already there, and the shapes a
    /// quoted word takes.
    fn around_the_name() -> impl Strategy<Value = Vec<Option<String>>> {
        proptest::collection::vec(
            proptest::option::weighted(
                0.6,
                prop_oneof![
                    "[ \t\n./\\\\:;,)(\\[\\]'’\"`~$%=a-z]{0,3}",
                    ".{0,3}",
                    "[a-z]{1,4}/[a-z]{0,4}",
                    proptest::sample::select(vec![
                        "C:\\",
                        "C:/",
                        "//",
                        "http://",
                        "file:///",
                        "[user]",
                        "'\\''",
                        "''",
                        "\"$HOME\\",
                        "~/'",
                    ])
                    .prop_map(String::from),
                ],
            ),
            0..16,
        )
    }

    // `hide_user_in` takes text from outside, such as a server's words and a place the system
    // names, so no text and no home folder may make it panic (D-134), the same as `tilde_in`
    // (D-129). Any string, and short ones of a path's characters, so a short home folder and a
    // separator, a quote or a bracket beside the name come up often.
    proptest! {
        #[test]
        fn no_text_or_home_folder_makes_hiding_the_user_panic(
            home in proptest::option::of(
                prop_oneof![any::<String>(), "[a-z./\\\\~ '’(\\[\\]x]{1,5}"]
            ),
            pieces in proptest::collection::vec(
                proptest::option::of(
                    prop_oneof![any::<String>(), "[ \n./\\\\:a-z~'’(\\[\\]$%\"`]{0,3}"]
                ),
                0..8,
            ),
        ) {
            let home = home.map(PathBuf::from);
            let name = home
                .as_deref()
                .and_then(Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default();
            let text = pieced(name, &pieces);

            let _ = hide_user_in(&text, home.as_deref());

            prop_assert_eq!(hide_user_in(&text, None), text.as_str(), "no home folder");
            let disk = Path::new(MAIN_SEPARATOR_STR);
            prop_assert_eq!(hide_user_in(&text, Some(disk)), text.as_str(), "a whole disk");
        }

        // D-134: hidden once, a text is what it was hidden as, for a name with a space in it, for
        // one that is a word of `[user]`, and for any name at all, among every shape of the text
        // around it: the places and the quoted words are read the same before and after.
        #[test]
        fn hiding_the_user_again_changes_nothing(
            name in prop_oneof![user_name(), Just("user".to_owned()), "\\PC{3,10}"],
            pieces in around_the_name(),
        ) {
            let home = PathBuf::from("/Users").join(&name);
            let text = pieced(&name, &pieces);

            let once = hide_user_in(&text, Some(&home));

            prop_assert_eq!(hide_user_in(&once, Some(&home)), once);
        }

        // Read back as the name, a text that has the name hidden is the text it was: nothing but
        // the name is ever hidden.
        #[test]
        fn hiding_the_user_hides_nothing_but_the_name(
            name in user_name(),
            pieces in around_the_name(),
        ) {
            let home = path(&["/", "Users", &name]);
            let text = pieced(&name, &pieces);

            let hidden = hide_user_in(&text, Some(&home));

            prop_assert_eq!(hidden.replace(USER, &name), text.replace(USER, &name));
        }

        // D-134, the rule: the name never survives as a component of a place outside the home
        // folder, or as a part of one set off by `-` or `.`, whatever the root, the separator and
        // the words around the place, in a sentence, in quotes, after `=` and in brackets. The
        // fixed parts are upper case, and the name lower case, so nothing but the name can be hidden,
        // and every expected place is made from its parts.
        #[test]
        fn the_users_name_never_survives_in_a_place_outside_the_home_folder(
            name in "[a-z0-9_-][a-z0-9 ._-]{1,8}[a-z0-9]",
            root in prop_oneof![
                Just("/"),
                Just("C:/"),
                Just("C:\\"),
                Just("//SERVER/"),
                Just("\\\\SERVER\\"),
            ],
            separator in prop_oneof![Just("/"), Just("\\")],
            before in proptest::collection::vec("[A-Z]{1,5}", 0..3),
            (prefix, suffix) in prop_oneof![
                Just(("", "")),
                Just(("", "-OLD")),
                Just(("-Users-", "--WORK")),
                Just(("OLD.", "")),
            ],
            after in proptest::collection::vec("[A-Z]{1,5}", 0..3),
            lead in "([a-z]{1,6} ){0,2}",
            tail in "( [a-z]{1,6}){0,2}",
            form in 0usize..5,
        ) {
            let home = path(&["/", "Users", &name]);
            let place = |who: &str| {
                let named = format!("{prefix}{who}{suffix}");
                let parts: Vec<&str> = before
                    .iter()
                    .map(String::as_str)
                    .chain(std::iter::once(named.as_str()))
                    .chain(after.iter().map(String::as_str))
                    .collect();
                format!("{root}{}", parts.join(separator))
            };
            // The name is a word of the sentence around the place too, before and after it, where it
            // stays: only the place is read for it.
            let (lead, tail) = (format!("{name} {lead}"), format!("{tail} {name}"));
            let sentence = |place: &str| match form {
                0 => format!("{lead}{place}"),
                1 => format!("{lead}{place}: it is not valid{tail}"),
                2 => format!("{lead}'{place}'{tail}"),
                3 => format!("CLAUDE_CONFIG_DIR={place}{tail}"),
                _ => format!("{lead}({place}).{tail}"),
            };

            prop_assert_eq!(
                hide_user_in(&sentence(&place(&name)), Some(&home)),
                sentence(&place(USER))
            );
        }

        // D-134, the commands: for any user name, the commands for a place in the home folder, a
        // POSIX shell's and PowerShell's, whatever the place holds, come through doctor's whole
        // pass as they were written, and the name is among their words and in the place.
        #[test]
        fn the_commands_for_a_place_in_the_home_folder_come_through_whole_for_any_user(
            name in user_name(),
            root in prop_oneof![Just("/"), Just("C:\\")],
            rest in proptest::collection::vec(
                proptest::option::of("\\.?[A-Za-z0-9_-][A-Za-z0-9 ._'\"$`-]{0,8}"),
                0..4,
            ),
        ) {
            let home = path(&[root, "Users", &name]);
            let place = rest.iter().fold(home.clone(), |place, part| {
                place.join(part.as_deref().unwrap_or(&name))
            });
            let (posix, shell) = (
                shell_word(&place, Some(&home)),
                powershell_string(&place, Some(&home)),
            );

            for command in [
                posix.clone(),
                format!("chmod 600 {posix}"),
                format!("CLAUDE_CONFIG_DIR={posix} claude"),
                format!("run `CLAUDE_CONFIG_DIR={posix} claude` to sign in again."),
                format!("$env:CLAUDE_CONFIG_DIR = {shell}; claude"),
                format!("run `$env:CLAUDE_CONFIG_DIR = {shell}; claude` to sign in again."),
            ] {
                prop_assert_eq!(doctor_pass(&command, &home), command.clone());
            }
        }
    }
}
