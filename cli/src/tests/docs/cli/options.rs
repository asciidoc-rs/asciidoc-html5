use clap::Parser as _;

use crate::{tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/options.adoc");

// This crate's "CLI Options" page. It is a landing page for the CLI module:
// descriptive prose, cross-references to the task-specific option pages, and
// the `adoc --help` invocation that lists the supported options. The prose and
// cross-references carry no rule to verify; the `--help` invocation is driven
// by the test below.

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

// The remainder of the page cross-references the task-specific option pages and
// records the known limitations of `adoc`'s option coverage; neither carries a
// rule to verify here.
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

[NOTE]
.Known limitations
====
`adoc` implements a focused subset of the `asciidoctor` CLI. It does not accept
the full option catalog that `asciidoctor` documents in its man page, and it has
no `asciidoctor(1)` man page of its own, so `adoc --help` is the authoritative
list of the options it supports. Generating a man(1) page for `adoc` is tracked
in https://github.com/asciidoc-rs/asciidoc-html5/issues/94[issue #94].
====
"#
);
