//! Unit coverage for `main.rs`'s input-resolution helpers: `resolve_inputs`,
//! which turns the command's positional arguments into the ordered list of
//! sources to convert (files, expanded globs, or standard input), and
//! `expand_glob`, which it calls to match a pattern. The page-driven suites
//! exercise the common file and glob cases; these cover the edges they do not:
//! a `-` mixed in with file arguments, and a pattern that is not valid UTF-8.

use std::path::PathBuf;

use crate::{expand_glob, resolve_inputs};

// A lone `-` (or no argument at all) reads standard input, but a `-` alongside
// other arguments is just one more source in the list: `adoc` resolves it to a
// standard-input source in place, leaving the surrounding file arguments
// intact.
#[test]
fn a_dash_among_file_arguments_is_a_stdin_source() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-dash-among-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create dir");
    let file = dir.join("a.adoc");
    std::fs::write(&file, "= A\n").expect("write file");

    let sources = resolve_inputs(&[file.clone(), PathBuf::from("-")]).expect("resolve inputs");
    let _ = std::fs::remove_dir_all(&dir);

    // The file resolves to itself, and the `-` resolves to standard input.
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0].file(), Some(file.as_path()));
    assert_eq!(sources[1].file(), None);
}

// `expand_glob` needs a UTF-8 pattern to hand to the glob matcher, so an
// argument that is not valid UTF-8 (and names no existing file, so it is
// treated as a pattern) is rejected as invalid input rather than silently
// ignored.
#[cfg(unix)]
#[test]
fn a_non_utf8_pattern_is_rejected() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

    // A lone continuation byte (0x80) never forms valid UTF-8, and no such file
    // exists, so resolution falls through to the glob path and fails there.
    let bad = PathBuf::from(OsStr::from_bytes(b"no-such-\x80-*.adoc"));

    let err = expand_glob(&bad).expect_err("non-UTF-8 pattern is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    // The same argument fails the whole resolution, since `resolve_inputs`
    // propagates the error.
    let err = resolve_inputs(&[bad]).expect_err("resolution fails");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}

// A glob can match directories as well as files; `expand_glob` keeps only the
// files, so a pattern never resolves to a directory `adoc` would then fail to
// read. Here `*` matches both a subdirectory and a file, and only the file
// comes back.
#[test]
fn a_directory_match_is_dropped() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-dir-drop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("subdir")).expect("create subdir");
    let file = dir.join("doc.adoc");
    std::fs::write(&file, "= Doc\n").expect("write file");

    let pattern = dir.join("*");
    let matches = expand_glob(&pattern).expect("expand glob");
    let _ = std::fs::remove_dir_all(&dir);

    // The subdirectory is dropped; only the file remains.
    assert_eq!(matches, vec![file]);
}

// A malformed glob pattern (here, a recursive `**` that is not a whole path
// component) is likewise reported as invalid input, rather than being taken as
// a literal file name.
#[test]
fn a_malformed_glob_pattern_is_rejected() {
    let bad = PathBuf::from("no-such-dir/**bad/*.adoc");

    let err = expand_glob(&bad).expect_err("malformed pattern is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    // And the error propagates out of resolution.
    let err = resolve_inputs(&[bad]).expect_err("resolution fails");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
