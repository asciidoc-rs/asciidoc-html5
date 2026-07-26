//! Unit coverage for `main.rs`'s path helpers: `output_dir`, which chooses
//! where `adoc` roots companion-file writes (the `copycss` stylesheet copy) for
//! a given output file; `same_file`, which detects when the copy would collide
//! with the output file; and `relative_subdir`, the `-R`/`--source-dir` routing
//! that computes an input's directory beneath the source root. The end-to-end
//! copy and collision behavior is exercised by the `docs` suites and the binary
//! tests in `tests/cli.rs`.

use std::path::{Path, PathBuf};

use crate::{output_dir, relative_subdir, same_file};

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

// `-R` routing: an input nested under the source root yields its subdirectory
// relative to that root, which `-D` then mirrors.
#[test]
fn relative_subdir_returns_the_nested_directory() {
    assert_eq!(
        relative_subdir(
            Path::new("/work/src"),
            Path::new("/work/src/sub/index.adoc")
        ),
        Some(PathBuf::from("sub"))
    );
}

// An input directly inside the source root has no subdirectory to mirror, so
// the relative directory is empty and joining it onto `-D` leaves it unchanged.
#[test]
fn relative_subdir_is_empty_for_an_input_in_the_root() {
    assert_eq!(
        relative_subdir(Path::new("/work/src"), Path::new("/work/src/index.adoc")),
        Some(PathBuf::new())
    );
}

// `..` components are collapsed before the comparison, matching Asciidoctor's
// `File.expand_path`. An input whose `..` climbs out of the source root is not
// treated as living under it — so its mirrored output cannot escape `-D` —
// while a `..` that stays within the root resolves to the real nested
// directory.
#[test]
fn relative_subdir_collapses_parent_components() {
    // `src/../outside` resolves out of the root, so it is not under it.
    assert_eq!(
        relative_subdir(
            Path::new("/work/src"),
            Path::new("/work/src/../outside/doc.adoc")
        ),
        None
    );

    // `a/../sub` resolves back inside the root, to the real `sub` subdirectory.
    assert_eq!(
        relative_subdir(
            Path::new("/work/src"),
            Path::new("/work/src/a/../sub/doc.adoc")
        ),
        Some(PathBuf::from("sub"))
    );
}

// An input entirely outside the source root has no relative subdirectory, so
// `-R` leaves it to the plain `-D` behavior.
#[test]
fn relative_subdir_is_none_outside_the_root() {
    assert_eq!(
        relative_subdir(Path::new("/work/src"), Path::new("/other/doc.adoc")),
        None
    );
}
