//! Unit coverage for `main.rs`'s path helpers: `output_dir`, which chooses
//! where `adoc` roots companion-file writes (the `copycss` stylesheet copy) for
//! a given output file, and `same_file`, which detects when the copy would
//! collide with the output file. The end-to-end copy and collision behavior is
//! exercised by the `docs` suites and the binary tests in `tests/cli.rs`.

use std::path::{Path, PathBuf};

use crate::{output_dir, same_file};

// A bare output file name has no directory component, so companion files are
// rooted at the current directory.
#[test]
fn bare_output_name_roots_at_the_current_directory() {
    assert_eq!(output_dir(Path::new("out.html")), PathBuf::from("."));
}

// An output file inside a directory roots companion files in that directory.
#[test]
fn output_in_a_directory_roots_there() {
    assert_eq!(
        output_dir(Path::new("public/out.html")),
        PathBuf::from("public")
    );
}

// Two spellings of the same path compare equal; distinct names do not.
#[test]
fn same_file_compares_absolute_forms() {
    assert!(same_file(
        Path::new("dir/out.css"),
        Path::new("dir/./out.css")
    ));
    assert!(!same_file(
        Path::new("dir/out.css"),
        Path::new("dir/other.css")
    ));
}

// When a path cannot be made absolute (an empty path), `same_file` reports no
// collision rather than erroring.
#[test]
fn same_file_is_false_when_a_path_cannot_be_absolutized() {
    assert!(!same_file(Path::new(""), Path::new("out.css")));
}
