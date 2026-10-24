//! Licence keys for QuotaDial Pro (M5, D-099). A key is a licence signed with QuotaDial's Ed25519
//! key: its id, its plan, the day it was issued and, for a licence that ends, its last day.
//! QuotaDial checks a key on this machine against the public keys it carries, so a key never needs
//! a network call, and a copy with no key is QuotaDial Free.
//!
//! A key reads `QD1.` + the licence as base64url JSON + `.` + the base64url Ed25519 signature of
//! everything before the second dot. `docs/LICENSING.md` describes it for whatever issues keys.

use std::fmt;

use base64::Engine as _;
use base64::alphabet::URL_SAFE;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use chrono::{Local, NaiveDate, Utc};
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::Tier;

/// What every key starts with: QuotaDial's first key format.
pub const PREFIX: &str = "QD1";

/// The public half of QuotaDial's root key. Its private half stays offline, on Matt's Mac and in
/// his password manager, and signs only by hand (`docs/LICENSING.md`).
pub const QUOTADIAL: [u8; 32] = [
    0x08, 0x30, 0x07, 0x67, 0xee, 0xdd, 0xbd, 0xfc, 0xf2, 0x61, 0xb5, 0xa8, 0x74, 0xb5, 0xe4, 0xac,
    0x01, 0x67, 0xb1, 0x56, 0x14, 0x19, 0x7c, 0xf5, 0x80, 0xe2, 0x93, 0x9a, 0x69, 0xf9, 0x69, 0x91,
];

/// The public half of QuotaDial's issuing key, which the website's key page signs with (#54). Its
/// private half lives online, as the page's secret, so it is the one most likely to leak: a release
/// that drops it, with a new issuing key beside the root, ends what a leaked one signed.
pub const ISSUER: [u8; 32] = [
    0x92, 0x71, 0x03, 0x90, 0x9c, 0x8d, 0xc7, 0x29, 0x93, 0x1e, 0x1b, 0xdc, 0xe9, 0xcf, 0x5a, 0x3f,
    0x58, 0x75, 0xa9, 0x2c, 0x5a, 0xc3, 0xf9, 0xd1, 0x1b, 0x27, 0xa6, 0x3b, 0xbe, 0x3f, 0xa1, 0x38,
];

/// The private half of the test key, public on purpose: tests sign licences with it, and only a
/// debug build trusts it, so no release ever takes a key it signed.
pub const TEST_SEED: [u8; 32] = *b"QuotaDial's licence test key 001";

/// Base64url, with or without padding: keys are written without it, and a reader may add it.
const BASE64: GeneralPurpose = GeneralPurpose::new(
    &URL_SAFE,
    GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// A licence, as its key holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Licence {
    /// The licence's id, from whatever issued it.
    pub id: String,
    /// What the licence is for.
    pub plan: Plan,
    /// The day it was issued.
    pub issued: NaiveDate,
    /// Its last day, for a licence that ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<NaiveDate>,
}

/// What a licence is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    /// QuotaDial Pro.
    Pro,
}

/// Why a key gives nothing. Each says what to do, as said when the key is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// The text is not a QuotaDial key at all.
    NotAKey,
    /// The text is the key from a Lemon Squeezy receipt, which the key page turns into
    /// QuotaDial's.
    OrderKey,
    /// The signature does not match: part of the key is missing or changed, or QuotaDial did not
    /// issue it.
    DoesNotCheckOut,
    /// The key is for a plan, or in a format, that this version of QuotaDial does not know.
    Newer,
}

impl fmt::Display for KeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotAKey => {
                "That isn't a QuotaDial licence key, which starts with QD1. Copy the whole key from \
                 the email it came in."
            }
            Self::OrderKey => {
                "That is your order's key from the receipt. Its \"Get your QuotaDial key\" link \
                 gives the QuotaDial licence key, which starts with QD1."
            }
            Self::DoesNotCheckOut => {
                "That licence key doesn't check out, so part of it may be missing. Copy the whole \
                 key from the email it came in and try again."
            }
            Self::Newer => {
                "That licence key is for a newer QuotaDial. Update QuotaDial, then add it."
            }
        })
    }
}

impl std::error::Error for KeyError {}

/// The fields of a key's licence as written, before its plan is known.
#[derive(Deserialize)]
struct Written {
    id: String,
    plan: String,
    issued: NaiveDate,
    #[serde(default)]
    until: Option<NaiveDate>,
}

impl Licence {
    /// The licence `key` holds, when QuotaDial's key signed it. Spaces, line breaks and invisible
    /// characters are left out first, since a key is often copied from an email that wrapped it.
    ///
    /// # Errors
    ///
    /// A [`KeyError`] saying why the key gives nothing.
    pub fn verify(key: &str) -> Result<Self, KeyError> {
        Self::verify_with(key, &trusted())
    }

    /// [`Licence::verify`] against the public keys `trusted`, such as a key `quotadial-issue`
    /// made.
    ///
    /// # Errors
    ///
    /// A [`KeyError`] saying why the key gives nothing.
    pub fn verify_with(key: &str, trusted: &[[u8; 32]]) -> Result<Self, KeyError> {
        let key = clean(key);
        let mut parts = key.split('.');
        let format = parts.next().unwrap_or_default();
        if order_key(&key) {
            return Err(KeyError::OrderKey);
        }
        if !format.eq_ignore_ascii_case(PREFIX) {
            let bytes = format.as_bytes();
            let later = bytes.len() > 2
                && bytes[..2].eq_ignore_ascii_case(b"QD")
                && bytes[2..].iter().all(u8::is_ascii_digit);
            return Err(if later {
                KeyError::Newer
            } else {
                KeyError::NotAKey
            });
        }
        let (Some(body), Some(signature), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(KeyError::DoesNotCheckOut);
        };
        let signature = BASE64
            .decode(signature)
            .ok()
            .and_then(|bytes| <[u8; 64]>::try_from(bytes.as_slice()).ok())
            .map(|bytes| Signature::from_bytes(&bytes))
            .ok_or(KeyError::DoesNotCheckOut)?;
        let signed = format!("{PREFIX}.{body}");
        let checks_out = trusted.iter().any(|public| {
            VerifyingKey::from_bytes(public)
                .is_ok_and(|public| public.verify_strict(signed.as_bytes(), &signature).is_ok())
        });
        if !checks_out {
            return Err(KeyError::DoesNotCheckOut);
        }
        let written: Written = BASE64
            .decode(body)
            .ok()
            .and_then(|json| serde_json::from_slice(&json).ok())
            .ok_or(KeyError::DoesNotCheckOut)?;
        let plan = match written.plan.as_str() {
            "pro" => Plan::Pro,
            _ => return Err(KeyError::Newer),
        };
        Ok(Self {
            id: written.id,
            plan,
            issued: written.issued,
            until: written.until,
        })
    }

    /// Whether the licence holds on `today`: through its last day, or always when it does not end.
    #[must_use]
    pub fn holds_on(&self, today: NaiveDate) -> bool {
        self.until.is_none_or(|until| today <= until)
    }

    /// The key for this licence, signed with `seed`, the private half of a signing key:
    /// `quotadial-issue` signs with QuotaDial's, and tests with [`TEST_SEED`].
    #[must_use]
    pub fn sign(&self, seed: &[u8; 32]) -> String {
        let mut fields = Map::new();
        fields.insert("id".into(), Value::from(self.id.as_str()));
        fields.insert("plan".into(), Value::from(self.plan.name()));
        fields.insert("issued".into(), Value::from(self.issued.to_string()));
        if let Some(until) = self.until {
            fields.insert("until".into(), Value::from(until.to_string()));
        }
        let body = BASE64.encode(Value::Object(fields).to_string());
        let signed = format!("{PREFIX}.{body}");
        let signature = SigningKey::from_bytes(seed).sign(signed.as_bytes());
        format!("{signed}.{}", BASE64.encode(signature.to_bytes()))
    }
}

impl Plan {
    /// The plan's name in a key, such as `pro`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pro => "pro",
        }
    }
}

/// The public half of the signing key whose private half is `seed`.
#[must_use]
pub fn public_key(seed: &[u8; 32]) -> [u8; 32] {
    SigningKey::from_bytes(seed).verifying_key().to_bytes()
}

/// Whether `text` holds all of a key's parts, with a whole signature, though not whether it checks
/// out: for reading a key pasted over several lines until the last one comes.
#[must_use]
pub fn whole(text: &str) -> bool {
    let text = clean(text);
    let parts: Vec<&str> = text.split('.').collect();
    matches!(parts.as_slice(), [_, body, signature]
        if !body.is_empty()
            && BASE64.decode(signature).is_ok_and(|bytes| bytes.len() == 64))
}

/// `key` as it is kept: on one line, without what copying it added, starting `QD1.`.
#[must_use]
pub fn canonical(key: &str) -> String {
    let key = clean(key);
    match key.get(..PREFIX.len()) {
        Some(format) if format.eq_ignore_ascii_case(PREFIX) => {
            format!("{PREFIX}{}", &key[PREFIX.len()..])
        }
        _ => key,
    }
}

/// Today, for a licence's last day: the earlier of the local and the UTC date, so a licence holds
/// through its last day wherever its user is.
#[must_use]
pub fn today() -> NaiveDate {
    Local::now().date_naive().min(Utc::now().date_naive())
}

/// What a copy of QuotaDial is, from the licence key it keeps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Standing {
    /// QuotaDial Free or QuotaDial Pro.
    pub tier: Tier,
    /// The licence its key holds, even one that has ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence: Option<Licence>,
    /// One sentence saying which, and why a key gives nothing.
    pub text: String,
}

impl Standing {
    /// What a copy keeping `key` is on `today`. `None` is a copy with no key.
    #[must_use]
    pub fn of(key: Option<&str>, today: NaiveDate) -> Self {
        let free = |text: &str| Self {
            tier: Tier::Free,
            licence: None,
            text: text.to_owned(),
        };
        let Some(key) = key else {
            return free("QuotaDial Free, with no licence key.");
        };
        match Licence::verify(key) {
            Ok(licence) => match licence.until {
                Some(until) if !licence.holds_on(today) => Self {
                    tier: Tier::Free,
                    text: format!(
                        "QuotaDial Free: licence {} ended on {until}. Renewing it at {} gives a \
                         new key.",
                        licence.id,
                        crate::PRO_URL
                    ),
                    licence: Some(licence),
                },
                Some(until) => Self {
                    tier: Tier::Pro,
                    text: format!("QuotaDial Pro until {until}, licence {}.", licence.id),
                    licence: Some(licence),
                },
                None => Self {
                    tier: Tier::Pro,
                    text: format!("QuotaDial Pro, licence {}.", licence.id),
                    licence: Some(licence),
                },
            },
            Err(KeyError::Newer) => free(
                "QuotaDial Free: its licence key is for a newer QuotaDial. Update QuotaDial to use \
                 it.",
            ),
            Err(KeyError::NotAKey | KeyError::OrderKey | KeyError::DoesNotCheckOut) => free(
                "QuotaDial Free: its licence key doesn't check out. Copy the key again from the \
                 email it came in, all of it, and add it again.",
            ),
        }
    }
}

impl Standing {
    /// What the copy is without the licence's id, for text that may be shared, such as the
    /// diagnostics Send feedback attaches.
    #[must_use]
    pub fn without_id(&self) -> String {
        match (&self.licence, self.tier) {
            (Some(licence), Tier::Pro) => match licence.until {
                Some(until) => format!("QuotaDial Pro until {until}"),
                None => "QuotaDial Pro, with a licence that does not end".to_owned(),
            },
            (
                Some(Licence {
                    until: Some(until), ..
                }),
                Tier::Free,
            ) => {
                format!("QuotaDial Free: the licence ended on {until}")
            }
            // With no licence, the sentence names none.
            _ => self.text.clone(),
        }
    }
}

/// Whether `key` has the shape of a Lemon Squeezy licence key: a UUID, as 8, 4, 4, 4 and 12 hex
/// digits joined by dashes.
fn order_key(key: &str) -> bool {
    let groups: Vec<&str> = key.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(group, length)| {
            group.len() == length && group.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

/// The keys a licence may be signed with: QuotaDial's root and issuing keys, and in a debug build
/// the test key.
fn trusted() -> Vec<[u8; 32]> {
    let mut keys = vec![QUOTADIAL, ISSUER];
    if cfg!(debug_assertions) {
        keys.push(public_key(&TEST_SEED));
    }
    keys
}

/// `key` without what copying it from an email adds: spaces, line breaks, and invisible characters
/// such as zero-width spaces and soft hyphens.
fn clean(key: &str) -> String {
    key.chars()
        .filter(|c| {
            !c.is_whitespace()
                && !matches!(
                    c,
                    '\u{ad}' | '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}'
                )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn day(text: &str) -> NaiveDate {
        text.parse().expect("a date")
    }

    fn licence(until: Option<&str>) -> Licence {
        Licence {
            id: "ls-1234".into(),
            plan: Plan::Pro,
            issued: day("2026-10-24"),
            until: until.map(day),
        }
    }

    fn test_key() -> [u8; 32] {
        public_key(&TEST_SEED)
    }

    #[test]
    fn a_signed_licence_is_read_back() {
        for written in [licence(None), licence(Some("2027-10-24"))] {
            let key = written.sign(&TEST_SEED);

            assert!(key.starts_with("QD1."), "{key}");
            assert_eq!(Licence::verify_with(&key, &[test_key()]), Ok(written));
        }
    }

    #[test]
    fn a_key_survives_being_wrapped_and_pasted() {
        let key = licence(Some("2027-10-24")).sign(&TEST_SEED);
        let (start, end) = key.split_at(40);
        let wrapped = [
            format!("  {start}\n{end}\n"),
            format!("{start}\r\n  {end}"),
            format!("{start}\u{a0}{end}"),
            format!("{start}\u{200b}{end}\u{feff}"),
            format!("{start}\u{ad}{end}"),
            format!("qd1{}", &key[3..]),
        ];

        for pasted in wrapped {
            assert_eq!(
                Licence::verify_with(&pasted, &[test_key()]),
                Ok(licence(Some("2027-10-24"))),
                "{pasted:?}"
            );
        }
    }

    #[test]
    fn a_changed_or_cut_key_does_not_check_out() {
        let key = licence(None).sign(&TEST_SEED);
        let swap = |at: usize| {
            let mut changed = key.clone().into_bytes();
            changed[at] = if changed[at] == b'A' { b'B' } else { b'A' };
            String::from_utf8(changed).expect("ascii")
        };
        let changed = [
            swap(6),
            swap(key.len() - 2),
            key[..key.len() - 1].to_owned(),
            key[..key.rfind('.').expect("a dot")].to_owned(),
            format!("{key}.more"),
            "QD1.".to_owned(),
            "QD1".to_owned(),
        ];

        for key in changed {
            assert_eq!(
                Licence::verify_with(&key, &[test_key()]),
                Err(KeyError::DoesNotCheckOut),
                "{key}"
            );
        }
    }

    #[test]
    fn a_key_signed_with_any_other_key_does_not_check_out() {
        let forged = licence(None).sign(&[7; 32]);

        assert_eq!(
            Licence::verify_with(&forged, &[test_key()]),
            Err(KeyError::DoesNotCheckOut)
        );
        assert_eq!(Licence::verify(&forged), Err(KeyError::DoesNotCheckOut));
    }

    #[test]
    fn what_is_not_a_key_says_so_and_a_later_format_asks_for_an_update() {
        for text in [
            "",
            "   ",
            "hello",
            "80e15db5-c796-436b-850c-8f9c98a48ab",
            "QD.abc.def",
            "QDX.abc.def",
            // Characters of more than one byte where the format's digits would be, which once
            // panicked (the property test below found it).
            "ࠀ",
            "Qࠀ.abc.def",
            "QDé.abc.def",
        ] {
            assert_eq!(
                Licence::verify_with(text, &[test_key()]),
                Err(KeyError::NotAKey),
                "{text:?}"
            );
        }
        assert_eq!(
            Licence::verify_with("QD2.abc.def", &[test_key()]),
            Err(KeyError::Newer)
        );
    }

    #[test]
    fn a_receipt_s_order_key_says_where_quotadial_s_key_is() {
        for pasted in [
            "38b1460a-5104-4067-a91d-77b872934d51",
            " 38B1460A-5104-4067-A91D-77B872934D51\n",
        ] {
            assert_eq!(
                Licence::verify_with(pasted, &[test_key()]),
                Err(KeyError::OrderKey),
                "{pasted:?}"
            );
        }
        assert!(
            KeyError::OrderKey
                .to_string()
                .contains("Get your QuotaDial key")
        );
    }

    #[test]
    fn a_key_for_a_plan_this_version_does_not_know_asks_for_an_update() {
        let body = BASE64.encode(r#"{"id":"t-1","plan":"team","issued":"2026-10-24"}"#);
        let signed = format!("{PREFIX}.{body}");
        let signature = SigningKey::from_bytes(&TEST_SEED).sign(signed.as_bytes());
        let key = format!("{signed}.{}", BASE64.encode(signature.to_bytes()));

        assert_eq!(
            Licence::verify_with(&key, &[test_key()]),
            Err(KeyError::Newer)
        );
    }

    #[test]
    fn fields_a_later_issuer_adds_are_signed_and_left_alone() {
        let body = BASE64
            .encode(r#"{"id":"t-2","plan":"pro","issued":"2026-10-24","seats":5,"note":"later"}"#);
        let signed = format!("{PREFIX}.{body}");
        let signature = SigningKey::from_bytes(&TEST_SEED).sign(signed.as_bytes());
        let key = format!("{signed}.{}", BASE64.encode(signature.to_bytes()));

        assert_eq!(
            Licence::verify_with(&key, &[test_key()]).map(|licence| licence.id),
            Ok("t-2".into())
        );
    }

    #[test]
    fn a_licence_holds_through_its_last_day() {
        let ending = licence(Some("2027-10-24"));

        assert!(ending.holds_on(day("2026-10-24")));
        assert!(ending.holds_on(day("2027-10-24")));
        assert!(!ending.holds_on(day("2027-10-25")));
        assert!(licence(None).holds_on(day("2099-12-31")));
    }

    #[test]
    fn the_standing_says_what_a_copy_is_in_one_sentence() {
        let today = day("2026-11-01");
        let standing = |key: Option<&str>| Standing::of(key, today);

        let none = standing(None);
        assert_eq!(none.tier, Tier::Free);
        assert_eq!(none.text, "QuotaDial Free, with no licence key.");

        let pro = standing(Some(&licence(None).sign(&TEST_SEED)));
        assert_eq!(pro.tier, Tier::Pro);
        assert_eq!(pro.licence, Some(licence(None)));
        assert_eq!(pro.text, "QuotaDial Pro, licence ls-1234.");

        let until = standing(Some(&licence(Some("2027-10-24")).sign(&TEST_SEED)));
        assert_eq!(until.tier, Tier::Pro);
        assert_eq!(
            until.text,
            "QuotaDial Pro until 2027-10-24, licence ls-1234."
        );

        let ended = standing(Some(&licence(Some("2026-10-31")).sign(&TEST_SEED)));
        assert_eq!(ended.tier, Tier::Free);
        assert_eq!(ended.licence, Some(licence(Some("2026-10-31"))));
        assert_eq!(
            ended.text,
            "QuotaDial Free: licence ls-1234 ended on 2026-10-31. Renewing it at quotadial.com/pro \
             gives a new key."
        );

        let damaged = standing(Some("QD1.abc.def"));
        assert_eq!(damaged.tier, Tier::Free);
        assert_eq!(damaged.licence, None);
        assert!(
            damaged.text.contains("doesn't check out"),
            "{}",
            damaged.text
        );

        let newer = standing(Some("QD2.abc.def"));
        assert_eq!(newer.tier, Tier::Free);
        assert!(newer.text.contains("newer QuotaDial"), "{}", newer.text);
    }

    #[test]
    fn a_standing_without_its_id_is_safe_to_share() {
        let today = day("2026-11-01");
        let without = |key: Option<&str>| Standing::of(key, today).without_id();

        assert_eq!(
            without(Some(&licence(None).sign(&TEST_SEED))),
            "QuotaDial Pro, with a licence that does not end"
        );
        assert_eq!(
            without(Some(&licence(Some("2027-10-24")).sign(&TEST_SEED))),
            "QuotaDial Pro until 2027-10-24"
        );
        assert_eq!(
            without(Some(&licence(Some("2026-10-31")).sign(&TEST_SEED))),
            "QuotaDial Free: the licence ended on 2026-10-31"
        );
        assert_eq!(without(None), "QuotaDial Free, with no licence key.");
        for key in [
            licence(None),
            licence(Some("2027-10-24")),
            licence(Some("2026-10-31")),
        ] {
            assert!(
                !Standing::of(Some(&key.sign(&TEST_SEED)), today)
                    .without_id()
                    .contains("ls-1234")
            );
        }
    }

    #[test]
    fn a_standing_is_plain_json() {
        let pro = Standing::of(
            Some(&licence(Some("2027-10-24")).sign(&TEST_SEED)),
            day("2026-11-01"),
        );

        assert_eq!(
            serde_json::to_value(&pro).expect("serializes"),
            serde_json::json!({
                "tier": "pro",
                "licence": {"id": "ls-1234", "plan": "pro", "issued": "2026-10-24", "until": "2027-10-24"},
                "text": "QuotaDial Pro until 2027-10-24, licence ls-1234.",
            })
        );
        assert_eq!(
            serde_json::to_value(Standing::of(None, day("2026-11-01"))).expect("serializes"),
            serde_json::json!({"tier": "free", "text": "QuotaDial Free, with no licence key."})
        );
    }

    #[test]
    fn a_key_is_kept_on_one_line_as_it_was_signed() {
        let key = licence(None).sign(&TEST_SEED);
        let pasted = format!(" qd1{}\n{}\u{200b}", &key[3..40], &key[40..]);

        assert_eq!(canonical(&pasted), key);
        assert_eq!(canonical("ࠀ"), "ࠀ");
        assert_eq!(canonical(""), "");
    }

    #[test]
    fn a_key_is_whole_once_its_signature_is() {
        let key = licence(None).sign(&TEST_SEED);

        assert!(whole(&key));
        assert!(whole(&format!("{}\n{}", &key[..50], &key[50..])));
        assert!(!whole(&key[..key.len() - 5]));
        assert!(!whole(&key[..60]));
        assert!(!whole("QD1."));
        assert!(!whole(""));
    }

    /// `docs/LICENSING.md` gives these keys to issuers in other languages to check their output
    /// against, so QuotaDial must sign them byte for byte the same.
    #[test]
    fn the_documented_test_keys_are_what_quotadial_signs() {
        let documented = include_str!("../../../docs/LICENSING.md");
        let issued = day("2026-10-24");
        let keys = [
            Licence {
                id: "test-0001".into(),
                plan: Plan::Pro,
                issued,
                until: None,
            },
            Licence {
                id: "test-0002".into(),
                plan: Plan::Pro,
                issued,
                until: Some(day("2027-10-24")),
            },
        ]
        .map(|licence| licence.sign(&TEST_SEED));

        for key in &keys {
            assert!(documented.contains(&format!("\n{key}\n")), "{key}");
        }
        assert_eq!(
            keys[0],
            "QD1.eyJpZCI6InRlc3QtMDAwMSIsImlzc3VlZCI6IjIwMjYtMTAtMjQiLCJwbGFuIjoicHJvIn0.\
             TCnQdA2XJTPnuGVV280SOgfy_BH4maU7Nj8s11TSw6nWbpPT_icIfL-nf155_ZQXnAnvGsKDQ14V3X-72YchDw"
        );
    }

    #[test]
    fn quotadials_keys_are_sound_public_keys() {
        for public in [QUOTADIAL, ISSUER] {
            let key = VerifyingKey::from_bytes(&public).expect("a point on the curve");
            assert!(!key.is_weak());
            assert_ne!(public, test_key());
        }
        assert_ne!(
            QUOTADIAL, ISSUER,
            "the root and the issuing key are two keys"
        );
    }

    /// CI runs this in a release build too (`cargo test --release`), where the test key must not
    /// be trusted: a release takes only keys QuotaDial's key signed.
    #[test]
    fn only_a_debug_build_trusts_the_test_key() {
        let trusted = trusted();

        assert_eq!(trusted[..2], [QUOTADIAL, ISSUER]);
        assert_eq!(trusted.contains(&test_key()), cfg!(debug_assertions));
        let tested = licence(None).sign(&TEST_SEED);
        assert_eq!(Licence::verify(&tested).is_ok(), cfg!(debug_assertions));
    }

    proptest! {
        #[test]
        fn no_text_makes_the_reader_panic(text in ".*") {
            let _ = Licence::verify_with(&text, &[test_key()]);
            let _ = whole(&text);
            let _ = canonical(&text);
            let _ = Standing::of(Some(&text), day("2026-11-01"));
        }

        #[test]
        fn no_key_shaped_text_makes_the_reader_panic(
            format in "(QD|qd)[0-9]{0,3}",
            body in "[A-Za-z0-9_=-]{0,200}",
            signature in "[A-Za-z0-9_=-]{0,100}",
        ) {
            let key = format!("{format}.{body}.{signature}");
            prop_assert!(Licence::verify_with(&key, &[test_key()]).is_err());
        }

        #[test]
        fn every_licence_signed_is_read_back(
            id in "[ -~]{1,64}",
            issued in 0_i32..40_000,
            length in proptest::option::of(0_i32..4_000),
        ) {
            let start = day("2000-01-01");
            let issued = start + chrono::Days::new(u64::try_from(issued).unwrap_or(0));
            let until = length.map(|days| issued + chrono::Days::new(u64::try_from(days).unwrap_or(0)));
            let written = Licence { id, plan: Plan::Pro, issued, until };

            prop_assert_eq!(Licence::verify_with(&written.sign(&TEST_SEED), &[test_key()]), Ok(written));
        }
    }
}
