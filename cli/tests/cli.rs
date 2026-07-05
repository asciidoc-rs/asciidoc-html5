//! End-to-end tests for the `adoc` binary.
//!
//! These drive the compiled command exactly as a user would, verifying the
//! baseline simplest case from the documentation's "Convert Your First File"
//! steps: `adoc document.adoc` converts a document and writes the HTML5 to a
//! file whose name is derived from the input.

use std::{fs, process::Command};

/// Runs the `adoc` binary on an AsciiDoc file and checks that a complete HTML5
/// document is written to a file whose name is derived from the input by
/// swapping its extension for `.html`, and that nothing is printed on success.
#[test]
fn converts_a_file_and_writes_html_to_derived_file() {
    let input = std::env::temp_dir().join(format!("adoc-cli-derive-{}.adoc", std::process::id()));
    let derived = input.with_extension("html");
    fs::write(&input, "= Hello\n\nWorld.").expect("write temp input");

    // The exact command shown in the "Convert Your First File" steps:
    // `adoc document.adoc`, with the output file name derived from the input.
    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .output()
        .expect("run the adoc binary");

    let html = fs::read_to_string(&derived).unwrap_or_default();
    let _ = fs::remove_file(&input);
    let _ = fs::remove_file(&derived);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );

    // On success adoc prints no messages, matching Asciidoctor.
    assert!(output.stdout.is_empty(), "adoc wrote to stdout on success");
    assert!(output.stderr.is_empty(), "adoc wrote to stderr on success");

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.contains("<p>World.</p>"));
}

/// Runs `adoc <input> -o -` and checks the HTML5 is written to standard output.
#[test]
fn writes_html_to_stdout_with_dash() {
    let input = std::env::temp_dir().join(format!("adoc-cli-stdout-{}.adoc", std::process::id()));
    fs::write(&input, "= Hello\n\nWorld.").expect("write temp input");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .arg("-o")
        .arg("-")
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

/// Runs `adoc --help` and checks the enriched help lists the usage examples.
#[test]
fn help_shows_usage_examples() {
    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg("--help")
        .output()
        .expect("run the adoc binary");

    assert!(
        output.status.success(),
        "adoc --help exited with {}",
        output.status
    );

    let help = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(help.contains("Examples:"));
    assert!(help.contains("adoc document.adoc -o out.html"));
    assert!(help.contains("read from standard input") || help.contains("standard input"));
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
