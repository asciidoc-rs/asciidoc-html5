//! Coverage of the AsciiDoc language description's *Syntax Quick Reference*
//! page — a cheatsheet that assembles, via `include::example$` directives, one
//! or two examples of nearly every AsciiDoc construct.
//!
//! Because the page is built almost entirely from includes, each section here
//! reproduces the page's literal include lines (so the `sdd` tool can align the
//! coverage) and drives the *expansion* of the included example — reconstructed
//! from the tracked example file — through `convert`, asserting the construct
//! renders as documented. The framing prose, the section headings, and the few
//! constructs this renderer does not deliver (URL includes and the `data-uri`
//! embed, both out of scope; the server-side `rouge` highlighter) are tracked
//! as non-normative.

use crate::{
    convert, convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/ROOT/pages/syntax-quick-reference.adoc");

// Renders a standalone document so the header (byline, revision line) and an
// auto-generated table of contents appear in the output.
fn convert_standalone(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

non_normative!(
    r#"
= AsciiDoc Syntax Quick Reference
:navtitle: Syntax Quick Reference
:description: The quick reference for common AsciiDoc document and text formatting markup.
:collapsible:
:url-char-xml: https://en.wikipedia.org/wiki/List_of_XML_and_HTML_character_entity_references
:url-data-uri: https://developer.mozilla.org/en-US/docs/data_URIs
:!table-frame:
:!table-grid:
// release-version is used for an example; it's not the release version for this document
:release-version: 2.4.3

////
This document is not meant to be a replacement for the documentation of the AsciiDoc language itself.
It's meant to be a helpful guide you can give to a writer to refer to while in the thick of writing.
Think of it a quick reminder of the most common syntax and scenarios.
It should not go into any depth about AsciiDoc processing or the options you can use when converting to an output format.
////

[IMPORTANT]
The examples on this page demonstrate the output produced by the built-in HTML converter.
An AsciiDoc converter is expected to produce complementary output when generating other output formats, such as PDF, EPUB, and DocBook.

"#
);

non_normative!(
    r#"
== Paragraphs

"#
);

// A paragraph needs no markup; consecutive lines form one paragraph, and an
// empty line begins the next.
#[test]
fn paragraph_renders_a_paragraph_block() {
    verifies!(
        r#"
.Paragraph
[#ex-normal]
----
include::text:example$text.adoc[tag=b-para]
----

.View result of <<ex-normal>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=b-para]
====

"#
    );

    let output = convert(
        "Paragraphs don't require special markup in AsciiDoc.\n\
         A paragraph is defined by one or more consecutive lines of text.\n\
         Line breaks within a paragraph are not displayed.\n\n\
         Leave at least one empty line to begin a new paragraph.\n",
    );
    assert_css(&output, "div.paragraph", 2);
}

// An indented paragraph becomes a preformatted literal block, set apart from a
// normal paragraph.
#[test]
fn literal_paragraph_is_preformatted() {
    verifies!(
        r#"
.Literal paragraph
[#ex-literal]
----
include::verbatim:example$literal.adoc[tag=qr-para]
----

.View result of <<ex-literal>>
[%collapsible.result]
====
include::verbatim:example$literal.adoc[tag=qr-para]
====

"#
    );

    let output = convert(
        "A normal paragraph.\n\n\
         \x20A literal paragraph.\n\
         \x20One or more consecutive lines indented by at least one space.\n\n\
         \x20The text is shown in a fixed-width (typically monospace) font.\n\
         \x20The lines are preformatted (i.e., as formatted in the source).\n\
         \x20Spaces and newlines,\n\
         \x20like the ones in this sentence,\n\
         \x20are preserved.\n",
    );
    assert_css(&output, "div.paragraph", 1);
    assert_css(&output, "div.literalblock", 2);
}

// A trailing `+` forces a line break, and the `%hardbreaks` option breaks every
// line of the paragraph.
#[test]
fn hard_line_breaks() {
    verifies!(
        r#"
.Hard line breaks
[#ex-hardbreaks]
----
include::text:example$text.adoc[tag=hb-all]
----

.View result of <<ex-hardbreaks>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=hb-all]
====

"#
    );

    let output = convert(
        "Roses are red, +\nviolets are blue.\n\n\
         [%hardbreaks]\nA ruby is red.\nJava is black.\n",
    );
    assert_css(&output, "div.paragraph", 2);
    assert_css(&output, "br", 2);
}

// The `lead` role styles a paragraph as a lead paragraph.
#[test]
fn lead_paragraph_role() {
    verifies!(
        r#"
.Lead paragraph
[#ex-lead]
----
include::text:example$text.adoc[tag=qr-lead]
----

.View result of <<ex-lead>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=qr-lead]
====

"#
    );

    let output = convert(
        "[.lead]\n\
         This text will be styled as a lead paragraph (i.e., larger font).\n\n\
         This paragraph will not be.\n",
    );
    assert_css(&output, "div.paragraph.lead", 1);
    assert_css(&output, "div.paragraph", 2);
}

non_normative!(
    r#"
TIP: The default Asciidoctor stylesheet automatically styles the first paragraph of the preamble as a xref:blocks:preamble-and-lead.adoc[lead paragraph] if no role is specified on that paragraph.

"#
);

non_normative!(
    r#"
== Text formatting

"#
);

// Constrained bold (`*`), italic (`_`), and monospace (`` ` ``) formatting,
// plus their combinations.
#[test]
fn constrained_bold_italic_monospace() {
    verifies!(
        r#"
.Constrained bold, italic, and monospace
[#ex-constrained]
----
include::text:example$text.adoc[tag=constrained-bold-italic-mono]
----

.View result of <<ex-constrained>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=constrained-bold-italic-mono]
====

"#
    );

    let output = convert(
        "It has *strong* significance to me.\n\n\
         I _cannot_ stress this enough.\n\n\
         Type `OK` to accept.\n\n\
         That *_really_* has to go.\n\n\
         Can't pick one? Let's use them `*_all_*`.\n",
    );
    assert_css(&output, "strong", 3);
    assert_css(&output, "em", 3);
    assert_css(&output, "code", 2);
}

// Unconstrained formatting doubles the marks (`**`, `__`, ` `` `) so it can
// apply inside a word.
#[test]
fn unconstrained_bold_italic_monospace() {
    verifies!(
        r#"
.Unconstrained bold, italic, and monospace
[#ex-unconstrained]
----
include::text:example$text.adoc[tag=unconstrained-bold-italic-mono]
----

.View result of <<ex-unconstrained>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=unconstrained-bold-italic-mono]
====

"#
    );

    let output = convert(
        "**C**reate, **R**ead, **U**pdate, and **D**elete (CRUD)\n\n\
         That's fan__freakin__tastic!\n\n\
         Don't pass generic ``Object``s to methods that accept ``String``s!\n\n\
         It was Beatle**__mania__**!\n",
    );
    assert_css(&output, "strong", 5);
    assert_css(&output, "em", 2);
    assert_css(&output, "code", 2);
}

// Highlight (`#`), and the `underline`, `line-through`, and custom roles
// applied with the `[.role]#text#` form.
#[test]
fn highlight_underline_strikethrough_custom_role() {
    verifies!(
        r#"
.Highlight, underline, strikethrough, and custom role
[#ex-lines]
----
include::text:example$text.adoc[tag=qr-all]
----

.View result of <<ex-lines>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=qr-all]
====

"#
    );

    let output = convert(
        "Mark my words, #automation is essential#.\n\n\
         ##Mark##up refers to text that contains formatting ##mark##s.\n\n\
         Where did all the [.underline]#cores# go?\n\n\
         We need [.line-through]#ten# twenty VMs.\n\n\
         A [.myrole]#custom role# must be fulfilled by the theme.\n",
    );
    assert_css(&output, "mark", 3);
    assert_css(&output, "span.underline", 1);
    assert_css(&output, "span.line-through", 1);
    assert_css(&output, "span.myrole", 1);
}

// Superscript (`^`) and subscript (`~`).
#[test]
fn superscript_and_subscript() {
    verifies!(
        r#"
.Superscript and subscript
[#ex-sub-sup]
----
include::text:example$text.adoc[tag=b-sub-sup]
----

.View result of <<ex-sub-sup>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=b-sub-sup]
====

"#
    );

    let output = convert("^super^script\n\n~sub~script\n");
    assert_css(&output, "sup", 1);
    assert_css(&output, "sub", 1);
}

// Curved double/single quotes (``"`...`"``, `` '`...`' ``) and the typographic
// apostrophe.
#[test]
fn smart_quotes_and_apostrophes() {
    verifies!(
        r#"
.Smart quotes and apostrophes
[#ex-curved]
----
include::text:example$text.adoc[tag=b-c-quote]
----

.View result of <<ex-curved>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=b-c-quote]
====

"#
    );

    let output = convert(
        "\"`double curved quotes`\"\n\n\
         '`single curved quotes`'\n\n\
         Olaf's desk was a mess.\n",
    );

    // Curved double quotes, curved single quotes, and the smart apostrophe.
    assert!(output.contains("&#8220;double curved quotes&#8221;"));
    assert!(output.contains("&#8216;single curved quotes&#8217;"));
    assert!(output.contains("Olaf&#8217;s desk"));
}

non_normative!(
    r#"
== Links

"#
);

// A bare URL autolinks, the URL macro sets link text, and the `mailto:` macro
// links an email address (with optional subject and body).
#[test]
fn autolinks_url_and_mailto_macros() {
    verifies!(
        r#"
.Autolinks, URL macro, and mailto macro
[#ex-urls]
----
include::macros:example$url.adoc[tag=b-base]

include::macros:example$url.adoc[tag=b-scheme]
----

.View result of <<ex-urls>>
[%collapsible.result]
====
include::macros:example$url.adoc[tag=b-base]

include::macros:example$url.adoc[tag=b-scheme]
====

"#
    );

    let output = convert(
        "https://asciidoctor.org - automatic!\n\n\
         https://asciidoctor.org[Asciidoctor]\n\n\
         devel@discuss.example.org\n\n\
         mailto:devel@discuss.example.org[Discuss]\n\n\
         mailto:join@discuss.example.org[Subscribe,Subscribe me,I want to join!]\n",
    );
    assert_css(&output, "a.bare", 1);
    assert_xpath(
        &output,
        r#"//a[@href="https://asciidoctor.org"][text()="Asciidoctor"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//a[@href="mailto:devel@discuss.example.org"][text()="Discuss"]"#,
        1,
    );
    assert!(output.contains(
        "mailto:join@discuss.example.org?subject=Subscribe%20me&amp;body=I%20want%20to%20join%21"
    ));
}

// Named URL attributes (`role`, `window`) and the `^` shorthand that opens the
// link in a new window.
#[test]
fn url_macro_attributes() {
    verifies!(
        r#"
.URL macros with attributes
[#ex-linkattrs]
----
include::macros:example$url.adoc[tag=b-linkattrs]
----

.View result of <<ex-linkattrs>>
[%collapsible.result]
====
include::macros:example$url.adoc[tag=b-linkattrs]
====

"#
    );

    let output = convert(
        "https://chat.asciidoc.org[Discuss AsciiDoc,role=external,window=_blank]\n\n\
         https://chat.asciidoc.org[Discuss AsciiDoc^]\n",
    );
    assert_css(&output, "a.external[target=\"_blank\"]", 1);
    assert_css(&output, "a[target=\"_blank\"][rel=\"noopener\"]", 2);
}

non_normative!(
    r#"
IMPORTANT: The `link:` macro prefix is _not_ required when the target starts with a URL scheme like `https:`.
The URL scheme acts as an implicit macro prefix.

CAUTION: If the link text contains a comma and the text is followed by one or more named attributes, you must enclose the text in double quotes.
Otherwise, the text will be cut off at the comma (and the remaining text will get pulled into the attribute parsing).

"#
);

// A link target with spaces or special characters is wrapped in `link:++...++`
// or percent-encoded.
#[test]
fn urls_with_spaces_and_special_characters() {
    verifies!(
        r#"
.URLs with spaces and special characters
----
include::macros:example$url.adoc[tag=b-spaces]
----

"#
    );

    let output = convert(
        "link:++https://example.org/?q=[a b]++[URL with special characters]\n\n\
         https://example.org/?q=%5Ba%20b%5D[URL with special characters]\n",
    );
    assert_css(&output, "a", 2);
    assert_xpath(&output, r#"//a[@href="https://example.org/?q=[a b]"]"#, 1);
}

// The `link:` macro targets a relative file path.
#[test]
fn link_to_relative_file() {
    verifies!(
        r#"
.Link to relative file
----
link:index.html[Docs]
----

"#
    );

    let output = convert("link:index.html[Docs]\n");
    assert_xpath(&output, r#"//a[@href="index.html"][text()="Docs"]"#, 1);
}

// The `link:` macro accepts a Windows UNC path.
#[test]
fn link_using_a_windows_unc_path() {
    verifies!(
        r#"
.Link using a Windows UNC path
----
include::macros:example$url.adoc[tag=b-windows]
----

"#
    );

    let output = convert("link:\\\\server\\share\\whitepaper.pdf[Whitepaper]\n");
    assert_xpath(&output, r#"//a[text()="Whitepaper"]"#, 1);
}

// Inline anchors declared with `[[id]]`, the shorthand `[#id]#text#`, the
// `anchor:` macro, and an anchor carrying an xreflabel.
#[test]
fn inline_anchors() {
    verifies!(
        r#"
.Inline anchors
----
include::attributes:example$id.adoc[tag=anchor]
----

"#
    );

    let output = convert(
        "[[bookmark-a]]Inline anchors make arbitrary content referenceable.\n\n\
         [#bookmark-b]#Inline anchors can be applied to a phrase like this one.#\n\n\
         anchor:bookmark-c[]Use a cross reference to link to this location.\n\n\
         [[bookmark-d,last paragraph]]The xreflabel attribute will be used as link text in the cross-reference link.\n",
    );
    assert_css(&output, "a#bookmark-a", 1);
    assert_css(&output, "span#bookmark-b", 1);
    assert_css(&output, "a#bookmark-c", 1);
    assert_css(&output, "a#bookmark-d", 1);
}

// A cross reference links to an anchor by id (`<<id>>`) or with custom text
// (`<<id,text>>`).
#[test]
fn cross_references() {
    verifies!(
        r#"
.Cross references
[#ex-xrefs]
----
include::macros:example$xref.adoc[tag=b-base]
----

.View result of <<ex-xrefs>>
[%collapsible.result]
====
include::macros:example$xref.adoc[tag=b-base]
====

"#
    );

    let output = convert(
        "[#paragraphs]\n== Paragraphs\n\n\
         See <<paragraphs>> to learn how to write paragraphs.\n\n\
         Learn how to organize the document into <<section-titles,sections>>.\n\n\
         [#section-titles]\n== Section titles\n",
    );
    assert_xpath(&output, r##"//a[@href="#paragraphs"]"##, 1);
    assert_xpath(
        &output,
        r##"//a[@href="#section-titles"][text()="sections"]"##,
        1,
    );
}

// An inter-document cross reference targets another document, optionally at a
// specific anchor.
#[test]
fn inter_document_cross_references() {
    verifies!(
        r#"
.Inter-document cross references
----
include::macros:example$xref.adoc[tag=b-inter]
----

"#
    );

    let output = convert(
        "Refer to xref:document-b.adoc#section-b[Section B of Document B] for more information.\n\n\
         If you never return from xref:document-b.adoc[Document B], we'll send help.\n",
    );
    assert_xpath(&output, r#"//a[@href="document-b.html#section-b"]"#, 1);
    assert_xpath(
        &output,
        r#"//a[@href="document-b.html"][text()="Document B"]"#,
        1,
    );
}

non_normative!(
    r#"
== Document header

The xref:document:header.adoc[document header] is optional.
The header may not contain any empty lines and must be separated from the content by at least one empty line.

"#
);

// The document title is the level-0 heading at the top of the header.
#[test]
fn header_title() {
    verifies!(
        r#"
.Title
----
include::document:example$title.adoc[tag=qr-title]
----

"#
    );

    let output = convert_standalone("= Document Title\n\nThis document provides...\n");
    assert_xpath(&output, r#"//h1[text()="Document Title"]"#, 1);
}

// The author line directly beneath the title sets the author name and email.
#[test]
fn header_title_and_author_line() {
    verifies!(
        r#"
.Title and author line
----
include::document:example$header.adoc[tag=qr-author]
----

"#
    );

    let output = convert_standalone(
        "= Document Title\n\
         Author Name <author@email.org>\n\n\
         This document provides...\n",
    );
    assert_xpath(&output, r#"//span[@id="author"][text()="Author Name"]"#, 1);
    assert_xpath(&output, r#"//a[@href="mailto:author@email.org"]"#, 1);
}

// The revision line beneath the author line sets the version and date; multiple
// authors are separated by a semicolon.
#[test]
fn header_title_author_and_revision_line() {
    verifies!(
        r#"
.Title, author line, and revision line
----
include::document:example$header.adoc[tag=qr-rev]
----

"#
    );

    let output = convert_standalone(
        "= Document Title\n\
         Author Name <author@email.org>; Another Author <a.author@email.org>\n\
         v2.0, 2019-03-22\n\n\
         This document provides...\n",
    );
    assert_xpath(&output, r#"//span[@id="author"][text()="Author Name"]"#, 1);
    assert_xpath(
        &output,
        r#"//span[@id="author2"][text()="Another Author"]"#,
        1,
    );
    assert_css(&output, "span#revnumber", 1);
    assert_css(&output, "span#revdate", 1);
}

non_normative!(
    r#"
IMPORTANT: You cannot have a xref:document:revision-line.adoc[revision line] without an xref:document:author-line.adoc[author line].

"#
);

// Attribute entries in the header set document attributes such as `toc` and a
// custom `homepage`.
#[test]
fn document_header_with_attribute_entries() {
    verifies!(
        r#"
.Document header with attribute entries
----
include::document:example$header.adoc[tag=qr-attributes]
----

"#
    );

    let output = convert_standalone(
        "= Document Title\n\
         Author Name <author@email.org>\n\
         v2.0, 2019-03-22\n\
         :toc:\n\
         :homepage: https://example.org\n\n\
         This document provides...\n\n\
         == A Section\n",
    );
    assert_css(&output, "div#toc", 1);
}

non_normative!(
    r#"
[#section-titles]
== Section titles

When the document type is `article` (the default), the document can only have one level 0 section title (`=`), which is the document title (i.e., doctitle).

"#
);

// In an article, `=` is the doctitle and `==` .. `======` produce nested
// section levels 1 through 5.
#[test]
fn article_section_levels() {
    verifies!(
        r#"
.Article section levels
[#ex-article]
----
include::sections:example$section.adoc[tag=base]
----

.View result of <<ex-article>>
[%collapsible.result]
====
include::sections:example$section.adoc[tag=b-base]
====

"#
    );

    let output = convert(
        "= Document Title (Level 0)\n\n\
         == Level 1 Section Title\n\n\
         === Level 2 Section Title\n\n\
         ==== Level 3 Section Title\n\n\
         ===== Level 4 Section Title\n\n\
         ====== Level 5 Section Title\n\n\
         == Another Level 1 Section Title\n",
    );
    assert_css(&output, "div.sect1", 2);
    assert_css(&output, "div.sect2", 1);
    assert_css(&output, "div.sect3", 1);
    assert_css(&output, "div.sect4", 1);
    assert_css(&output, "div.sect5", 1);
}

non_normative!(
    r#"
The `book` document type can have additional level 0 section titles, which are interpreted as xref:sections:parts.adoc[parts].
The presence of at least one part implicitly makes the document a multi-part book.

"#
);

// In a book, a level-0 title below the doctitle is a part; a document with a
// part is a multi-part book.
#[test]
fn book_section_levels() {
    verifies!(
        r#"
.Book section levels
----
include::sections:example$section.adoc[tag=book]
----

"#
    );

    let output = convert(
        ":doctype: book\n\n\
         = Document Title (Level 0)\n\n\
         == Level 1 Section Title\n\n\
         = Level 0 Section Title (Part)\n\n\
         == Level 1 Section Title\n",
    );
    assert_css(&output, "h1.sect0", 1);
}

non_normative!(
    r#"
////
xref:sections:title-links.adoc#link[sectlinks]::
When the document attribute `sectlinks` is set, section titles become self-links.
This feature allows a reader to easily bookmark the section.

xref:sections:title-links.adoc#anchor[sectanchors]::
When the document attribute `sectanchors` is set, a floating section icon anchor appears in front of the section title on hover.
This feature provides an alternate way for the reader to easily bookmark the section.
Section title anchors depend on support from the stylesheet to render properly.
////

"#
);

// A `[discrete]` heading looks like a section title but is a standalone
// heading, not a section, so following blocks are its siblings.
#[test]
fn discrete_heading() {
    verifies!(
        r#"
.Discrete heading (not a section)
[#ex-discrete]
----
[discrete]
=== I'm an independent heading!

This paragraph is its sibling, not its child.
----

.View result of <<ex-discrete>>
[%collapsible.result]
====
[discrete]
=== I'm an independent heading!

This paragraph is its sibling, not its child.
====

"#
    );

    let output = convert(
        "[discrete]\n=== I'm an independent heading!\n\n\
         This paragraph is its sibling, not its child.\n",
    );
    assert_css(&output, "h3.discrete", 1);

    // The heading does not wrap the paragraph in a section body.
    assert_css(&output, "div.sect2", 0);
    assert_css(&output, "div.paragraph", 1);
}

non_normative!(
    r#"
== Automatic TOC

"#
);

// Setting the `toc` attribute in the header inserts a table of contents.
#[test]
fn activate_table_of_contents() {
    verifies!(
        r#"
.Activate Table of Contents for a document
----
= Document Title
Doc Writer <doc.writer@email.org>
:toc:
----

"#
    );

    let output = convert_standalone(
        "= Document Title\n\
         Doc Writer <doc.writer@email.org>\n\
         :toc:\n\n\
         Some content.\n\n\
         == A Section\n",
    );
    assert_css(&output, "div#toc.toc", 1);
    assert_css(&output, "div#toctitle", 1);
}

non_normative!(
    r#"
The Table of Contents`' xref:toc:title.adoc[title], xref:toc:levels.adoc[displayed section depth], and xref:toc:position.adoc[position] can be customized.

"#
);

non_normative!(
    r#"
== Includes

"#
);

// The include directive pulls one document into another; the cheatsheet shows
// the directive syntax escaped so it displays literally.
#[test]
fn include_document_parts() {
    verifies!(
        r#"
.Include document parts
----
include::directives:example$include.adoc[tag=base]
----

"#
    );

    let output = convert(
        "= Reference Documentation\nLead Developer\n\n\
         This is documentation for project X.\n\n\
         \\include::basics.adoc[]\n\n\
         \\include::installation.adoc[]\n\n\
         \\include::example.adoc[]\n",
    );
    assert!(output.contains("include::basics.adoc[]"));
    assert!(output.contains("include::installation.adoc[]"));
    assert!(output.contains("include::example.adoc[]"));
}

// An include can select a tagged region (`tag=`) or a line range (`lines=`).
#[test]
fn include_by_tagged_regions_or_lines() {
    verifies!(
        r#"
.Include content by tagged regions or lines
----
include::directives:example$include.adoc[tag=include-with-tag]

include::directives:example$include.adoc[tag=line]
----

"#
    );

    let output = convert(
        "\\include::filename.txt[tag=definition]\n\n\
         \\include::filename.txt[lines=5..10]\n",
    );
    assert!(output.contains("include::filename.txt[tag=definition]"));
    assert!(output.contains("include::filename.txt[lines=5..10]"));
}

// Reading an include from a URL is out of scope for this renderer (per the
// README it never reads over the network), and Asciidoctor disables it under
// SECURE, so the URL-include behavior shown here is non-normative.
non_normative!(
    r#"
.Include content from a URL
----
include::directives:example$include.adoc[tag=uri]
----

WARNING: Including content from a URL is potentially dangerous, so it's disabled if the safe mode is SECURE or greater.
Assuming the safe mode is less than SECURE, you must also set the `allow-uri-read` attribute to permit the AsciiDoc processor to read content from a URL.

"#
);

non_normative!(
    r#"
== Lists

"#
);

// An unordered list nests by repeating the `*` marker; leading whitespace
// before a marker is not significant.
#[test]
fn unordered_list() {
    verifies!(
        r#"
.Unordered list
[#ex-ul]
----
include::lists:example$unordered.adoc[tag=qr-base]
----

.View result of <<ex-ul>>
[%collapsible.result]
====
include::lists:example$unordered.adoc[tag=qr-base]
====

"#
    );

    let output = convert(
        "* List item\n\
         ** Nested list item\n\
         *** Deeper nested list item\n\
         * List item\n\
         \x20** Another nested list item\n\
         * List item\n",
    );
    assert_css(&output, "div.ulist", 4);
    assert_css(&output, "li", 6);
}

non_normative!(
    r#"
TIP: An empty line is required before and after a list to separate it from other blocks.
You can force two adjacent lists apart by adding an empty attribute list (i.e., `[]`) above the second list or by inserting an empty line followed by a line comment after the first list.
If you use a line comment, the convention is to use `//-` to provide a hint to other authors that it's serving as a list divider.

"#
);

// An unordered list nests up to five levels.
#[test]
fn unordered_list_max_nesting() {
    verifies!(
        r#"
.Unordered list max level nesting
[#ex-ul-max]
----
include::lists:example$unordered.adoc[tag=max]
----

.View result of <<ex-ul-max>>
[%collapsible.result]
====
include::lists:example$unordered.adoc[tag=max]
====

"#
    );

    let output = convert(
        "* Level 1 list item\n\
         ** Level 2 list item\n\
         *** Level 3 list item\n\
         **** Level 4 list item\n\
         ***** Level 5 list item\n\
         ****** etc.\n\
         * Level 1 list item\n",
    );
    assert_css(&output, "div.ulist", 6);
    assert_css(&output, "li", 7);
}

non_normative!(
    r#"
The xref:lists:unordered.adoc#markers[unordered list marker] can be changed using a list style (e.g., `square`).

"#
);

// An ordered list nests by repeating the `.` marker.
#[test]
fn ordered_list() {
    verifies!(
        r#"
.Ordered list
[#ex-ol]
----
include::lists:example$ordered.adoc[tag=nest]
----

.View result of <<ex-ol>>
[%collapsible.result]
====
include::lists:example$ordered.adoc[tag=nest]
====

"#
    );

    let output = convert(". Step 1\n. Step 2\n.. Step 2a\n.. Step 2b\n. Step 3\n");
    assert_css(&output, "div.olist", 2);
    assert_css(&output, "ol", 2);
    assert_css(&output, "li", 5);
}

// An ordered list nests up to five levels.
#[test]
fn ordered_list_max_nesting() {
    verifies!(
        r#"
.Ordered list max level nesting
[#ex-ol-max]
----
include::lists:example$ordered.adoc[tag=max]
----

.View result of <<ex-ol-max>>
[%collapsible.result]
====
include::lists:example$ordered.adoc[tag=max]
====

"#
    );

    let output = convert(
        ". Level 1 list item\n\
         .. Level 2 list item\n\
         ... Level 3 list item\n\
         .... Level 4 list item\n\
         ..... Level 5 list item\n\
         . Level 1 list item\n",
    );
    assert_css(&output, "div.olist", 5);
    assert_css(&output, "li", 6);
}

non_normative!(
    r#"
Ordered lists support xref:lists:ordered.adoc#styles[numeration styles] such as `lowergreek` and `decimal-leading-zero`.

"#
);

// A checklist marks items with `[*]`/`[x]` (checked) or `[ ]` (unchecked).
#[test]
fn checklist() {
    verifies!(
        r#"
.Checklist
[#ex-check]
----
include::lists:example$checklist.adoc[tag=check]
----

.View result of <<ex-check>>
[%collapsible.result]
====
include::lists:example$checklist.adoc[tag=check]
====

"#
    );

    let output =
        convert("* [*] checked\n* [x] also checked\n* [ ] not checked\n* normal list item\n");
    assert_css(&output, "ul.checklist", 1);
    assert_css(&output, "li", 4);

    // Checked items render a filled box, unchecked an empty one.
    assert!(output.contains("&#10003;"));
    assert!(output.contains("&#10063;"));
}

// A description list pairs each term (`term::`) with its description.
#[test]
fn description_list() {
    verifies!(
        r#"
.Description list
[#ex-dlist]
----
include::lists:example$description.adoc[tag=qr-base]
----

.View result of <<ex-dlist>>
[%collapsible.result]
====
include::lists:example$description.adoc[tag=qr-base]
====

"#
    );

    let output = convert(
        "First term:: The description can be placed on the same line\n\
         as the term.\n\
         Second term::\n\
         Description of the second term.\n\
         The description can also start on its own line.\n",
    );
    assert_css(&output, "div.dlist", 1);
    assert_css(&output, "dt.hdlist1", 2);
    assert_css(&output, "dd", 2);
}

// The `qanda` style renders a description list as a numbered
// question-and-answer list.
#[test]
fn question_and_answer_list() {
    verifies!(
        r#"
.Question and answer list
[#ex-qa]
----
include::lists:example$description.adoc[tag=qa]
----

.View result of <<ex-qa>>
[%collapsible.result]
====
include::lists:example$description.adoc[tag=qa]
====

"#
    );

    let output = convert(
        "[qanda]\n\
         What is the answer?::\n\
         This is the answer.\n\n\
         Are cameras allowed?::\n\
         Are backpacks allowed?::\n\
         No.\n",
    );
    assert_css(&output, "div.qlist.qanda", 1);
    assert_css(&output, "ol", 1);
    assert_css(&output, "ol > li", 2);
}

// Description, ordered, and unordered lists nest freely within one another.
#[test]
fn mixed_nested_lists() {
    verifies!(
        r#"
.Mixed
[#ex-mixed]
----
include::lists:example$description.adoc[tag=3-mix]
----

.View result of <<ex-mixed>>
[%collapsible.result]
====
include::lists:example$description.adoc[tag=3-mix]
====

"#
    );

    let output = convert(
        "Operating Systems::\n\
         \x20 Linux:::\n\
         \x20   . Fedora\n\
         \x20     * Desktop\n\
         \x20   . Ubuntu\n\
         \x20     * Desktop\n\
         \x20     * Server\n\
         \x20 BSD:::\n\
         \x20   . FreeBSD\n\
         \x20   . NetBSD\n\n\
         Cloud Providers::\n\
         \x20 PaaS:::\n\
         \x20   . OpenShift\n\
         \x20   . CloudBees\n\
         \x20 IaaS:::\n\
         \x20   . Amazon EC2\n\
         \x20   . Rackspace\n",
    );
    assert_css(&output, "div.dlist", 3);
    assert_css(&output, "div.olist", 4);
}

non_normative!(
    r#"
TIP: Lists can be indented.
Leading whitespace is not significant.

"#
);

// A list item may hold compound content: extra paragraphs and blocks joined by
// a list continuation (`+`), a literal paragraph, a nested list, and a table.
#[test]
fn complex_content_in_outline_lists() {
    verifies!(
        r#"
.Complex content in outline lists
[#ex-complex]
----
include::lists:example$complex.adoc[tag=b-complex]
----

.View result of <<ex-complex>>
[%collapsible.result]
====
include::lists:example$complex.adoc[tag=b-complex]
====

"#
    );

    let output = convert(
        "* Every list item has at least one paragraph of content,\n\
         \x20 which may be wrapped, even using a hanging indent.\n\
         +\n\
         Additional paragraphs or blocks are adjoined by putting\n\
         a list continuation on a line adjacent to both blocks.\n\
         +\n\
         list continuation:: a plus sign (`{plus}`) on a line by itself\n\n\
         * A literal paragraph does not require a list continuation.\n\n\
         \x20$ cd projects/my-book\n\n\
         * AsciiDoc lists may contain any compound content.\n\
         +\n\
         |===\n\
         |Column 1, Header Row |Column 2, Header Row\n\n\
         |Column 1, Row 1\n\
         |Column 2, Row 1\n\
         |===\n",
    );
    assert_css(&output, "div.ulist", 1);
    assert_css(&output, "div.dlist", 1);
    assert_css(&output, "div.literalblock", 1);
    assert_css(&output, "table", 1);
}

non_normative!(
    r#"
== Images

You can use the xref:macros:images-directory.adoc[imagesdir attribute] to avoid hard coding the common path to your images in every image macro.
The value of this attribute can be an absolute path, relative path, or base URL.
If the image target is a relative path, the attribute's value is prepended (i.e., it's resolved relative to the value of the `imagesdir` attribute).
If the image target is a URL or absolute path, the attribute's value is _not_ prepended.

"#
);

// The block image macro (`image::`) renders a figure, optionally with alt text,
// a title, sizing, a link, or a URL/absolute target.
#[test]
fn block_image_macro() {
    verifies!(
        r#"
.Block image macro
[#ex-image-blocks]
----
include::macros:example$image.adoc[tag=base]

include::macros:example$image.adoc[tag=alt]

include::macros:example$image.adoc[tag=qr-attr]

include::macros:example$image.adoc[tag=ab-url]
----

.View result of <<ex-image-blocks>>
[%collapsible.result]
====
include::macros:example$image.adoc[tag=qr-base]

include::macros:example$image.adoc[tag=qr-alt]

include::macros:example$image.adoc[tag=qr-attr]

include::macros:example$image.adoc[tag=ab-url]
====

"#
    );

    let output = convert(
        "image::sunset.jpg[]\n\n\
         image::sunset.jpg[Sunset]\n\n\
         .A mountain sunset\n\
         [#img-sunset,caption=\"Figure 1: \",link=https://www.flickr.com/photos/javh/5448336655]\n\
         image::macros:sunset.jpg[Sunset,200,100]\n\n\
         image::https://asciidoctor.org/images/octocat.jpg[GitHub mascot]\n",
    );
    assert_css(&output, "div.imageblock", 4);
    assert_css(&output, "img", 4);
    assert_css(&output, "a.image", 1);
    assert_css(&output, "div.imageblock div.title", 1);
}

non_normative!(
    r#"
Two colons following the image keyword in the macro (i.e., `image::`) indicates a block image (aka figure), whereas one colon following the image keyword (i.e., `image:`) indicates an inline image.
(All macros follow this pattern).
You use an inline image when you need to place the image in a line of text.
Otherwise, you should prefer the block form.

"#
);

// The inline image macro (`image:`) places an image within a line of text.
#[test]
fn inline_image_macro() {
    verifies!(
        r#"
.Inline image macro
[#ex-image-inline]
----
include::macros:example$image.adoc[tag=inline]
----

.View result of <<ex-image-inline>>
[%collapsible.result]
====
include::macros:example$image.adoc[tag=qr-inline]
====

"#
    );

    let output = convert(
        "Click image:play.png[] to get the party started.\n\n\
         Click image:pause.png[title=Pause] when you need a break.\n",
    );
    assert_css(&output, "span.image", 2);
    assert_css(&output, "img", 2);
}

// A positioning role (e.g. `right`) on an inline image adds it as a class.
#[test]
fn inline_image_with_positioning_role() {
    verifies!(
        r#"
.Inline image macro with positioning role
[#ex-image-role]
----
include::macros:example$image.adoc[tag=in-role]
----

.View result of <<ex-image-role>>
[%collapsible.result]
====
include::macros:example$image.adoc[tag=qr-role]
====

"#
    );

    let output = convert("image:sunset.jpg[Sunset,150,150,role=right] What a beautiful sunset!\n");
    assert_css(&output, "span.image.right", 1);
    assert_css(&output, "img", 1);
}

// The data-uri attribute embeds images as data URIs; this renderer does not yet
// embed resources (issue #51), so this behavior is non-normative here.
non_normative!(
    r#"
.Embedded
----
include::macros:example$image.adoc[tag=data]
----

When the `data-uri` attribute is set, all images in the document--including admonition icons--are embedded into the document as {url-data-uri}[data URIs].
You can also pass it as a command line argument using `-a data-uri`.

"#
);

non_normative!(
    r#"
== Audio

"#
);

// The block audio macro (`audio::`) embeds an audio player; attributes set the
// start time and autoplay.
#[test]
fn block_audio_macro() {
    verifies!(
        r#"
.Block audio macro
----
include::macros:example$audio.adoc[tag=basic]

include::macros:example$audio.adoc[tag=attrs]
----

"#
    );

    let output = convert(
        "audio::ocean-waves.wav[]\n\n\
         audio::ocean-waves.wav[start=60,opts=autoplay]\n",
    );
    assert_css(&output, "div.audioblock", 2);
    assert_css(&output, "audio", 2);
    assert_css(&output, "audio[autoplay]", 1);
}

non_normative!(
    r#"
You can control the audio settings using xref:macros:audio-and-video.adoc[additional attributes and options] on the macro.

"#
);

non_normative!(
    r#"
== Videos

"#
);

// The block video macro (`video::`) embeds a self-hosted video; attributes set
// the width, start time, and autoplay.
#[test]
fn block_video_macro() {
    verifies!(
        r#"
.Block video macro
----
include::macros:example$video.adoc[tag=base]

include::macros:example$video.adoc[tag=attr]
----

"#
    );

    let output = convert(
        "video::video-file.mp4[]\n\n\
         video::video-file.mp4[width=640,start=60,opts=autoplay]\n",
    );
    assert_css(&output, "div.videoblock", 2);
    assert_css(&output, "video", 2);
    assert_css(&output, "video[width=\"640\"]", 1);
}

// The `youtube` target embeds a YouTube video in an iframe.
#[test]
fn embedded_youtube_video() {
    verifies!(
        r#"
.Embedded YouTube video
----
include::macros:example$video.adoc[tag=youtube]
----

"#
    );

    let output = convert("video::RvRhUHTV_8k[youtube]\n");
    assert_css(&output, "div.videoblock", 1);
    assert!(output.contains("https://www.youtube.com/embed/RvRhUHTV_8k"));
}

// The `vimeo` target embeds a Vimeo video in an iframe.
#[test]
fn embedded_vimeo_video() {
    verifies!(
        r#"
.Embedded Vimeo video
----
include::macros:example$video.adoc[tag=vimeo]
----

"#
    );

    let output = convert("video::67480300[vimeo]\n");
    assert_css(&output, "div.videoblock", 1);
    assert!(output.contains("https://player.vimeo.com/video/67480300"));
}

non_normative!(
    r#"
You can control the video settings using xref:macros:audio-and-video.adoc[additional attributes and options] on the macro.

"#
);

non_normative!(
    r#"
== Keyboard, button, and menu macros

IMPORTANT: You must set the `experimental` attribute in the document header to enable these macros.

"#
);

// The `kbd:` macro renders a keyboard shortcut; a `+`-joined combination
// becomes a key sequence.
#[test]
fn keyboard_macro() {
    verifies!(
        r#"
.Keyboard macro
[#ex-kbd]
----
include::macros:example$ui.adoc[tag=qr-key]
----

.View result of <<ex-kbd>>
[%collapsible.result]
====
include::macros:example$ui.adoc[tag=qr-key]
====

"#
    );

    let output = convert(
        ":experimental:\n\n\
         |===\n\
         |Shortcut |Purpose\n\n\
         |kbd:[F11]\n\
         |Toggle fullscreen\n\n\
         |kbd:[Ctrl+T]\n\
         |Open a new tab\n\
         |===\n",
    );
    assert_css(&output, "kbd", 3);
    assert_css(&output, "span.keyseq", 1);
}

// The `menu:` macro renders a menu selection, including a nested submenu path.
#[test]
fn menu_macro() {
    verifies!(
        r#"
.Menu macro
[#ex-menu]
----
include::macros:example$ui.adoc[tag=menu]
----

.View result of <<ex-menu>>
[%collapsible.result]
====
include::macros:example$ui.adoc[tag=menu]
====

"#
    );

    let output = convert(
        ":experimental:\n\n\
         To save the file, select menu:File[Save].\n\n\
         Select menu:View[Zoom > Reset] to reset the zoom level to the default setting.\n",
    );
    assert_css(&output, "span.menuseq", 2);
    assert_css(&output, "b.menu", 2);
}

// The `btn:` macro renders a user-interface button.
#[test]
fn button_macro() {
    verifies!(
        r#"
.Button macro
[#ex-button]
----
include::macros:example$ui.adoc[tag=button]
----

.View result of <<ex-button>>
[%collapsible.result]
====
include::macros:example$ui.adoc[tag=button]
====

"#
    );

    let output = convert(
        ":experimental:\n\n\
         Press the btn:[OK] button when you are finished.\n\n\
         Select a file in the file navigator and click btn:[Open].\n",
    );
    assert_css(&output, "b.button", 2);
}

non_normative!(
    r#"
== Literals and source code

"#
);

non_normative!(
    r#"
////
.Inline monospace only
[#ex-inline-code]
----
include::text:example$text.adoc[tag=b-mono-code]
----

.View result of <<ex-inline-code>>
[%collapsible.result]
====
include::text:example$text.adoc[tag=b-mono-code]
====
////

"#
);

// Enclosing text in `` `+...+` `` outputs literal monospace, suppressing
// substitutions within.
#[test]
fn inline_literal_monospace() {
    verifies!(
        r#"
.Inline literal monospace
[#ex-inline-literal]
----
include::pass:example$pass.adoc[tag=backtick-plus]
----

.View result of <<ex-inline-literal>>
[%collapsible.result]
====
include::pass:example$pass.adoc[tag=backtick-plus]
====

"#
    );

    let output = convert(
        "Output literal monospace text, such as `+{backtick}+` or `+http://localhost:8080+`, by enclosing the text in a pair of pluses surrounded by a pair of backticks.\n",
    );
    assert_css(&output, "code", 2);

    // The attribute reference is not resolved inside the literal monospace span.
    assert!(output.contains("<code>{backtick}</code>"));
}

// Indenting a line by one space makes it a literal line, set between normal
// paragraphs.
#[test]
fn literal_paragraph_indented() {
    verifies!(
        r#"
.Literal paragraph
[#ex-literal-line]
----
include::verbatim:example$literal.adoc[tag=b-imp-code]
----

.View result of <<ex-literal-line>>
[%collapsible.result]
====
include::verbatim:example$literal.adoc[tag=b-imp-code]
====

"#
    );

    let output = convert(
        "Normal line.\n\n\
         \x20Indent line by one space to create a literal line.\n\n\
         Normal line.\n",
    );
    assert_css(&output, "div.literalblock", 1);
    assert_css(&output, "div.paragraph", 2);
}

// A `....` delimited block renders its content verbatim as a literal block.
#[test]
fn literal_block() {
    verifies!(
        r#"
.Literal block
[#ex-literal-block]
----
include::verbatim:example$literal.adoc[tag=b-block]
----

.View result of <<ex-literal-block>>
[%collapsible.result]
====
include::verbatim:example$literal.adoc[tag=b-block]
====

"#
    );

    let output = convert(
        "....\n\
         error: 1954 Forbidden search\n\
         absolutely fatal: operation lost in the dodecahedron of doom\n\n\
         Would you like to try again? y/n\n\
         ....\n",
    );
    assert_css(&output, "div.literalblock", 1);
    assert!(output.contains("Forbidden search"));
}

// A `----` delimited listing block, with a block title, renders verbatim.
#[test]
fn listing_block_with_title() {
    verifies!(
        r#"
.Listing block with title
[#ex-listing]
------
include::verbatim:example$listing.adoc[tag=qr-listing]
------

.View result of <<ex-listing>>
[%collapsible.result]
====
[caption="Listing 1. "]
[listing]
include::verbatim:example$listing.adoc[tag=qr-listing]
====

"#
    );

    let output = convert(
        ".Gemfile.lock\n\
         ----\n\
         GEM\n\
         \x20 remote: https://rubygems.org/\n\
         \x20 specs:\n\
         \x20   asciidoctor (2.0.15)\n\n\
         PLATFORMS\n\
         \x20 ruby\n\n\
         DEPENDENCIES\n\
         \x20 asciidoctor (~> 2.0.15)\n\
         ----\n",
    );
    assert_css(&output, "div.listingblock", 1);
    assert_css(&output, "div.listingblock div.title", 1);
    assert!(output.contains("asciidoctor (2.0.15)"));
}

// A `[source,ruby]` listing block sets the code language for highlighting.
#[test]
fn source_block_with_title_and_language() {
    verifies!(
        r#"
.Source block with title and syntax highlighting
[#ex-highlight]
------
.Some Ruby code
include::verbatim:example$source.adoc[tag=src-base]
------

.View result of <<ex-highlight>>
[%collapsible.result]
====
[caption="Listing 1. "]
.Some Ruby code
include::verbatim:example$source.adoc[tag=src-base]
====

"#
    );

    let output = convert(
        ".Some Ruby code\n\
         [source,ruby]\n\
         ----\n\
         require 'sinatra'\n\n\
         get '/hi' do\n\
         \x20 \"Hello World!\"\n\
         end\n\
         ----\n",
    );
    assert_css(&output, "div.listingblock", 1);
    assert_css(&output, "code.language-ruby", 1);
}

// The rouge highlighter shown here is a server-side highlighter this renderer
// does not run (issue #223), so enabling it is non-normative.
non_normative!(
    r#"
[IMPORTANT]
====
You must enable xref:verbatim:source-highlighter.adoc[source highlighting] by setting the `source-highlighter` attribute in the document header, CLI, or API.

----
:source-highlighter: rouge
----

See xref:asciidoctor:syntax-highlighting:index.adoc[] to learn which values are accepted when using Asciidoctor.
====

"#
);

// Callout markers (`<1>`) in a source block link to a numbered callout list.
#[test]
fn source_block_with_callouts() {
    verifies!(
        r#"
.Source block with callouts
[#ex-callouts,subs=-callouts]
------
include::verbatim:example$callout.adoc[tag=b-src]
------

.View result of <<ex-callouts>>
[%collapsible.result]
====
include::verbatim:example$callout.adoc[tag=b-src]
====

"#
    );

    let output = convert(
        "[source,ruby]\n\
         ----\n\
         require 'sinatra' // <1>\n\n\
         get '/hi' do // <2>\n\
         \x20 \"Hello World!\" // <3>\n\
         end\n\
         ----\n\
         <1> Library import\n\
         <2> URL mapping\n\
         <3> HTTP response body\n",
    );
    assert_css(&output, "b.conum", 3);
    assert_css(&output, "div.colist", 1);
    assert_css(&output, "div.colist li", 3);
}

// Callouts placed behind a line comment are hidden from selection but still map
// to the callout list.
#[test]
fn make_callouts_non_selectable() {
    verifies!(
        r#"
.Make callouts non-selectable
[#ex-hide-callouts,subs=-callouts]
------
include::verbatim:example$callout.adoc[tag=b-nonselect]
------

.View result of <<ex-hide-callouts>>
[%collapsible.result]
====
include::verbatim:example$callout.adoc[tag=b-nonselect]
====

"#
    );

    let output = convert(
        "----\n\
         line of code // <1>\n\
         line of code # <2>\n\
         line of code ;; <3>\n\
         line of code <!--4-->\n\
         ----\n\
         <1> A callout behind a line comment for C-style languages.\n\
         <2> A callout behind a line comment for Ruby, Python, Perl, etc.\n\
         <3> A callout behind a line comment for Clojure.\n\
         <4> A callout behind a line comment for XML or SGML languages like HTML.\n",
    );
    assert_css(&output, "b.conum", 4);
    assert_css(&output, "div.colist li", 4);
}

// An include directive inside a source block pulls in file content; the escaped
// directive displays literally here.
#[test]
fn source_block_content_included_from_a_file() {
    verifies!(
        r#"
.Source block content included from a file
------
include::verbatim:example$source.adoc[tag=src-inc]
------

"#
    );

    let output = convert("[,ruby]\n----\n\\include::app.rb[]\n----\n");
    assert_css(&output, "code.language-ruby", 1);
    assert!(output.contains("include::app.rb[]"));
}

// A source-directory attribute resolves an included file path relative to it.
#[test]
fn source_block_content_included_relative_to_source_directory() {
    verifies!(
        r#"
.Source block content included from file relative to source directory
------
include::verbatim:example$source.adoc[tag=rel]
------

"#
    );

    let output = convert(
        ":sourcedir: src/main/java\n\n\
         [source,java]\n----\n\\include::{sourcedir}/org/asciidoctor/Asciidoctor.java[]\n----\n",
    );
    assert_css(&output, "code.language-java", 1);
    assert!(output.contains("Asciidoctor.java[]"));
}

// The `indent=0` include option strips leading indentation from included
// content.
#[test]
fn strip_leading_indentation_from_partial_file_content() {
    verifies!(
        r#"
.Strip leading indentation from partial file content
------
include::verbatim:example$source.adoc[tag=ind]
------

"#
    );

    let output = convert("[source,ruby]\n----\n\\include::lib/app.rb[tag=main,indent=0]\n----\n");
    assert_css(&output, "code.language-ruby", 1);
    assert!(output.contains("include::lib/app.rb[tag=main,indent=0]"));
}

non_normative!(
    r#"
[NOTE]
====
The xref:directives:include-with-indent.adoc[indent attribute] is frequently used when including source code by xref:directives:include-tagged-regions.adoc[tagged region] or xref:directives:include-lines.adoc[lines].
It can be specified on the include directive itself or the enclosing literal, listing, or source block.

When indent is 0, the leading block indent is stripped.

When indent is greater than 0, the leading block indent is first stripped, then a block is indented by the number of columns equal to this value.
====

"#
);

// The source paragraph style renders a single-paragraph source block without
// delimiters.
#[test]
fn source_paragraph() {
    verifies!(
        r#"
.Source paragraph (no empty lines)
[#ex-source-para]
----
include::verbatim:example$source.adoc[tag=src-para]
----

.View result of <<ex-source-para>>
[%collapsible.result]
====
include::verbatim:example$source.adoc[tag=src-para]
====

"#
    );

    let output = convert(
        "[source,xml]\n\
         <meta name=\"viewport\"\n\
         \x20 content=\"width=device-width, initial-scale=1.0\">\n\n\
         This is normal content.\n",
    );
    assert_css(&output, "div.listingblock", 1);
    assert_css(&output, "code.language-xml", 1);
    assert_css(&output, "div.paragraph", 1);
}

non_normative!(
    r#"
== Admonitions

"#
);

// An admonition label (`NOTE:`, `TIP:`, ...) on a paragraph renders a labeled
// admonition block, one per built-in type.
#[test]
fn admonition_paragraph() {
    verifies!(
        r#"
.Admonition paragraph
[#ex-admon-para]
----
include::blocks:example$admonition.adoc[tag=b-para]
----

.View result of <<ex-admon-para>>
[%collapsible.result]
====
include::blocks:example$admonition.adoc[tag=b-para]
====

"#
    );

    let output = convert(
        "NOTE: An admonition draws the reader's attention to auxiliary information.\n\n\
         Here are the other built-in admonition types:\n\n\
         IMPORTANT: Don't forget the children!\n\n\
         TIP: Look for the warp zone under the bridge.\n\n\
         CAUTION: Slippery when wet.\n\n\
         WARNING: The software you're about to use is untested.\n\n\
         IMPORTANT: Sign off before stepping away from your computer.\n",
    );
    assert_css(&output, "div.admonitionblock", 6);
    assert_css(&output, "div.admonitionblock.note", 1);
    assert_css(&output, "div.admonitionblock.important", 2);
    assert_css(&output, "div.admonitionblock.tip", 1);
    assert_css(&output, "div.admonitionblock.caution", 1);
    assert_css(&output, "div.admonitionblock.warning", 1);
}

// The `[NOTE]` block style renders an admonition that may hold compound
// content.
#[test]
fn admonition_block() {
    verifies!(
        r#"
.Admonition block
[#ex-admon-block]
----
include::blocks:example$admonition.adoc[tag=b-bl]
----

.View result of <<ex-admon-block>>
[%collapsible.result]
=====
include::blocks:example$admonition.adoc[tag=b-bl]
=====

"#
    );

    let output = convert(
        "[NOTE]\n\
         ====\n\
         An admonition block may contain compound content.\n\n\
         .A list\n\
         - one\n\
         - two\n\
         - three\n\n\
         Another paragraph.\n\
         ====\n",
    );
    assert_css(&output, "div.admonitionblock.note", 1);
    assert_css(&output, "td.content div.ulist", 1);
    assert_css(&output, "td.content div.paragraph", 2);
}

non_normative!(
    r#"
== More delimited blocks

Any block can have a title.
A block title is defined using a line of text above the block that starts with a dot.
That dot cannot be followed by a space.
For block images, the title is displayed below the block.
For all other blocks, the title is typically displayed above it.

"#
);

// A `****` delimited block renders a sidebar, optionally with a title.
#[test]
fn sidebar_block() {
    verifies!(
        r#"
.Sidebar block
[#ex-sidebar]
----
include::blocks:example$sidebar.adoc[tag=delimited]
----

.View result of <<ex-sidebar>>
[%collapsible.result]
====
include::blocks:example$sidebar.adoc[tag=delimited]
====

"#
    );

    let output = convert(
        ".Optional Title\n\
         ****\n\
         Sidebars are used to visually separate auxiliary bits of content\n\
         that supplement the main text.\n\
         ****\n",
    );
    assert_css(&output, "div.sidebarblock", 1);
    assert_css(&output, "div.sidebarblock div.title", 1);
}

// A `====` delimited block renders an example that can enclose other blocks.
#[test]
fn example_block() {
    verifies!(
        r#"
.Example block
[#ex-example]
------
include::blocks:example$example.adoc[tag=base]
------

.View result of <<ex-example>>
[example%collapsible.result]
--
include::blocks:example$example.adoc[tag=base]
--

"#
    );

    let output = convert(
        "====\n\
         Here's a sample AsciiDoc document:\n\n\
         ----\n\
         = Title of Document\n\
         Doc Writer\n\
         :toc:\n\n\
         This guide provides...\n\
         ----\n\n\
         The document header is useful, but not required.\n\
         ====\n",
    );
    assert_css(&output, "div.exampleblock", 1);
    assert_css(&output, "div.exampleblock div.listingblock", 1);
}

// Quote blocks and paragraphs: with attribution and citation, attribution only,
// no citation, a citation link, and the abbreviated `-- attribution` form.
#[test]
fn blockquotes() {
    verifies!(
        r#"
.Blockquotes
[#ex-quotes]
----
include::blocks:example$quote.adoc[tag=bl]

include::blocks:example$quote.adoc[tag=para]

include::blocks:example$quote.adoc[tag=no-cite]

include::blocks:example$quote.adoc[tag=link-text]

include::blocks:example$quote.adoc[tag=abbr]
----

.View result of <<ex-quotes>>
[%collapsible.result]
====
include::blocks:example$quote.adoc[tag=bl]

include::blocks:example$quote.adoc[tag=para]

include::blocks:example$quote.adoc[tag=no-cite]

include::blocks:example$quote.adoc[tag=link-text]

include::blocks:example$quote.adoc[tag=abbr]
====

"#
    );

    let output = convert(
        "[quote,Abraham Lincoln,Address delivered at the dedication of the Cemetery at Gettysburg]\n\
         ____\n\
         Four score and seven years ago our fathers brought forth\n\
         on this continent a new nation...\n\
         ____\n\n\
         [quote,Albert Einstein]\n\
         A person who never made a mistake never tried anything new.\n\n\
         ____\n\
         A person who never made a mistake never tried anything new.\n\
         ____\n\n\
         [quote,Charles Lutwidge Dodgson,'Mathematician and author, also known as https://en.wikipedia.org/wiki/Lewis_Carroll[Lewis Carroll]']\n\
         ____\n\
         If you don't know where you are going, any road will get you there.\n\
         ____\n\n\
         \"I hold it that a little rebellion now and then is a good thing,\n\
         and as necessary in the political world as storms in the physical.\"\n\
         -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11\n",
    );
    assert_css(&output, "div.quoteblock", 5);
    assert_css(&output, "div.attribution", 4);
    assert_css(&output, "div.attribution a", 1);
}

// A `--` open block is an anonymous container; with a style it masquerades as
// another block (e.g. a source block).
#[test]
fn open_blocks() {
    verifies!(
        r#"
.Open blocks
[#ex-open]
----
include::blocks:example$open.adoc[tag=base]

include::blocks:example$open.adoc[tag=src]
----

.View result of <<ex-open>>
[%collapsible.result]
====
include::blocks:example$open.adoc[tag=base]

include::blocks:example$open.adoc[tag=src]
====

"#
    );

    let output = convert(
        "--\n\
         An open block can be an anonymous container,\n\
         or it can masquerade as any other block.\n\
         --\n\n\
         [source]\n\
         --\n\
         puts \"I'm a source block!\"\n\
         --\n",
    );
    assert_css(&output, "div.openblock", 1);
    assert_css(&output, "div.listingblock", 1);
}

// A `++++` passthrough block emits its content unprocessed, so raw HTML passes
// straight to the output.
#[test]
fn passthrough_block() {
    verifies!(
        r#"
.Passthrough block
[#ex-pass-block]
----
include::pass:example$pass.adoc[tag=b-bl]
----

.View result of <<ex-pass-block>>
[%collapsible.result]
====
include::pass:example$pass.adoc[tag=b-bl]
====

"#
    );

    let output = convert(
        "++++\n\
         <p>\n\
         Content in a passthrough block is passed to the output unprocessed.\n\
         That means you can include raw HTML, like this embedded Gist:\n\
         </p>\n\n\
         <script src=\"https://gist.github.com/mojavelinux/5333524.js\">\n\
         </script>\n\
         ++++\n",
    );
    assert!(output.contains("<script src=\"https://gist.github.com/mojavelinux/5333524.js\">"));

    // Passthrough content is emitted with no block wrapper.
    assert_css(&output, "div.paragraph", 0);
}

// The `subs=attributes+` block attribute adds attribute substitution to a
// source block, so an attribute reference is resolved.
#[test]
fn customize_block_substitutions() {
    verifies!(
        r#"
.Customize block substitutions
[#ex-block-subs,subs=+macros]
------
include::verbatim:example$listing.adoc[tag=subs]
------

.View result of <<ex-block-subs>>
[%collapsible.result]
====
include::verbatim:example$listing.adoc[tag=subs-out]
====

"#
    );

    let output = convert(
        ":release-version: 2.4.3\n\n\
         [source,xml,subs=attributes+]\n\
         ----\n\
         <dependency>\n\
         \x20 <groupId>org.asciidoctor</groupId>\n\
         \x20 <artifactId>asciidoctorj</artifactId>\n\
         \x20 <version>{release-version}</version>\n\
         </dependency>\n\
         ----\n",
    );
    assert_css(&output, "code.language-xml", 1);

    // The attribute reference resolves; the literal braces are gone.
    assert!(output.contains("2.4.3"));
    assert!(!output.contains("{release-version}"));
}

non_normative!(
    r#"
== Tables

"#
);

// A table with a block title and a header row inferred from the first line
// followed by an empty line.
#[test]
fn table_with_title_header_and_rows() {
    verifies!(
        r#"
.Table with a title, two columns, a header row, and two rows of content
[#ex-header-row]
----
include::tables:example$table.adoc[tag=b-base-h-co]
----
<.> Unless the `cols` attribute is specified, the number of columns is equal to the number of cell separators on the first (non-empty) line.
<.> When an empty line immediately follows a non-empty line at the start of the table, the cells in the first line get promoted to the table header.

.View result of <<ex-header-row>>
[%collapsible.result]
====
[caption="Table 1. "]
include::tables:example$table.adoc[tag=b-base-h]
====

"#
    );

    let output = convert(
        ".Table Title\n\
         |===\n\
         |Column 1, Header Row |Column 2, Header Row\n\n\
         |Cell in column 1, row 1\n\
         |Cell in column 2, row 1\n\n\
         |Cell in column 1, row 2\n\
         |Cell in column 2, row 2\n\
         |===\n",
    );
    assert_css(&output, "table", 1);
    assert_css(&output, "caption.title", 1);
    assert_css(&output, "thead th", 2);
    assert_css(&output, "tbody tr", 2);
}

// The `cols` attribute with the repeat operator (`2*`) sets the column count;
// the `%header` option promotes the first row.
#[test]
fn table_with_cols_and_header() {
    verifies!(
        r#"
.Table with two columns, a header row, and two rows of content
[#ex-cols]
----
include::tables:example$table.adoc[tag=b-col-h-co]
----
<.> The `+*+` in the `cols` attribute is the repeat operator.
It means repeat the column specification across the remaining columns.
In this case, we are repeating the default formatting across 2 columns.
When the cells in the header are not defined on a single line, you must use the `cols` attribute to set the number of columns in the table and the `%header` option (or `options=header` attribute) to promote the first row to the table header.

.View result of <<ex-cols>>
[%collapsible.result]
====
include::tables:example$table.adoc[tag=b-col-h]
====

"#
    );

    let output = convert(
        "[%header,cols=2*]\n\
         |===\n\
         |Name of Column 1\n\
         |Name of Column 2\n\n\
         |Cell in column 1, row 1\n\
         |Cell in column 2, row 1\n\n\
         |Cell in column 1, row 2\n\
         |Cell in column 2, row 2\n\
         |===\n",
    );
    assert_css(&output, "thead th", 2);
    assert_css(&output, "tbody tr", 2);
}

// A `cols` value of relative widths sets both the column count and their
// widths.
#[test]
fn table_with_three_columns_and_widths() {
    verifies!(
        r#"
.Table with three columns, a header row, and two rows of content
[#ex-cols-widths]
----
include::tables:example$table.adoc[tag=b-col-indv-co]
----
<.> In this example, the `cols` attribute has two functions.
It specifies that this table has three columns, and it sets their relative widths.

.View result of <<ex-cols-widths>>
[%collapsible.result]
====
[caption="Table 1. "]
include::tables:example$table.adoc[tag=b-col-indv]
====

"#
    );

    let output = convert(
        ".Applications\n\
         [cols=\"1,1,2\"]\n\
         |===\n\
         |Name |Category |Description\n\n\
         |Firefox\n\
         |Browser\n\
         |Mozilla Firefox is an open source web browser.\n\
         It's designed for standards compliance,\n\
         performance, portability.\n\n\
         |Arquillian\n\
         |Testing\n\
         |An innovative and highly extensible testing platform.\n\
         Empowers developers to easily create real, automated tests.\n\
         |===\n",
    );
    assert_css(&output, "col", 3);
    assert_css(&output, "thead th", 3);
    assert_css(&output, "tbody tr", 2);
}

// An `a`-styled column parses its cells as AsciiDoc, so a cell may hold a list
// and a link.
#[test]
fn table_column_with_asciidoc_content() {
    verifies!(
        r#"
.Table with column containing AsciiDoc content
[#ex-table-adoc]
----
include::tables:example$table.adoc[tag=b-col-a]
----

.View result of <<ex-table-adoc>>
[%collapsible.result]
====
include::tables:example$table.adoc[tag=b-col-a]
====

"#
    );

    let output = convert(
        "[cols=\"2,2,5a\"]\n\
         |===\n\
         |Firefox\n\
         |Browser\n\
         |Mozilla Firefox is an open source web browser.\n\n\
         It's designed for:\n\n\
         * standards compliance\n\
         * performance\n\
         * portability\n\n\
         https://getfirefox.com[Get Firefox]!\n\
         |===\n",
    );
    assert_css(&output, "table", 1);
    assert_css(&output, "td div.ulist", 1);
    assert_css(&output, "td a", 1);
}

// The `,===` shorthand builds a table from CSV data.
#[test]
fn table_from_csv_shorthand() {
    verifies!(
        r#"
.Table from CSV data using shorthand
[#ex-csv]
----
include::tables:example$data.adoc[tag=s-csv]
----

.View result of <<ex-csv>>
[%collapsible.result]
====
include::tables:example$data.adoc[tag=s-csv]
====

"#
    );

    let output = convert(
        ",===\n\
         Artist,Track,Genre\n\n\
         Baauer,Harlem Shake,Hip Hop\n\
         ,===\n",
    );
    assert_css(&output, "thead th", 3);
    assert_css(&output, "tbody tr", 1);
}

// The `format=csv` attribute builds a table from CSV rows in a normal table
// block.
#[test]
fn table_from_csv_data() {
    verifies!(
        r#"
.Table from CSV data
[#ex-csv-formal]
----
include::tables:example$data.adoc[tag=csv]
----

.View result of <<ex-csv-formal>>
[%collapsible.result]
====
include::tables:example$data.adoc[tag=csv]
====

"#
    );

    let output = convert(
        "[%header,format=csv]\n\
         |===\n\
         Artist,Track,Genre\n\
         Baauer,Harlem Shake,Hip Hop\n\
         The Lumineers,Ho Hey,Folk Rock\n\
         |===\n",
    );
    assert_css(&output, "thead th", 3);
    assert_css(&output, "tbody tr", 2);
}

// A CSV table can include its data from a file; the escaped directive displays
// literally here.
#[test]
fn table_from_csv_included_from_file() {
    verifies!(
        r#"
.Table from CSV data included from file
[#ex-csv-include]
----
include::tables:example$data.adoc[tag=i-csv]
----

"#
    );

    let output = convert(",===\n\\include::customers.csv[]\n,===\n");
    assert_css(&output, "table", 1);
    assert!(output.contains("include::customers.csv[]"));
}

// The `:===` shorthand builds a table from DSV (colon-delimited) data.
#[test]
fn table_from_dsv_shorthand() {
    verifies!(
        r#"
.Table from DSV data using shorthand
[#ex-dsv]
----
include::tables:example$data.adoc[tag=s-dsv]
----

.View result of <<ex-dsv>>
[%collapsible.result]
====
include::tables:example$data.adoc[tag=s-dsv]
====

"#
    );

    let output = convert(
        ":===\n\
         Artist:Track:Genre\n\n\
         Robyn:Indestructible:Dance\n\
         :===\n",
    );
    assert_css(&output, "thead th", 3);
    assert_css(&output, "tbody tr", 1);
}

// Column and cell specifiers format (`e`/`m`/`s`), align (`^`/`>`), and merge
// (`2+`, `.3+`) cells.
#[test]
fn table_with_formatted_aligned_merged_cells() {
    verifies!(
        r#"
.Table with formatted, aligned and merged cells
[#ex-cell-format]
----
include::tables:example$cell.adoc[tag=b-spec]
----

.View result of <<ex-cell-format>>
[%collapsible.result]
====
include::tables:example$cell.adoc[tag=b-spec]
====

"#
    );

    let output = convert(
        "[cols=\"e,m,^,>s\",width=\"25%\"]\n\
         |===\n\
         |1 >s|2 |3 |4\n\
         ^|5 2.2+^.^|6 .3+<.>m|7\n\
         ^|8\n\
         |9 2+>|10\n\
         |===\n",
    );
    assert_css(&output, "table", 1);
    assert_css(&output, "td[colspan]", 2);
    assert_css(&output, "td[rowspan]", 2);
    assert_css(&output, "td.valign-middle", 1);
}

non_normative!(
    r#"
== IDs, roles, and options

"#
);

// The shorthand `[#id.role]` assigns a block id (anchor) and role.
#[test]
fn shorthand_id_and_role() {
    verifies!(
        r#"
.Shorthand method for assigning block ID (anchor) and role
----
[#goals.incremental]
* Goal 1
* Goal 2
----

"#
    );

    let output = convert("[#goals.incremental]\n* Goal 1\n* Goal 2\n");
    assert_css(&output, "div.ulist#goals.incremental", 1);
}

non_normative!(
    r#"
[TIP]
====
* To specify multiple roles using the shorthand syntax, delimit them by dots.
* The order of `id` and `role` values in the shorthand syntax does not matter.
====

"#
);

// The formal `[id="...",role="..."]` form assigns the same block id and role.
#[test]
fn formal_id_and_role() {
    verifies!(
        r#"
.Formal method for assigning block ID (anchor) and role
----
[id="goals",role="incremental"]
* Goal 1
* Goal 2
----

"#
    );

    let output = convert("[id=\"goals\",role=\"incremental\"]\n* Goal 1\n* Goal 2\n");
    assert_css(&output, "div.ulist#goals.incremental", 1);
}

// The shorthand `[#id]` above a section title sets the section's id (anchor).
#[test]
fn explicit_section_id() {
    verifies!(
        r#"
.Explicit section ID (anchor)
----
[#null-values]
== Primitive types and null values
----

"#
    );

    let output = convert("[#null-values]\n== Primitive types and null values\n");
    assert_css(&output, "h2#null-values", 1);
}

// The shorthand `[#id.role]` before inline formatted text assigns its id and
// role.
#[test]
fn id_and_role_on_inline_formatted_text() {
    verifies!(
        r#"
.Assign ID (anchor) and role to inline formatted text
----
[#id-name.role-name]`monospace text`

[#free-world.goals]*free the world*
----

"#
    );

    let output = convert(
        "[#id-name.role-name]`monospace text`\n\n\
         [#free-world.goals]*free the world*\n",
    );
    assert_css(&output, "code#id-name.role-name", 1);
    assert_css(&output, "strong#free-world.goals", 1);
}

// The shorthand `[%opt%opt]` assigns block options such as `header`, `footer`,
// and `autowidth`.
#[test]
fn shorthand_block_options() {
    verifies!(
        r#"
.Shorthand method for assigning block options
----
[%header%footer%autowidth]
|===
|Header A |Header B
|Footer A |Footer B
|===
----

"#
    );

    let output = convert(
        "[%header%footer%autowidth]\n\
         |===\n\
         |Header A |Header B\n\
         |Footer A |Footer B\n\
         |===\n",
    );
    assert_css(&output, "table.fit-content", 1);
    assert_css(&output, "thead", 1);
    assert_css(&output, "tfoot", 1);
}

// The formal `[options="..."]` (or `[opts="..."]`) form assigns the same block
// options.
#[test]
fn formal_block_options() {
    verifies!(
        r#"
.Formal method for assigning block options
----
[options="header,footer,autowidth"]
|===
|Header A |Header B
|Footer A |Footer B
|===

// options can be shorted to opts
[opts="header,footer,autowidth"]
|===
|Header A |Header B
|Footer A |Footer B
|===
----

"#
    );

    let output = convert(
        "[options=\"header,footer,autowidth\"]\n\
         |===\n\
         |Header A |Header B\n\
         |Footer A |Footer B\n\
         |===\n\n\
         [opts=\"header,footer,autowidth\"]\n\
         |===\n\
         |Header A |Header B\n\
         |Footer A |Footer B\n\
         |===\n",
    );
    assert_css(&output, "table", 2);
    assert_css(&output, "thead", 2);
    assert_css(&output, "tfoot", 2);
}

non_normative!(
    r#"
== Comments

"#
);

// A line comment (`//`) and a `////` block comment are dropped from the output.
#[test]
fn line_and_block_comments() {
    verifies!(
        r#"
.Line and block comments
----
// A single-line comment

////
A multi-line comment.

Notice it's a delimited block.
////
----

"#
    );

    let output = convert(
        "// A single-line comment\n\n\
         ////\n\
         A multi-line comment.\n\n\
         Notice it's a delimited block.\n\
         ////\n",
    );
    assert!(output.trim().is_empty());
}

non_normative!(
    r#"
== Breaks

"#
);

// A `'''` line renders a thematic break (horizontal rule) between paragraphs.
#[test]
fn thematic_break() {
    verifies!(
        r#"
.Thematic break (aka horizontal rule)
[#ex-thematic]
----
before

'''

after
----

.View result of <<ex-thematic>>
[%collapsible.result]
====
before

'''

after
====

"#
    );

    let output = convert("before\n\n'''\n\nafter\n");
    assert_css(&output, "hr", 1);
    assert_css(&output, "div.paragraph", 2);
}

// A `<<<` line renders a page break.
#[test]
fn page_break() {
    verifies!(
        r#"
.Page break
----
<<<
----

"#
    );

    let output = convert("<<<\n");
    assert!(output.contains("style=\"page-break-after: always;\""));
}

non_normative!(
    r#"
== Attributes and substitutions

"#
);

// Attribute entries declare reusable values (including multi-line values joined
// with a trailing backslash and a passthrough) that resolve where referenced.
#[test]
fn attribute_declaration_and_usage() {
    verifies!(
        r#"
.Attribute declaration and usage
[#ex-attributes]
----
:url-home: https://asciidoctor.org
:link-docs: https://asciidoctor.org/docs[documentation]
:summary: AsciiDoc is a mature, plain-text document format for \
       writing notes, articles, documentation, books, and more. \
       It's also a text processor & toolchain for translating \
       documents into various output formats (i.e., backends), \
       including HTML, DocBook, PDF and ePub.
:checkedbox: pass:normal[{startsb}&#10004;{endsb}]

Check out {url-home}[Asciidoctor]!

{summary}

Be sure to read the {link-docs} too!

{checkedbox} That's done!
----

"#
    );

    let output = convert(
        ":url-home: https://asciidoctor.org\n\
         :link-docs: https://asciidoctor.org/docs[documentation]\n\
         :summary: AsciiDoc is a mature, plain-text document format for \\\n\
         \x20      writing notes, articles, documentation, books, and more. \\\n\
         \x20      It's also a text processor & toolchain for translating \\\n\
         \x20      documents into various output formats (i.e., backends), \\\n\
         \x20      including HTML, DocBook, PDF and ePub.\n\
         :checkedbox: pass:normal[{startsb}&#10004;{endsb}]\n\n\
         Check out {url-home}[Asciidoctor]!\n\n\
         {summary}\n\n\
         Be sure to read the {link-docs} too!\n\n\
         {checkedbox} That's done!\n",
    );
    assert_xpath(
        &output,
        r#"//a[@href="https://asciidoctor.org"][text()="Asciidoctor"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//a[@href="https://asciidoctor.org/docs"][text()="documentation"]"#,
        1,
    );
    assert!(output.contains("mature, plain-text document format"));
    assert!(output.contains("&#10004;"));
}

// The page shows the result of the example above via a table (a rendering
// workaround so the nested attributes resolve); the same behavior is verified
// above.
non_normative!(
    r#"
.View result of <<ex-attributes>>
[%collapsible.result]
====
// I have to use a nested doc hack here, otherwise the attributes won't resolve
[.unstyled]
|===
a|
:url-home: https://asciidoctor.org
:link-docs: https://asciidoctor.org/docs[documentation]
:summary: AsciiDoc is a mature, plain-text document format for \
       writing notes, articles, documentation, books, and more. \
       It's also a text processor & toolchain for translating \
       documents into various output formats (i.e., backends), \
       including HTML, DocBook, PDF and ePub.
:checkedbox: pass:normal[{startsb}&#10004;{endsb}]

Check out {url-home}[Asciidoctor]!

{summary}

Be sure to read the {link-docs} too!

{checkedbox} That's done!
|===
====

To learn more about the available attributes and substitution groups see:

* xref:attributes:document-attributes-ref.adoc[]
* xref:attributes:character-replacement-ref.adoc[]
* xref:subs:apply-subs-to-blocks.adoc#subs-groups[Substitution Groups]

"#
);

// The `counter`/`counter2` attributes emit an auto-incrementing value each time
// they are referenced.
#[test]
fn counter_attributes() {
    verifies!(
        r#"
.Counter attributes
[#ex-counter]
----
include::attributes:example$counter.adoc[tag=base]
----

.View result of <<ex-counter>>
[%collapsible.result]
====
[caption="Table 1. "]
include::attributes:example$counter.adoc[tag=base]
====

"#
    );

    let output = convert(
        ".Parts{counter2:index:0}\n\
         |===\n\
         |Part Id |Description\n\n\
         |PX-{counter:index}\n\
         |Description of PX-{index}\n\n\
         |PX-{counter:index}\n\
         |Description of PX-{index}\n\
         |===\n",
    );
    assert!(output.contains("PX-1"));
    assert!(output.contains("Description of PX-1"));
    assert!(output.contains("PX-2"));
    assert!(output.contains("Description of PX-2"));
}

non_normative!(
    r#"
== Text replacements

"#
);

// Textual shorthands are replaced with their symbols: (C), (R), (TM), em dash,
// ellipsis, arrows, and the typographic apostrophe.
#[test]
fn textual_symbol_replacements() {
    verifies!(
        r#"
[frame=none,grid=rows]
include::subs:partial$subs-symbol-repl.adoc[]

"#
    );

    let output = convert(
        "(C) (R) (TM)\n\n\
         a -- b\n\n\
         and so on...\n\n\
         -> => <- <=\n\n\
         Sam's\n",
    );
    assert!(output.contains("&#169;"));
    assert!(output.contains("&#174;"));
    assert!(output.contains("&#8482;"));
    assert!(output.contains("&#8212;"));
    assert!(output.contains("&#8230;"));
    assert!(output.contains("&#8594;"));
    assert!(output.contains("&#8658;"));
    assert!(output.contains("&#8592;"));
    assert!(output.contains("&#8656;"));
    assert!(output.contains("Sam&#8217;s"));
}

non_normative!(
    r#"
Any named, numeric or hexadecimal {url-char-xml}[XML character reference^] is supported.

"#
);

non_normative!(
    r#"
== Escaping substitutions

"#
);

// A leading backslash escapes a substitution so the literal characters are
// preserved (attribute reference, bold, entity, arrow, anchors, index term,
// URL).
#[test]
fn escape_with_backslash() {
    verifies!(
        r#"
.Backslash
[#ex-slash]
----
include::subs:example$subs.adoc[tag=backslash]
----

.View result of <<ex-slash>>
[%collapsible.result]
====
include::subs:example$subs.adoc[tag=backslash]
====

"#
    );

    let output = convert(
        "In /items/\\{id}, the id attribute isn't replaced.\n\
         The curly braces around it are preserved.\n\n\
         \\*Stars* isn't displayed as bold text.\n\
         The asterisks around it are preserved.\n\n\
         \\&sect; appears as an entity reference.\n\
         It's not converted into the section symbol (&#167;).\n\n\
         \\=> The backslash prevents the equals sign followed by a greater\n\
         than sign from combining to form a double arrow character (=>).\n\n\
         The URL \\https://example.org isn't converted into an active link.\n",
    );
    assert!(output.contains("/items/{id}"));
    assert!(output.contains("*Stars*"));
    assert!(output.contains("&amp;sect;"));

    // The escaped URL is not turned into a link.
    assert_css(&output, "a", 0);
}

// Single and double pluses form an inline passthrough that suppresses
// substitutions while still escaping special characters.
#[test]
fn single_and_double_plus_passthroughs() {
    verifies!(
        r#"
.Single and double plus inline passthroughs
[#ex-single-plus]
----
include::pass:example$pass.adoc[tag=plus]
----

.View result of <<ex-single-plus>>
[%collapsible.result]
====
include::pass:example$pass.adoc[tag=plus]
====

"#
    );

    let output = convert(
        "A word or phrase between single pluses, such as +/user/{id}+,\n\
         is not substituted.\n\
         However, the special characters like +<+ and +>+ are still\n\
         escaped in the output.\n\n\
         An attribute reference within a word, such as dev++{conf}++,\n\
         is not replaced.\n\n\
         A plus passthrough will escape standalone formatting marks,\n\
         like +``+, or formatting marks within a word, like all-natural++*++.\n",
    );
    assert!(output.contains("/user/{id}"));
    assert!(output.contains("&lt;"));
    assert!(output.contains("&gt;"));
    assert!(output.contains("dev{conf}"));
}

// The triple-plus passthrough and the `pass:[]` macro emit their content
// unprocessed, so raw HTML passes through.
#[test]
fn triple_plus_and_inline_pass_macro() {
    verifies!(
        r#"
.Triple plus inline passthrough and inline pass macro
[#ex-inline-pass]
----
include::pass:example$pass.adoc[tag=b-3p-macro]
----

.View result of <<ex-inline-pass>>
[%collapsible.result]
====
include::pass:example$pass.adoc[tag=b-3p-macro]
====

"#
    );

    let output = convert(
        "+++<del>strike this</del>+++ is marked as deleted.\n\n\
         pass:[<del>strike this</del>] is also marked as deleted.\n",
    );
    assert_css(&output, "del", 2);
}

non_normative!(
    r#"
== Bibliography

"#
);

// A bibliography section holds bibliography anchors (`[[[id]]]`); inbound
// references (`<<id>>`) link to them.
#[test]
fn bibliography_with_inbound_references() {
    verifies!(
        r#"
.Bibliography with inbound references
[#ex-biblio]
----
include::sections:example$bibliography.adoc[tag=base]
----

.View result of <<ex-biblio>>
[%collapsible.result]
====
|===
a|
include::sections:example$bibliography.adoc[tag=base]
|===
====

"#
    );

    let output = convert(
        "_The Pragmatic Programmer_ <<pp>> should be required reading for all developers.\n\
         To learn all about design patterns, refer to the book by the \"`Gang of Four`\" <<gof>>.\n\n\
         [bibliography]\n\
         == References\n\n\
         * [[[pp]]] Andy Hunt & Dave Thomas. The Pragmatic Programmer:\n\
         From Journeyman to Master. Addison-Wesley. 1999.\n\
         * [[[gof,gang]]] Erich Gamma, Richard Helm, Ralph Johnson & John Vlissides.\n\
         Design Patterns: Elements of Reusable Object-Oriented Software. Addison-Wesley. 1994.\n",
    );
    assert_css(&output, "div.ulist.bibliography", 1);
    assert_css(&output, "a#pp", 1);
    assert_css(&output, "a#gof", 1);
    assert_xpath(&output, r##"//a[@href="#pp"]"##, 1);
}

non_normative!(
    r#"
[#section-footnotes]
== Footnotes

"#
);

// A normal footnote and a reusable footnote (referenced again by id) both add a
// numbered reference and a definition in the footnotes block.
#[test]
fn normal_and_reusable_footnotes() {
    verifies!(
        r#"
.Normal and reusable footnotes
[#ex-footnotes]
----
include::macros:example$footnote.adoc[tag=base]
----

.View result of <<ex-footnotes>>
[%collapsible.result]
====
[.unstyled]
|===
a|
include::macros:example$footnote.adoc[tag=base]
|===
====

"#
    );

    let output = convert(
        "A statement.footnote:[Clarification about this statement.]\n\n\
         A bold statement!footnote:disclaimer[Opinions are my own.]\n\n\
         Another bold statement.footnote:disclaimer[]\n",
    );
    assert_css(&output, "div#footnotes", 1);
    assert_css(&output, "div.footnote", 2);
    assert_css(&output, "sup.footnote", 2);
}

non_normative!(
    r#"
[#markdown-compatibility]
== Markdown compatibility

Markdown compatible syntax is an optional feature of the AsciiDoc language and is currently only available when using Asciidoctor.

"#
);

// Markdown-style `#` headings map to AsciiDoc section levels.
#[test]
fn markdown_style_headings() {
    verifies!(
        r#"
.Markdown-style headings
[#ex-md-headings]
----
include::sections:example$section.adoc[tag=md]
----

.View result of <<ex-md-headings>>
[%collapsible.result]
====
include::sections:example$section.adoc[tag=b-md]
====

"#
    );

    let output = convert(
        "# Document Title (Level 0)\n\n\
         ## Section Level 1\n\n\
         ### Section Level 2\n\n\
         #### Section Level 3\n\n\
         ##### Section Level 4\n\n\
         ###### Section Level 5\n",
    );
    assert_css(&output, "div.sect1", 1);
    assert_css(&output, "h2#_section_level_1", 1);
}

// A Markdown-style ```` ``` ```` fenced block with a language becomes a source
// block.
#[test]
fn markdown_style_fenced_code_block() {
    verifies!(
        r#"
.Fenced code block with syntax highlighting
[#ex-fenced]
----
include::verbatim:example$source.adoc[tag=fence]
----

.View result of <<ex-fenced>>
[%collapsible.result]
====
include::verbatim:example$source.adoc[tag=fence]
====

"#
    );

    let output = convert(
        "```ruby\n\
         require 'sinatra'\n\n\
         get '/hi' do\n\
         \x20 \"Hello World!\"\n\
         end\n\
         ```\n",
    );
    assert_css(&output, "div.listingblock", 1);
    assert_css(&output, "code.language-ruby", 1);
}

// A Markdown-style `>` blockquote maps to a quote block, with `--` attribution.
#[test]
fn markdown_style_blockquote() {
    verifies!(
        r#"
.Markdown-style blockquote
[#ex-md-quote]
----
include::blocks:example$quote.adoc[tag=md]
----

.View result of <<ex-md-quote>>
[%collapsible.result]
====
include::blocks:example$quote.adoc[tag=md]
====

"#
    );

    let output = convert(
        "> I hold it that a little rebellion now and then is a good thing,\n\
         > and as necessary in the political world as storms in the physical.\n\
         > -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11\n",
    );
    assert_css(&output, "div.quoteblock", 1);
    assert_css(&output, "cite", 1);
}

// A nested Markdown-style blockquote with block content maps to nested quote
// blocks holding the content.
#[test]
fn markdown_style_blockquote_with_block_content() {
    verifies!(
        r#"
.Markdown-style blockquote with block content
[#ex-md-blockquote]
----
include::blocks:example$quote.adoc[tag=md-alt]
----

.View result of <<ex-md-blockquote>>
[%collapsible.result]
====
include::blocks:example$quote.adoc[tag=md-alt]
====

"#
    );

    let output = convert(
        "> > What's new?\n\
         >\n\
         > I've got Markdown in my AsciiDoc!\n\
         >\n\
         > > Like what?\n\
         >\n\
         > * Blockquotes\n\
         > * Headings\n\
         > * Fenced code blocks\n\
         >\n\
         > > Is there more?\n\
         >\n\
         > Yep. AsciiDoc and Markdown share a lot of common syntax already.\n",
    );

    // The outer blockquote plus its three nested `> >` blockquotes.
    assert_css(&output, "div.quoteblock", 4);
    assert_css(&output, "ul", 1);
}

// Markdown-style thematic breaks (`---`, `- - -`, `***`, `* * *`) each render a
// horizontal rule.
#[test]
fn markdown_style_thematic_breaks() {
    verifies!(
        r#"
.Markdown-style thematic breaks
[#ex-md-breaks]
----
---

- - -

***

* * *
----

.View result of <<ex-md-breaks>>
[%collapsible.result]
====
---

- - -

***

* * *
====

"#
    );

    let output = convert("---\n\n- - -\n\n***\n\n* * *\n");
    assert_css(&output, "hr", 4);
}

non_normative!(
    r#"

////
Possible change for future to `%collapsible` blocks

.Normal
----
Paragraphs don't require any special markup in AsciiDoc.
A paragraph is just one or more lines of consecutive text.

To begin a new paragraph, separate it by at least one empty line.
Line breaks within a paragraph are not displayed.
----

.View Result (Normal)
[%collapsible.result]
====
Paragraphs don't require any special markup in AsciiDoc.
A paragraph is just one or more lines of consecutive text.

To begin a new paragraph, separate it by at least one empty line.
Line breaks within a paragraph are not displayed.
====

'''

.Normal
[tabs]
====
Source::
+
----
Paragraphs don't require any special markup in AsciiDoc.
A paragraph is just one or more lines of consecutive text.

To begin a new paragraph, separate it by at least one empty line.
Line breaks within a paragraph are not displayed.
----

Output::
+
--
Paragraphs don't require any special markup in AsciiDoc.
A paragraph is just one or more lines of consecutive text.

To begin a new paragraph, separate it by at least one empty line.
Line breaks within a paragraph are not displayed.
--
====
////
"#
);
