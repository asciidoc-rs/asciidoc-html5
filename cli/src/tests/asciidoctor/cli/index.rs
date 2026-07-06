use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/cli/pages/index.adoc");

// Asciidoctor's "Process AsciiDoc Using the CLI" overview, tracked from the CLI
// crate. It walks through confirming the CLI is installed (`--version`),
// converting a file, and printing help (`--help`). This crate's `adoc` command
// supports the same three invocations, so each is driven by a test below. The
// parts with no `adoc` counterpart are non-normative here: the Ruby runtime
// banner `asciidoctor --version` prints (adoc is a native binary with no such
// banner), the option catalog reached through the man page, and the `manpage`
// and `syntax` help topics, which `adoc` does not provide.

non_normative!(
    r#"
= Process AsciiDoc Using the CLI

////
command-line-usage.adoc
Command line usage quick start for Asciidoctor
included in the install-toolchain and user-manual documents
////

When the Asciidoctor gem is installed successfully, the Asciidoctor command line interface (CLI) named `asciidoctor` will be available on your PATH.

"#
);

// The "Version and runtime" section: `asciidoctor --version`, and its `-v`
// shorthand, print the processor version. `adoc` mirrors this with `--version`;
// its short form follows the Rust/clap convention `-V` rather than
// Asciidoctor's `-v`, so this test drives `-V` as the closest available
// behavior. The Ruby runtime-environment banner that follows has no counterpart
// in this native binary and is tracked as non-normative below.
#[test]
fn version_flag_prints_the_version() {
    verifies!(
        r#"
== Version and runtime

To confirm that the CLI is available, execute the following command in your terminal application:

 $ asciidoctor --version

Alternately, you can shorten the `--version` CLI option flag to `-v`:

 $ asciidoctor -v

"#
    );

    non_normative!(
        r#"
If this command completes successfully, information about Asciidoctor and the runtime environment will be printed to the standard output of your terminal:

[subs=attributes+]
 Asciidoctor {release-version} [https://asciidoctor.org]
 Runtime Environment ({ruby-description}) (lc:UTF-8 fs:UTF-8 in:UTF-8 ex:UTF-8)

The runtime environment information varies based on the version of Ruby you're using and the encoding settings of your operating system.

"#
    );

    // clap reports a version request as a `DisplayVersion` "error" whose message
    // is the version string. `--version` and the short `-V` produce the same
    // output, `adoc <version>`.
    let long = Cli::try_parse_from(["adoc", "--version"]).expect_err("--version displays version");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayVersion);
    assert!(long.to_string().starts_with("adoc "));

    let short = Cli::try_parse_from(["adoc", "-V"]).expect_err("-V displays version");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayVersion);
    assert_eq!(short.to_string(), long.to_string());
}

// The "Convert an AsciiDoc file" section: `asciidoctor <file>` converts the
// document and, with no output option, writes an `.html` file next to the input
// whose base name matches. `adoc <file>` mirrors that derivation, which this
// test drives end to end.
#[test]
fn converts_a_file_and_derives_the_output() {
    verifies!(
        r#"
== Convert an AsciiDoc file

To invoke Asciidoctor from the CLI and convert an `.adoc` file, execute:

 $ asciidoctor <asciidoc-file>

This will use the built-in defaults for options and create a new file in the same directory as the input file, with the same base name, but with the `.html` extension.

"#
    );

    non_normative!(
        r#"
The Asciidoctor CLI accepts numerous options that control the behavior of the processor, from setting additional attributes (`-a`) to where the output file is written (`-o`).
Most options have both a longhand (e.g., `--out-file`) and shorthand form (e.g., `-o`).
The shorthand form is used throughout this documentation.
Once you're familiar with the options, the shorthand form is preferred and most common since it requires a lot less typing.

xref:man1/asciidoctor.adoc#options[CLI Options] describes the available options and parameters for the Asciidoctor CLI.

"#
    );

    // Hand `adoc` a `<name>.adoc` file with no `-o` and confirm it derives
    // `<name>.html` alongside it, writes a complete HTML5 document there, and
    // prints nothing.
    let source = "= Hello\n\nWorld.";
    let path = std::env::temp_dir().join(format!("adoc-cli-index-{}.adoc", std::process::id()));
    let derived = path.with_extension("html");
    std::fs::write(&path, source).expect("write temp input");

    let cli = Cli::parse_from(["adoc", path.to_str().expect("temp path is UTF-8")]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");

    assert!(stdout.is_empty(), "adoc wrote to stdout on success");
    assert!(
        derived.exists(),
        "adoc did not create the derived output file"
    );
    let html = std::fs::read_to_string(&derived).expect("read derived output file");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&derived);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));
    assert!(html.trim_end().ends_with("</body>\n</html>"));
}

// The "Help topics" section: `asciidoctor --help`, and its `-h` shorthand,
// print the usage statement. `adoc --help` and `adoc -h` do the same. The topic
// grouping, and the `manpage` and `syntax` topics, are Asciidoctor features
// this native binary does not provide, so they are tracked as non-normative.
#[test]
fn help_flag_prints_the_usage_statement() {
    verifies!(
        r#"
== Help topics

"#
    );

    non_normative!(
        r#"
The `--help` option provides self-describing documentation for the `asciidoctor` command, grouped by topic.
"#
    );

    verifies!(
        r#"
If you don't specify a topic, the `--help` option prints the usage statement for the `asciidoctor` command:

 $ asciidoctor --help

Alternately, you can shorten the `--help` CLI option flag to `-h`:

 $ asciidoctor -h

"#
    );

    non_normative!(
        r#"
You can generate the full documentation (i.e., man page) for the `asciidoctor` command by passing the `manpage` topic to the `--help` option.
You can pipe that output to the `man` pager to view it:

 $ asciidoctor --help manpage | man -l -

You can also find the man page for the `asciidoctor` command rendered as HTML in this documentation, which you can view in a browser instead.
See xref:man1/asciidoctor.adoc[asciidoctor(1)].

You can print an AsciiDoc syntax crib sheet by passing the `syntax` topic to the `--help` option.

 $ asciidoctor --help syntax

The crib sheet itself is composed in AsciiDoc.
You can convert it to HTML by piping the output back into the `asciidoctor` command.

 $ asciidoctor --help syntax | asciidoctor -o syntax.html -

Navigate to the [.path]_syntax.html_ file in your browser to see what the examples in the crib sheet look like when converted to HTML.
"#
    );

    // clap reports a help request as a `DisplayHelp` "error" whose message is
    // the rendered help. Both the long `--help` and the short `-h` carry the
    // usage statement for the `adoc` command.
    let long = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(long.to_string().contains("Usage: adoc"));

    let short = Cli::try_parse_from(["adoc", "-h"]).expect_err("-h displays help");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(short.to_string().contains("Usage: adoc"));
}
