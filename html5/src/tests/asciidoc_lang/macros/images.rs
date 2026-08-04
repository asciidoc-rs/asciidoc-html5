//! Coverage of the AsciiDoc language description's *Images* page.
//!
//! The block `image::` macro renders `<div class="imageblock"><img>` and the
//! inline `image:` macro renders `<span class="image"><img>`. Alt text (first
//! positional, double-quoted when it contains a comma), an id, a title (with an
//! automatic "Figure N." caption unless `figure-caption` is unset), dimensions,
//! and a `link` are all supported. All verified through `convert`, matching
//! Asciidoctor 2.0.26.
//!
//! The examples are pulled in with `include::example$image.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/images.adoc");

non_normative!(
    r#"
= Images

There are two AsciiDoc image macro types, block and inline.
As with all macros, the block and inline forms differ by the number of colons that follow the macro name.
The block form uses two colons (`::`), whereas the inline form only uses one (`:`).

== Block image macro

A [.term]*block image* is displayed as a discrete element, i.e., on its own line, in a document.
A block image is designated by `image` macro name and followed by two colons (`::`)
It's preceded by an empty line, entered on a line by itself, and then followed by an empty line.

"#
);

// A block image renders a `<div class="imageblock">` wrapping an `<img>` whose
// `alt` defaults to the target's base name.
#[test]
fn block_image() {
    verifies!(
        r#"
.Block image macro
[source#ex-block]
----
Content in document.

include::example$image.adoc[tag=base-co]

Content in document
----
<.> To insert a block image, type the `image` macro name directly followed by two colons (`::`).
<.> After the colons, enter the image file target.
Type a pair of square brackets (`+[]+`) directly after the target to complete the macro.

The result of <<ex-block>> is displayed below.

include::example$image.adoc[tag=base]

include::partial$image-target.adoc[]

"#
    );

    // The `tag=base` snippet.
    let output = convert("image::sunset.jpg[]");
    assert!(output.contains(r#"<div class="imageblock">"#));
    assert!(output.contains(r#"<img src="sunset.jpg" alt="sunset">"#));
}

// The first positional attribute is the alt text.
#[test]
fn alt_text() {
    verifies!(
        r#"
You can specify a comma-separated list of optional attributes inside the square brackets or leave them empty.
If you want to specify alt text, enter it inside the square brackets.

.Block image macro with alt text
[source#ex-alt]
----
include::example$image.adoc[tag=alt]
----
"#
    );

    // The `tag=alt` snippet.
    assert!(convert("image::sunset.jpg[Sunset]").contains(r#"<img src="sunset.jpg" alt="Sunset">"#));
}

// Alt text containing a comma must be double-quoted so the comma is not parsed
// as an attribute separator.
#[test]
fn alt_text_with_comma() {
    verifies!(
        r#"

If the alt text contains a comma or starts with a valid attribute name followed by an equals sign, you must enclose the alt text in double quotes.
The double quote enclosure effectively escapes the comma from being interpreted as an attribute separator.
See xref:attributes:positional-and-named-attributes.adoc#attribute-list-parsing[Attribute list parsing] to learn how the attribute list in a macro is parsed.

.Block image macro with alt text that contains a comma
[source#ex-alt-with-comma]
----
include::example$image.adoc[tag=alt-with-comma]
----
"#
    );

    // The `tag=alt-with-comma` snippet.
    assert!(
        convert(r#"image::sunset.jpg["Mesa Verde Sunset, by JAVH"]"#)
            .contains(r#"<img src="sunset.jpg" alt="Mesa Verde Sunset, by JAVH">"#)
    );
}

non_normative!(
    r#"

NOTE: Although you could enclose the alt text in single quotes to escape the comma, doing so implicitly enables substitutions.
Unless you need substitutions to be applied to the alt text, prefer using double quotes as the enclosure.

"#
);

// The attribute list gives the image an id, a title, dimensions, and a link.
#[test]
fn attribute_list() {
    verifies!(
        r#"
You can also give the image an ID, title, set its dimensions and make it a link.

.Block image macro with attribute list
[source#ex-attributes]
----
include::example$image.adoc[tag=attr-co]
----
<.> Defines the title of the block image, which gets displayed underneath the image when rendered.
<.> Assigns an ID to the block and makes the image a link.
The `link` attribute can also be defined inside the attribute list of the block macro.
<.> The first positional attribute, _Sunset_, is the image's alt text.
<.> The second and third positional attributes define the width and height, respectively.

The result of <<ex-attributes>> is displayed below.

include::example$image.adoc[tag=attr]

"#
    );

    // The `tag=attr` snippet: title, id, link, and dimensions.
    let output = convert(
        ".A mountain sunset\n\
         [#img-sunset,link=https://www.flickr.com/photos/javh/5448336655]\n\
         image::sunset.jpg[Sunset,200,100]",
    );
    assert!(output.contains(r#"id="img-sunset""#));
    assert!(output
        .contains(r#"<a class="image" href="https://www.flickr.com/photos/javh/5448336655">"#));
    assert!(output.contains(r#"<img src="sunset.jpg" alt="Sunset" width="200" height="100">"#));
    assert!(output.contains(r#"<div class="title">Figure 1. A mountain sunset</div>"#));
}

// A block image's title is prefixed with an automatic "Figure N." caption
// unless the `figure-caption` attribute is unset.
#[test]
fn figure_caption_label() {
    verifies!(
        r#"
=== Figure caption label

When a title is defined on a block image, the image title will be prefixed by a caption label (Figure) and numbered automatically.
To turn off figure caption labels and numbers, unset the `figure-caption` attribute in the document header.

[source]
----
= Document Title
:figure-caption!:
----
"#
    );

    // With `figure-caption` set (the default), the title is prefixed and numbered.
    assert!(convert(".A mountain sunset\nimage::sunset.jpg[Sunset]")
        .contains(r#"<div class="title">Figure 1. A mountain sunset</div>"#));

    // Unsetting `figure-caption` removes the label and number.
    assert!(
        convert(":figure-caption!:\n\n.A mountain sunset\nimage::sunset.jpg[Sunset]")
            .contains(r#"<div class="title">A mountain sunset</div>"#)
    );
}

non_normative!(
    r#"

== Inline image macro

An [.term]*inline image* is displayed in the flow of another element, such as a paragraph or sidebar block.
The inline image macro is almost identical to the block image macro, except its macro name is followed by a single colon (`:`).

"#
);

// An inline image renders a `<span class="image">` wrapping an `<img>` in the
// flow of text; a `title` becomes a tooltip.
#[test]
fn inline_image() {
    verifies!(
        r#"
.Inline image macro
[source#ex-inline]
----
Click image:play.png[] to get the party started. <.>

Click image:pause.png[title=Pause] when you need a break. <.>
----
<.> In the flow of an element, enter the macro name and a single colon (`+image:+`), followed by the image target.
Complete the macro with a pair of square brackets (`+[]+`).
<.> You can specify a comma-separated list of attributes inside the square brackets or leave them empty.

The result of <<ex-inline>> is displayed below.

include::example$image.adoc[tag=inline]

"#
    );

    // The `tag=inline` snippet: a plain inline image and one with a `title`.
    let output = convert(
        "Click image:play.png[] to get the party started.\n\n\
         Click image:pause.png[title=Pause] when you need a break.",
    );
    assert!(output.contains(r#"<span class="image"><img src="play.png" alt="play"></span>"#));
    assert!(output.contains(r#"<img src="pause.png" alt="pause" title="Pause">"#));
}

non_normative!(
    r#"
include::partial$image-target.adoc[]

The alt text for an inline image has the same requirements as for a block image, with the added restriction that a closing square bracket must be escaped.

For inline images, the optional title is displayed as a tooltip.
"#
);
