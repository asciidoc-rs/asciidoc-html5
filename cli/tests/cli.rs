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

/// Runs `adoc` on `source` from standard input with the given extra arguments,
/// returning the exit status, standard output, and standard error. Used by the
/// `-a`/`--attribute` tests, which need only vary the arguments and the source.
fn run_adoc(args: &[&str], source: &str) -> (std::process::ExitStatus, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the adoc binary");

    // `adoc` validates its arguments before it reads standard input, so an
    // invalid invocation (see `empty_attribute_name_is_rejected`) can exit
    // before consuming the source, breaking the pipe mid-write. That is
    // expected here; the meaningful assertions are on the exit status and the
    // captured output. Any other write error is a genuine failure. Drop stdin
    // afterward so the child sees EOF (otherwise `wait_with_output` deadlocks).
    let mut stdin = child.stdin.take().expect("child stdin is piped");
    if let Err(err) = stdin.write_all(source.as_bytes()) {
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::BrokenPipe,
            "write to child stdin: {err}"
        );
    }
    drop(stdin);

    let output = child.wait_with_output().expect("wait for the adoc binary");
    (
        output.status,
        String::from_utf8(output.stdout).expect("stdout is UTF-8"),
        String::from_utf8(output.stderr).expect("stderr is UTF-8"),
    )
}

/// A custom `stylesheet` set in the document header is read from the input
/// file's directory and embedded, under the `adoc` default (`unsafe`) mode.
/// This is the end-to-end custom-stylesheet path: the binary resolves the file
/// on disk without any extra flags.
#[test]
fn custom_stylesheet_is_embedded_from_the_input_directory() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-css-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("doc.adoc");
    fs::write(&input, "= Doc\n:stylesheet: my-theme.css\n\nBody.").expect("write input");
    fs::write(dir.join("my-theme.css"), "body { color: #ff0000; }\n").expect("write stylesheet");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .arg("-o")
        .arg("-")
        .output()
        .expect("run the adoc binary");
    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );
    assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
    // A custom stylesheet suppresses the default stylesheet and the web fonts.
    assert!(!html.contains("/*! Asciidoctor default stylesheet"));
    assert!(!html.contains("fonts.googleapis.com"));
}

/// Under `--safe-mode=secure`, the same custom stylesheet is *linked* (at its
/// normalized web path) rather than read from disk.
#[test]
fn custom_stylesheet_is_linked_under_secure() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-css-link-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("doc.adoc");
    fs::write(&input, "= Doc\n:stylesheet: my-theme.css\n\nBody.").expect("write input");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .arg("--safe-mode=secure")
        .arg("-o")
        .arg("-")
        .output()
        .expect("run the adoc binary");
    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./my-theme.css\">"));
    assert!(!html.contains("<style>"));
}

/// `-a name=value` supplies a document attribute and, being an override, wins
/// over an assignment of the same name in the document header.
#[test]
fn attribute_override_beats_the_document_header() {
    let source = "= Doc\n:webfonts: from-header\n\nBody.";
    let (status, html, _) = run_adoc(&["-a", "webfonts=from-cli", "-o", "-"], source);

    assert!(status.success(), "adoc exited with {status}");
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=from-cli\">"
    ));
}

/// `-a name=value@` is a soft default, so a document-header assignment of the
/// same name wins over it.
#[test]
fn soft_attribute_yields_to_the_document_header() {
    let source = "= Doc\n:webfonts: from-header\n\nBody.";
    let (status, html, _) = run_adoc(&["-a", "webfonts=from-cli@", "-o", "-"], source);

    assert!(status.success(), "adoc exited with {status}");
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=from-header\">"
    ));
}

/// `-a name!` unsets an attribute; here it drops the web-font `<link>`.
#[test]
fn attribute_unset_drops_the_web_font_link() {
    let (status, html, _) = run_adoc(&["-a", "webfonts!", "-o", "-"], "= Doc\n\nBody.");

    assert!(status.success(), "adoc exited with {status}");
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
    // The default stylesheet is still embedded.
    assert!(html.contains("<style>"));
}

/// The leading-bang form `-a !name` unsets an attribute too, matching
/// Asciidoctor's alternative to the trailing `name!`.
#[test]
fn attribute_leading_bang_unsets() {
    let (status, html, _) = run_adoc(&["-a", "!webfonts", "-o", "-"], "= Doc\n\nBody.");

    assert!(status.success(), "adoc exited with {status}");
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
    // The default stylesheet is still embedded.
    assert!(html.contains("<style>"));
}

/// A `!` on either end of the key unsets the attribute and discards any
/// `=value`, matching Asciidoctor: `-a name!=value` and `-a !name=value` both
/// unset `name` rather than assigning the value.
#[test]
fn bang_key_unsets_and_discards_value() {
    for spec in ["webfonts!=X:400", "!webfonts=X:400"] {
        let (status, html, _) = run_adoc(&["-a", spec, "-o", "-"], "= Doc\n\nBody.");

        assert!(status.success(), "adoc exited with {status} for -a {spec}");
        assert!(
            !html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"),
            "-a {spec} should unset webfonts and drop the font link"
        );
        // The discarded value must not leak through as a font family.
        assert!(
            !html.contains("family=X:400"),
            "-a {spec} must discard the value"
        );
    }
}

/// `-a linkcss` links the stylesheet instead of embedding it.
#[test]
fn attribute_set_links_the_stylesheet() {
    let (status, html, _) = run_adoc(&["-a", "linkcss", "-o", "-"], "= Doc\n\nBody.");

    assert!(status.success(), "adoc exited with {status}");
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!html.contains("<style>"));
}

/// The soft-toggle forms `-a name@` (set) and `-a name!@` (unset) apply when
/// the document is silent but yield to a document assignment of the same name.
#[test]
fn soft_toggle_attributes_yield_to_the_document() {
    // `-a linkcss@` softly sets `linkcss`: with the document silent, the
    // stylesheet is linked...
    let (status, linked, _) = run_adoc(&["-a", "linkcss@", "-o", "-"], "= Doc\n\nBody.");
    assert!(status.success(), "adoc exited with {status}");
    assert!(linked.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));

    // ...but the document can turn it back off, so the stylesheet is embedded.
    let (_, relinked, _) = run_adoc(&["-a", "linkcss@", "-o", "-"], "= Doc\n:linkcss!:\n\nBody.");
    assert!(!relinked.contains("./asciidoctor.css"));
    assert!(relinked.contains("<style>"));

    // `-a webfonts!@` softly unsets `webfonts`: with the document silent, the
    // web-font link is gone...
    let (_, unfonted, _) = run_adoc(&["-a", "webfonts!@", "-o", "-"], "= Doc\n\nBody.");
    assert!(!unfonted.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

    // ...but a document assignment of `webfonts` wins.
    let (_, refonted, _) = run_adoc(
        &["-a", "webfonts!@", "-o", "-"],
        "= Doc\n:webfonts: X:400\n\nBody.",
    );
    assert!(refonted.contains(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=X:400\">"
    ));
}

/// When the output file cannot be written (its parent directory does not
/// exist), adoc fails with a nonzero exit status and an `adoc:`-prefixed error,
/// writing nothing to standard output.
#[test]
fn reports_failure_when_output_cannot_be_written() {
    let input =
        std::env::temp_dir().join(format!("adoc-cli-badout-in-{}.adoc", std::process::id()));
    // A destination inside a directory that does not exist, so the write fails.
    let unwritable = std::env::temp_dir()
        .join(format!("adoc-cli-missing-dir-{}", std::process::id()))
        .join("out.html");
    fs::write(&input, "= Hello\n\nWorld.").expect("write temp input");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .arg("-o")
        .arg(&unwritable)
        .output()
        .expect("run the adoc binary");
    let _ = fs::remove_file(&input);

    assert!(
        !output.status.success(),
        "adoc should fail when the output cannot be written, but exited with {}",
        output.status
    );
    assert!(output.stdout.is_empty(), "adoc wrote to stdout on failure");

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.contains("adoc:"),
        "error should be prefixed with `adoc:`, got: {stderr}"
    );
}

/// An unrecognized `-S`/`--safe-mode` value is rejected with a nonzero exit
/// status and an `adoc:`-prefixed error naming the offending mode.
#[test]
fn invalid_safe_mode_is_rejected() {
    let (status, stdout, stderr) = run_adoc(&["-S", "bogus", "-o", "-"], "= Doc\n\nBody.");

    assert!(
        !status.success(),
        "adoc should reject an unrecognized safe mode"
    );
    assert!(stdout.is_empty(), "adoc wrote to stdout on failure");
    assert!(
        stderr.contains("adoc:")
            && stderr.contains("invalid safe mode")
            && stderr.contains("bogus"),
        "error should name the invalid safe mode, got: {stderr}"
    );
}

/// A shared docinfo file sitting next to the input is read from disk (via the
/// base directory `adoc` derives from the input path) and its content is
/// injected at each of the three positions: the head docinfo lands at the
/// bottom of `<head>`, the header docinfo just before `<div id="header">`, and
/// the footer docinfo just after the footer `<div>`. This exercises docinfo
/// resolution end to end.
#[test]
fn docinfo_files_are_read_from_disk_and_injected() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-docinfo-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create docinfo dir");
    let input = dir.join("document.adoc");
    let derived = dir.join("document.html");

    fs::write(&input, "= Doc\n:docinfo: shared\n\nBody.").expect("write input");
    fs::write(dir.join("docinfo.html"), "<meta name=\"di-head\">\n").expect("write head docinfo");

    fs::write(
        dir.join("docinfo-header.html"),
        "<div class=\"di-header\"></div>\n",
    )
    .expect("write header docinfo");

    fs::write(
        dir.join("docinfo-footer.html"),
        "<p class=\"di-footer\"></p>\n",
    )
    .expect("write footer docinfo");

    let status = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .status()
        .expect("run the adoc binary");

    assert!(status.success(), "adoc exited with {status}");

    let html = fs::read_to_string(&derived).unwrap_or_default();
    let _ = fs::remove_dir_all(&dir);

    // Head docinfo appears flush above `</head>` (its trailing newline chomped).
    assert!(
        html.contains("<meta name=\"di-head\">\n</head>"),
        "head docinfo not placed at the bottom of <head>: {html}"
    );

    // Header docinfo appears immediately before the header div.
    assert!(
        html.contains("<div class=\"di-header\"></div>\n<div id=\"header\">"),
        "header docinfo not placed before the header div"
    );

    // Footer docinfo appears immediately after the footer div, before `</body>`.
    assert!(
        html.contains("</div>\n<p class=\"di-footer\"></p>\n</body>"),
        "footer docinfo not placed after the footer div"
    );
}

/// Under the `secure` safe mode, docinfo files are not read even when the
/// `docinfo` attribute is set and the file exists — matching Asciidoctor, which
/// disables docinfo at `secure` and above.
#[test]
fn secure_safe_mode_disables_docinfo() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-docinfo-secure-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create docinfo dir");
    let input = dir.join("document.adoc");

    fs::write(&input, "= Doc\n:docinfo: shared\n\nBody.").expect("write input");
    fs::write(dir.join("docinfo.html"), "<meta name=\"di-head\">\n").expect("write head docinfo");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .args(["-S", "secure", "-o", "-"])
        .output()
        .expect("run the adoc binary");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );

    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    assert!(
        !html.contains("di-head"),
        "docinfo should be disabled under the secure safe mode"
    );
}

/// Under the jailed `--safe` mode, a docinfo file inside the base directory is
/// still read and injected — the jail confines reads to the base directory but
/// does not forbid them, matching Asciidoctor.
#[test]
fn safe_mode_still_reads_in_base_docinfo() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-docinfo-safe-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create docinfo dir");
    let input = dir.join("document.adoc");

    fs::write(&input, "= Doc\n:docinfo: shared\n\nBody.").expect("write input");
    fs::write(dir.join("docinfo.html"), "<meta name=\"di-head\">\n").expect("write head docinfo");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .args(["--safe", "-o", "-"])
        .output()
        .expect("run the adoc binary");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );

    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    assert!(
        html.contains("<meta name=\"di-head\">\n</head>"),
        "in-base docinfo should still be injected under --safe"
    );
}

/// Under the `server` safe mode, a document that enables docinfo itself
/// (`:docinfo: shared` in its header) is ignored — Asciidoctor's SERVER
/// "prevents the document from setting … docinfo". The file is present but must
/// not be read.
#[test]
fn server_ignores_document_set_docinfo() {
    let dir = std::env::temp_dir().join(format!("adoc-cli-docinfo-server-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create docinfo dir");
    let input = dir.join("document.adoc");

    fs::write(&input, "= Doc\n:docinfo: shared\n\nBody.").expect("write input");
    fs::write(dir.join("docinfo.html"), "<meta name=\"di-head\">\n").expect("write head docinfo");

    let output = Command::new(env!("CARGO_BIN_EXE_adoc"))
        .arg(&input)
        .args(["-S", "server", "-o", "-"])
        .output()
        .expect("run the adoc binary");

    let _ = fs::remove_dir_all(&dir);

    assert!(
        output.status.success(),
        "adoc exited with {}",
        output.status
    );

    let html = String::from_utf8(output.stdout).expect("stdout is UTF-8");

    assert!(
        !html.contains("di-head"),
        "a document-set docinfo must be ignored under the server safe mode"
    );
}

/// A `-a` spec with no attribute name is rejected with a nonzero exit status
/// and an `adoc:`-prefixed error.
#[test]
fn empty_attribute_name_is_rejected() {
    let (status, stdout, stderr) = run_adoc(&["-a", "=value", "-o", "-"], "= Doc\n\nBody.");

    assert!(
        !status.success(),
        "adoc should reject an empty attribute name"
    );
    assert!(stdout.is_empty(), "adoc wrote to stdout on failure");
    assert!(
        stderr.contains("adoc:") && stderr.contains("attribute name"),
        "error should explain the missing attribute name, got: {stderr}"
    );
}
