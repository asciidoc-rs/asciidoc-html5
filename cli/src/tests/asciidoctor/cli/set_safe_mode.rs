use asciidoc_html5::SafeMode;
use clap::Parser as _;

use crate::{resolve_safe_mode, run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/cli/pages/set-safe-mode.adoc");

// Asciidoctor's "Set the Safe Mode Using the CLI" page, tracked from the CLI
// crate. `adoc` mirrors Asciidoctor's CLI: the safe mode defaults to `UNSAFE`,
// `-S`/`--safe-mode` assigns a named level, and `--safe` selects `SAFE`. Each
// claim drives `adoc`'s own option parsing (`Cli` + `resolve_safe_mode`), and
// the default/secure cases are confirmed end to end through `run`.
//
// The hidden `-B`/`--base-dir` note (tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/44) and the
// `asciidoctor-safe` command alias have no counterpart in `adoc`, so those are
// non-normative.

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

// The hidden `-B`/`--base-dir` note (a base-directory chroot `adoc` does not
// provide, tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/44)
// and the closing cross-references carry no rule to verify here.
non_normative!(
    r#"
////
-B, --base-dir=DIR
Base directory containing the document and resources. Defaults to the directory containing the source file, or the working directory if the source is read from a stream. Can be used as a way to chroot the execution of the program.
////

You can also set the xref:api:set-safe-mode.adoc[safe mode from the API] and xref:ROOT:reference-safe-mode.adoc[enable or disable content based on the current safe mode].
"#
);
