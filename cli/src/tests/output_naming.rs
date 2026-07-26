//! Unit coverage for `main.rs`'s output-name helpers: `output_suffix`, which
//! reads the effective `outfilesuffix` from the `-a` specs, and
//! `derive_output_path`, which swaps an input's extension for that suffix. The
//! page-driven and `invoker_test.rb` suites exercise the common cases through
//! `run`; these cover the parsing edges they do not — a bare attribute, a
//! non-matching key, a `!`-guarded key, the soft-default `@`, repeated
//! assignments, and a suffix given without its leading dot.

use std::path::{Path, PathBuf};

use clap::Parser as _;

use crate::{build_options, derive_output_path, output_suffix, Cli};

/// The suffix `adoc` would derive output names with for the given `-a` specs.
fn suffix_for(specs: &[&str]) -> String {
    let mut argv = vec!["adoc"];
    for spec in specs {
        argv.push("-a");
        argv.push(spec);
    }
    argv.push("doc.adoc");
    output_suffix(&Cli::parse_from(argv))
}

// With no `outfilesuffix` assignment, the derived suffix defaults to `.html`.
// A bare attribute (no `=`) and an unrelated `key=value` both leave the default
// untouched — the first skips the value branch entirely, the second fails the
// name match.
#[test]
fn suffix_defaults_to_html_without_an_outfilesuffix_value() {
    assert_eq!(suffix_for(&[]), ".html");
    assert_eq!(suffix_for(&["linkcss"]), ".html");
    assert_eq!(suffix_for(&["docname=other"]), ".html");
}

// An `outfilesuffix=value` assignment sets the suffix, matched
// case-insensitively (the library lowercases attribute names), and the last
// assignment wins.
#[test]
fn outfilesuffix_value_sets_the_suffix() {
    assert_eq!(suffix_for(&["outfilesuffix=.htm"]), ".htm");
    assert_eq!(suffix_for(&["OUTFILESUFFIX=.HTM"]), ".HTM");
    assert_eq!(suffix_for(&["outfilesuffix=.a", "outfilesuffix=.b"]), ".b");
}

// A trailing `@` marks a soft default; it is stripped before the value is read,
// so the suffix is still taken.
#[test]
fn outfilesuffix_soft_default_still_sets_the_suffix() {
    assert_eq!(suffix_for(&["outfilesuffix=.soft@"]), ".soft");
}

// A `!` on either end of the key makes the spec an unset, which carries no
// usable suffix, so the default is kept and the `=value` is ignored.
#[test]
fn a_bang_guarded_outfilesuffix_key_keeps_the_default() {
    assert_eq!(suffix_for(&["outfilesuffix!=.htm"]), ".html");
    assert_eq!(suffix_for(&["!outfilesuffix=.htm"]), ".html");
}

// `derive_output_path` swaps the input's extension for the suffix, accepting
// the suffix with or without its leading dot, and writing beside the input when
// no destination directory is given.
#[test]
fn derive_output_path_swaps_the_extension() {
    assert_eq!(
        derive_output_path(Path::new("sub/doc.adoc"), None, ".html"),
        PathBuf::from("sub/doc.html")
    );
    assert_eq!(
        derive_output_path(Path::new("sub/doc.adoc"), None, ".htm"),
        PathBuf::from("sub/doc.htm")
    );

    // A suffix given without the leading dot is treated the same.
    assert_eq!(
        derive_output_path(Path::new("doc.adoc"), None, "html"),
        PathBuf::from("doc.html")
    );
}

// With a destination directory, the derived base name is placed inside it,
// dropping the input's own directory.
#[test]
fn derive_output_path_honors_the_destination_directory() {
    assert_eq!(
        derive_output_path(Path::new("sub/doc.adoc"), Some(Path::new("build")), ".html"),
        PathBuf::from("build/doc.html")
    );
}

// An attribute spec with no name is rejected. `build_options` surfaces the
// `validate_name` error in process (the end-to-end path spawns the binary, so
// it otherwise leaves this branch unexercised in the library's own coverage).
#[test]
fn build_options_rejects_a_nameless_attribute() {
    let err = build_options(false, &["=value".to_string()])
        .expect_err("a nameless attribute is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("attribute name"));

    // A bare `!` is likewise nameless once the unset marker is stripped.
    let err = build_options(false, &["!".to_string()]).expect_err("a bare bang is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    // An empty spec is a nameless bare key (no value, no unset marker).
    let err = build_options(false, &[String::new()]).expect_err("an empty spec is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
