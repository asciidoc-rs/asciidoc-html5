//! Coverage for the read-error paths of `main.rs`'s conversion pipeline:
//! `run_with_input` propagating a resolve-inputs failure, and `read_input`
//! propagating a standard-input read failure. The success paths are exercised
//! throughout the page and `invoker_test.rb` suites; these drive the `?` error
//! edges those never trigger.

use clap::Parser as _;

use crate::{run_with_input, Cli};

// A malformed glob argument fails input resolution, and the error propagates
// out of `run_with_input` rather than being silently dropped. (`**bad` is a
// recursive wildcard that is not a whole path component, which the glob matcher
// rejects.)
#[test]
fn a_malformed_input_glob_fails_the_run() {
    let cli = Cli::parse_from(["adoc", "no-such-dir/**bad/*.adoc"]);
    let mut stdin = &b""[..];
    let mut stdout = Vec::new();
    let err =
        run_with_input(&cli, &mut stdin, &mut stdout).expect_err("a malformed glob fails the run");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        stdout.is_empty(),
        "nothing should be written on a resolution failure"
    );
}

// A standard-input stream that is not valid UTF-8 fails the read, and the error
// propagates out of the pipeline. With no input argument, `adoc` reads the
// document from standard input, so the invalid bytes reach `read_input`'s
// `read_to_string`.
#[test]
fn an_unreadable_stdin_fails_the_run() {
    let cli = Cli::parse_from(["adoc"]);
    let mut stdin = &[0xff, 0xfe, 0xfd][..];
    let mut stdout = Vec::new();
    let err = run_with_input(&cli, &mut stdin, &mut stdout)
        .expect_err("invalid UTF-8 on stdin fails the run");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        stdout.is_empty(),
        "nothing should be written on a read failure"
    );
}
