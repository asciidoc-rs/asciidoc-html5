use std::path::{Path, PathBuf};

use clap::Parser as _;

use crate::{output_dir, output_target, run, run_with_input, tests::sdd::*, Cli, OutputTarget};

track_file!("docs/modules/cli/pages/output-file.adoc");

// This crate's "Specify an Output File" page. It documents how `adoc` chooses
// where to write: with no output option it derives the `.html` name and writes
// beside the input; `-o` names the output file (a relative path resolved
// against the current directory, which also fixes the companion-file
// directory); `-D` names the output directory while the file name defaults, and
// a relative `-o` is then resolved inside it; `-D` applies per input, so it
// also governs a multi-file conversion; and a piped document writes to standard
// output unless `-o` names a file. Each invocation is verified through `adoc`'s
// own routing (`Cli` plus the private `output_target`/`output_dir` helpers)
// and, for the file-writing cases, end to end.

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
        "adoc-docs-output-file-{label}-{}",
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
:navtitle: Specify an Output File
:description: How to choose the output file or directory for the HTML5 that the adoc command writes.

"#
);

// With no output option, `adoc mydoc.adoc` derives the `.html` name from the
// input and writes it in the input's own directory.
#[test]
fn default_output_is_derived_beside_the_input() {
    verifies!(
        r#"
By default, the `adoc` command writes the converted HTML5 to the same directory
as the input file, deriving the output file name from the input by replacing its
file extension with `.html`. So `adoc mydoc.adoc` writes [.path]_mydoc.html_
alongside the input.

"#
    );

    // The derived name keeps the input's directory, `.adoc` swapped for `.html`.
    assert_eq!(
        output_file(&["adoc", "sub/mydoc.adoc"]),
        Some(PathBuf::from("sub/mydoc.html"))
    );

    // End to end, the HTML lands beside the input under the derived name.
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
[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

You can instruct `adoc` to write the output to a different file or directory.
That is useful when you want to give the output a different name (perhaps
appending a version string), write it to a different directory, or capture piped
content, which has no input file name to derive an output name from.

"#
);

// `-o` names the output file; prefixing it with a folder writes into that
// folder, which `adoc` creates if it is missing.
#[test]
fn the_output_option_names_the_file_and_creates_its_folder() {
    verifies!(
        r#"
== Name the output file

To choose the output file, use the `-o` (longhand `--output`) option. For
example, to convert [.path]_mydoc.adoc_ and write the output to a name that
includes a version string:

 $ adoc -o mydoc-v2.html mydoc.adoc

You can write it to another directory by prefixing the output file with a folder
name; `adoc` creates the directory if it does not exist:

 $ adoc -o build/mydoc-v2.html mydoc.adoc

"#
    );

    // `-o <name>` and its `--output` longhand both name the output file.
    assert_eq!(
        output_file(&["adoc", "-o", "mydoc-v2.html", "mydoc.adoc"]),
        Some(PathBuf::from("mydoc-v2.html"))
    );
    assert_eq!(
        output_file(&["adoc", "--output", "mydoc-v2.html", "mydoc.adoc"]),
        Some(PathBuf::from("mydoc-v2.html"))
    );

    // A folder-prefixed output keeps the folder in the path.
    assert_eq!(
        output_file(&["adoc", "-o", "build/mydoc-v2.html", "mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc-v2.html"))
    );

    // End to end: the `build/` folder does not exist yet; `adoc` creates it and
    // writes the HTML inside.
    let dir = sandbox("output-option");
    let input = dir.join("mydoc.adoc");
    std::fs::write(&input, "= Doc\n\nBody.\n").expect("write input");
    let out = dir.join("build").join("mydoc-v2.html");
    let stdout = run_argv(&["adoc", "-o", out.to_str().unwrap(), input.to_str().unwrap()]);
    assert!(
        stdout.is_empty(),
        "adoc wrote to stdout instead of the file"
    );
    let html = std::fs::read_to_string(&out).expect("read output file");
    assert!(html.contains("<p>Body.</p>"));
    let _ = std::fs::remove_dir_all(&dir);
}

// A relative `-o` is taken relative to the current directory, not the input's
// directory, and naming the output file also fixes the directory companion
// files are written to.
#[test]
fn relative_output_is_resolved_against_the_current_directory() {
    verifies!(
        r#"
[CAUTION]
====
If you specify the output file as a relative path, it is resolved relative to the
current working directory, not the directory of the input file. Specifying the
output file therefore also sets the directory that companion files (such as a
linked stylesheet) are written to.
====

"#
    );

    // The relative `-o` value is not joined onto the input's directory.
    assert_eq!(
        output_file(&["adoc", "-o", "out.html", "sub/mydoc.adoc"]),
        Some(PathBuf::from("out.html"))
    );

    // Companion files are rooted at the output file's own directory.
    assert_eq!(
        output_dir(Path::new("build/out.html")),
        PathBuf::from("build")
    );
    assert_eq!(output_dir(Path::new("out.html")), PathBuf::from("."));
}

// `-D` names the output directory while the file name defaults to the derived
// `.html` name; a relative `-o` is then resolved inside that directory, while
// an absolute `-o` path is left unchanged. Because the destination applies per
// input, `-D` with several files writes each derived name into that directory.
#[test]
fn the_destination_dir_option_defaults_the_filename() {
    verifies!(
        r#"
== Set the output directory

If you only want to choose the output directory and let the file name default,
use the `-D` (longhand `--destination-dir`) option. `adoc` writes the derived
`.html` name into that directory, creating it if necessary:

 $ adoc -D build mydoc.adoc

When you also pass `-o` with a relative path, it is resolved inside the `-D`
directory; an absolute `-o` path is used unchanged.

`-D` also applies when you xref:cli:process-multiple-files.adoc[convert several
files at once]: each input's derived `.html` name is written into the destination
directory. So this command converts every `.adoc` file in the current directory
into [.path]_build_:

 $ adoc -D build '*.adoc'

"#
    );

    // With `-D` and no `-o`, the derived name is written into the directory.
    assert_eq!(
        output_file(&["adoc", "-D", "build", "mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc.html"))
    );
    assert_eq!(
        output_file(&["adoc", "--destination-dir", "build", "mydoc.adoc"]),
        Some(PathBuf::from("build/mydoc.html"))
    );

    // A relative `-o` is resolved inside `-D`; an absolute `-o` is used as given.
    assert_eq!(
        output_file(&["adoc", "-D", "build", "-o", "out.html", "mydoc.adoc"]),
        Some(PathBuf::from("build/out.html"))
    );
    assert_eq!(
        output_file(&["adoc", "-D", "build", "-o", "/abs/out.html", "mydoc.adoc"]),
        Some(PathBuf::from("/abs/out.html"))
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

    // End to end with a glob: `-D build '*.adoc'` converts every input in the
    // directory, writing each derived name into `build`.
    let dir = sandbox("destination-dir-glob");
    std::fs::write(dir.join("a.adoc"), "= A\n\nAlpha.\n").expect("write a");
    std::fs::write(dir.join("b.adoc"), "= B\n\nBravo.\n").expect("write b");
    let build = dir.join("build");
    let stdout = run_argv(&[
        "adoc",
        "-D",
        build.to_str().unwrap(),
        dir.join("*.adoc").to_str().unwrap(),
    ]);
    assert!(stdout.is_empty(), "adoc wrote to stdout instead of files");
    assert!(std::fs::read_to_string(build.join("a.html"))
        .expect("read a output")
        .contains("Alpha."));
    assert!(std::fs::read_to_string(build.join("b.html"))
        .expect("read b output")
        .contains("Bravo."));
    let _ = std::fs::remove_dir_all(&dir);
}

// When piping, output goes to standard output unless `-o` names a file.
#[test]
fn piping_writes_stdout_unless_an_output_file_is_named() {
    verifies!(
        r#"
== Write a file when piping

When you pipe content to `adoc`, the default is to write the output to standard
output (STDOUT). To write to a file in that case, you have to name one with `-o`:

 $ cat mydoc.adoc | adoc -o build/mydoc.html -

"#
    );

    // A piped document with no `-o` writes to standard output.
    assert!(goes_to_stdout(&["adoc", "-"]));
    let stdout = run_piped(&["adoc", "-"], "= Doc\n\nBody.\n");
    let html = String::from_utf8(stdout).expect("output is UTF-8");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<p>Body.</p>"));

    // Naming a file with `-o` captures the piped output there instead.
    assert_eq!(
        output_file(&["adoc", "-o", "build/mydoc.html", "-"]),
        Some(PathBuf::from("build/mydoc.html"))
    );
    let dir = sandbox("piping");
    let out = dir.join("build").join("mydoc.html");
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
