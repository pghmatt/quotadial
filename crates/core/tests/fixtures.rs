//! Every committed provider fixture is safe to publish: no token and no real email address (#4).
//! Unredacted captures live in `fixtures/<provider>/raw/`, which is gitignored and skipped here.

use std::fs;
use std::path::{Path, PathBuf};

fn fixture_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("reads the fixtures directory") {
        let path = entry.expect("reads an entry").path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "raw") {
                fixture_files(&path, files);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json" || extension == "jsonl")
        {
            files.push(path);
        }
    }
}

/// Every email-like domain in `text`: what follows an `@`.
fn email_domains(text: &str) -> Vec<&str> {
    text.match_indices('@')
        .map(|(at, _)| {
            let domain = &text[at + 1..];
            let end = domain
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-'))
                .unwrap_or(domain.len());
            domain[..end].trim_end_matches('.')
        })
        .collect()
}

#[test]
fn committed_fixtures_hold_no_token_or_real_email_address() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut files = Vec::new();
    fixture_files(&root, &mut files);
    assert!(files.len() >= 8, "found only {} fixtures", files.len());

    for path in files {
        let text = fs::read_to_string(&path).expect("fixtures are UTF-8");
        let name = path.display();
        assert!(!text.contains("eyJ"), "{name} holds a JWT-like string");
        let lower = text.to_ascii_lowercase();
        assert!(
            !lower.contains("sk-"),
            "{name} holds an API-key-like string"
        );
        assert!(!lower.contains("bearer"), "{name} mentions a bearer token");
        for domain in email_domains(&text) {
            assert!(
                domain == "example.com" || domain.ends_with(".example.com"),
                "{name} holds an email address at {domain}"
            );
        }
    }
}

#[test]
fn the_email_check_finds_domains() {
    assert_eq!(
        email_domains("a user@example.com, b@corp.example.org."),
        ["example.com", "corp.example.org"]
    );
    assert!(email_domains("no address here").is_empty());
}
