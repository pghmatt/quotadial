//! `packaging/completions` and `packaging/man`: the shell completions and the manual pages, written
//! from the commands' own definitions, so they cannot drift from what the CLI accepts. That
//! includes the commands a provider adds (P-018), since they come from the same `command` the CLI
//! runs. Every release archive carries them and the package managers install them. This test fails
//! when a file and the code disagree, and `QUOTADIAL_WRITE_PACKAGING=1 cargo test -p quotadial
//! packaging` writes them again.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Command, CommandFactory};
use clap_complete::Shell;
use clap_mangen::Man;
use quotadial_core::Provider;
use quotadial_daemon::{Config, Licensing};
use roff::{Roff, bold, italic, roman};

use crate::Cli;

/// `packaging/`, from this crate.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging")
}

/// The providers this build ships, set up with the default settings and no licence, so no folder
/// is read.
fn shipped() -> Vec<Arc<dyn Provider>> {
    quotadial_providers::with_config(&Config::default(), &Licensing::new(None))
}

/// The command as `quotadial` prints its help.
fn command() -> Command {
    let mut command = crate::command(&shipped());
    command.build();
    command
}

/// `command` as the pages describe it: without clap's `help` subcommand, which `--help` and the
/// pages themselves cover.
fn for_pages(command: Command) -> Command {
    let mut command = command.disable_help_subcommand(true);
    command.build();
    command
}

/// Every file under `packaging/completions` and `packaging/man`, by its path there, with what it
/// should hold.
fn files() -> BTreeMap<String, Vec<u8>> {
    files_of(&crate::command(&shipped()))
}

/// The completions and pages for `command`, by their paths under `packaging/`.
fn files_of(command: &Command) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    for bin in ["quotadial", "qd"] {
        for (shell, name) in [
            (Shell::Bash, format!("{bin}.bash")),
            (Shell::Zsh, format!("_{bin}")),
            (Shell::Fish, format!("{bin}.fish")),
            (Shell::PowerShell, format!("{bin}.ps1")),
        ] {
            let mut script = Vec::new();
            clap_complete::generate(shell, &mut command.clone(), bin, &mut script);
            files.insert(format!("completions/{name}"), script);
        }
    }
    pages(&for_pages(command.clone()), true, &mut files);
    files
}

/// The page for `command`, then one for each of its subcommands.
fn pages(command: &Command, top: bool, files: &mut BTreeMap<String, Vec<u8>>) {
    let man = Man::new(command.clone())
        .date("")
        .manual("QuotaDial")
        .source(format!("QuotaDial {}", env!("CARGO_PKG_VERSION")));
    let mut page = Vec::new();
    let rendered = "a page renders into memory";
    man.render_title(&mut page).expect(rendered);
    man.render_name_section(&mut page).expect(rendered);
    man.render_synopsis_section(&mut page).expect(rendered);
    man.render_description_section(&mut page).expect(rendered);
    if command.get_arguments().any(|arg| !arg.is_hide_set()) {
        man.render_options_section(&mut page).expect(rendered);
    }
    if command.has_subcommands() {
        man.render_subcommands_section(&mut page).expect(rendered);
    }
    let mut more = Roff::new();
    if let Some(examples) = command.get_after_long_help() {
        more.control("SH", ["EXAMPLES"]).control("nf", []);
        for line in examples.to_string().lines().skip(1) {
            more.text([roman(line.trim_start())]);
        }
        more.control("fi", []);
    }
    if top {
        about_the_command(&mut more);
    }
    more.control("SH", ["SEE ALSO"]);
    let parent = command.get_display_name().unwrap_or("quotadial");
    let others: Vec<String> = if top {
        documented(command)
            .map(|sub| format!("{parent}-{}(1)", sub.get_name()))
            .collect()
    } else {
        vec!["quotadial(1)".to_owned()]
    };
    more.text([roman(format!(
        "{}. The full guide is docs/CLI.md in QuotaDial's repository, \
         https://github.com/pghmatt/quotadial, and quotadial.com.",
        others.join(", ")
    ))]);
    page.extend(more.render().into_bytes());
    if command.get_version().is_some() {
        man.render_version_section(&mut page).expect(rendered);
    }
    files.insert(format!("man/{}", man.get_filename()), page);
    for sub in documented(command) {
        pages(sub, false, files);
    }
}

/// The subcommands that get a page.
fn documented(command: &Command) -> impl Iterator<Item = &Command> {
    command.get_subcommands().filter(|sub| !sub.is_hide_set())
}

/// The sections only `quotadial(1)` has: what it exits with, what it reads from the environment,
/// and where it keeps its files, as docs/CLI.md says at more length.
fn about_the_command(roff: &mut Roff) {
    roff.control("SH", ["EXIT STATUS"]);
    for (code, meaning) in [
        (
            "0",
            "It worked. For status, every account produced a snapshot, whatever its status: \
             check each snapshot's status.",
        ),
        (
            "1",
            "Something failed: a provider or account produced no snapshot, a running daemon did \
             not answer, or the output could not be written. What went wrong, and what to do, is \
             on standard error.",
        ),
        ("2", "The arguments were not understood."),
    ] {
        roff.control("TP", [])
            .text([bold(code)])
            .text([roman(meaning)]);
    }
    roff.control("SH", ["ENVIRONMENT"]);
    for (name, meaning) in [
        (
            "QUOTADIAL_DATA_DIR",
            "The folder QuotaDial keeps its history, settings, licence key and the local API's \
             token in, in place of the system's own.",
        ),
        (
            "CLAUDE_CODE_OAUTH_TOKEN",
            "The Claude sign-in to read first, as Claude Code does. QuotaDial only reads it.",
        ),
        (
            "CLAUDE_CONFIG_DIR",
            "The folder Claude Code keeps its sign-in for, in place of ~/.claude. QuotaDial also \
             finds every other Claude Code sign-in on the machine.",
        ),
        (
            "CODEX_HOME",
            "The folder the Codex CLI keeps auth.json in, in place of ~/.codex.",
        ),
        (
            "GEMINI_CLI_HOME",
            "The folder Gemini CLI keeps its .gemini folder in, in place of the home folder. \
             QuotaDial only checks that its sign-in is there and reads one field of its settings.",
        ),
    ] {
        roff.control("TP", [])
            .text([bold(name)])
            .text([roman(meaning)]);
    }
    roff.control("SH", ["FILES"]);
    for (path, meaning) in [
        (
            "~/Library/Application Support/quotadial",
            "On macOS: the history (history.db), config.json, the licence key, daemon.json while \
             the daemon runs, and the local API's token.",
        ),
        (
            "$XDG_DATA_HOME/quotadial or ~/.local/share/quotadial",
            "The same, on Linux.",
        ),
        ("%APPDATA%\\quotadial", "The same, on Windows."),
    ] {
        roff.control("TP", [])
            .text([italic(path)])
            .text([roman(meaning)]);
    }
}

#[test]
#[cfg_attr(
    feature = "fake",
    ignore = "the files describe what a release ships; the example build's daemon has its own \
              default port (D-117)"
)]
fn the_completions_and_manual_pages_are_the_commands() {
    let root = root();
    let files = files();
    if std::env::var_os("QUOTADIAL_WRITE_PACKAGING").is_some() {
        for folder in ["completions", "man"] {
            let _ = std::fs::remove_dir_all(root.join(folder));
            std::fs::create_dir_all(root.join(folder)).expect("makes the folder");
        }
        for (path, content) in &files {
            std::fs::write(root.join(path), content).expect("writes the file");
        }
    }
    let mut found = BTreeMap::new();
    for folder in ["completions", "man"] {
        for entry in std::fs::read_dir(root.join(folder))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            found.insert(
                format!("{folder}/{}", name_of(&path)),
                std::fs::read(&path).unwrap_or_default(),
            );
        }
    }
    let differ: Vec<&String> = files
        .keys()
        .chain(found.keys())
        .filter(|path| files.get(*path) != found.get(*path))
        .collect();
    assert!(
        differ.is_empty(),
        "packaging/ is not what the commands say for {differ:?}; write it again with \
         QUOTADIAL_WRITE_PACKAGING=1 cargo test -p quotadial packaging"
    );
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[test]
fn every_command_and_argument_says_what_it_is_for() {
    fn check(command: &Command, path: &str) {
        assert!(
            command.get_about().is_some(),
            "`{path} --help` says nothing about what it does"
        );
        for arg in command.get_arguments().filter(|arg| !arg.is_hide_set()) {
            assert!(
                arg.get_help().is_some(),
                "`{path} --help` says nothing about {}",
                arg.get_id()
            );
        }
        for sub in command.get_subcommands() {
            check(sub, &format!("{path} {}", sub.get_name()));
        }
    }
    check(&command(), "quotadial");
}

#[test]
fn there_is_a_manual_page_for_every_command() {
    let files = files();
    let mut missing = Vec::new();
    for sub in for_pages(crate::command(&shipped())).get_subcommands() {
        let page = format!("man/quotadial-{}.1", sub.get_name());
        if !files.contains_key(&page) {
            missing.push(page);
        }
    }
    assert!(missing.is_empty(), "no page: {missing:?}");
    let top = String::from_utf8(files["man/quotadial.1"].clone()).expect("roff is text");
    assert!(
        !top.contains("quotadial-help"),
        "a page names clap's help subcommand"
    );
    for section in [
        "EXAMPLES",
        "EXIT STATUS",
        "ENVIRONMENT",
        "FILES",
        "SEE ALSO",
    ] {
        assert!(top.contains(section), "quotadial(1) has no {section}");
    }
    // Each variable a shipped provider reads to find a sign-in, Gemini CLI's too (D-124).
    for variable in [
        "QUOTADIAL_DATA_DIR",
        "CLAUDE_CODE_OAUTH_TOKEN",
        "CLAUDE_CONFIG_DIR",
        "CODEX_HOME",
        "GEMINI_CLI_HOME",
    ] {
        assert!(
            top.contains(variable),
            "quotadial(1) does not name {variable}"
        );
    }
}

#[test]
fn a_providers_own_commands_get_completions_and_manual_pages() {
    let providers: Vec<Arc<dyn Provider>> = vec![Arc::new(crate::tests::ChattyProvider)];
    let mut with_commands = crate::command(&providers);
    with_commands.build();

    let files = files_of(&with_commands);

    assert!(files.contains_key("man/quotadial-chatty.1"));
    let page = String::from_utf8(files["man/quotadial-chatty-echo.1"].clone()).expect("roff");
    assert!(page.contains(crate::tests::ECHO_ABOUT), "{page}");
    for (path, script) in files
        .iter()
        .filter(|(path, _)| path.starts_with("completions/"))
    {
        let script = String::from_utf8_lossy(script);
        assert!(
            script.contains("chatty") && script.contains("echo"),
            "{path}"
        );
    }
    let echo = with_commands
        .find_subcommand("chatty")
        .and_then(|group| group.find_subcommand("echo"))
        .expect("the provider's command");
    for arg in echo.get_arguments().filter(|arg| !arg.is_hide_set()) {
        assert!(arg.get_help().is_some(), "{} has no help", arg.get_id());
    }
}

#[test]
fn a_provider_with_no_commands_of_its_own_changes_no_file() {
    let providers: Vec<Arc<dyn Provider>> =
        vec![Arc::new(crate::tests::CountingProvider::default())];

    assert!(files_of(&crate::command(&providers)) == files_of(&Cli::command()));
}
