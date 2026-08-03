//! Coverage of the AsciiDoc language description's *Images Reference* page.
//!
//! A recap of the `imagesdir` document attribute and the block/inline image
//! attributes. The attributes this crate's HTML5 backend acts on — `imagesdir`,
//! `id`, `alt`, `title`, `width`/`height`, `link`/`window`, `align`/`float`/
//! `role`, and the link `opts` — are verified through `convert`. The
//! DocBook/PDF-only attributes (`scale`, `scaledwidth`, `pdfwidth`), the
//! per-image `imagesdir` override (unsupported before Asciidoctor 2.1), the
//! `format` attribute (a no-op for a plain `<img>`), and the SVG `fallback` /
//! inline options (not implemented here) are non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-ref.adoc");

non_normative!(
    r#"
= Images Reference

.Document attributes and values
[cols=2;2;3;3]
|===
|Attribute |Value(s) |Example Syntax |Comments

"#
);

// `imagesdir` is prepended to a relative image target.
#[test]
fn imagesdir_attribute() {
    verifies!(
        r#"
|`imagesdir`
|empty, filesystem path, or base URL
|`:imagesdir: images`
|Added in front of a relative image target, joined using a file separator if needed.
Not used if the image target is an absolute URL or path.
Default value is empty.
|===
"#
    );

    assert!(convert(":imagesdir: images\n\nimage::sunset.jpg[]")
        .contains(r#"<img src="images/sunset.jpg" alt="sunset">"#));
}

non_normative!(
    r#"

.Block and inline image attributes and values
[cols=2;2;3;3]
|===
|Attribute |Value(s) |Example Syntax |Comments

"#
);

// `id` sets the image block's id; `alt` (first positional or named) sets the
// alt text.
#[test]
fn id_and_alt() {
    verifies!(
        r#"
|`id`
|User defined text
|`id=sunset-img` +
(or `+[[sunset-img]]+` or `[#sunset-img]` above block macro)
|

|`alt`
|User defined text in first position of attribute list or named attribute
|`image::sunset.jpg[Brilliant sunset]` +
(or `alt=Sunset`)
|

"#
    );

    assert!(convert("image::sunset.jpg[Sunset,id=sunset-img]").contains(r#"id="sunset-img""#));
    assert!(convert("image::sunset.jpg[Brilliant sunset]")
        .contains(r#"<img src="sunset.jpg" alt="Brilliant sunset">"#));
}

// The `fallback` image applies only to an interactive SVG `<object>`, which
// this crate does not render (see the *SVG Images* page), so it is
// non-normative.
non_normative!(
    r#"
|`fallback`
|Image path relative to `imagesdir` or an absolute path or URL
|`image::tiger.svg[fallback=tiger.png]`
|Only applicable if target is an SVG and opts=interactive

"#
);

// `title` on an inline image becomes the `title` attribute (a tooltip).
#[test]
fn title_attribute() {
    verifies!(
        r#"
|`title`
|User defined text
|in attrlist: `title="A mountain sunset"` (enclosing quotes only required if value contains a comma) +
above block macro: `.A mountain sunset`
|Blocks: title displayed below image +
Inline: title displayed as tooltip

"#
    );

    // For an inline image the `title` attribute renders as the `<img>` tooltip.
    assert!(convert("Click image:pause.png[title=Pause] now.")
        .contains(r#"<img src="pause.png" alt="pause" title="Pause">"#));
}

// `format` (image format) does not change a plain `<img>`'s markup, and
// `caption` is a block-caption prefix supplied through the block title; neither
// is verifiable via the recap examples here.
non_normative!(
    r#"
|`format`
|The format of the image, specified as a sub-MIME type (except in the case of an SVG, which is specified as `svg`).
|`format=svg`
|Only necessary when the converter needs to know the format of the image and the target does not end in a file extension (or otherwise cannot be detected).

|`caption`
|User defined text
|`caption="Figure 8: "`
|Only applies to block images.

"#
);

// The second and third positional attributes (or `width=`/`height=`) set the
// `<img>` dimensions.
#[test]
fn width_and_height() {
    verifies!(
        r#"
|`width`
|User defined size in pixels
|`image::sunset.jpg[Sunset,300]` +
(or `width=300`)
|

|`height`
|User defined size in pixels
|`image::sunset.jpg[Sunset,300,200]` +
(or `height=200`)
|The height should only be set if the width attribute is set and must respect the aspect ratio of the image.

"#
    );

    assert!(convert("image::sunset.jpg[Sunset,300]")
        .contains(r#"<img src="sunset.jpg" alt="Sunset" width="300">"#));
    assert!(convert("image::sunset.jpg[Sunset,300,200]")
        .contains(r#"<img src="sunset.jpg" alt="Sunset" width="300" height="200">"#));
}

// `link` wraps the image in an `<a class="image">`; `window` sets the link's
// `target`.
#[test]
fn link_and_window() {
    verifies!(
        r#"
|`link`
|User defined location of external URI
|`link=https://www.flickr.com/photos/javh/5448336655`
|

|`window`
|User defined window target for the `link` attribute
|`window=_blank`
|

"#
    );

    let output = convert(
        "image::sunset.jpg[Sunset,link=https://www.flickr.com/photos/javh/5448336655,window=_blank]",
    );
    assert!(output.contains(
        r#"<a class="image" href="https://www.flickr.com/photos/javh/5448336655" target="_blank" rel="noopener">"#
    ));
}

// `scale`, `scaledwidth`, and `pdfwidth` size images for DocBook / Asciidoctor
// PDF only; they have no effect on this crate's HTML output.
non_normative!(
    r#"
|`scale`
|A scaling factor to apply to the intrinsic image dimensions
|`scale=80`
|DocBook only

|`scaledwidth`
|User defined width for block images
|`scaledwidth=25%`
|DocBook and Asciidoctor PDF only

|`pdfwidth`
|User defined width for images in a PDF
|`pdfwidth=80vw`
|Asciidoctor PDF only

"#
);

// `align`, `float`, and `role` add positioning classes to a block image's
// wrapper.
#[test]
fn align_float_and_role() {
    verifies!(
        r#"
|`align`
|`left`, `center`, `right`
|`align=left`
|Block images only.
`align` and `float` attributes are mutually exclusive.

|`float`
|`left`, `right`
|`float=right`
|Block images only.
`float` and `align` attributes are mutually exclusive.
To scope the float, use a xref:image-position.adoc#control-float[float group].

|`role`
|user-defined, `left`, `right`, `th`, `thumb`, `related`, `rel`
|`role="thumb right"` +
(or `[.thumb.right]` above block macro)
|The role is preferred to specify the float position for an image.
Role shorthand (`.`) can only be used in block attribute list above a block image.

"#
    );

    // `align=left` maps to the `text-left` class.
    assert!(convert("image::sunset.jpg[Sunset,align=left]")
        .contains(r#"<div class="imageblock text-left">"#));

    // `float=right` maps to the `right` class.
    assert!(convert("image::tiger.png[Tiger,float=right]")
        .contains(r#"<div class="imageblock right">"#));

    // A multi-token `role` adds each token as a class.
    assert!(convert(r#"image::tiger.png[Tiger,role="thumb right"]"#)
        .contains(r#"<div class="imageblock thumb right">"#));
}

// The per-image `imagesdir` override was introduced in Asciidoctor 2.1 and is
// not applied here.
non_normative!(
    r#"
|`imagesdir`
|empty, filesystem path, or base URL
|`imagesdir=ch1/images`
|Overrides the `imagesdir` set on the document.
If not specified, the `imagesdir` from the document is used.
(Not supported until Asciidoctor 2.1).

"#
);

// The link `opts` (`nofollow`, `noopener`) apply to an image link; the SVG-only
// `inline`/`interactive` options are covered on the *SVG Images* page and are
// not implemented here.
#[test]
fn opts_attribute() {
    verifies!(
        r#"
|`opts`
|Additional options for link creation and SVG targets.
|`image::sunset.jpg[Sunset, link=https://example.org, opts=nofollow]`

`image::chart.svg[opts=inline]`
|Option names include: `nofollow`, `noopener`, `inline` (SVG only), `interactive` (SVG only)
|===
"#
    );

    // The `nofollow` link option is emitted in the link's `rel`.
    assert!(
        convert("image::sunset.jpg[Sunset, link=https://example.org, opts=nofollow]")
            .contains(r#"<a class="image" href="https://example.org" rel="nofollow">"#)
    );
}
