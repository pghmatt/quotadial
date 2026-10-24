//! The app's Help (D-101): what is new in this version, read from the `CHANGELOG.md` built into the
//! app, so it needs no network; Send feedback, which opens a new GitHub issue with what
//! `quotadial doctor` prints filled in; the support address; and About, with the version, the
//! licence and the credits.

use serde::Serialize;

/// The changelog, as the app was built with it.
pub const CHANGELOG: &str = include_str!("../../../../CHANGELOG.md");

/// What the app's Help shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HelpView {
    /// The app's version, such as `0.3.0`.
    pub version: String,
    /// What changed in it.
    pub whats_new: WhatsNew,
    /// What this copy is, such as "QuotaDial Pro, licence 1234."
    pub licence: String,
    /// Where people write for help.
    pub support: String,
    /// Whose work QuotaDial uses, and under which licence.
    pub credits: Vec<String>,
}

/// One version's part of the changelog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WhatsNew {
    /// The version, such as `0.3.0`, or `Unreleased` for a build between releases.
    pub version: String,
    /// Each kind of change, such as "Added", with its lines.
    pub sections: Vec<Section>,
}

/// A kind of change and its lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Section {
    /// Such as "Added" or "Fixed".
    pub heading: String,
    /// The changes, as plain text.
    pub items: Vec<String>,
}

/// Whose work QuotaDial uses: NOTICE's credits, as About shows them.
pub const CREDITS: [&str; 3] = [
    "QuotaDial is free software under the MIT licence.",
    "Its provider integrations and cost scan are ported from CodexBar by Peter Steinberger, used \
     under the MIT licence.",
    "Model prices come from models.dev, used under the MIT licence.",
];

/// What changed in `version`, from `changelog`: its own section, or Unreleased when it has none,
/// as in a build between releases.
#[must_use]
pub fn whats_new(changelog: &str, version: &str) -> WhatsNew {
    let section = |name: &str| {
        let heading = format!("## [{name}]");
        let mut lines = changelog
            .lines()
            .skip_while(|line| !line.starts_with(&heading));
        lines.next().map(|_| {
            lines
                .take_while(|line| !line.starts_with("## [") && !line.starts_with('['))
                .collect::<Vec<_>>()
        })
    };
    let (shown, lines) = match section(version) {
        Some(lines) => (version, lines),
        None => ("Unreleased", section("Unreleased").unwrap_or_default()),
    };
    let mut sections: Vec<Section> = Vec::new();
    for line in lines {
        if let Some(heading) = line.strip_prefix("### ") {
            sections.push(Section {
                heading: heading.trim().to_owned(),
                items: Vec::new(),
            });
        } else if let Some(item) = line.strip_prefix("- ")
            && let Some(section) = sections.last_mut()
        {
            section.items.push(plain(item));
        }
    }
    sections.retain(|section| !section.items.is_empty());
    WhatsNew {
        version: shown.to_owned(),
        sections,
    }
}

/// `markdown` as plain text: code, bold and links lose their marks, and a link keeps its words.
fn plain(markdown: &str) -> String {
    let mut text = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find('[') {
        let link = rest[start..].find("](").and_then(|middle| {
            let after = start + middle + 2;
            rest[after..]
                .find(')')
                .map(|end| (start + middle, after + end))
        });
        match link {
            Some((middle, end)) => {
                text.push_str(&rest[..start]);
                text.push_str(&rest[start + 1..middle]);
                rest = &rest[end + 1..];
            }
            None => {
                text.push_str(&rest[..=start]);
                rest = &rest[start + 1..];
            }
        }
    }
    text.push_str(rest);
    text.replace("**", "").replace('`', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANGES: &str = "# Changelog\n\nIntro.\n\n## [Unreleased]\n\n### Added\n\n- A new \
                           `quotadial doctor`.\n\n## [0.2.0] - 2026-10-24\n\n### Added\n\n- **Pace** \
                           for each window.\n- See [the docs](https://example.com) for more.\n\n\
                           ### Fixed\n\n- A fix.\n\n## [0.1.0] - 2026-10-23\n\n### Added\n\n- The \
                           first.\n\n[Unreleased]: https://example.com/compare\n";

    #[test]
    fn a_released_version_shows_its_own_section_in_plain_words() {
        assert_eq!(
            whats_new(CHANGES, "0.2.0"),
            WhatsNew {
                version: "0.2.0".into(),
                sections: vec![
                    Section {
                        heading: "Added".into(),
                        items: vec![
                            "Pace for each window.".into(),
                            "See the docs for more.".into(),
                        ],
                    },
                    Section {
                        heading: "Fixed".into(),
                        items: vec!["A fix.".into()],
                    },
                ],
            }
        );
        assert_eq!(
            whats_new(CHANGES, "0.1.0").sections[0].items,
            ["The first."]
        );
    }

    #[test]
    fn a_build_between_releases_shows_what_is_unreleased() {
        let unreleased = whats_new(CHANGES, "0.3.0");

        assert_eq!(unreleased.version, "Unreleased");
        assert_eq!(unreleased.sections[0].items, ["A new quotadial doctor."]);
        assert!(whats_new("# Changelog\n", "0.3.0").sections.is_empty());
    }

    #[test]
    fn the_changelog_built_in_has_this_version_or_unreleased() {
        let shown = whats_new(CHANGELOG, env!("CARGO_PKG_VERSION"));

        assert!(!shown.sections.is_empty(), "{shown:?}");
        assert!(shown.sections.iter().all(|section| {
            section
                .items
                .iter()
                .all(|item| !item.contains('`') && !item.contains("]("))
        }));
    }

    #[test]
    fn plain_text_keeps_what_is_not_markdown() {
        assert_eq!(plain("a [b] c"), "a [b] c");
        assert_eq!(plain("[x](y"), "[x](y");
        assert_eq!(plain("`a` **b** [c](d) e"), "a b c e");
    }
}
