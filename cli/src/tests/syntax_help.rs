//! Coverage for `main.rs`'s `--help syntax` topic: `adoc --help syntax` prints
//! an AsciiDoc syntax crib sheet, matching Asciidoctor's `--help syntax`. The
//! `syntax_help_topic` detector is driven directly for the argument forms it
//! recognizes (and the ones it must not), and the embedded crib sheet is
//! confirmed to be valid AsciiDoc that this crate renders without falling back
//! to any `unsupported` marker — the guarantee that lets
//! `adoc --help syntax | adoc -o -` produce an accurate HTML5 preview.

use std::ffi::OsString;

use crate::{syntax_help_topic, SYNTAX_CRIB_SHEET};

/// Builds the `OsString` argument vector `syntax_help_topic` inspects.
fn args(argv: &[&str]) -> Vec<OsString> {
    argv.iter().map(OsString::from).collect()
}

// The topic is recognized whether it trails the flag as its own argument or is
// joined with `=`, in both the long and short spellings — the forms the issue
// (#31) and Asciidoctor's CLI accept.
#[test]
fn recognizes_the_syntax_topic_in_every_form() {
    for argv in [
        ["adoc", "--help", "syntax"].as_slice(),
        ["adoc", "-h", "syntax"].as_slice(),
        ["adoc", "--help=syntax"].as_slice(),
        ["adoc", "-h=syntax"].as_slice(),
    ] {
        assert!(
            syntax_help_topic(&args(argv)),
            "should detect the syntax topic in {argv:?}"
        );
    }
}

// Plain help, an unrecognized topic, and an ordinary conversion are all left to
// clap: only the `syntax` topic is intercepted, so everything else follows the
// usual path (a bare `--help` renders usage, as does an unknown topic).
#[test]
fn leaves_other_invocations_to_clap() {
    for argv in [
        ["adoc", "--help"].as_slice(),
        ["adoc", "-h"].as_slice(),
        ["adoc", "--help", "manpage"].as_slice(),
        ["adoc", "syntax"].as_slice(),
        ["adoc", "document.adoc"].as_slice(),
        ["adoc"].as_slice(),
    ] {
        assert!(
            !syntax_help_topic(&args(argv)),
            "should not detect the syntax topic in {argv:?}"
        );
    }
}

// The crib sheet leads with the document title, so the printed output is itself
// a well-formed AsciiDoc document.
#[test]
fn the_crib_sheet_is_an_asciidoc_document() {
    assert!(
        SYNTAX_CRIB_SHEET.starts_with("= AsciiDoc Syntax\n"),
        "the crib sheet should open with its document title"
    );
}

// The whole point of the topic: the crib sheet is valid AsciiDoc that this
// crate's own renderer converts, so `adoc --help syntax | adoc -o -` yields an
// accurate HTML5 preview. Because the crib sheet is scoped to supported
// constructs, that render must contain no `unsupported` fallback marker.
#[test]
fn the_crib_sheet_renders_without_unsupported_markers() {
    let html = asciidoc_html5::convert(SYNTAX_CRIB_SHEET);

    assert!(
        html.contains("<h2 id=\"_paragraphs\">Paragraphs</h2>"),
        "the crib sheet should render its section headings"
    );
    assert!(
        !html.contains("unsupported"),
        "every construct in the crib sheet should render, but found an unsupported marker:\n{html}"
    );
}
