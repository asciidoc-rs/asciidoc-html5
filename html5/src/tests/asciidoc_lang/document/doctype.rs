//! Coverage of the AsciiDoc language description's *Document Type* page.
//!
//! The doctype declares a document's expected structure. This crate models the
//! `article` doctype structurally and also implements Asciidoctor's `inline`
//! doctype: matching Asciidoctor 2.0.26, it reads a single block, applies
//! inline formatting, and emits the result without the surrounding paragraph
//! tags. That inline behavior is verified through `convert_with`; the doctype
//! descriptions (`book` and `manpage` are out of scope here) are non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/doctype.adoc");

// The document title, the page alias, the overview of the doctypes, and the
// per-doctype descriptions. Descriptive prose.
non_normative!(
    r#"
= Document Type
:page-aliases: doctypes.adoc

The document type (aka doctype) declares the expected structure of an AsciiDoc document.
AsciiDoc defines a fixed set of document types.
Each document type provides a slight variation on the permitted structure of an AsciiDoc document to accommodate different uses cases.

The default doctype is `article`, which provides the foundation structure on which other doctypes build.
The `book` doctype permits multiple level-0 sections that act as part sections.
The `manpage` doctype provides an extended header for defining standard metadata of a manpage, such as the volume number, man, and purpose.
The `inline` doctype is intended for embedded scenarios.

== Document types

Article (`article`)::
The default doctype.
In DocBook, this includes the appendix, abstract, bibliography, glossary, and index sections.
Unless you are making a book or a man page, you don't need to worry about the doctype.
The default will suffice.

Book (`book`)::
Builds on the article doctype with the additional ability to use a top-level title as part titles, includes the appendix, dedication, preface, bibliography, glossary, index, and colophon.
There's also the concept of a multi-part book, but the distinction from a regular book is determined by the content.
A book only has chapters and special sections, whereas a multi-part book is divided by parts that each contain one or more chapters or special sections.

Man page (`manpage`)::
Used for producing a roff or HTML-formatted manual page (man page) for Unix and Unix-like operating systems.
This doctype instructs the parser to recognize a special document header and section naming conventions for organizing the AsciiDoc content as a man page.
See xref:asciidoctor:manpage-backend:index.adoc[] for details on how structure a man page using AsciiDoc and generate it using Asciidoctor.

Inline (`inline`)::
There may be cases when you only want to apply inline AsciiDoc formatting to input text without wrapping it in a block element.
For example, in the Asciidoclet project (AsciiDoc in Javadoc), only the inline formatting is needed for the text in Javadoc tags.
// {asciidoclet-ref}[Asciidoclet project]

== Inline doctype rules

"#
);

// Under the inline doctype only the first block is read, inline formatting is
// applied, and the result is emitted without the wrapping paragraph tags.
#[test]
fn inline_doctype_emits_unwrapped_inline_formatting() {
    verifies!(
        r#"
The rules for the inline doctype are as follows:

* Only a single paragraph is read from the AsciiDoc source.
* Inline formatting is applied.
* The output is not wrapped in the normal paragraph tags.

Given the following input:

[source]
https://asciidoctor.org[AsciiDoc] is a _lightweight_ markup language...

Processing it with the options `doctype=inline` and `backend=html5` produces:

[source,html]
<a href="https://asciidoctor.org">AsciiDoc</a> is a <em>lightweight</em> markup language&#8230;

The inline doctype allows the AsciiDoc processor to cover the full range of applications, from unstructured (inline) text to full, standalone documents!
"#
    );

    let output = convert_with(
        "https://asciidoctor.org[AsciiDoc] is a _lightweight_ markup language...",
        &Options::new().doctype("inline"),
    );
    // Inline formatting is applied (link, emphasis, the `...` replacement) and
    // the output is not wrapped in `<p>` tags. Asciidoctor 2.0.26 appends a
    // word-joiner (`&#8203;`) after the ellipsis; the page predates it.
    assert!(output.contains(
        r#"<a href="https://asciidoctor.org">AsciiDoc</a> is a <em>lightweight</em> markup language&#8230;&#8203;"#
    ));
    assert!(!output.contains("<p>"));

    // Only the first block is read; a second paragraph is dropped.
    let single = convert_with(
        "first _para_\n\nsecond para",
        &Options::new().doctype("inline"),
    );
    assert!(single.contains("first <em>para</em>"));
    assert!(!single.contains("second"));
}
