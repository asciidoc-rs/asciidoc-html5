//! Coverage of the AsciiDoc language description's *Subtitle* page.
//!
//! A document title may carry a subtitle after a separator. Matching
//! Asciidoctor 2.0.26, this crate's HTML5 output does *not* split the subtitle
//! out — the whole title is rendered as the `<h1>` and `<title>` — but the
//! partition is still available through the parser API (`Header::main_title`
//! and `Header::subtitle`), which honors the final colon-space separator and an
//! overridden separator. The HTML rendering is verified through `convert` and
//! the partition through `load`; the Ruby API prose is non-normative.

use crate::{
    convert_with, load,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/subtitle.adoc");

// Renders a standalone document so the header `<h1>` and the `<title>` element
// are present in the output.
fn convert_standalone(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the introduction. Descriptive prose.
non_normative!(
    r#"
= Subtitle
//From @graphitefriction: this page has some weird complexity for such a simple thing

An optional subtitle can be appended to a xref:title.adoc[document title].

"#
);

// The HTML5 converter does not split the subtitle out: the whole title is
// rendered in the `<h1>` and `<title>`. The partition is nonetheless available
// through the parser API.
#[test]
fn html5_output_keeps_the_whole_title() {
    verifies!(
        r#"
NOTE: The HTML 5 converter does not currently split the subtitle out from the document title when generating HTML from AsciiDoc.
The document title is only partitioned into a main and subtitle in the output of the DocBook, EPUB 3, and PDF converters.
However, the subtitle is still available via the API, so you could add support for it by extending the HTML 5 converter.

"#
    );

    let output = convert_standalone("= Main Title: Subtitle\n\nBody.\n");
    // The `<h1>` and `<title>` carry the whole, unpartitioned title.
    assert_xpath(&output, r#"//h1[text()="Main Title: Subtitle"]"#, 1);
    assert!(output.contains("<title>Main Title: Subtitle</title>"));

    // The subtitle is still available through the parser API.
    let document = load("= Main Title: Subtitle\n\nBody.\n");
    assert_eq!(document.header().subtitle(), Some("Subtitle"));
}

// The section heading. Descriptive prose.
non_normative!(
    r#"
== Subtitle syntax

"#
);

// The final colon-space sequence partitions the title into a main title and a
// subtitle; only the last occurrence is used as the separator.
#[test]
fn the_final_colon_space_partitions_the_title() {
    verifies!(
        r#"
When the document title contains a colon followed by a space (i.e, `:{sp}`), the text after the final colon-space sequence is treated as a subtitle.

.A document title and subtitle
[source]
----
= Main Title: Subtitle
----

The separator is searched for from the end of the text.
Therefore, only the last occurrence of the separator (i.e, `:{sp}`) is used for partitioning the title.

.A document title that contains more than one colon-space sequence
[source]
----
= Main Title: Main Title Continued: Subtitle
----

"#
    );

    let simple = load("= Main Title: Subtitle\n\nBody.\n");
    assert_eq!(simple.header().main_title(), Some("Main Title"));
    assert_eq!(simple.header().subtitle(), Some("Subtitle"));

    // Only the final colon-space separates the subtitle; earlier ones stay in
    // the main title.
    let multi = load("= Main Title: Main Title Continued: Subtitle\n\nBody.\n");
    assert_eq!(
        multi.header().main_title(),
        Some("Main Title: Main Title Continued")
    );
    assert_eq!(multi.header().subtitle(), Some("Subtitle"));
}

// The subsection heading. Descriptive prose.
non_normative!(
    r#"
=== Modify the title separator

"#
);

// The separator can be overridden with the `separator` block attribute above
// the title or the `title-separator` document attribute; both repartition the
// title around the custom separator.
#[test]
fn the_title_separator_can_be_overridden() {
    verifies!(
        r#"
You can change the title separator by specifying the `separator` block attribute explicitly above the document title.
A space will automatically be appended to the separator value.

.Assign separator to the document title
[source]
----
[separator=::]
= Main Title:: Subtitle
----

You can also assign a separator using a document attribute `title-separator` in the header.

.Assign title-separator to the document title
[source]
----
= Main Title:: Subtitle
:title-separator: ::
----

"#
    );

    // The `separator` block attribute above the title.
    let block = load("[separator=::]\n= Main Title:: Subtitle\n\nBody.\n");
    assert_eq!(block.header().main_title(), Some("Main Title"));
    assert_eq!(block.header().subtitle(), Some("Subtitle"));

    // The `title-separator` document attribute in the header.
    let attr = load("= Main Title:: Subtitle\n:title-separator: ::\n\nBody.\n");
    assert_eq!(attr.header().main_title(), Some("Main Title"));
    assert_eq!(attr.header().subtitle(), Some("Subtitle"));
}

// Assigning the separator via the CLI, and the Ruby API for partitioning the
// title (plus a large commented-out block of alternative examples).
// Ruby-specific; non-normative here.
non_normative!(
    r#"
`title-separator` can also be assigned via the CLI.

....
$ asciidoctor -a title-separator=:: document.adoc
....

== Partition the title using the API

You can partition the title from the API when calling the `doctitle` method on Document:

.Retrieving a partitioned document title
[source,ruby]
----
title_parts = document.doctitle partition: true
puts title_parts.title
puts title_parts.subtitle
----

You can partition the title in an arbitrary way by passing the separator as a value to the partition option.
In this case, the partition option both activates subtitle partitioning and passes in a custom separator.

.Retrieving a partitioned document title with a custom separator
[source,ruby]
----
title_parts = document.doctitle partition: '::'
puts title_parts.title
puts title_parts.subtitle
----

////
.Document with a subtitle
[source]
----
include::example$title.adoc[tag=sub-1]
----

In this example, the following is true:

Main title:: The Dangerous and Thrilling Documentation Chronicles
Subtitle:: A Tale of Caffeine and Words

.Document with a subtitle and multiple colons
[source]
----
include::example$title.adoc[tag=sub-2]
----

In this example, the following is true:

Main title:: A Cautionary Tale: The Dangerous and Thrilling Documentation Chronicles
Subtitle:: A Tale of Caffeine and Words

Instead of using a colon followed by a space as the separator characters between the main title and the subtitle, you can specify a custom separator using the `title-separator` attribute.

.Document with a subtitle using a custom separator
[source]
----
include::example$title.adoc[tag=sub-3]
----

Note that a space is always appended to the value of the `title-separator` (making the default value of the `title-separator` effectively a single colon).

This content needs to be moved or reconsidered:

Asciidoctor also provides an API for extracting the title and subtitle.
See the API docs for the https://www.rubydoc.info/gems/asciidoctor/Asciidoctor/Document/Title[Document::Title] for more information.
Support for subtitle functionality for other sections is being considered.
Refer to https://github.com/asciidoctor/asciidoctor/issues/1493[Asciidoctor issue #1493].
////
"#
);
