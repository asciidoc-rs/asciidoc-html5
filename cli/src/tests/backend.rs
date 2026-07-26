//! Coverage for `main.rs`'s `-b`/`--backend` handling: `adoc` accepts `html5`
//! (and the default) for command-line compatibility with `asciidoctor -b html5
//! …`, and rejects every other backend Asciidoctor documents rather than
//! silently ignoring it. The `check_backend` gate is driven directly for the
//! parse edges, and `run_with_input` confirms an unsupported backend fails the
//! run before any output is produced.

use clap::Parser as _;

use crate::{check_backend, run_with_input, Cli};

/// Parses `args` and runs `check_backend`, returning its result.
fn check(args: &[&str]) -> std::io::Result<()> {
    check_backend(&Cli::parse_from(args))
}

// With no `-b`, there is no backend to validate, so the check passes: the HTML5
// backend is the only thing `adoc` produces, and it needs no opting in.
#[test]
fn the_default_backend_is_accepted() {
    check(&["adoc", "doc.adoc"]).expect("the default backend is accepted");
}

// `-b html5` (and its `--backend=` long form) is a no-op accepted for parity,
// so an existing `asciidoctor -b html5 …` invocation runs unchanged. The value
// is matched case-insensitively.
#[test]
fn the_html5_backend_is_accepted() {
    check(&["adoc", "-b", "html5", "doc.adoc"]).expect("-b html5 is accepted");
    check(&["adoc", "--backend=html5", "doc.adoc"]).expect("--backend=html5 is accepted");
    check(&["adoc", "-b", "HTML5", "doc.adoc"]).expect("-b HTML5 is accepted");
}

// Every other backend Asciidoctor documents — `xhtml5` included, with no
// special treatment — is not implemented here, so it is rejected with a message
// explaining that `adoc` only produces the HTML5 backend, rather than being
// silently ignored.
#[test]
fn other_backends_are_rejected() {
    for backend in ["xhtml5", "docbook5", "manpage", "pdf", "revealjs"] {
        let err = check(&["adoc", "-b", backend, "doc.adoc"])
            .expect_err("an unsupported backend is rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

        let message = err.to_string();
        assert!(message.contains(backend), "names the offending backend");
        assert!(message.contains("html5"), "explains only html5 is produced");
    }
}

// An unsupported backend fails the whole run before anything is read or
// converted, so nothing is written to standard output.
#[test]
fn an_unsupported_backend_fails_the_run() {
    let cli = Cli::parse_from(["adoc", "-b", "docbook5"]);
    let mut stdin = &b"= Doc\n\nBody.\n"[..];
    let mut stdout = Vec::new();

    let err = run_with_input(&cli, &mut stdin, &mut stdout)
        .expect_err("an unsupported backend fails the run");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        stdout.is_empty(),
        "nothing should be written when the backend is rejected"
    );
}
