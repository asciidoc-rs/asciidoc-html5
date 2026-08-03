//! Coverage of the AsciiDoc language description's *SVG Images* page.
//!
//! An SVG image target accepts an `opts` value controlling how the SVG is
//! referenced: `none` (the default, an `<img>`), `interactive` (an `<object>`),
//! or `inline` (an embedded `<svg>`). The `none` and `interactive` referencing
//! (including the `<object>`'s `fallback`) are implemented and verified through
//! `convert`; the security-sensitive `interactive` behavior takes effect only
//! below the `Secure` safe mode, matching Asciidoctor. The `inline` option
//! embeds the SVG file's contents and is not yet implemented for block images
//! (tracked as <https://github.com/asciidoc-rs/asciidoc-html5/issues/275>), so
//! that material — and the `viewBox`/dimensions authoring guidance — is
//! non-normative.

use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-svg.adoc");

// Renders below the `Secure` safe mode, where the interactive SVG referencing
// is enabled (at `Secure`, the API default, an SVG always renders as an
// `<img>`).
fn convert_unsafe(source: &str) -> String {
    convert_with(source, &Options::new().safe_mode(SafeMode::Unsafe))
}

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

// The default (`none`) SVG referencing renders an ordinary `<img>`.
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

// The demonstration table illustrates the runtime hover behavior of each option
// (a visual, stylesheet-driven effect), so it is non-normative.
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

"#
);

// `none` renders an `<img>` and `interactive` renders an `<object>`. The
// `inline` (`<svg>`) referencing embeds the SVG file's contents and is not yet
// implemented for block images (issue #275); it still renders an `<img>` here.
#[test]
fn svg_referencing_options() {
    verifies!(
        r#"
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

"#
    );

    // `none` (default): an `<img>`.
    assert!(convert_unsafe("image::sample.svg[Diagram,300]")
        .contains(r#"<img src="sample.svg" alt="Diagram" width="300">"#));

    // `interactive`: an `<object>` (block image, below Secure), with the alt text
    // nested as the fallback.
    assert!(convert_unsafe("image::sample.svg[Diagram,300,opts=interactive]").contains(
        r#"<object type="image/svg+xml" data="sample.svg" width="300"><span class="alt">Diagram</span></object>"#
    ));

    // The inline image macro's `interactive` option likewise renders an
    // `<object>` (this path is handled by the parser's inline substitution).
    assert!(convert_unsafe("See image:sample.svg[Diagram,opts=interactive] here.").contains(
        r#"<span class="image"><object type="image/svg+xml" data="sample.svg"><span class="alt">Diagram</span></object></span>"#
    ));
}

// The `interactive` `<object>` accepts a `fallback` image (nested for user
// agents that can't render the object); a relative fallback path is resolved
// under `imagesdir`.
#[test]
fn interactive_fallback() {
    verifies!(
        r#"
When using the `interactive` option, you can specify a fallback image using the `fallback` attribute.
The fallback image is used if the browser does not support the `<object>` tag.
If the value of the fallback attribute is a relative path, it will be prefixed with the value of the `imagesdir` document attribute.

"#
    );

    // The `fallback` image is nested inside the `<object>` instead of the alt
    // text.
    assert!(convert_unsafe("image::sample.svg[Diagram,opts=interactive,fallback=fallback.png]")
        .contains(
            r#"<object type="image/svg+xml" data="sample.svg"><img src="fallback.png" alt="Diagram"></object>"#
        ));

    // A relative `fallback` path (and the target) are resolved under `imagesdir`.
    assert!(
        convert_unsafe(":imagesdir: img\n\nimage::sample.svg[Diagram,opts=interactive,fallback=fallback.png]")
            .contains(
                r#"<object type="image/svg+xml" data="img/sample.svg"><img src="img/fallback.png" alt="Diagram"></object>"#
            )
    );
}

// The `viewBox`/namespace requirements and the `inline`-option dimension
// stripping concern the embedded `<svg>` (issue #275) or its authoring, and the
// closing links are descriptive.
non_normative!(
    r#"
When using the `inline` or `interactive` options, the `viewBox` attribute must be defined on the root `<svg>` element in order for scaling to work properly.

When using the `inline` option, if you specify a width or height on the image macro in AsciiDoc, the `width`, `height` and `style` attributes on the `<svg>` element will be removed. Additionally, when using `inline` the primary SVG elements (e.g., `<svg>`) cannot have a namespace.

If using the `interactive` option, you must link to the CSS that declares the fonts in the SVG file using an XML stylesheet declaration.

If you're inserting an SVG using either the `inline` or `interactive` options, we strongly recommend you optimize your SVG using a tool like {url-svgo}[svgo^] or {url-svg-editor}[SVG Editor^].

As you work with SVG, you'll become more comfortable making the decision about which method to employ given the circumstances.
It's only confusing when you first encounter the choice.
To learn more about using SVG on the web, consult the online book https://svgontheweb.com/[SVG on the Web: A Practical Guide^] as well as https://www.sarasoueidan.com/tags/svg/[these articles about SVG^].
"#
);
