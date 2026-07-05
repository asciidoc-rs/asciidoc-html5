//! End-to-end tests for the `adoc` binary.
//!
//! These drive the compiled command exactly as a user would: the simplest case
//! from the documentation's "Convert Your First File" steps (`adoc
//! document.adoc` writes the HTML5 to a file whose name is derived from the
//! input), the `-o`/`-o -` variants, reading from standard input, and the
//! failure path when the input cannot be read.

use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

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

/// Pipes AsciiDoc into the binary with no input argument and checks that the
/// HTML5 is read from standard input and written to standard output — the
/// `cat document.adoc | adoc` case.
#[test]
fn reads_stdin_and_writes_html_to_stdout() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn the adoc binary");

    child
        .stdin
        .take()
        .expect("child stdin is piped")
        .write_all(b"= Hello\n\nWorld.")
        .expect("write to child stdin");

    let output = child.wait_with_output().expect("wait for the adoc binary");

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

/// Runs the binary on a file that does not exist and checks it fails with a
/// nonzero exit status and an `adoc:`-prefixed error on standard error, writing
/// nothing to standard output — the documented "exit status 1 if the input
/// cannot be read" behavior.
#[test]
fn reports_failure_when_input_cannot_be_read() {
    let missing =
        std::env::temp_dir().join(format!("adoc-cli-missing-{}.adoc", std::process::id()));
    // Make sure the input really is absent, whatever ran before.
    let _ = fs::remove_file(&missing);

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&missing)
        .output()
        .expect("run the adoc binary");

    assert!(
        !output.status.success(),
        "adoc should fail on a missing input file, but exited with {}",
        output.status
    );
    assert!(output.stdout.is_empty(), "adoc wrote to stdout on failure");

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.contains("adoc:"),
        "error should be prefixed with `adoc:`, got: {stderr}"
    );
}
