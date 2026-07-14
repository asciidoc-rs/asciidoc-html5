use std::path::{Path, PathBuf};

use clap::Parser as _;

use crate::{output_dir, output_target, run, run_with_input, tests::sdd::*, Cli, OutputTarget};

track_file!("ref/asciidoctor/docs/modules/cli/pages/output-file.adoc");

// Asciidoctor's "Specify an Output File" page, tracked from the CLI crate.
// `adoc` mirrors this part of the interface: with no output option it derives
// the output file name from the input (extension swapped for `.html`) and
// writes it beside the input; `-o` names the output file (a relative path
// resolved against the current directory, so specifying it also fixes the
// companion-file directory); `-D` names the output directory while the file
// name defaults; and a piped document writes to standard output unless `-o`
// names a file. Each invocation drives `adoc`'s own routing (`Cli` plus the
// private `output_target`/`output_dir` helpers), and the file-writing cases are
// confirmed end to end.
//
// One passage stays non-normative: `asciidoctor -D build *.adoc` converts many
// input files in one run. `adoc` converts a single document per invocation, so
// it has no matching multi-input behavior to verify; the divergence is noted
// where the passage appears.

/// The output file `adoc` would write this invocation's HTML to, or `None` when
/// it writes to standard output.
fn output_file(args: &[&str]) -> Option<PathBuf> {
    match output_target(&Cli::parse_from(args)) {
        OutputTarget::File(path) => Some(path),
        OutputTarget::Stdout => None,
    }
}

/// Whether `adoc` would send this invocation's rendered HTML to standard
/// output.
fn goes_to_stdout(args: &[&str]) -> bool {
    matches!(output_target(&Cli::parse_from(args)), OutputTarget::Stdout)
}

/// A unique temporary directory for one test `label`, freshly emptied.
fn sandbox(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "adoc-cli-output-file-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create sandbox");
    dir
}

/// Runs `adoc` with the full `args` (input file included), returning its
/// stdout.
fn run_argv(args: &[&str]) -> Vec<u8> {
    let cli = Cli::parse_from(args);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts");
    stdout
}

/// Pipes `source` through `adoc` as standard input, returning captured stdout.
fn run_piped(args: &[&str], source: &str) -> Vec<u8> {
    let cli = Cli::parse_from(args);
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("adoc converts");
    stdout
}

non_normative!(
    r#"
= Specify an Output File
// Included in user-manual: Specifying an output file

"#
);

// With no output option, `adoc` derives the output file name from the input by
// swapping its extension for `.html` and writes it in the same directory as the
// input, exactly as Asciidoctor does.
#[test]
fn default_output_is_derived_beside_the_input() {
    verifies!(
        r#"
By default, the Asciidoctor CLI writes the converted output file to the same directory as the input file.
If an output file is not specified, the name of the output file is derived from the input file by replacing its file extension with the file extension that matches the output format (e.g., replacing .adoc with .html).

"#
    );

    // The derived name keeps the input's directory, with `.adoc` swapped for
    // `.html`.
    assert_eq!(
        output_file(&["adoc", "sub/mydoc.adoc"]),
        Some(PathBuf::from("sub/mydoc.html"))
    );

    // End to end: with no `-o`, the HTML lands beside the input under the derived
    // name, and nothing goes to stdout.
    let dir = sandbox("default");
    let input = dir.join("mydoc.adoc");
    std::fs::write(&input, "= Doc\n\nBody.\n").expect("write input");
    let stdout = run_argv(&["adoc", input.to_str().unwrap()]);
    assert!(stdout.is_empty(), "adoc wrote to stdout instead of a file");
    let html = std::fs::read_to_string(dir.join("mydoc.html")).expect("read derived output");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

non_normative!(
    r#"
You can instruct the Asciidoctor CLI to write content to a different output file (or directory).
There are several circumstances when you'll want to specify a different output file:

* You want to write the output file to a different name, perhaps to append a qualifier such as a version string.
* You want to write the output file to a different directory.
* You are piping content to the CLI, but want to write the output to a file (in this case, an output file is required).

"#
);

// A relative `-o` path is taken relative to the current directory, not the
// input's directory: `-o out.html` for an input under `sub/` routes to
// `out.html`, not `sub/out.html`. And because companion files are rooted at the
// output file's own directory, naming the output implicitly names that
// directory too.
#[test]
fn relative_output_is_resolved_against_the_current_directory() {
    verifies!(
        r#"
CAUTION: If you specify the output file as a relative path, it will be resolved relative to the current working directory instead of the directory of the input file (i.e., specifying the output file implicitly sets the output directory too).

"#
    );

    // The relative `-o` value is not joined onto the input's directory.
    assert_eq!(
        output_file(&["adoc", "-o", "out.html", "sub/mydoc.adoc"]),
        Some(PathBuf::from("out.html"))
    );

    // Specifying the output file sets the directory companion files are written
    // to: a folder-prefixed output roots them at that folder, and a bare name
    // roots them at the current directory.
    assert_eq!(
        output_dir(Path::new("build/out.html")),
        PathBuf::from("build")
    );
    assert_eq!(output_dir(Path::new("out.html")), PathBuf::from("."));
}

// The `-o` option names the output file directly. `adoc` routes `-o <name>` to
// that file and, end to end, writes the HTML there while leaving stdout empty.
#[test]
fn the_output_option_names_the_output_file() {
    verifies!(
        r#"
To specify the output file, you'll use the `-o` option.
For example, let's say we want to convert [.path]_mydoc.adoc_ and write the output to a filename that includes the current date.
You'd use:

 $ asciidoctor -o mydoc-$(date +%Y-%m-%d).html mydoc.adoc

"#
    );

    assert_eq!(
        output_file(&["adoc", "-o", "mydoc-2024-01-01.html", "mydoc.adoc"]),
        Some(PathBuf::from("mydoc-2024-01-01.html"))
    );

    // End to end, `-o <name>` writes the HTML to that file.
    let dir = sandbox("output-option");
    let input = dir.join("mydoc.adoc");
    std::fs::write(&input, "= Doc\n\nBody.\n").expect("write input");
    let out = dir.join("mydoc-2024-01-01.html");
    let stdout = run_argv(&["adoc", "-o", out.to_str().unwrap(), input.to_str().unwrap()]);
    assert!(
        stdout.is_empty(),
        "adoc wrote to stdout instead of the file"
    );
    let html = std::fs::read_to_string(&out).expect("read output file");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

// Prefixing the output file with a folder name writes it into that folder.
// `adoc` keeps the folder-prefixed path and creates the folder if it does not
// yet exist.
#[test]
fn a_folder_prefixed_output_writes_into_that_folder() {
    verifies!(
        r#"
We could write it to another folder as well by prefixing the output file with a folder name:

 $ asciidoctor -o build/mydoc-$(date +%Y-%m-%d).html mydoc.adoc

"#
    );

    assert_eq!(
        output_file(&["adoc", "-o", "build/mydoc-2024-01-01.html", "mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc-2024-01-01.html"))
    );

    // End to end: the `build/` folder does not exist yet; `adoc` creates it and
    // writes the HTML inside.
    let dir = sandbox("output-folder");
    let input = dir.join("mydoc.adoc");
    std::fs::write(&input, "= Doc\n\nBody.\n").expect("write input");
    let out = dir.join("build").join("mydoc-2024-01-01.html");
    let stdout = run_argv(&["adoc", "-o", out.to_str().unwrap(), input.to_str().unwrap()]);
    assert!(
        stdout.is_empty(),
        "adoc wrote to stdout instead of the file"
    );
    let html = std::fs::read_to_string(&out).expect("read output file");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

// The `-D` option names the output directory while the file name defaults to
// the derived `.html` name. `adoc` places the derived name in that directory,
// creating it if needed.
#[test]
fn the_destination_dir_option_defaults_the_filename() {
    verifies!(
        r#"
If you only want to specify the output directory, but let the filename be defaulted, use the `-D` option:

 $ asciidoctor -D build mydoc.adoc

"#
    );

    // The derived name is placed in the `-D` directory, using the input's base
    // name regardless of the input's own directory.
    assert_eq!(
        output_file(&["adoc", "-D", "build", "mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc.html"))
    );
    assert_eq!(
        output_file(&["adoc", "-D", "build", "sub/mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc.html"))
    );

    // End to end: the destination directory does not exist yet; `adoc` creates it
    // and writes the derived name inside.
    let dir = sandbox("destination-dir");
    let input = dir.join("mydoc.adoc");
    std::fs::write(&input, "= Doc\n\nBody.\n").expect("write input");
    let build = dir.join("build");
    let stdout = run_argv(&[
        "adoc",
        "-D",
        build.to_str().unwrap(),
        input.to_str().unwrap(),
    ]);
    assert!(stdout.is_empty(), "adoc wrote to stdout instead of a file");
    let html = std::fs::read_to_string(build.join("mydoc.html")).expect("read derived output");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

// Divergence: Asciidoctor's `-D` also applies when several input files are
// given at once. `adoc` converts a single document per invocation, so it has no
// multi-input form to verify; run `adoc -D <dir>` once per file instead.
non_normative!(
    r#"
The `-D` option can also be used when processing multiple input files:

 $ asciidoctor -D build *.adoc

"#
);

// When piping, `adoc` writes to standard output by default; to capture the
// output in a file you must name one with `-o`, just as with Asciidoctor.
#[test]
fn piping_writes_stdout_unless_an_output_file_is_named() {
    verifies!(
        r#"
If you are piping content to the CLI, the default is to write the output to STDOUT.
If you want to write the output to a file in this case, you have to specify one:

 $ cat mydoc.adoc | asciidoctor -o build/mydoc-$(date +%Y-%m-%d).html -

"#
    );

    // With `-`, and no `-o`, the piped document goes to standard output.
    assert!(goes_to_stdout(&["adoc", "-"]));
    let stdout = run_piped(&["adoc", "-"], "= Doc\n\nBody.\n");
    let html = String::from_utf8(stdout).expect("output is UTF-8");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<p>Body.</p>"));

    // Naming a file with `-o` captures the piped output there instead.
    assert_eq!(
        output_file(&["adoc", "-o", "build/out.html", "-"]),
        Some(PathBuf::from("build/out.html"))
    );
    let dir = sandbox("piping");
    let out = dir.join("build").join("out.html");
    let cli = Cli::parse_from(["adoc", "-o", out.to_str().unwrap(), "-"]);
    let mut stdin = "= Doc\n\nBody.\n".as_bytes();
    let mut sink = Vec::new();
    run_with_input(&cli, &mut stdin, &mut sink).expect("adoc converts");
    assert!(sink.is_empty(), "adoc wrote to stdout instead of the file");
    let html = std::fs::read_to_string(&out).expect("read output file");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

non_normative!(
    r#"
See xref:io-piping.adoc[] to learn more.
"#
);
