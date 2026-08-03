//! Coverage of the AsciiDoc language description's *Document Title* page.
//!
//! The document title is a level-0 heading in the header. Matching Asciidoctor
//! 2.0.26, this crate renders it as the header `<h1>` and the `<title>` of a
//! standalone document, shows or hides it with `showtitle`/`showtitle!` (on by
//! default standalone, off by default embedded, and with no byline in the
//! embedded case), assigns it to the `doctitle` attribute, and lets a `title`
//! attribute override the HTML `<title>` (or supply a fallback). Those
//! behaviors are verified through `convert`; the doctype-structure prose (whose
//! book-part behavior is a known limitation) and the screenshots are
//! non-normative.

use crate::{
    convert, convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/title.adoc");

// Renders a standalone document so the header `<h1>` and the `<title>` element
// are present in the output.
fn convert_standalone(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the introduction. Descriptive prose.
non_normative!(
    r#"
= Document Title

A document title (aka doctitle) is defined in the document header, typically on the first line of the document.
Like all elements of the document header, the document title is optional.

== Title syntax

"#
);

// A `= ` line in the header becomes the document title: the header `<h1>` of a
// standalone document.
#[test]
fn document_title_renders_as_the_header_heading() {
    verifies!(
        r#"
A document title is specified using a single equals sign (`=`), followed by a space, then the title text.

.Document with a title
[source#ex-title]
----
include::example$document-title.adoc[]
----

In <<ex-title>>, notice the empty line between the document title and the first line of prose.
That empty line is what separates the document header from the document body.

"#
    );

    // The full contents of `example$document-title.adoc`.
    let source = "\
= The Intrepid Chronicles

This adventure begins on a frigid morning.
";

    let output = convert_standalone(source);
    assert_xpath(&output, r#"//h1[text()="The Intrepid Chronicles"]"#, 1);
    assert!(output.contains("<title>The Intrepid Chronicles</title>"));
}

// The screenshot and the doctype/title structure prose (the book multi-part
// behavior is a known limitation of this crate). Descriptive prose.
non_normative!(
    r#"
image::document-title.png[Title of document]

=== Doctypes and titles

Technically, a document title is a level 0 section title (`=`).
The `article` and `manpage` document types (`doctype`) can only have one level 0 section.

The `book` document type permits multiple level 0 section titles.
When the `doctype` is `book`, the title of the level 0 section in the header is used as the document's title.
Subsequent level 0 section titles in the document body are interpreted as xref:sections:parts.adoc[part titles], unless labeled with a xref:sections:styles.adoc[style].

[#hide-or-show]
== Hide or show the document title

"#
);

// `showtitle` governs the title's visibility: on by default for a standalone
// document (and removable with `showtitle!`), off by default for an embeddable
// one (and addable with `showtitle`), and the embeddable title is never
// accompanied by the author/revision byline.
#[test]
fn showtitle_controls_the_title_visibility() {
    verifies!(
        r#"
When converting a standalone document, the document title is shown by default.
You can control whether the document title appears with the `showtitle` attribute.
If you don't want the title to be shown, unset the `showtitle` attribute using `showtitle!` in the document header or via the CLI or API.

//Need to link to a definition of embeddable doc
When converted to an embeddable document, the document title isn't shown by default.
To show the title in the embeddable document, set `showtitle` in the document header or via the CLI or API.
The author and revision information isn't shown below the document title in the embeddable version of the document like it is in the standalone document, even when `showtitle` is set.

"#
    );

    // Standalone: shown by default, removed by `showtitle!`.
    let shown = convert_standalone("= The Intrepid Chronicles\nKismet Lee\n\nBody.\n");
    assert_css(&shown, "#header h1", 1);
    let hidden =
        convert_standalone("= The Intrepid Chronicles\nKismet Lee\n:showtitle!:\n\nBody.\n");
    assert_css(&hidden, "h1", 0);

    // Embeddable: hidden by default, added by `showtitle` — but with no byline.
    let embedded = convert("= The Intrepid Chronicles\nKismet Lee\n\nBody.\n");
    assert_css(&embedded, "h1", 0);
    let embedded_shown = convert("= The Intrepid Chronicles\nKismet Lee\n:showtitle:\n\nBody.\n");
    assert_css(&embedded_shown, "h1", 1);
    assert_css(&embedded_shown, "span.author", 0);
}

// The section heading. Descriptive prose.
non_normative!(
    r#"
[#reference-doctitle]
== Reference the document title

"#
);

// The title is assigned to the `doctitle` attribute, so `{doctitle}` resolves
// to the title wherever it is referenced.
#[test]
fn doctitle_attribute_holds_the_title() {
    verifies!(
        r#"
The level 0 section title in a document's header, that is, its title, is automatically assigned to the document attribute `doctitle`.
You can reference the `doctitle` attribute anywhere in your document and the document's title will be displayed.

.Reference the doctitle attribute
[source#ex-doctitle]
----
include::example$doctitle.adoc[]
----

"#
    );

    // The full contents of `example$doctitle.adoc`.
    let source = "\
= The Intrepid Chronicles

{doctitle} begin on a frigid morning.
";

    let output = convert_standalone(source);
    assert!(output.contains("The Intrepid Chronicles begin on a frigid morning."));
}

// The screenshot, the note about explicitly setting `doctitle`, and the section
// heading. Descriptive prose.
non_normative!(
    r#"
image::doctitle.png[The document title is displayed wherever the doctitle attribute is referenced]

The `doctitle` attribute can also be explicitly set and assigned a value using an attribute entry in the header.
//Its value is identical to the value returned by `Document#doctitle`.

[#title-attr]
== title attribute

"#
);

// The `title` attribute overrides the HTML `<title>` value (while the on-page
// heading keeps the document title), and serves as a fallback document title
// when no level-0 title or `doctitle` is present.
#[test]
fn title_attribute_sets_the_html_title() {
    verifies!(
        r#"
By default, the text of the document title is used as the value of the HTML `<title>` element and main DocBook `<info>` element.
You can override this behavior by setting the `title` attribute in the header with an attribute entry.
If neither a level 0 section title or `doctitle` is specified in the header, but `title` is, its value is used as a fallback document title.
"#
    );

    // `title` overrides the `<title>` element; the header `<h1>` still shows the
    // document title.
    let overridden =
        convert_standalone("= The Intrepid Chronicles\n:title: Custom Tab Title\n\nBody.\n");
    assert!(overridden.contains("<title>Custom Tab Title</title>"));
    assert_xpath(&overridden, r#"//h1[text()="The Intrepid Chronicles"]"#, 1);

    // With no document title, `title` supplies the fallback `<title>`.
    let fallback = convert_standalone(":title: Only Title\n\nBody.\n");
    assert!(fallback.contains("<title>Only Title</title>"));
}
