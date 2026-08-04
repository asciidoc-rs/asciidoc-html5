//! Coverage of the AsciiDoc language description's *Specify Image Format* page.
//!
//! The `format` attribute tells a converter an image's format when the target's
//! file extension can't. Its one behavior observable in this crate's HTML
//! output is distinguishing an SVG target: `format=svg` makes an extensionless
//! target eligible for the SVG `interactive` (`<object>`) referencing, which is
//! verified through `convert` (below the `Secure` safe mode). The data-URI and
//! reencoding scenarios (`data-uri`, PDF) don't affect this crate's output and
//! are non-normative.

use crate::{convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-format.adoc");

// Renders below `Secure`, where an SVG target's `interactive` referencing is
// enabled — the path in which `format=svg` recognition is observable.
fn convert_unsafe(source: &str) -> String {
    convert_with(source, &Options::new().safe_mode(SafeMode::Unsafe))
}

non_normative!(
    r#"
= Specify Image Format

Although to a reader, an image is just an image, different image formats must be processed differently by the converter in order to be referenced by or embedded in the output format.
Some image formats, such as SVG, even activate additional behavior.
On this page, you learn how an AsciiDoc processor determines the format of an image and how it can be specified explicitly using the `format` attribute if the format cannot be determined.

== Automatic image format

Most of the time, the image format can be determined from the file extension (e.g., `.svg`) of the target.
Here's an example that shows an image target that has a file extension:

----
image::avatar.svg[]
----

In this case, the converter can determine that this is an SVG image based on the file extension `.svg`.
So the author does not need to specify the image format.
The author only needs to specify the format when the image format cannot be determined.

"#
);

// `format=svg` marks an extensionless target as an SVG, so it becomes eligible
// for the SVG `interactive` referencing (an `<object>`) it would otherwise be
// denied — the one effect of `format` observable in this crate's HTML output.
#[test]
fn format_attribute_recognizes_svg() {
    verifies!(
        r#"
== format attribute

When the file extension is not present, or--in the case of a URL--not in its usual place, the author must specify the image format explicitly.
The `format` attribute on a block or inline image macro can be used to specify the format of an image.

----
image::https://example.org/avatar[format=svg]
----

In this case, the converter would not be able to determine that this is an SVG image because the target has no file extension.
But since the author has specified `format=svg`, the converter can recognize this as an SVG image.

"#
    );

    // With `format=svg`, the extensionless URL is recognized as an SVG, so
    // `opts=interactive` yields an `<object>`.
    assert!(convert_unsafe(
        "image::https://example.org/avatar[Avatar,format=svg,opts=interactive]"
    )
    .contains(r#"<object type="image/svg+xml" data="https://example.org/avatar">"#));

    // Without `format=svg`, the same extensionless target is not recognized as an
    // SVG, so it renders as a plain `<img>` even with `opts=interactive`.
    assert!(
        convert_unsafe("image::https://example.org/avatar[Avatar,opts=interactive]")
            .contains(r#"<img src="https://example.org/avatar" alt="Avatar">"#)
    );
}

// The remaining scenarios in which a converter needs the format — data-URI
// embedding (`data-uri`) and reencoding (PDF) — don't affect this crate's HTML
// output, and the MIME-type guidance is descriptive.
non_normative!(
    r#"
Here are few cases when the image format cannot be determined automatically:

* the target does not have a file extension
* the target is using an unrecognized file extension
* the target is a URL that contains a query string (thus hiding the file extension)

The value of the format attribute is the sub-MIME type (the part after the forward slash) (e.g., `png` for a PNG image).
One exception to this rule is the format for SVG, which is specified as `svg` instead of `svg+xml`.
You can find a list of common image formats along with their MIME type values on the https://developer.mozilla.org/en-US/docs/Web/Media/Formats/Image_types[image file type and format guide^] page.

== When is the format used?

There are several scenarios when a converter almost always needs to know the image format.
One is to distinguish an SVG target.
Another is to convert the image to a data URI.
Yet another is when a converter must reencode the image.

AsciiDoc recognizes extra settings for xref:image-svg.adoc[SVG images], such as how to include it in the HTML output and whether to provide a fallback image.
If the converter needs to read and process the file, it often must use a different library to process an SVG since the contents of an SVG is an XML document.
So being able to recognize an SVG target is essential for almost any converter.

If the `data-uri` attribute is set on the document, an AsciiDoc processor must convert the image to a data URI.
To start, the data URI for an SVG differs from a data URI for any other format.
But even when creating a data URI for a non-SVG, the MIME type of that image must be included in the data URI, in which case the image format must be known.

Some converters, such as a PDF converter, have to reencode the image data.
This means the converter must parse the image data, which may require loading an additional library.
Doing that processing almost certainly requires knowing the format of the image.
It also provides an opportunity for the converter to inform the author whether the image uses a format that cannot be processed.

As you can see, while the image format not always needed, it's needed often enough that you should specify the format if it's not apparent from the file extension of the target.
"#
);
