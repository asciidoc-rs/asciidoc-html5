//! Coverage of the AsciiDoc language description's *Document Structure* page.
//!
//! This is an orientation page: it sketches what an AsciiDoc document is made
//! of — documents, lines, blocks, and text — without pinning down syntax. Most
//! of it is conceptual prose, and several examples illustrate line-oriented
//! parsing rules (an attribute entry must occupy a line, and may be continued
//! with a trailing backslash) rather than rendered output; those are
//! non-normative here. The concrete, whole-document examples the page shows — a
//! one-sentence document, a two-paragraph document, a document with a header,
//! and a section title on its own line — are this crate's behavior and are
//! verified through `convert`.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/ROOT/pages/document-structure.adoc");

// The page's framing and the opening of the "Documents" section: AsciiDoc is
// plain text with no boilerplate, and a document can be as short as a single
// sentence.
non_normative!(
    r#"
= Document Structure
:page-aliases: document.adoc

On this page, you'll learn about the overall structure of an AsciiDoc document.
Don't worry about the details of the syntax at this point.
That topic will be covered thoroughly later in the documentation.
Right now, we're just aiming to get a sense of what makes up an AsciiDoc document.

== Documents

AsciiDoc is a plain text writing format with no boilerplate enclosure or prologue.
An AsciiDoc document may consist of only a single sentence (or even a single character, to be academic).

"#
);

// The minimal valid document: a single sentence renders as a single paragraph.
#[test]
fn a_single_sentence_is_a_valid_document() {
    verifies!(
        r#"
The example below is a valid AsciiDoc document.
It contains a single paragraph that consists of a single sentence.

----
This is a basic AsciiDoc document.
----

"#
    );

    let output = convert("This is a basic AsciiDoc document.\n");
    assert_css(&output, "div.paragraph", 1);
    assert!(output.contains("<p>This is a basic AsciiDoc document.</p>"));
}

// A document is a linewise stack of blocks, typically offset by empty lines.
non_normative!(
    r#"
Of course, you can have more content than a single sentence!
What we want to emphasize here is that it's simple to get started.

An AsciiDoc document is a series of blocks that are stacked linewise.
These blocks are typically offset from one another by an empty line.
(While these empty lines aren't always required, we do recommend using them for readability.)

"#
);

// Separating the source with an empty line yields two distinct paragraphs.
#[test]
fn an_empty_line_separates_two_paragraphs() {
    verifies!(
        r#"
To expand the previous document from one paragraph to two, you'd separate the two paragraphs by an empty line:

----
This is a basic AsciiDoc document.

This document contains two paragraphs.
----

"#
    );

    let output =
        convert("This is a basic AsciiDoc document.\n\nThis document contains two paragraphs.\n");
    assert_css(&output, "div.paragraph", 2);
}

// A document may open with a header that sets the title and document
// attributes. Rendering the header's source (shown here as a `[source]`
// listing) resolves the author line and the `{author}` reference in the body,
// and produces the two body paragraphs.
#[test]
fn a_document_may_begin_with_a_header() {
    verifies!(
        r#"
An AsciiDoc document may begin with a document header.
Although the document header is optional, it's often used because it allows you to specify a document title as well as document-wide configuration and reusable text in the form of document attributes.

[source]
----
= Document Title
Author Name
:reproducible:

This is a basic AsciiDoc document by {author}.

This document contains two paragraphs.
It also has a header that specifies the document title and some attibutes.
----

"#
    );

    let output = convert(
        "= Document Title\nAuthor Name\n:reproducible:\n\nThis is a basic AsciiDoc document by {author}.\n\nThis document contains two paragraphs.\nIt also has a header that specifies the document title and some attibutes.\n",
    );
    assert_css(&output, "div.paragraph", 2);
    assert!(output.contains("This is a basic AsciiDoc document by Author Name."));
}

// The rest of the "Documents" section and the opening of "Lines": documents can
// range from a sentence to a book, and AsciiDoc is a line-oriented language in
// which many constructs must occupy a whole line.
non_normative!(
    r#"
Almost any combination of blocks constitutes a valid AsciiDoc document (with some structural requirements dictated by the xref:document:doctypes.adoc[document type]).
Documents can range from a single sentence to a multi-part book.

== Lines

The line is a significant building block in AsciiDoc.
A line is defined as text that's separated on either side by a newline character or the boundary of the document.
Many aspects of the syntax must occupy a whole line.
That's why we say AsciiDoc is a line-oriented language.

For example, a section title must be on a line by itself.
The same is true for an attribute entry, a block title, a block attribute list, a block macro, a list item, a block delimiter, and so forth.

"#
);

// A section title occupies a line by itself and renders as a section heading.
#[test]
fn a_section_title_occupies_its_own_line() {
    verifies!(
        r#"
.Example of a section title, which must occupy a single line
[source]
----
== Section Title
----

"#
    );

    let output = convert("== Section Title\n");
    assert_css(&output, "div.sect1", 1);
    assert!(output.contains("<h2 id=\"_section_title\">Section Title</h2>"));
}

// The attribute-entry examples illustrate a line-oriented *parsing* rule (an
// entry occupies a line and may be continued with a trailing backslash) rather
// than a rendered result — a bare attribute entry produces no output — so they
// are non-normative here, alongside the remaining conceptual sections on
// blocks, text and inline elements, and file encodings.
non_normative!(
    r#"
.Example of an attribute entry, which must also occupy at least one line
[source]
-----
:name: value
-----

.Example of an attribute entry that extends to two lines
[source]
-----
:name: value \
more value
-----

Notice that the attribute entry must be explicitly continued to extend onto another line.

Empty lines can also be significant.
For example, a single empty line separates the header from the body.
Many blocks are also separated by an empty line, as you saw in the example earlier with two paragraphs.

In contrast, lines within paragraph content are insignificant.
Keep these points in mind as you're learning about the AsciiDoc syntax.

== Blocks

Blocks in an AsciiDoc document lay down the document structure.
Some blocks may contain other blocks, so the document structure is inherently hierarchical (i.e., a tree structure).
You can inspect this section structure, for example, by enabling the automatic table of contents.
Examples of blocks include paragraphs, sections, lists, delimited blocks, tables, and block macros.

Blocks are easy to identify because they're usually offset from other blocks by an empty line (though not always required).
Blocks always start on a new line, terminate at the end of a line, and are aligned to the left margin.

Every block can have one or more lines of block metadata.
This metadata can be in the form of block attributes, a block anchor, or a block title.
These metadata lines must be above and directly adjacent to the block itself.

Sections, non-verbatim delimited blocks, and AsciiDoc table cells may contain other blocks.
Despite the fact that blocks form a hierarchy, even nested blocks start at the left margin.
By requiring blocks to start at the left margin, it avoids the tedium of having to track and maintain levels of indentation.
It also happens to make the content more reusable.

== Text and inline elements

Surrounded by the markers, delimiters, and metadata lines is the text.
The text is the main focus of a document and the reason the AsciiDoc syntax gives it so much room to breathe.
Text is most often found in the lines of a block (e.g., paragraph), the block title (e.g., section title), and in list items, though there are other places where it can exist.

Text is subject to substitutions.
Substitutions interpret markup as text formatting, replace macros with text or non-text elements, expand attribute references, and perform other sorts of text replacement.

Normal text is subject to all substitutions, unless specified otherwise.
Verbatim text is subject to a minimal set of substitutions to allow it to be displayed in the output as it appears in the source.
It's also possible to disable all substitutions in order to pass the text through to the output unmodified (i.e., raw).
The parsing of text ends up being a mix of inline elements and other forms of transformations.

== Encodings and AsciiDoc files

An AsciiDoc file is a text file that has the _.adoc_ file extension (e.g., [.path]_document.adoc_).
Most AsciiDoc processors assume the text in the file uses UTF-8 encoding.
UTF-16 encodings are supported only if the file starts with a BOM.

An AsciiDoc processor can process AsciiDoc from a string (i.e., character sequence).
However, most of the time you'll save your AsciiDoc documents to a file.
"#
);
