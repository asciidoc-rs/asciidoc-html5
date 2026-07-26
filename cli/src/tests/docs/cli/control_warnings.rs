use clap::Parser as _;

use crate::{run_with_streams, tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/control-warnings.adoc");

// This crate's "Control Warnings" page. It documents how `adoc` surfaces parser
// warnings: printed to standard error by default, silenced by `-q`, expanded to
// the info level by `-v`, turned into a non-zero exit code by
// `--failure-level`, and accepting Asciidoctor's `-w` as a no-op. Descriptive
// prose is tracked as non-normative; each `adoc` invocation and the behavior
// claimed for it is verified by driving the command end to end and inspecting
// the stream it writes to and the exit status it computes.

/// Pipes `source` through `adoc` with `args`, driving the fully injectable
/// [`run_with_streams`] core so both the warning stream and the failure-level
/// exit status are observable. Returns `(failure_reached, stdout, stderr)`.
fn run_piped(args: &[&str], source: &str) -> (bool, String, String) {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let failed =
        run_with_streams(&cli, &mut stdin, &mut stdout, &mut stderr).expect("adoc converts");
    (
        failed,
        String::from_utf8(stdout).expect("stdout is UTF-8"),
        String::from_utf8(stderr).expect("stderr is UTF-8"),
    )
}

non_normative!(
    r#"
= Control Warnings
:navtitle: Control Warnings
:description: How the adoc command reports parser warnings, and the options that silence them, add detail, or turn them into a failing exit code.

When `adoc` parses a document, anything ambiguous or likely unintended -- an
ordered list whose item numbers skip, a delimited block that is never closed, a
cross reference to an id that is not defined -- is reported as a warning. By
default `adoc` prints these warnings to standard error and still writes the
converted HTML, matching Asciidoctor.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// By default a document that provokes a warning prints one line to standard
// error — naming `<stdin>`, the line, and the condition — while the converted
// HTML goes to standard output, the two streams kept separate.
#[test]
fn warnings_print_to_standard_error_by_default() {
    verifies!(
        r#"
== Warnings print to standard error

Give a document that provokes a warning -- here an ordered list whose explicit
item numbers skip from 1 to 3 -- and `adoc` prints a warning line to standard
error while the HTML goes to standard output:

 $ printf '1. first\n3. third\n' | adoc -o -

The warning names the source (`<stdin>` for piped input, or the input file's
path), the line, and what was found:

 adoc: WARNING: <stdin>: line 2: list item index: expected 2, got 3

Because warnings go to standard error, they never mix into the converted HTML on
standard output.

"#
    );

    let (failed, stdout, stderr) = run_piped(&["-o", "-"], "1. first\n3. third\n");

    // The warning line is emitted verbatim on standard error.
    assert_eq!(
        stderr.trim_end(),
        "adoc: WARNING: <stdin>: line 2: list item index: expected 2, got 3"
    );

    // The HTML lands on standard output, with no warning text mixed in, and the
    // default failure level (FATAL) leaves the exit status successful.
    assert!(stdout.starts_with("<!DOCTYPE html>"));
    assert!(!stdout.contains("WARNING"));
    assert!(!failed);
}

// `-q`/`--quiet` suppresses the warning stream while still converting: the HTML
// is written but standard error stays empty.
#[test]
fn q_silences_warnings() {
    verifies!(
        r#"
== Silence warnings with -q

Pass `-q` (`--quiet`) to suppress all log output, warnings included. The
conversion still runs and its HTML is still written; only the messages are
silenced:

 $ printf '1. first\n3. third\n' | adoc -q -o -

"#
    );

    let (_failed, stdout, stderr) = run_piped(&["-q", "-o", "-"], "1. first\n3. third\n");
    assert!(stderr.is_empty(), "expected no output, got: {stderr}");
    assert!(stdout.starts_with("<!DOCTYPE html>"));
}

// `-v`/`--verbose` reports the lower, info-level diagnostics that are held back
// by default; `-q` and `-v` are mutually exclusive.
#[test]
fn v_shows_info_level_diagnostics() {
    verifies!(
        r#"
== Show more detail with -v

A few low-severity diagnostics -- a reference to a missing attribute, or a cross
reference whose target is not defined -- are held back by default, since they are
easily false positives. Pass `-v` (`--verbose`) to report them too, at the `INFO`
level:

 $ printf 'See {no-such-attr}.\n' | adoc -v -a attribute-missing=warn -o -

The reference above then reports:

 adoc: INFO: <stdin>: line 1: skipping reference to missing attribute: no-such-attr

`-q` and `-v` are opposites and cannot be combined.

"#
    );

    let source = "See {no-such-attr}.\n";

    // Under `-v`, the info-level diagnostic is reported verbatim.
    let (_failed, _stdout, stderr) =
        run_piped(&["-v", "-a", "attribute-missing=warn", "-o", "-"], source);
    assert_eq!(
        stderr.trim_end(),
        "adoc: INFO: <stdin>: line 1: skipping reference to missing attribute: no-such-attr"
    );

    // Without `-v`, the same run reports nothing: the info-level diagnostic is
    // held back by default.
    let (_failed, _stdout, default_stderr) =
        run_piped(&["-a", "attribute-missing=warn", "-o", "-"], source);
    assert!(default_stderr.is_empty(), "{default_stderr}");

    // `-q` and `-v` cannot be combined.
    let conflict = Cli::try_parse_from(["adoc", "-q", "-v"]).expect_err("-q and -v conflict");
    assert_eq!(conflict.kind(), clap::error::ErrorKind::ArgumentConflict);
}

// `--failure-level` turns diagnostics at or above the named level into a
// non-zero exit code (WARN catching any warning), and the level is evaluated
// even under `-q`, so the run can fail silently.
#[test]
fn failure_level_sets_the_exit_code() {
    verifies!(
        r#"
== Fail the build with --failure-level

By default a warning does not change the exit code: `adoc` reports it and exits
0. Pass `--failure-level` to make diagnostics at or above a level fail the run
instead, so a build script can stop on them. The accepted levels are `INFO`,
`WARN`, `ERROR`, and `FATAL` (the default, effectively never reached by a
parse).

Setting the level to `WARN` makes any warning exit non-zero:

 $ printf '1. first\n3. third\n' | adoc --failure-level=WARN -o -

The failure level is evaluated even under `-q`, so you can fail the build on a
warning without printing anything:

 $ printf '1. first\n3. third\n' | adoc -q --failure-level=WARN -o -

"#
    );

    let source = "1. first\n3. third\n";

    // Every documented level parses; `WARN` is case-insensitive and `WARNING`
    // is accepted as its alias.
    for level in ["INFO", "WARN", "warning", "ERROR", "FATAL"] {
        let arg = format!("--failure-level={level}");
        run_piped(&[&arg, "-o", "-"], source);
    }

    // `--failure-level=WARN` makes the warning fail the run (non-zero exit),
    // while the warning is still printed.
    let (failed, _stdout, stderr) = run_piped(&["--failure-level=WARN", "-o", "-"], source);
    assert!(failed);
    assert!(stderr.contains("WARNING"));

    // Adding `-q` keeps the failure but silences the message.
    let (failed, _stdout, stderr) = run_piped(&["-q", "--failure-level=WARN", "-o", "-"], source);
    assert!(failed);
    assert!(stderr.is_empty(), "{stderr}");

    // Without `--failure-level`, the default is never reached by the warning, so
    // the exit status stays successful.
    let (failed, _stdout, _stderr) = run_piped(&["-o", "-"], source);
    assert!(!failed);
}

// Asciidoctor's `-w`/`--warnings` is accepted for compatibility but has no
// effect: parser warnings print by default whether or not it is given.
#[test]
fn w_is_accepted_but_has_no_effect() {
    verifies!(
        r#"
== Compatibility with asciidoctor -w

Asciidoctor's `-w` (`--warnings`) turns on the Ruby interpreter's own script
warnings, which have no analog in this native binary. `adoc` accepts `-w` so
`asciidoctor -w` invocations keep working, but the flag has no effect: parser
warnings are printed by default regardless, and `-q` is what silences them.

 $ printf '1. first\n3. third\n' | adoc -w -o -
"#
    );

    let source = "1. first\n3. third\n";

    // `-w` parses into its flag and does not disturb the default output: the
    // warning is printed exactly as it is without the flag.
    assert!(Cli::parse_from(["adoc", "-w", "-o", "-", "-"]).warnings);
    let (_failed, _stdout, with_w) = run_piped(&["-w", "-o", "-"], source);
    let (_failed, _stdout, without_w) = run_piped(&["-o", "-"], source);
    assert!(with_w.contains("WARNING"));
    assert_eq!(with_w, without_w);
}
