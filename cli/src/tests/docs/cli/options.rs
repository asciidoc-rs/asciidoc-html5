use clap::Parser as _;

use crate::{tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/options.adoc");

// This crate's "CLI Options" page. It is the reference page for the CLI module:
// descriptive prose, a table of the supported options, an out-of-scope section,
// and cross-references to the task-specific option pages. The prose and
// cross-references carry no rule to verify; the `--help` invocation, the
// supported-options table, and the out-of-scope list are driven by the tests
// below.

non_normative!(
    r#"
= CLI Options
:navtitle: CLI Options
:description: The command line options the adoc command accepts, and where each one is described.

The `adoc` command accepts a set of options that control how it reads input and
writes output. To see the full list of the options it supports, print the usage
statement:

"#
);

// The `adoc --help` invocation prints the usage statement, which lists the
// options `adoc` supports; the short `-h` flag prints a shorter summary.
#[test]
fn help_lists_the_options() {
    verifies!(
        r#"
 $ adoc --help

You can shorten the `--help` flag to `-h`.

"#
    );

    non_normative!(
        r#"
[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
    );

    // clap surfaces a help request as a `DisplayHelp` "error" carrying the
    // rendered help. Both the long `--help` and the short `-h` include the usage
    // statement and the `Options:` section that lists the supported options, and
    // `-h` is the shorter of the two.
    let long = Cli::try_parse_from(["adoc", "--help"]).expect_err("--help displays help");
    assert_eq!(long.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(long.to_string().contains("Usage: adoc"));
    assert!(long.to_string().contains("Options:"));

    let short = Cli::try_parse_from(["adoc", "-h"]).expect_err("-h displays help");
    assert_eq!(short.kind(), clap::error::ErrorKind::DisplayHelp);
    assert!(short.to_string().contains("Usage: adoc"));
    assert!(
        short.to_string().len() < long.to_string().len(),
        "-h summary should be shorter than --help"
    );
}

// Every option listed in the "Supported options" table is one `adoc` actually
// accepts, in each of the forms the table documents.
#[test]
fn supported_options_table_is_accepted() {
    verifies!(
        r#"
== Supported options

The options below mirror the `asciidoctor` command of the same name, so an
existing invocation that uses only these options runs unchanged under `adoc`:

[cols="1,3",options="header"]
|===
| Option | Description

| `-o`, `--output` _FILE_
| Write the rendered HTML5 to _FILE_ (default: derived from the input path); use `-o -` to write to standard output.

| `-D`, `--destination-dir` _DIR_
| Write output into _DIR_ instead of alongside the input file.

| `-a`, `--attribute` _NAME[=VALUE]_
| Set a document attribute in the form `name`, `name!`, or `name=value`; may be repeated.

| `-B`, `--base-dir` _DIR_
| Base directory that filesystem-relative resources (such as `include::` targets) resolve against.

| `-S`, `--safe-mode` _SAFE_MODE_
| Set the safe mode level explicitly: `unsafe`, `safe`, `server`, or `secure` (default: `unsafe`).

| `--safe`
| Set the safe mode level to `safe`; provided for compatibility with the `asciidoc` command.

| `-e`, `--embedded`
| Suppress the enclosing document structure and output an embedded document. Accepts `-s` and `--no-header-footer` as aliases.

| `-h`, `--help`
| Print the usage statement; the short `-h` prints a briefer summary.

| `-V`, `--version`
| Print the version.
|===

"#
    );

    // Every option form the table documents -- short flag, long flag, and
    // alias alike -- is one the parser actually accepts, so parsing an
    // invocation that uses it never fails with an `UnknownArgument` error.
    // (Checking long-option substrings in `--help` would let a dropped short
    // flag or alias, such as `-s`, slip through while the table still lists
    // it.) Value-taking options are given a minimal valid value; `-h`/`--help`
    // and `-V`/`--version` short-circuit with their own recognized error kinds
    // rather than `UnknownArgument`.
    let forms: &[&[&str]] = &[
        &["-o", "out.html"],
        &["--output", "out.html"],
        &["-D", "build"],
        &["--destination-dir", "build"],
        &["-a", "name=value"],
        &["--attribute", "name=value"],
        &["-B", "base"],
        &["--base-dir", "base"],
        &["-S", "unsafe"],
        &["--safe-mode", "unsafe"],
        &["--safe"],
        &["-e"],
        &["--embedded"],
        &["-s"],
        &["--no-header-footer"],
        &["-h"],
        &["--help"],
        &["-V"],
        &["--version"],
    ];

    for form in forms {
        let mut argv = vec!["adoc"];
        argv.extend_from_slice(form);
        argv.push("doc.adoc");

        if let Err(e) = Cli::try_parse_from(argv) {
            assert_ne!(
                e.kind(),
                clap::error::ErrorKind::UnknownArgument,
                "documented option form {form:?} should be accepted"
            );
        }
    }
}

// The remainder of the page cross-references the task-specific option pages;
// neither the cross-references nor the attribute pointer carries a rule to
// verify here.
non_normative!(
    r#"
Each option is described in depth on the page for the task it serves:

* xref:output-file.adoc[Specify an Output File] covers `-o` (`--output`) to name
the output file and `-D` (`--destination-dir`) to set the output directory.
* xref:set-safe-mode.adoc[Set Safe Mode] covers `-S` (`--safe-mode`), `--safe`,
and `-B` (`--base-dir`).
* xref:io-piping.adoc[Pipe Content] covers reading from standard input and
writing the result to standard output.
* xref:process-multiple-files.adoc[Process Multiple Files] covers converting more
than one file in a single invocation.

Setting document attributes from the command line with `-a` (`--attribute`) is
shown in xref:index.adoc[Process AsciiDoc Using the CLI].

"#
);

// The out-of-scope options really are unknown to `adoc`: passing one is a parse
// error rather than a silently accepted no-op.
#[test]
fn out_of_scope_options_are_rejected() {
    verifies!(
        r#"
== Out-of-scope options

A few `asciidoctor` options depend on the Ruby runtime or on output formats that
`asciidoc-html5` does not produce. These are permanently out of scope: `adoc`
will not implement them.

* *Ruby template engines* -- `-T` (`--template-dir`), `-E` (`--template-engine`),
and `--eruby` load custom converter templates through Ruby template engines such
as Tilt and ERB. `adoc` is a native binary with no Ruby runtime and a single
built-in HTML5 converter, so there is no template engine to load.
* *Ruby runtime hooks* -- `-I` (`--load-path`) and `-r` (`--require`) extend the
Ruby `$LOAD_PATH` and `require` additional Ruby libraries before processing. They
have no meaning outside a Ruby process.
* *Non-HTML5 backends* -- `asciidoctor -b` can select the `xhtml5`, `docbook5`, or
`manpage` backends, among others. `asciidoc-html5` produces only the HTML5
backend; `xhtml5` in particular is a permanent non-goal.

"#
    );

    // The Ruby template-engine and runtime-hook options are not defined on the
    // `adoc` CLI, so clap rejects each with an `UnknownArgument` error. (The
    // `-b`/`--backend` flag is tracked separately in issue #179 and is
    // deliberately left out of this check.)
    for flag in [
        "-T",
        "--template-dir",
        "-E",
        "--template-engine",
        "--eruby",
        "-I",
        "--load-path",
        "-r",
        "--require",
    ] {
        let err = Cli::try_parse_from(["adoc", flag, "value", "doc.adoc"])
            .expect_err("out-of-scope option is rejected");

        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::UnknownArgument,
            "{flag} should be an unknown argument"
        );
    }
}

// The closing note records the known limitations of `adoc`'s option coverage;
// it carries no rule to verify here.
non_normative!(
    r#"
[NOTE]
.Known limitations
====
Beyond the out-of-scope options above, several `asciidoctor` options are simply
not implemented yet. `adoc --help` is the authoritative list of the options it
currently supports, and it has no `asciidoctor(1)` man page of its own.
Generating a man(1) page for `adoc` is tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/94[issue #94].
====
"#
);
