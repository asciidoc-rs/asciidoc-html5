use clap::Parser as _;

use crate::{run_with_streams, tests::sdd::*, Cli, TEST_STDIN_NOT_A_TERMINAL};

track_file!("docs/modules/cli/pages/report-timings.adoc");

// This crate's "Report Timings" page. It documents `-t` (`--timings`): after
// each conversion `adoc` prints a timing report to standard error, breaking the
// time into reading-and-parsing, converting, and the total. Descriptive prose
// is tracked as non-normative; each `adoc` invocation and the report it
// produces is verified by driving the command end to end and inspecting the
// streams it writes.

/// Pipes `source` through `adoc` with `args`, returning `(stdout, stderr)` so
/// the timing report on standard error can be inspected apart from the HTML on
/// standard output.
fn run_piped(args: &[&str], source: &str) -> (String, String) {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with_streams(
        &cli,
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect("adoc converts");
    (
        String::from_utf8(stdout).expect("stdout is UTF-8"),
        String::from_utf8(stderr).expect("stderr is UTF-8"),
    )
}

/// Runs `adoc` with `args` (which already name real input files), reading no
/// standard input, and returns the standard-error text — where the per-input
/// timing reports land.
fn run_files(args: &[&str]) -> String {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = std::io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run_with_streams(
        &cli,
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect("adoc converts");
    String::from_utf8(stderr).expect("stderr is UTF-8")
}

non_normative!(
    r#"
= Report Timings
:navtitle: Report Timings
:description: How the adoc command reports the time a conversion took, with the -t option.

By default `adoc` converts a document and says nothing about how long the work
took. Pass `-t` (`--timings`) to print a short timing report to standard error
after each input is converted, matching Asciidoctor.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// `-t`/`--timings` prints a report to standard error naming the input (`-` for
// piped stdin) and breaking the time into read-and-parse, convert, and the
// total. The figures vary run to run, so the labels — not the numbers — are
// what is verified, and the report stays out of the HTML on standard output.
#[test]
fn report_a_conversions_timings() {
    verifies!(
        r#"
== Report a conversion's timings

Pass `-t` (`--timings`) to report the time `adoc` spent on a conversion. The
report goes to standard error, so it never mixes into the converted HTML on
standard output:

 $ printf 'Sample *AsciiDoc*' | adoc -t -o -

The report names the input (`-` for piped input, or the input file's path) and
breaks the time down into reading and parsing the source, converting it to
HTML5, and the total of the two:

 Input file: -
   Time to read and parse source: 0.00042
   Time to convert document: 0.00013
   Total time (read, parse and convert): 0.00055

The individual figures vary from run to run, so only the report's shape is fixed:
the input name, then three labeled lines, each a count of seconds.

"#
    );

    let (stdout, stderr) = run_piped(&["-t", "-o", "-"], "Sample *AsciiDoc*");

    // The report names the piped input `-` and carries the three labeled lines.
    assert!(stderr.contains("Input file: -"), "{stderr}");
    assert!(
        stderr.contains("Time to read and parse source:"),
        "{stderr}"
    );
    assert!(stderr.contains("Time to convert document:"), "{stderr}");
    assert!(
        stderr.contains("Total time (read, parse and convert):"),
        "{stderr}"
    );

    // The HTML lands on standard output, with no timing report mixed into it.
    assert!(stdout.starts_with("<!DOCTYPE html>"));
    assert!(!stdout.contains("Total time"));
}

// The timing report is not a parser warning, so `-q` does not silence it: a
// quiet run of a warning-provoking document still prints the report while
// suppressing the warning.
#[test]
fn timings_are_independent_of_q() {
    verifies!(
        r#"
== Timings are independent of -q

The timing report is not a warning, so `-q` (`--quiet`) does not silence it. A
quiet run still prints the report, having suppressed only the parser's warnings:

 $ printf 'Sample *AsciiDoc*' | adoc -t -q -o -

"#
    );

    // The documented invocation: `-q` leaves the timing report in place.
    let (_stdout, stderr) = run_piped(&["-t", "-q", "-o", "-"], "Sample *AsciiDoc*");
    assert!(stderr.contains("Total time"), "{stderr}");

    // And `-q` still does its own job: a document that would warn (an ordered
    // list whose numbers skip) prints the report but not the warning.
    let (_stdout, stderr) = run_piped(&["-t", "-q", "-o", "-"], "1. first\n3. third\n");
    assert!(stderr.contains("Total time"), "{stderr}");
    assert!(!stderr.contains("WARNING"), "{stderr}");
}

// Converting several files in one invocation prints one report per input, each
// labeled with that file's path.
#[test]
fn a_report_per_input() {
    verifies!(
        r#"
== A report per input

Converting several files in one invocation prints a separate report for each,
labeled with that file's path, so you can see where the time went file by file.
"#
    );

    let dir = std::env::temp_dir().join(format!("adoc-cli-timings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let one = dir.join("one.adoc");
    let two = dir.join("two.adoc");
    std::fs::write(&one, "First *file*.\n").expect("write one.adoc");
    std::fs::write(&two, "Second *file*.\n").expect("write two.adoc");

    let stderr = run_files(&["-t", one.to_str().unwrap(), two.to_str().unwrap()]);

    // Each input gets its own report, labeled with its path, so both paths and
    // two totals appear.
    assert!(
        stderr.contains(&format!("Input file: {}", one.display())),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("Input file: {}", two.display())),
        "{stderr}"
    );
    assert_eq!(
        stderr.matches("Total time").count(),
        2,
        "one report per input: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
