//! Coverage for the input/output collision guard in `main.rs`'s conversion
//! pipeline: `adoc` refuses to write an output onto any input in the
//! invocation, so a conversion never truncates a source file. The
//! `invoker_test.rb` port covers the plain self-collision (`-o file` on `file`,
//! and an `outfilesuffix` landing on the input's own extension); these cover
//! the ways that guard could otherwise be bypassed — an output that aliases an
//! input through a symlink or a hard link, and an output that lands on a
//! *sibling* input in a multi-file run — plus a non-colliding batch, to guard
//! against false positives.

use std::path::PathBuf;

use clap::Parser as _;

use crate::{run, Cli};

/// A fresh, empty temp directory for one test `label`.
fn tempdir(label: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("adoc-cli-collision-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

// An output path that aliases the input through a symlink is rejected: writing
// through the link would follow it and truncate the source. The guard resolves
// symlinks, so it catches this where a plain path-string comparison would not,
// and the source is left intact.
#[cfg(unix)]
#[test]
fn an_output_symlinked_to_the_input_is_rejected_without_truncating_it() {
    let dir = tempdir("symlink-alias");
    let input = dir.join("doc.adoc");
    std::fs::write(&input, "= Doc\n\nOriginal source.\n").expect("write input");

    // `link.html` is a symlink to the input; `-o link.html` would write through
    // it onto `doc.adoc`.
    let link = dir.join("link.html");
    std::os::unix::fs::symlink(&input, &link).expect("create symlink");

    let cli = Cli::parse_from([
        "adoc",
        "-o",
        link.to_str().unwrap(),
        input.to_str().unwrap(),
    ]);
    let mut stdout = Vec::new();
    let err = run(&cli, &mut stdout).expect_err("an output aliasing the input is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    let after = std::fs::read_to_string(&input).expect("read input back");
    assert!(
        after.contains("Original source."),
        "the input was clobbered through the symlink: {after:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// An output path that is a *hard link* to the input shares the input's inode
// under a distinct pathname, so canonicalization alone would not equate them.
// The guard compares file identity (device + inode), so it is caught, and
// writing does not replace the shared inode's contents.
#[cfg(unix)]
#[test]
fn an_output_hard_linked_to_the_input_is_rejected_without_truncating_it() {
    let dir = tempdir("hardlink-alias");
    let input = dir.join("doc.adoc");
    std::fs::write(&input, "= Doc\n\nOriginal source.\n").expect("write input");

    // `out.html` is a second directory entry for the input's inode.
    let out = dir.join("out.html");
    std::fs::hard_link(&input, &out).expect("create hard link");

    let cli = Cli::parse_from(["adoc", "-o", out.to_str().unwrap(), input.to_str().unwrap()]);
    let mut stdout = Vec::new();
    let err = run(&cli, &mut stdout).expect_err("an output hard-linked to the input is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    let after = std::fs::read_to_string(&input).expect("read input back");
    assert!(
        after.contains("Original source."),
        "the input was clobbered through the hard link: {after:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// In a multi-file run, one source's resolved output must not land on a
// *sibling* input. Converting `a.txt` with `outfilesuffix=.adoc` derives
// `a.adoc`, which is also an input; the guard rejects the whole run before
// `a.txt` is converted, so the sibling source is not overwritten.
#[test]
fn an_output_matching_a_sibling_input_is_rejected_without_clobbering_it() {
    let dir = tempdir("cross-input");
    let first = dir.join("a.txt");
    let second = dir.join("a.adoc");
    std::fs::write(&first, "= A\n\nAlpha.\n").expect("write first input");
    std::fs::write(&second, "= Sibling\n\nKeep me.\n").expect("write sibling input");

    let cli = Cli::parse_from([
        "adoc",
        "-a",
        "outfilesuffix=.adoc",
        first.to_str().unwrap(),
        second.to_str().unwrap(),
    ]);
    let mut stdout = Vec::new();
    let err = run(&cli, &mut stdout).expect_err("an output landing on a sibling input is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    let after = std::fs::read_to_string(&second).expect("read sibling back");
    assert!(
        after.contains("Keep me."),
        "the sibling input was clobbered: {after:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// A batch whose outputs collide with no input converts normally — the guard
// does not fire on ordinary multi-file runs.
#[test]
fn a_batch_with_no_input_output_collision_converts() {
    let dir = tempdir("no-collision");
    let a = dir.join("a.adoc");
    let b = dir.join("b.adoc");
    std::fs::write(&a, "= A\n\nAlpha.\n").expect("write a");
    std::fs::write(&b, "= B\n\nBravo.\n").expect("write b");

    let cli = Cli::parse_from(["adoc", a.to_str().unwrap(), b.to_str().unwrap()]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("a batch with no input/output collision converts");

    assert!(dir.join("a.html").exists());
    assert!(dir.join("b.html").exists());

    let _ = std::fs::remove_dir_all(&dir);
}
