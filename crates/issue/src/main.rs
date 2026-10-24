//! `quotadial-issue`: makes QuotaDial's licence signing key and signs licence keys with it (D-099).
//! It is for whoever issues keys, by hand or to check what an issuing service makes. It is not part
//! of QuotaDial, and no release carries it.
//!
//! The signing key is kept as a PKCS #8 file, readable only by its owner, which OpenSSL and
//! WebCrypto read too. It is never in this repository and never printed.

// Printing results is this tool's job.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use quotadial_core::licence::{self, ISSUER, KeyError, Licence, Plan, QUOTADIAL, TEST_SEED};
use zeroize::Zeroizing;

/// Make QuotaDial's licence signing key, or sign a licence key with it.
#[derive(Debug, Parser)]
#[command(name = "quotadial-issue", version)]
struct Cli {
    /// The signing key's file: `quotadial-issue/signing-key.pem` in the system's config folder
    /// unless given.
    #[arg(long, global = true)]
    key_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Make a new signing key, in a file readable only by you, and print its public half. A key
    /// that is there already is never replaced. QuotaDial has two: the root key, offline, and the
    /// issuing key the website signs with (`--key-file .../issuing-key.pem`).
    Keygen,
    /// Print the signing key's public half, as base64 and as the bytes crates/core keeps.
    Public,
    /// Print a licence key for QuotaDial Pro.
    Sign {
        /// The licence's id, such as the order it came from: up to 64 letters, digits and
        /// punctuation, with no spaces.
        #[arg(long)]
        id: String,
        /// The day it was issued: today unless given.
        #[arg(long)]
        issued: Option<NaiveDate>,
        /// Its last day, for a licence that ends.
        #[arg(long)]
        until: Option<NaiveDate>,
        /// Sign with the test key, which only debug builds trust, instead of the signing key.
        #[arg(long)]
        test: bool,
    },
    /// Say what a licence key holds, and which of QuotaDial's keys, or the test key, signed it.
    Check {
        /// The key.
        key: String,
    },
}

/// The DER before an Ed25519 private key's 32 bytes in a PKCS #8 file (RFC 8410).
const PKCS8: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
];

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(path) = cli.key_file.or_else(default_key_file) else {
        eprintln!("quotadial-issue: this system names no config folder; give --key-file.");
        return ExitCode::FAILURE;
    };
    let said = match cli.command {
        Command::Keygen => keygen(&path).map(|public| {
            format!(
                "Made a signing key in {}, readable only by you. Keep a copy somewhere safe, such \
                 as a password manager: without it no new licence keys can be made.\n{}",
                path.display(),
                describe(&public)
            )
        }),
        Command::Public => read_seed(&path).map(|seed| describe(&licence::public_key(&seed))),
        Command::Sign {
            id,
            issued,
            until,
            test,
        } => {
            let seed = if test {
                Ok(Zeroizing::new(TEST_SEED))
            } else {
                read_seed(&path)
            };
            seed.and_then(|seed| sign(&seed, id, issued.unwrap_or_else(licence::today), until))
        }
        Command::Check { key } => Ok(check(&key)),
    };
    match said {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("quotadial-issue: {error}");
            ExitCode::FAILURE
        }
    }
}

fn default_key_file() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("quotadial-issue").join("signing-key.pem"))
}

/// Makes a signing key at `path`, which must not exist, and returns its public half.
fn keygen(path: &Path) -> Result<[u8; 32], String> {
    let mut seed = Zeroizing::new([0_u8; 32]);
    getrandom::fill(&mut *seed).map_err(|error| format!("no random source: {error}"))?;
    write_new_private(path, pem(&seed).as_bytes()).map_err(|error| match error.kind() {
        io::ErrorKind::AlreadyExists => format!(
            "{} holds a signing key already, and it is never replaced.",
            path.display()
        ),
        _ => format!("{} could not be written: {error}", path.display()),
    })?;
    Ok(licence::public_key(&seed))
}

/// The licence key for `id`, signed with `seed`, refusing a signing key that is not one QuotaDial
/// carries (the root or the issuing key) or the test key, since keys it signed would not check out.
fn sign(
    seed: &[u8; 32],
    id: String,
    issued: NaiveDate,
    until: Option<NaiveDate>,
) -> Result<String, String> {
    let public = licence::public_key(seed);
    if ![QUOTADIAL, ISSUER, licence::public_key(&TEST_SEED)].contains(&public) {
        return Err(
            "this signing key is not the one QuotaDial carries, so keys it signed would not check \
             out. `quotadial-issue public` prints the bytes for crates/core/src/licence.rs."
                .into(),
        );
    }
    if id.is_empty() || id.len() > 64 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err("--id must be 1 to 64 letters, digits or punctuation, with no spaces.".into());
    }
    if until.is_some_and(|until| until < issued) {
        return Err("--until must not come before the day the licence is issued.".into());
    }
    Ok(Licence {
        id,
        plan: Plan::Pro,
        issued,
        until,
    }
    .sign(seed))
}

/// What `key` holds and who signed it, or why it holds nothing.
fn check(key: &str) -> String {
    let signer = [
        ("QuotaDial's root key", QUOTADIAL),
        ("QuotaDial's issuing key", ISSUER),
        (
            "the test key, which only debug builds trust",
            licence::public_key(&TEST_SEED),
        ),
    ];
    let mut why = KeyError::DoesNotCheckOut;
    for (name, public) in signer {
        match Licence::verify_with(key, &[public]) {
            Ok(licence) => {
                let until = licence.until.map_or_else(
                    || "it does not end".to_owned(),
                    |day| format!("until {day}"),
                );
                return format!(
                    "Signed with {name}: licence {}, QuotaDial {}, issued {}, {until}.",
                    licence.id,
                    licence.plan.name(),
                    licence.issued
                );
            }
            Err(error) => why = error,
        }
    }
    why.to_string()
}

/// The public half, as base64 and as Rust bytes.
fn describe(public: &[u8; 32]) -> String {
    let rows: Vec<String> = public
        .chunks(16)
        .map(|row| {
            let bytes: Vec<String> = row.iter().map(|byte| format!("{byte:#04x}")).collect();
            format!("    {},", bytes.join(", "))
        })
        .collect();
    let carried = if *public == QUOTADIAL {
        "This is QuotaDial's root key, which QuotaDial carries as QUOTADIAL."
    } else if *public == ISSUER {
        "This is QuotaDial's issuing key, which QuotaDial carries as ISSUER."
    } else {
        "QuotaDial carries neither: put these bytes in crates/core/src/licence.rs, as QUOTADIAL \
         for the root key or ISSUER for the issuing key."
    };
    format!(
        "Public half (base64): {}\nAs bytes: [\n{}\n]\n{carried}",
        STANDARD.encode(public),
        rows.join("\n")
    )
}

/// The seed as a PKCS #8 PEM file.
fn pem(seed: &[u8; 32]) -> Zeroizing<String> {
    let mut der = Zeroizing::new(Vec::with_capacity(PKCS8.len() + seed.len()));
    der.extend_from_slice(&PKCS8);
    der.extend_from_slice(seed);
    Zeroizing::new(format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        STANDARD.encode(&*der)
    ))
}

/// The seed in a PKCS #8 PEM file's text, if it holds an Ed25519 key.
fn seed_of(text: &str) -> Option<Zeroizing<[u8; 32]>> {
    let body: Zeroizing<String> = Zeroizing::new(
        text.lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("-----"))
            .collect(),
    );
    let der = Zeroizing::new(STANDARD.decode(body.as_bytes()).ok()?);
    let seed: [u8; 32] = der.strip_prefix(&PKCS8)?.try_into().ok()?;
    Some(Zeroizing::new(seed))
}

fn read_seed(path: &Path) -> Result<Zeroizing<[u8; 32]>, String> {
    let text = Zeroizing::new(fs::read_to_string(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => format!(
            "there is no signing key at {}; `quotadial-issue keygen` makes one, or give --key-file.",
            path.display()
        ),
        _ => format!("{} could not be read: {error}", path.display()),
    })?);
    seed_of(&text).ok_or_else(|| format!("{} is not an Ed25519 signing key.", path.display()))
}

/// Writes `bytes` to `path`, which must not exist, readable only by the user.
fn write_new_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(folder)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(text: &str) -> NaiveDate {
        text.parse().expect("a date")
    }

    #[test]
    fn a_signing_key_is_made_once_privately_and_read_back() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("issuer/signing-key.pem");

        let public = keygen(&path).expect("makes a key");
        let seed = read_seed(&path).expect("reads it");

        assert_eq!(licence::public_key(&seed), public);
        let text = fs::read_to_string(&path).expect("reads the file");
        assert!(text.starts_with("-----BEGIN PRIVATE KEY-----\n"), "{text}");
        assert!(
            keygen(&path)
                .expect_err("never replaced")
                .contains("never replaced")
        );
        assert_eq!(
            licence::public_key(&read_seed(&path).expect("reads")),
            public
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode =
                |path: &Path| fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
            assert_eq!(mode(&path), 0o600);
            assert_eq!(mode(path.parent().expect("a folder")), 0o700);
        }
    }

    #[test]
    fn the_file_is_the_pkcs8_that_openssl_and_webcrypto_read() {
        // RFC 8410's example private key, section 10.3.
        let rfc = "-----BEGIN PRIVATE KEY-----\n\
                   MC4CAQAwBQYDK2VwBCIEINTuctv5E1hK1bbY8fdp+K06/nwoy/HU++CXqI9EdVhC\n\
                   -----END PRIVATE KEY-----\n";
        let seed = seed_of(rfc).expect("an Ed25519 key");

        assert_eq!(pem(&seed).as_str(), rfc);
        assert!(seed_of("-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----").is_none());
        assert!(seed_of("not a key").is_none());
    }

    #[test]
    fn a_test_key_signs_what_a_debug_build_takes() {
        let key = sign(&TEST_SEED, "order-1".into(), day("2026-10-24"), None).expect("signs");

        assert_eq!(
            Licence::verify_with(&key, &[licence::public_key(&TEST_SEED)]).map(|l| l.id),
            Ok("order-1".into())
        );
        assert_eq!(
            check(&key),
            "Signed with the test key, which only debug builds trust: licence order-1, QuotaDial \
             pro, issued 2026-10-24, it does not end."
        );
    }

    #[test]
    fn only_quotadials_key_or_the_test_key_signs() {
        let refused = sign(&[9; 32], "order-1".into(), day("2026-10-24"), None);

        assert!(
            refused
                .expect_err("refused")
                .contains("not the one QuotaDial carries")
        );
        assert!(describe(&ISSUER).contains("QuotaDial's issuing key"));
        assert!(describe(&QUOTADIAL).contains("QuotaDial's root key"));
    }

    #[test]
    fn a_licence_needs_a_plain_id_and_a_last_day_after_its_first() {
        for id in ["", "has space", "x".repeat(65).as_str(), "naïve"] {
            assert!(
                sign(&TEST_SEED, id.into(), day("2026-10-24"), None).is_err(),
                "{id:?}"
            );
        }
        assert!(
            sign(
                &TEST_SEED,
                "o-1".into(),
                day("2026-10-24"),
                Some(day("2026-10-23"))
            )
            .is_err()
        );
        assert!(
            sign(
                &TEST_SEED,
                "o-1".into(),
                day("2026-10-24"),
                Some(day("2026-10-24"))
            )
            .is_ok()
        );
    }

    #[test]
    fn check_says_why_a_key_holds_nothing() {
        assert_eq!(check("hello"), KeyError::NotAKey.to_string());
        let forged = Licence {
            id: "o-1".into(),
            plan: Plan::Pro,
            issued: day("2026-10-24"),
            until: None,
        }
        .sign(&[9; 32]);
        assert_eq!(check(&forged), KeyError::DoesNotCheckOut.to_string());
    }
}
