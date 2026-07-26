//! Coverage for `main.rs`'s `-d`/`--doctype` handling: `adoc` accepts `article`
//! (and the default) for command-line compatibility with `asciidoctor -d
//! article …`, and rejects the other doctypes Asciidoctor defines (`book`,
//! `manpage`, `inline`) rather than silently ignoring them. The `check_doctype`
//! gate is driven directly for the parse edges, and `run_with_input` confirms
//! an unsupported doctype fails the run before any output is produced.

use clap::Parser as _;

use crate::{check_doctype, run_with_input, Cli};

/// Parses `args` and runs `check_doctype`, returning its result.
fn check(args: &[&str]) -> std::io::Result<()> {
    check_doctype(&Cli::parse_from(args))
}

// With no `-d`, there is no doctype to validate, so the check passes: `article`
// is the only doctype `adoc` models, and it needs no opting in.
#[test]
fn the_default_doctype_is_accepted() {
    check(&["adoc", "doc.adoc"]).expect("the default doctype is accepted");
}

// `-d article` (and its `--doctype=` long form) is a no-op accepted for parity,
// so an existing `asciidoctor -d article …` invocation runs unchanged. The
// value is matched case-insensitively.
#[test]
fn the_article_doctype_is_accepted() {
    check(&["adoc", "-d", "article", "doc.adoc"]).expect("-d article is accepted");
    check(&["adoc", "--doctype=article", "doc.adoc"]).expect("--doctype=article is accepted");
    check(&["adoc", "-d", "ARTICLE", "doc.adoc"]).expect("-d ARTICLE is accepted");
}

// The other doctypes Asciidoctor defines — `book`, `manpage`, `inline` — are
// not modeled here, so each is rejected with a message explaining that `adoc`
// only produces the article doctype, rather than being silently ignored.
#[test]
fn other_doctypes_are_rejected() {
    for doctype in ["book", "manpage", "inline"] {
        let err = check(&["adoc", "-d", doctype, "doc.adoc"])
            .expect_err("an unsupported doctype is rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

        let message = err.to_string();
        assert!(message.contains(doctype), "names the offending doctype");
        assert!(
            message.contains("article"),
            "explains only article is produced"
        );
    }
}

// An unsupported doctype fails the whole run before anything is read or
// converted, so nothing is written to standard output.
#[test]
fn an_unsupported_doctype_fails_the_run() {
    let cli = Cli::parse_from(["adoc", "-d", "book"]);
    let mut stdin = &b"= Doc\n\nBody.\n"[..];
    let mut stdout = Vec::new();

    let err = run_with_input(&cli, &mut stdin, &mut stdout)
        .expect_err("an unsupported doctype fails the run");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        stdout.is_empty(),
        "nothing should be written when the doctype is rejected"
    );
}
