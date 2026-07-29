//! Coverage for the warning-reporting internals of `main.rs`: the severity
//! labels, the `--failure-level` parser's error path, the `writeln!` error edge
//! when the warning stream fails (and the matching edge when the
//! bare-invocation usage summary cannot be written), and the include-origin
//! branch that attributes a warning to the file a nested directive actually
//! lives in. The happy paths are exercised by the `invoker_test.rb` and docs
//! "Control Warnings" suites; these drive the edges those never reach.

use std::io::{self, Read, Write};

use clap::Parser as _;

use crate::{parse_failure_level, run_with_streams, Cli, LogLevel, TEST_STDIN_NOT_A_TERMINAL};

// Each severity renders the label Asciidoctor's basic formatter prints for it —
// `WARN` as `WARNING` and `FATAL` as `FAILED`. Only `Info` and `Warn` are ever
// displayed (parser diagnostics never reach the `Error`/`Fatal` levels), so the
// higher labels are covered here rather than through a rendered warning line.
#[test]
fn each_severity_has_its_asciidoctor_label() {
    assert_eq!(LogLevel::Info.label(), "INFO");
    assert_eq!(LogLevel::Warn.label(), "WARNING");
    assert_eq!(LogLevel::Error.label(), "ERROR");
    assert_eq!(LogLevel::Fatal.label(), "FAILED");
}

// A `--failure-level` value that names no known level is rejected, so the flag
// surfaces a clear error rather than silently defaulting.
#[test]
fn an_unknown_failure_level_is_rejected() {
    let err = parse_failure_level("bogus").expect_err("an unknown level is rejected");
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("invalid failure level 'bogus'"));

    // Every documented level parses, in either the `warn`/`warning` spelling and
    // regardless of case.
    for level in ["info", "WARN", "Warning", "error", "FATAL"] {
        assert!(parse_failure_level(level).is_ok(), "{level} should parse");
    }
}

/// A [`Write`] that fails every write, used to drive the error edge where
/// emitting a warning to the standard-error stream fails.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "stderr is closed",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "stderr is closed",
        ))
    }
}

// When a warning must be printed but the warning stream fails, the write error
// propagates out of the run rather than being swallowed.
#[test]
fn a_failing_warning_stream_propagates_the_error() {
    // A piped document with an out-of-sequence list item raises a warning that
    // the default (non-quiet) reporter tries to print.
    let cli = Cli::parse_from(["adoc", "-o", "-", "-"]);
    let mut stdin: &[u8] = b"1. first\n3. third\n";
    let mut stdout = Vec::new();
    let mut stderr = FailingWriter;

    let err = run_with_streams(
        &cli,
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect_err("a failing warning stream fails the run");
    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

// The usage summary printed for a bare invocation at a terminal goes to the
// same stream; when that write fails, the error propagates rather than being
// dropped.
#[test]
fn a_failing_usage_stream_propagates_the_error() {
    // No input argument with `stdin_is_terminal` set diverts to printing usage on
    // stderr, which here refuses every write.
    let cli = Cli::parse_from(["adoc"]);
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = FailingWriter;

    let err = run_with_streams(&cli, true, &mut stdin, &mut stdout, &mut stderr)
        .expect_err("a failing usage stream fails the run");
    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

/// Captures a run's `(stdout, stderr)` from a file input, reading no standard
/// input (the file-based invocations here never touch it).
fn run_file(args: &[&str]) -> (bool, String) {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let failed = run_with_streams(
        &cli,
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin as &mut dyn Read,
        &mut stdout,
        &mut stderr,
    )
    .expect("adoc converts");
    (failed, String::from_utf8(stderr).expect("stderr is UTF-8"))
}

// A directive buried in an include-expanded AsciiDoc table cell carries its own
// pre-resolved origin, so its warning is attributed to the file and line the
// directive actually lives in — not the cell's line in the primary document.
// Here an owned cell (its first line an `include::`) pulls in a file whose
// unterminated conditional raises the warning, which is reported against that
// included file.
#[test]
fn a_warning_from_an_included_directive_names_its_origin_file() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-warn-origin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(
        dir.join("main.adoc"),
        "|===\na|include::cond.adoc[]\n|===\n",
    )
    .expect("write main");
    std::fs::write(dir.join("cond.adoc"), "ifdef::never[]\nbody\n").expect("write include");
    let main = dir.join("main.adoc");

    let (_failed, stderr) = run_file(&[main.to_str().unwrap(), "-o", "-"]);
    let _ = std::fs::remove_dir_all(&dir);

    // The warning names `cond.adoc` (the included file), line 1 — the origin the
    // parser pre-resolved for the buried directive — rather than the cell's line
    // in `main.adoc`.
    assert!(
        stderr.contains("cond.adoc: line 1:"),
        "expected the warning to name the included file's origin, got: {stderr}"
    );
    assert!(stderr.contains("unterminated preprocessor conditional directive"));
}
