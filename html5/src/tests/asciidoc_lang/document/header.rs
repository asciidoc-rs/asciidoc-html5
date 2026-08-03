//! Coverage of the AsciiDoc language description's *Document Header* page.
//!
//! The header is the contiguous run of lines at the top of a document, ended by
//! the first empty line. Matching Asciidoctor 2.0.26, this crate skips a
//! leading comment, renders the header elements, starts the body after the
//! first empty line, and suppresses the whole header block when `noheader` is
//! set. Those behaviors are verified through `convert`; the structural prose
//! (and the attribute-scoping note, where this crate reflects body-set document
//! attributes in the head) is non-normative.

use crate::{
    convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/header.adoc");

// Renders a standalone document so the header `<div>` and the `<head>` metadata
// are present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the definition of the header, its structural rules, and
// the list of elements it may contain. Descriptive prose and setup.
non_normative!(
    r#"
= Document Header

An AsciiDoc document may begin with a document header.
The document header encapsulates the document title, author and revision information, document-wide attributes, and other document metadata.

== Document header structure

The optional document header is a series of contiguous lines at the start of the AsciiDoc source, after skipping any empty or comment lines.
If a document has a header, _no content blocks are permitted above it_.
In other words, the document must start with a document header if it has one.

[IMPORTANT]
====
[.lead]
*The document header may not contain empty lines.*
The first empty line the processor encounters after the document header begins marks the <<when-does-the-document-header-end,end of the document header>> and the start of the document body.
====

A header typically begins with a xref:title.adoc[].
When a document title is specified, it may be immediately followed by one or two designated lines of content.
These implicit content lines are used to assign xref:author-information.adoc[] and xref:revision-information.adoc[] to the document.

The header may contain the following elements as long as there aren't any empty lines between them:

* optional document title (a level-0 heading)
* optional author line or author and revision lines if the document title is present (should immediately follow the document title)
* optional document-wide attributes (built-in and user-defined) declared using xref:attributes:attribute-entries.adoc[attribute entries],
** includes optional xref:metadata.adoc[metadata], such as a description or keywords
* optional xref:ROOT:comments.adoc#comment-lines[comment lines]

Notice in <<ex-basic-header>> that there are no empty lines between any of the entries.
In other words, the lines are contiguous.

"#
);

// A leading comment is skipped, the title and attribute entries are read as the
// header (the `description` becomes `<head>` metadata), and the body begins
// after the first empty line.
#[test]
fn a_basic_header_is_read_and_the_body_follows_the_blank_line() {
    verifies!(
        r#"
.Common elements in a header
[source#ex-basic-header]
----
// this comment line is ignored
= Document Title <.>
Kismet R. Lee <kismet@asciidoctor.org> <.>
:description: The document's description. <.>
:sectanchors: <.>
:url-repo: https://my-git-repo.com <.>
<.>
The document body starts here.
----
<.> Document title
<.> Author line
<.> Attribute entry assigning metadata to a built-in document attribute
<.> Attribute entry setting a built-in document attribute
<.> Attribute entry assigning a value to a user-defined document attribute
<.> The document body is separated from the document header by an empty line

There are a few attribute entries in <<ex-basic-header>>.
Each attribute entry, whether built-in or user-defined, must be entered on its own line.
While attribute entries may be placed anywhere in the header, including above the document title, the preferred placement is below the title, if it's present.
Since the document title is optional, it's possible for the header to only consist of attribute entries.

"#
    );

    let source = "\
// this comment line is ignored
= Document Title
Kismet R. Lee <kismet@asciidoctor.org>
:description: The document's description.
:sectanchors:
:url-repo: https://my-git-repo.com

The document body starts here.
";

    let output = convert(source);
    // The leading comment is not rendered.
    assert!(!output.contains("this comment line is ignored"));
    // The title and the header-set description become header/head output.
    assert_xpath(&output, r#"//h1[text()="Document Title"]"#, 1);
    assert!(output.contains(r#"<meta name="description" content="The document's description.">"#));
    // The body begins after the empty line.
    assert_xpath(
        &output,
        r#"//p[text()="The document body starts here."]"#,
        1,
    );
}

// The section heading. Descriptive prose.
non_normative!(
    r#"
== When does the document header end?

"#
);

// The first empty line ends the header; the next content line begins the body.
#[test]
fn the_first_empty_line_ends_the_header() {
    verifies!(
        r#"
*The first empty line in the document marks the end of the header.*
The next line after the first empty line that contains content is interpreted as the beginning of the document's body.

.Terminating a document header
[source#ex-terminate]
----
= Document Title
Kismet R. Lee <kismet@asciidoctor.org>
:url-repo: https://my-git-repo.com
<.>
This is the first line of content in the document body. <.>
----
<.> An empty line ends the document header.
<.> After the empty line, the next line with content starts the body of the document.

"#
    );

    let source = "\
= Document Title
Kismet R. Lee <kismet@asciidoctor.org>
:url-repo: https://my-git-repo.com

This is the first line of content in the document body.
";

    let output = convert(source);
    assert_xpath(&output, r#"//h1[text()="Document Title"]"#, 1);
    assert_xpath(
        &output,
        r#"//p[text()="This is the first line of content in the document body."]"#,
        1,
    );
}

// The kinds of content the body may start with; the note that below-header
// attributes are not scoped document-wide (this crate reflects body-set
// document attributes in the head, a known divergence); the per-doctype header
// requirements; and the level-0 warning (book/manpage doctypes are out of scope
// here). Descriptive prose.
non_normative!(
    r#"
The first line of the document body can be any valid AsciiDoc content, such as a section heading, paragraph, table, include directive, image, etc.
Any attributes defined below the first empty line are not part of the document header and will not be scoped to the entire document.

== Header requirements per doctype

The header is optional when the `doctype` is `article` or `book`.
A header is required when the document type is `manpage`.
See the xref:asciidoctor:manpage-backend:index.adoc[manpage doctype] section for manual page (man page) requirements.

If you put content blocks above the document header when using the default article doctype, you will see the following warning:

....
level 0 sections can only be used when doctype is book
....

While this warning can be mitigated by changing the doctype to book, it may lead to a secondary warning about an invalid part.
That's because the document title will be repurposed as a part title and any lines that follow it as content blocks.
If you're going to use the book doctype, you must structure your document to use xref:sections:parts.adoc[].

== Header processing

"#
);

// The header is displayed by default in a standalone document and suppressed
// when `noheader` is set.
#[test]
fn noheader_suppresses_the_header_block() {
    verifies!(
        r#"
The information in the document header is displayed by default when converting to a standalone document.
If you don't want the header of a document to be displayed, set the `noheader` attribute in the document's header or via the CLI.

"#
    );

    // Displayed by default.
    let shown = convert("= Document Title\nKismet Lee\n\nBody.\n");
    assert_css(&shown, "#header", 1);

    // Suppressed by `noheader`.
    let hidden = convert("= Document Title\nKismet Lee\n:noheader:\n\nBody.\n");
    assert_css(&hidden, "#header", 0);
}

// The front-matter section, which points to a separate page. Descriptive prose.
non_normative!(
    r#"
== Front matter

Many static site generators, such as Jekyll and Middleman, rely on front matter added to the top of the document to determine how to convert the content.
Asciidoctor has a number of attributes available to correctly handle front matter.
See xref:asciidoctor:html-backend:skip-front-matter.adoc[] to learn more.
"#
);
