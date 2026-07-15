// Asciidoctor's `man1/asciidoctor.adoc` page is the AsciiDoc source of the
// `asciidoctor(1)` Unix man page: the complete command-line reference for the
// Ruby `asciidoctor` command, rendered into the roff/man output format. The
// page itself is a thin wrapper that sets a TOC level and includes the man page
// body from a partial (a symlink to the man page source that lives outside the
// documentation tree). It documents the `asciidoctor` command — its Ruby
// runtime, every option flag, and man-page-only sections such as SYNOPSIS,
// AUTHORS, and COPYING — none of which states a rendering rule for this HTML5
// renderer to satisfy, so the whole page is tracked as non-normative. See
// `sdd/README.md`.
//
// Only the two literal lines of this wrapper page are tracked: the sdd tool
// measures coverage against the tracked file's own lines and does not expand
// the `include::` directive, and partials fall outside the `pages/` content it
// scans.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/cli/pages/man1/asciidoctor.adoc");

non_normative!(
    r#"
:page-toclevels: 1
include::partial$man-asciidoctor.adoc[]
"#
);
