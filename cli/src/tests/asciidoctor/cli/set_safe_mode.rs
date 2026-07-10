use std::path::PathBuf;

use asciidoc_html5::SafeMode;
use clap::Parser as _;

use crate::{resolve_safe_mode, run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/cli/pages/set-safe-mode.adoc");

// Asciidoctor's "Set the Safe Mode Using the CLI" page, tracked from the CLI
// crate. `adoc` mirrors Asciidoctor's CLI: the safe mode defaults to `UNSAFE`,
// `-S`/`--safe-mode` assigns a named level, `--safe` selects `SAFE`, and
// `-B`/`--base-dir` sets the base directory. Each claim drives `adoc`'s own
// option parsing (`Cli` + `resolve_safe_mode`), and the default/secure and
// base-directory cases are confirmed end to end through `run`.
//
// The `asciidoctor-safe` command alias has no counterpart in `adoc`, so it
// stays non-normative.

/// Resolves the safe mode `adoc` would use for the given command-line
/// arguments, exercising the full `Cli` parse plus [`resolve_safe_mode`].
fn safe_mode_for(args: &[&str]) -> SafeMode {
    let cli = Cli::parse_from(args);
    resolve_safe_mode(&cli).expect("valid safe mode")
}

/// Runs `adoc` on `source` with `args` (a source file is written to a temp path
/// and appended to `args`, with `-o -` forcing output to the captured stdout).
fn run_adoc(label: &str, args: &[&str], source: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "adoc-cli-set-safe-mode-{label}-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, source).expect("write temp input");

    let mut full: Vec<&str> = vec!["adoc", "-o", "-"];
    full.extend_from_slice(args);
    let path_str = path.to_str().expect("temp path is UTF-8");
    full.push(path_str);

    let cli = Cli::parse_from(full);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts");
    let _ = std::fs::remove_file(&path);
    String::from_utf8(stdout).expect("adoc output is UTF-8")
}

/// A throwaway on-disk project rooted at a unique temp directory, used to
/// exercise `include::` resolution and the base-directory jail end to end.
struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Creates a fresh, empty project directory named for `label`.
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "adoc-cli-set-safe-mode-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create project dir");
        Self { dir }
    }

    /// The absolute path of `relative` within the project.
    fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }

    /// Writes `contents` to `relative`, creating parent directories as needed.
    fn write(&self, relative: &str, contents: &str) {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(path, contents).expect("write project file");
    }

    /// Runs `adoc` on the project's `input` file with `args` (with `-o -`
    /// forcing output to the captured stdout), returning the rendered HTML.
    fn run(&self, args: &[&str], input: &str) -> String {
        let input = self.path(input);
        let input = input.to_str().expect("input path is UTF-8");

        let mut full: Vec<&str> = vec!["adoc", "-o", "-"];
        full.extend_from_slice(args);
        full.push(input);

        let cli = Cli::parse_from(full);
        let mut stdout = Vec::new();
        run(&cli, &mut stdout).expect("adoc converts");
        String::from_utf8(stdout).expect("adoc output is UTF-8")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// The CLI default safe mode is `UNSAFE`. With no safe-mode flag, `adoc`
// resolves to `SafeMode::Unsafe`, which embeds the default stylesheet inline.
#[test]
fn the_cli_defaults_to_unsafe() {
    verifies!(
        r#"
= Set the Safe Mode Using the CLI
:navtitle: Set Safe Mode

When Asciidoctor is invoked via the CLI, the xref:ROOT:safe-modes.adoc[safe mode] is set to `UNSAFE` by default.

"#
    );

    assert_eq!(safe_mode_for(&["adoc", "doc.adoc"]), SafeMode::Unsafe);

    // End to end, the unsafe default embeds the stylesheet inline.
    let html = run_adoc("default", &[], "= Doc\n\nBody.");
    assert!(html.contains("<style>"));
}

// `-S`/`--safe-mode=SAFE_MODE` assigns the named level. `adoc` accepts each of
// `unsafe`, `safe`, `server`, and `secure` (case-insensitive) and resolves it
// to the matching `SafeMode`; end to end, `secure` links the stylesheet.
#[test]
fn safe_mode_flag_assigns_the_named_level() {
    verifies!(
        r#"
== Assign safe mode level

You can change the security level by executing one of the following commands:

`-S`, `--safe-mode=SAFE_MODE`::
Sets the safe mode level of the document according to the assigned level (`UNSAFE`, `SAFE`, `SERVER`, `SECURE`).

"#
    );

    for (name, mode) in [
        ("unsafe", SafeMode::Unsafe),
        ("safe", SafeMode::Safe),
        ("server", SafeMode::Server),
        ("secure", SafeMode::Secure),
    ] {
        // Both the long `--safe-mode=` and short `-S` forms assign the level.
        assert_eq!(
            safe_mode_for(&["adoc", &format!("--safe-mode={name}"), "doc.adoc"]),
            mode
        );
        assert_eq!(safe_mode_for(&["adoc", "-S", name, "doc.adoc"]), mode);
    }

    // End to end, `--safe-mode=secure` links the stylesheet instead of embedding.
    let html = run_adoc("secure", &["--safe-mode=secure"], "= Doc\n\nBody.");
    assert!(html.contains("./asciidoctor.css"));
    assert!(!html.contains("<style>"));
}

// `--safe` selects `SAFE`, for compatibility with the Python AsciiDoc `safe`
// command. (The separate `asciidoctor-safe` command alias has no `adoc`
// counterpart, so only the `--safe` flag is verified.)
#[test]
fn the_safe_flag_selects_safe() {
    verifies!(
        r#"
`--safe`, `asciidoctor-safe`::
Sets the safe mode level to `SAFE`.
Provided for compatibility with the python AsciiDoc `safe` command.

"#
    );

    assert_eq!(
        safe_mode_for(&["adoc", "--safe", "doc.adoc"]),
        SafeMode::Safe
    );
}

// `-B`/`--base-dir=DIR` sets the base directory, matching Asciidoctor. `adoc`
// accepts both forms into `Cli::base_dir`, and — since the base directory is
// the jail for `include::` resolution under a jailed safe mode — it can chroot
// the conversion: under `safe`, an include that climbs above the base directory
// is blocked, but widening the base directory with `-B` brings it back in
// reach.
#[test]
fn base_dir_option_sets_the_base_directory() {
    verifies!(
        r#"
////
-B, --base-dir=DIR
Base directory containing the document and resources. Defaults to the directory containing the source file, or the working directory if the source is read from a stream. Can be used as a way to chroot the execution of the program.
////

"#
    );

    // Both the short `-B` and long `--base-dir=` forms parse into `base_dir`.
    assert_eq!(
        Cli::parse_from(["adoc", "-B", "/docs/site", "doc.adoc"]).base_dir,
        Some(PathBuf::from("/docs/site"))
    );
    assert_eq!(
        Cli::parse_from(["adoc", "--base-dir=/docs/site", "doc.adoc"]).base_dir,
        Some(PathBuf::from("/docs/site"))
    );

    // The base directory is the jail for `include::` under a jailed safe mode.
    // A document in `base/` that includes `../note.adoc` reaches outside the
    // default base directory (its own folder), so under `safe` it is blocked...
    let project = TempProject::new("base-dir");
    project.write("base/main.adoc", "= Main\n\ninclude::../note.adoc[]\n");
    project.write("note.adoc", "Note from the parent directory.\n");

    let blocked = project.run(&["-S", "safe"], "base/main.adoc");
    assert!(!blocked.contains("Note from the parent directory."));

    // ...but pointing `-B` at the parent directory widens the jail to include
    // it, so the same include now resolves.
    let parent = project.dir.to_str().expect("path is UTF-8").to_owned();
    let allowed = project.run(&["-S", "safe", "-B", &parent], "base/main.adoc");
    assert!(allowed.contains("Note from the parent directory."));
}

// The closing cross-references carry no rule to verify here.
non_normative!(
    r#"
You can also set the xref:api:set-safe-mode.adoc[safe mode from the API] and xref:ROOT:reference-safe-mode.adoc[enable or disable content based on the current safe mode].
"#
);
