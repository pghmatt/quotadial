//! The app's Help (D-101): what is new in this version, read from the `CHANGELOG.md` built into the
//! app, so it needs no network; Send feedback, which opens a new GitHub issue with what
//! `quotadial doctor` prints filled in; the support address; About, with the version, the licence
//! and the credits; and what QuotaDial reads and sends, the one text the README and the website
//! also show (D-106).

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
    /// What QuotaDial reads and sends, from `docs/PRIVACY.md`.
    pub privacy: Vec<Block>,
    /// Every place QuotaDial connects to, as `privacy` names them ([`hosts`]).
    pub hosts: Vec<String>,
}

/// Every place QuotaDial connects to (`quotadial_core::privacy::HOSTS`), such as `chatgpt.com`,
/// which the privacy text names: the hosts the window finds in that text and writes as it writes a
/// host, breaking before a dot and never inside a word (D-141, extended to a host in a sentence). A
/// list, so that a file's name with a dot in it, such as `config.json`, is never taken for one.
#[must_use]
pub fn hosts() -> Vec<String> {
    quotadial_core::privacy::HOSTS
        .iter()
        .map(|host| (*host).to_owned())
        .collect()
}

/// A paragraph or a list of a text Help shows, in plain words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    Paragraph { text: String },
    List { items: Vec<String> },
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

/// `markdown`, such as `docs/PRIVACY.md`, as paragraphs and lists in plain words, without its
/// headings. A list item's wrapped lines join it.
#[must_use]
pub fn blocks(markdown: &str) -> Vec<Block> {
    fn end_paragraph(lines: &mut Vec<&str>, blocks: &mut Vec<Block>) {
        if !lines.is_empty() {
            blocks.push(Block::Paragraph {
                text: plain(&lines.join(" ")),
            });
            lines.clear();
        }
    }
    fn end_list(items: &mut Vec<String>, blocks: &mut Vec<Block>) {
        if !items.is_empty() {
            blocks.push(Block::List {
                items: items.drain(..).map(|item| plain(&item)).collect(),
            });
        }
    }
    let mut blocks = Vec::new();
    let mut lines = Vec::new();
    let mut items: Vec<String> = Vec::new();
    for line in markdown.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            end_paragraph(&mut lines, &mut blocks);
            end_list(&mut items, &mut blocks);
        } else if let Some(item) = line.strip_prefix("- ") {
            end_paragraph(&mut lines, &mut blocks);
            items.push(item.trim().to_owned());
        } else if let (true, Some(item)) = (line.starts_with("  "), items.last_mut()) {
            item.push(' ');
            item.push_str(line.trim());
        } else {
            end_list(&mut items, &mut blocks);
            lines.push(line.trim());
        }
    }
    end_paragraph(&mut lines, &mut blocks);
    end_list(&mut items, &mut blocks);
    blocks
}

/// `markdown` as plain text: code, bold and links lose their marks, and a link keeps its words.
pub(crate) fn plain(markdown: &str) -> String {
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
    fn what_quotadial_sends_reads_as_paragraphs_and_one_list_of_every_destination() {
        let shown = blocks(quotadial_core::privacy::PRIVACY);

        let lists: Vec<&Vec<String>> = shown
            .iter()
            .filter_map(|block| match block {
                Block::List { items } => Some(items),
                Block::Paragraph { .. } => None,
            })
            .collect();
        assert_eq!(lists.len(), 1, "{shown:?}");
        assert_eq!(lists[0].len(), 4);
        assert!(lists[0][0].starts_with("chatgpt.com, OpenAI's, with your Codex CLI sign-in"));
        for host in quotadial_core::privacy::HOSTS {
            assert!(lists[0].iter().any(|item| item.contains(host)), "{host}");
        }
        assert!(
            matches!(&shown[0], Block::Paragraph { text } if text.starts_with("QuotaDial runs on your machine."))
        );
        let text = format!("{shown:?}");
        assert!(!text.contains("**") && !text.contains('`') && !text.contains("# What"));
    }

    #[test]
    fn every_host_help_is_given_stands_in_the_privacy_text_where_the_window_finds_one() {
        // The window finds a host where it stands as a word of a sentence does (`hostRuns` in
        // `src/address.ts`): after the line's start, a space, a bracket or a quotation mark, and
        // before its end, a space, a mark or the stop that ends a sentence. It writes each one it
        // finds breaking before a dot, so a host the text names any other way would not break so.
        assert_eq!(hosts(), quotadial_core::privacy::HOSTS);
        let lines: Vec<String> = blocks(quotadial_core::privacy::PRIVACY)
            .into_iter()
            .flat_map(|block| match block {
                Block::Paragraph { text } => vec![text],
                Block::List { items } => items,
            })
            .collect();
        let starts = |before: Option<char>| {
            before.is_none_or(|before| before.is_whitespace() || "([\"'".contains(before))
        };
        let ends = |rest: &str| {
            let mut after = rest.chars();
            match after.next() {
                None => true,
                Some('.') => after.next().is_none_or(char::is_whitespace),
                Some(mark) => mark.is_whitespace() || ",;:!?)]\"'".contains(mark),
            }
        };
        for host in hosts() {
            let stands = lines.iter().any(|line| {
                line.match_indices(host.as_str()).any(|(at, _)| {
                    starts(line[..at].chars().next_back()) && ends(&line[at + host.len()..])
                })
            });
            assert!(
                stands,
                "{host} does not stand in the privacy text where the window finds a host"
            );
        }
        // Not where a longer name goes on from it, as "github.com" in "api.github.com" or
        // "github.com.au", or a link from it, as "github.com/pghmatt".
        assert!(!starts(Some('.')) && !starts(Some('@')));
        assert!(
            !ends(".au") && !ends("/pghmatt") && !ends("-mirror") && ends(". Next") && ends(", ")
        );
    }

    /// The window check's copies (`scripts/window-fixtures.mjs`) keep Help's text of what QuotaDial
    /// reads and sends exactly as the app sends it, a block a line, so that their own test
    /// (`scripts/whole-addresses.test.mjs`) can hold every copy's text to the app's words, the
    /// website's too. When `docs/PRIVACY.md` changes, this fails and says the lines to put there.
    #[test]
    fn the_window_check_keeps_help_s_privacy_text_as_the_app_sends_it() {
        const COPIES: &str = include_str!("../../scripts/window-fixtures.mjs");
        let lines: String = blocks(quotadial_core::privacy::PRIVACY)
            .iter()
            .map(|block| format!("  {},\n", serde_json::to_string(block).expect("JSON")))
            .collect();
        let kept = format!("export const PRIVACY = [\n{lines}];\n");
        assert!(
            COPIES.contains(&kept),
            "scripts/window-fixtures.mjs should keep the app's privacy text as\n{kept}"
        );
    }

    /// The window check's copy of ten accounts shows lines of the real changelog in What's new and
    /// in its update's notes, `CHANGES` in `scripts/window-fixtures.mjs`, a heading and its lines a
    /// line, so that the check holds lines that name files and commands as the app shows them.
    /// Each must be a line of the app's own What's new for the copies' version, under its heading
    /// and in its order. When `CHANGELOG.md` changes one or drops it, this fails and says the lines
    /// there are.
    #[test]
    fn the_window_check_s_changelog_lines_are_the_app_s_own() {
        // A heading and its lines, as `CHANGES` keeps them.
        #[derive(serde::Deserialize)]
        struct Kept {
            heading: String,
            items: Vec<String>,
        }
        // The copies' version, which their Help and What's new name.
        const VERSION: &str = "0.3.0";
        const COPIES: &str = include_str!("../../scripts/window-fixtures.mjs");

        assert!(COPIES.contains(&format!("version: \"{VERSION}\"")));
        let kept: Vec<Kept> = COPIES
            .split_once("export const CHANGES = [\n")
            .and_then(|(_, rest)| rest.split_once("];\n"))
            .expect(
                "scripts/window-fixtures.mjs should keep CHANGES, a heading and its lines a line",
            )
            .0
            .lines()
            .map(|line| {
                serde_json::from_str(line.trim().trim_end_matches(',')).expect("a line of JSON")
            })
            .collect();
        assert!(
            kept.iter().any(|section| !section.items.is_empty()),
            "CHANGES keeps no line"
        );
        let shown = whats_new(CHANGELOG, VERSION);
        let mut sections = shown.sections.iter();
        let mut not_the_app_s = Vec::new();
        for section in &kept {
            let mut items = sections
                .find(|own| own.heading == section.heading)
                .map(|own| own.items.iter());
            for item in &section.items {
                if !items
                    .as_mut()
                    .is_some_and(|items| items.any(|own| own == item))
                {
                    not_the_app_s.push(format!("{}: {item}", section.heading));
                }
            }
        }
        let lines: String = shown
            .sections
            .iter()
            .map(|section| format!("  {},\n", serde_json::to_string(section).expect("JSON")))
            .collect();
        assert!(
            not_the_app_s.is_empty(),
            "scripts/window-fixtures.mjs's CHANGES has lines that are not the app's What's new in \
             {VERSION}, under their heading and in its order:\n{}\nIts lines are:\n{lines}",
            not_the_app_s.join("\n")
        );
    }

    #[test]
    fn a_wrapped_line_joins_its_paragraph_or_its_item() {
        assert_eq!(
            blocks("# Title\n\nOne\nline.\n\n- An\n  item.\n- Two.\nAfter.\n"),
            [
                Block::Paragraph {
                    text: "One line.".into()
                },
                Block::List {
                    items: vec!["An item.".into(), "Two.".into()]
                },
                Block::Paragraph {
                    text: "After.".into()
                },
            ]
        );
    }

    #[test]
    fn plain_text_keeps_what_is_not_markdown() {
        assert_eq!(plain("a [b] c"), "a [b] c");
        assert_eq!(plain("[x](y"), "[x](y");
        assert_eq!(plain("`a` **b** [c](d) e"), "a b c e");
    }

    use proptest::strategy::Strategy as _;

    proptest::proptest! {
        #[test]
        fn no_text_makes_the_markdown_reader_panic(text in ".*") {
            let _ = plain(&text);
            let _ = blocks(&text);
            let _ = whats_new(&text, "0.3.0");
        }

        #[test]
        fn no_markdown_like_text_makes_it_panic(
            pieces in proptest::collection::vec(
                proptest::prop_oneof![
                    proptest::sample::select(
                        &["[", "]", "(", ")", "](", "**", "`", "- ", "  ", "\n", "# ", "### ",
                          "## [0.3.0]", "## [Unreleased]", "é", "😀"][..]
                    ).prop_map(str::to_owned),
                    ".{0,6}",
                ],
                0..40,
            ),
        ) {
            let text = pieces.concat();
            let _ = plain(&text);
            let _ = blocks(&text);
            let _ = whats_new(&text, "0.3.0");
        }
    }
}
