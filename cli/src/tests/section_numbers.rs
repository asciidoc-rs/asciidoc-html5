//! Coverage for `main.rs`'s `-n`/`--section-numbers` handling: the flag is
//! Asciidoctor's shorthand for setting the `sectnums` attribute, so `adoc -n`
//! numbers section headings (`1.`, `2.`, …) the way `asciidoctor -n` does. The
//! run is driven end to end through `run_with_input`, converting a piped
//! document and inspecting the rendered headings, and the flag's precedence
//! against an explicit `-a sectnums!` on the same command line is pinned down.

use clap::Parser as _;

use crate::{run_with_input, Cli};

/// A two-section document with no `sectnums` set in its own header, so whether
/// the headings are numbered is decided entirely by the command line.
const TWO_SECTIONS: &[u8] = b"= Doc\n\n== One\n\n== Two\n";

/// Converts `TWO_SECTIONS` under `args` (an `adoc …` command line) and returns
/// the rendered HTML.
fn convert(args: &[&str]) -> String {
    let cli = Cli::parse_from(args);
    let mut stdin = TWO_SECTIONS;
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("conversion succeeds");
    String::from_utf8(stdout).expect("output is UTF-8")
}

// Without `-n`, and with no `sectnums` in the document, section headings render
// unnumbered — the same baseline `asciidoctor` produces.
#[test]
fn headings_are_unnumbered_by_default() {
    let html = convert(&["adoc", "-o", "-"]);
    assert!(
        html.contains(r#"<h2 id="_one">One</h2>"#),
        "the first heading is unnumbered: {html}"
    );
    assert!(
        html.contains(r#"<h2 id="_two">Two</h2>"#),
        "the second heading is unnumbered: {html}"
    );
}

// `-n` sets the `sectnums` attribute, so each heading is prefixed with its
// dotted section number, matching `asciidoctor -n`.
#[test]
fn dash_n_numbers_the_headings() {
    let html = convert(&["adoc", "-n", "-o", "-"]);
    assert!(
        html.contains(r#"<h2 id="_one">1. One</h2>"#),
        "the first heading is numbered `1.`: {html}"
    );
    assert!(
        html.contains(r#"<h2 id="_two">2. Two</h2>"#),
        "the second heading is numbered `2.`: {html}"
    );
}

// The `--section-numbers` long form is equivalent to `-n`.
#[test]
fn the_long_form_numbers_the_headings() {
    let html = convert(&["adoc", "--section-numbers", "-o", "-"]);
    assert!(
        html.contains(r#"<h2 id="_one">1. One</h2>"#),
        "the long form numbers the first heading: {html}"
    );
    assert!(
        html.contains(r#"<h2 id="_two">2. Two</h2>"#),
        "the long form numbers the second heading: {html}"
    );
}

// `-n` is seeded before the `-a` specs, so an explicit `-a sectnums!` on the
// same command line still wins and leaves the headings unnumbered — matching
// `asciidoctor -n -a sectnums!`.
#[test]
fn an_explicit_unset_overrides_dash_n() {
    let html = convert(&["adoc", "-n", "-a", "sectnums!", "-o", "-"]);
    assert!(
        html.contains(r#"<h2 id="_one">One</h2>"#),
        "`-a sectnums!` overrides `-n`, leaving the heading unnumbered: {html}"
    );
}
