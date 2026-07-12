//! Unit coverage for `output_dir`, the helper that chooses where `adoc` roots
//! companion-file writes (the `copycss` stylesheet copy) for a given output
//! file. The end-to-end copy is exercised by the `docs` suites and by the
//! binary tests in `tests/cli.rs`.

use std::path::{Path, PathBuf};

use crate::output_dir;

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
