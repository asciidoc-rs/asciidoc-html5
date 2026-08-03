//! Coverage of the AsciiDoc language description's *SVG Images* page.
//!
//! An SVG image target accepts an `opts` value controlling how the SVG is
//! referenced: `none` (the default, an `<img>`), `interactive` (an `<object>`),
//! or `inline` (an embedded `<svg>`). This crate's HTML5 backend implements
//! only the default `none` behavior — it renders every SVG as an `<img>`
//! regardless of `opts` — which matches Asciidoctor's own output under the
//! default `Secure` safe mode (interactive/inline are disabled there). The
//! interactive and inline behaviors (available only below `Secure`, and needing
//! filesystem access) are not implemented here, so that material is
//! non-normative; the `viewBox`/dimensions authoring guidance is likewise
//! descriptive.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-svg.adoc");

non_normative!(
    r#"
= SVG Images
:url-svg-editor: https://petercollingridge.appspot.com/svg-editor
:url-svgo: https://github.com/svg/svgo

Both block and inline image macros have built-in support for scalable vector graphics (SVGs).
But there's more than one way to include an SVG into a web page, and the strategy used can affect how the SVG behaves (or misbehaves).
Therefore, these macros provide additional options to control how the SVG is included (i.e., referenced).

== SVG dimensions

The viewBox attribute on the root `<svg>` element is required.
The viewBox establishes the coordinate space on to which x and y values inside the SVG data are placed.
Without this information, converters cannot properly interpret the SVG data and translate it to the canvas.

We strongly recommend not using the width and height attributes on the root `<svg>` element.
You will find that SVGs that only specify a viewBox work best in a document.
That's because the SVG data itself is infinitely scalable.
By assigning an explicit width and height, you may end up limiting how the SVG can be sized or positioned in the document.
It's better to specify the width (or similar, such as pdfwidth) on the image macro instead, or using CSS.

You particularly want to avoid using a percentage width, such as `width="100%"`.
According to the SVG spec, this means using all available space, but without altering the aspect ratio.
As a result, you can get a large gap above and below the SVG in page-oriented media, such as PDF.
If you do specify a width and height, at least make sure the values are fixed and that they respect the aspect ratio of the data.

"#
);

// The default (`none`) SVG referencing renders an ordinary `<img>` — the only
// one of the three options this crate's HTML5 backend implements (and what
// Asciidoctor also emits under the default `Secure` safe mode).
#[test]
fn default_none_renders_img() {
    verifies!(
        r#"
== Options for SVG images

When the image target is an SVG, the `options` attribute (often abbreviated as `opts`) on the macro accepts one of the following values to control how the SVG is referenced:

* _none_ (default)
* `interactive`
* `inline`

The following table demonstrates the impact these options have.
"#
    );

    // A default SVG image (no `opts`) renders as an `<img>`.
    assert!(convert("image::sample.svg[Static,300]")
        .contains(r#"<img src="sample.svg" alt="Static" width="300">"#));
}

// The interactive (`<object>`) and inline (`<svg>`) referencing options — and
// the `fallback` attribute and viewBox/namespace requirements that go with them
// — are not implemented by this crate's HTML5 backend (it renders every SVG as
// an `<img>`), so the demonstration table, the option-values table, and the
// accompanying notes are non-normative.
non_normative!(
    r#"

.Demonstration of option values for SVG images
[cols=2*,frame=ends,grid=none]
|===
2+l|image::sample.svg[Static,300]
a|image::sample.svg[Static,300]
|Observe that the SVG does not respond to the hover event.

2+l|image::sample.svg[Interactive,300,opts=interactive]
a|image::sample.svg[Interactive,300,opts=interactive]
|Observe that the color changes when hovering over the SVG.

2+l|image::sample.svg[Embedded,300,opts=inline]
a|image::sample.svg[Embedded,300,opts=interactive]
// the output uses the interactive version as the documentation doesn't currently support the `inline` option.
|Observe that the color changes when hovering over the SVG.
The SVG also inherits CSS from the document stylesheets.
|===

How these options values work and when each should be used is described below:

.Option values that control how an SVG image is referenced in the HTML output
[%autowidth]
|===
|Option |HTML Element Used |Effect |When To Use

|_none_ (default)
|`<img>`
|Image is rasterized
|Static image, no interactivity, no custom fonts

|`interactive`
|`<object>`
|Image embedded as a live, interactive object
|For using CSS animations, scripting, webfonts, or when you want to specify a fallback image

|`inline`
|`<svg>`
|The SVG is embedded directly into the HTML itself
|For using CSS animations, scripting, webfonts, when you require search engines to search the SVG content

To allow SVG content reachable by JavaScript in the main DOM or to inherit styles from the main DOM
|===

When using the `interactive` option, you can specify a fallback image using the `fallback` attribute.
The fallback image is used if the browser does not support the `<object>` tag.
If the value of the fallback attribute is a relative path, it will be prefixed with the value of the `imagesdir` document attribute.

When using the `inline` or `interactive` options, the `viewBox` attribute must be defined on the root `<svg>` element in order for scaling to work properly.

When using the `inline` option, if you specify a width or height on the image macro in AsciiDoc, the `width`, `height` and `style` attributes on the `<svg>` element will be removed. Additionally, when using `inline` the primary SVG elements (e.g., `<svg>`) cannot have a namespace.

If using the `interactive` option, you must link to the CSS that declares the fonts in the SVG file using an XML stylesheet declaration.

If you're inserting an SVG using either the `inline` or `interactive` options, we strongly recommend you optimize your SVG using a tool like {url-svgo}[svgo^] or {url-svg-editor}[SVG Editor^].

As you work with SVG, you'll become more comfortable making the decision about which method to employ given the circumstances.
It's only confusing when you first encounter the choice.
To learn more about using SVG on the web, consult the online book https://svgontheweb.com/[SVG on the Web: A Practical Guide^] as well as https://www.sarasoueidan.com/tags/svg/[these articles about SVG^].
"#
);
