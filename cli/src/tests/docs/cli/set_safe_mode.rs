use asciidoc_html5::SafeMode;
use clap::Parser as _;

use crate::{resolve_safe_mode, run, tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/set-safe-mode.adoc");

// This crate's "Set the Safe Mode Using the CLI" page. It documents `adoc`'s
// `-S`/`--safe-mode` option, the `--safe` shorthand, and the `unsafe` default.
// Each invocation is verified through `adoc`'s own option parsing (`Cli` +
// `resolve_safe_mode`), with the default and `secure` cases confirmed end to
// end through `run`.

/// Resolves the safe mode `adoc` would use for the given command-line
/// arguments.
fn safe_mode_for(args: &[&str]) -> SafeMode {
    let cli = Cli::parse_from(args);
    resolve_safe_mode(&cli).expect("valid safe mode")
}

/// Runs `adoc` on `source` with `args` (a temp source file is appended, and
/// `-o -` forces output to the captured stdout).
fn run_adoc(label: &str, args: &[&str], source: &str) -> String {
    let path = std::env::temp_dir().join(format!(
        "adoc-docs-set-safe-mode-{label}-{}.adoc",
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

non_normative!(
    r#"
= Set the Safe Mode Using the CLI
:navtitle: Set Safe Mode
:description: How to choose the safe mode when converting with the adoc command.

When you run the `adoc` command, the default xref:ROOT:safe-modes.adoc[safe mode]
is `unsafe`. Choose a different mode with `-S`/`--safe-mode`, or the `--safe`
shorthand.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// `-S`/`--safe-mode` assigns any of the four modes; `secure` links the
// stylesheet.
#[test]
fn safe_mode_flag_assigns_the_mode() {
    verifies!(
        r#"
== Assign the safe mode

`-S`, `--safe-mode=SAFE_MODE`::
Set the safe mode to `unsafe`, `safe`, `server`, or `secure` (case-insensitive).
For example, `secure` links the default stylesheet instead of embedding it:
+
 $ adoc -S secure document.adoc

"#
    );

    for (name, mode) in [
        ("unsafe", SafeMode::Unsafe),
        ("safe", SafeMode::Safe),
        ("server", SafeMode::Server),
        ("secure", SafeMode::Secure),
    ] {
        assert_eq!(safe_mode_for(&["adoc", "-S", name, "doc.adoc"]), mode);
    }

    // `-S secure` links the stylesheet instead of embedding it.
    let html = run_adoc("secure", &["-S", "secure"], "= Doc\n\nBody.");
    assert!(html.contains("./asciidoctor.css"));
    assert!(!html.contains("<style>"));
}

// `--safe` is shorthand for `--safe-mode=safe`.
#[test]
fn the_safe_shorthand_selects_safe() {
    verifies!(
        r#"
`--safe`::
Shorthand that sets the safe mode to `safe`. Cannot be combined with
`--safe-mode`.

"#
    );

    assert_eq!(
        safe_mode_for(&["adoc", "--safe", "doc.adoc"]),
        SafeMode::Safe
    );
}

// With no safe-mode option, `adoc` runs `unsafe`, which embeds the stylesheet.
#[test]
fn the_default_is_unsafe() {
    verifies!(
        r#"
With no safe-mode option, `adoc` runs `unsafe`, which embeds the default
stylesheet.

"#
    );

    assert_eq!(safe_mode_for(&["adoc", "doc.adoc"]), SafeMode::Unsafe);

    let html = run_adoc("default", &[], "= Doc\n\nBody.");
    assert!(html.contains("<style>"));
}

non_normative!(
    r#"
You can also set the xref:api:set-safe-mode.adoc[safe mode from the API].
"#
);
