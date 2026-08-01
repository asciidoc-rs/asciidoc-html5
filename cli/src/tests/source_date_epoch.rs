//! Unit coverage for `main.rs`'s reproducible-clock plumbing: `unix_seconds`,
//! which converts a file's modification time to the epoch count that seeds the
//! `doc*` attributes; `resolve_source_date_epoch`, which applies the
//! `SOURCE_DATE_EPOCH` rules to a raw value; and the pinning path in
//! `run_with_streams_using`, which overrides an input file's mtime.
//!
//! These drive the resolved value directly rather than mutating the process
//! environment: `SOURCE_DATE_EPOCH` is global and read on every in-process
//! `adoc` run, so setting it would race the rest of the test binary (and a
//! multi-conversion test comparing two outputs would desync). The end-to-end
//! rendering of the footer stamp these feed is exercised by the `invoker_test`
//! port and the `html5` renderer tests.

use std::time::{Duration, UNIX_EPOCH};

use asciidoc_html5::ReferenceTime;
use clap::Parser as _;

use crate::{
    resolve_source_date_epoch, run_with_streams_using, unix_seconds, Cli, TEST_STDIN_NOT_A_TERMINAL,
};

// `unix_seconds` counts forward from the Unix epoch for a later instant.
#[test]
fn unix_seconds_counts_forward_from_the_epoch() {
    assert_eq!(unix_seconds(UNIX_EPOCH), 0);
    assert_eq!(
        unix_seconds(UNIX_EPOCH + Duration::from_secs(1_234_123_412)),
        1_234_123_412
    );
}

// `unix_seconds` returns a negative count for an instant before the epoch – a
// source file whose modification time predates 1970 – flooring a fractional
// pre-epoch instant toward the earlier second rather than truncating toward
// zero (which would round it up by a second).
#[test]
fn unix_seconds_is_negative_before_the_epoch() {
    assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_secs(1)), -1);
    assert_eq!(
        unix_seconds(UNIX_EPOCH - Duration::from_secs(86_400)),
        -86_400
    );

    // Fractional instants floor down: -0.5s -> -1, -1.5s -> -2.
    assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_millis(500)), -1);
    assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_millis(1_500)), -2);
}

// A valid raw value resolves to exactly its instant. `1234123412` is
// 2009-02-08T20:03:32Z.
#[test]
fn resolve_reads_a_valid_value() {
    assert_eq!(
        resolve_source_date_epoch(Some("1234123412".into())).expect("valid epoch"),
        Some(ReferenceTime::from_unix_timestamp(1_234_123_412))
    );
}

// An absent, empty, or all-whitespace value is ignored (the clock stays
// unpinned), matching Asciidoctor's `nil_or_empty?` guard.
#[test]
fn resolve_ignores_absent_or_empty_values() {
    assert!(matches!(resolve_source_date_epoch(None), Ok(None)));
    assert!(matches!(
        resolve_source_date_epoch(Some("".into())),
        Ok(None)
    ));
    assert!(matches!(
        resolve_source_date_epoch(Some("   ".into())),
        Ok(None)
    ));
}

// A malformed value fails resolution (which `main` turns into a non-zero exit)
// rather than being ignored.
#[test]
fn resolve_rejects_a_malformed_value() {
    let result = resolve_source_date_epoch(Some("not-an-epoch".into()));
    assert!(result.is_err(), "a malformed value must fail resolution");
    assert_eq!(
        result.expect_err("malformed epoch errors").kind(),
        std::io::ErrorKind::InvalidInput
    );
}

// A pinned epoch overrides the input file's modification time: with the clock
// resolved to 2009-02-08T20:03:32Z, the footer stamp reflects that instant, not
// the freshly written file's mtime.
#[test]
fn a_pinned_epoch_overrides_the_input_file_mtime() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-sde-pin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    let input = dir.join("sample.adoc");
    std::fs::write(&input, "= Sample\n\nBody.\n").expect("write input");

    let cli = Cli::parse_from([
        "adoc",
        input.to_str().expect("temp path is UTF-8"),
        "-o",
        "-",
    ]);
    let mut stdin = std::io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    run_with_streams_using(
        &cli,
        Some(ReferenceTime::from_unix_timestamp(1_234_123_412)),
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect("adoc converts");

    let output = String::from_utf8(stdout).expect("output is UTF-8");

    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        output.contains("Last updated 2009-02-08 20:03:32 UTC"),
        "a pinned epoch should drive the footer stamp: {output}"
    );
}
