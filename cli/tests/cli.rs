//! End-to-end tests for the `adoc` binary.
//!
//! These drive the compiled command exactly as a user would, verifying the
//! baseline simplest case from the documentation's "Basic usage" section:
//! `adoc document.adoc` converts a document and writes the HTML5 to stdout.

use std::{fs, process::Command};

/// Runs the `adoc` binary on an AsciiDoc file and checks that a complete HTML5
/// document is written to standard output.
#[test]
fn converts_a_file_and_writes_html_to_stdout() {
    let input = std::env::temp_dir().join(format!("adoc-cli-stdout-{}.adoc", std::process::id()));
    fs::write(&input, "= Hello\n\nWorld.").expect("write temp input");

    // The exact command shown on the introduction page: `adoc document.adoc`.
    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .output()
        .expect("run the adoc binary");
    let _ = fs::remove_file(&input);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );

    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<p>World.</p>"));
}

/// Runs `adoc <input> -o <output>` and checks the HTML5 lands in the file.
#[test]
fn writes_html_to_the_output_file() {
    let input = std::env::temp_dir().join(format!("adoc-cli-in-{}.adoc", std::process::id()));
    let output_path =
        std::env::temp_dir().join(format!("adoc-cli-out-{}.html", std::process::id()));
    fs::write(&input, "= Hello\n\nWorld.").expect("write temp input");

    let status = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .arg("-o")
        .arg(&output_path)
        .status()
        .expect("run the adoc binary");
    let html = fs::read_to_string(&output_path).unwrap_or_default();
    let _ = fs::remove_file(&input);
    let _ = fs::remove_file(&output_path);

    assert!(status.success(), "adoc exited with {status}");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
}
