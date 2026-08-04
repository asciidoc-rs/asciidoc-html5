//! Coverage of the AsciiDoc language description's *Add Link to Image* page.
//!
//! The `link` attribute wraps an image in a link (`<a class="image">`), for
//! both block and inline images. The link controls from the link macro
//! (`window`, and the `nofollow`/`noopener` options) apply, and a `_blank`
//! window automatically enables `noopener`. All verified through `convert`,
//! matching Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-link.adoc");

non_normative!(
    r#"
= Add Link to Image

You can turn an image into a link by using the `link` attribute.

== link attribute

"#
);

// A block image's `link` attribute wraps the `<img>` in an `<a class="image">`,
// whether declared on the block attribute line above the macro or inside it.
#[test]
fn block_image_link() {
    verifies!(
        r#"
The link attribute on a block or image macro acts as though the image is wrapped in a link macro.
While it's possible to wrap an inline image macro in a link macro, that combination is not well supported and may introduce subtle parsing problems.
Therefore, you should use the `link` attribute on the image macro instead.

The value of the `link` attribute is akin to the target of the link macro.
It can point to any URL or relative path.

For a block image macro, the `link` attribute can be added to the block attribute line above the macro or inside the contents of the macro.

----
[link=https://example.org]
image::logo.png[Logo]
----

or

----
image::logo.png[Logo,link=https://example.org]
----

"#
    );

    let expected =
        r#"<a class="image" href="https://example.org"><img src="logo.png" alt="Logo"></a>"#;

    // The `link` on the block attribute line above the macro.
    assert!(convert("[link=https://example.org]\nimage::logo.png[Logo]").contains(expected));

    // The `link` inside the macro's attribute list.
    assert!(convert("image::logo.png[Logo,link=https://example.org]").contains(expected));
}

// An inline image's `link` attribute (inside the macro) wraps the `<img>` in an
// `<a class="image">`, itself inside the `<span class="image">`.
#[test]
fn inline_image_link() {
    verifies!(
        r#"
For an inline macro, the `link` attribute must be added inside the contents of the macro.

----
image:apply.jpg[Apply,link=https://apply.example.org] today!
----

"#
    );

    let output = convert("image:apply.jpg[Apply,link=https://apply.example.org] today!");
    assert!(output.contains(
        r#"<span class="image"><a class="image" href="https://apply.example.org"><img src="apply.jpg" alt="Apply"></a></span>"#
    ));
}

non_normative!(
    r#"
== Link controls

When using the `link` attribute, you can also use the same controls supported by the link macro to control how the link is constructed.
Those controls are as follows:

* `window` attribute - instructs the browser to open the link in the specified named window
* `nofollow` option - instructs search engines to not follow the link
* `noopener` option - instructs the browser to navigate to the target without granting the new browsing context access to the original document

"#
);

// A `_blank` window sets `target` and automatically enables `noopener`; the
// `nofollow` option is also emitted, all in the link's `rel`.
#[test]
fn link_controls() {
    verifies!(
        r#"
When the value of `window` attribute is *_blank*, the `noopener` option is automatically enabled.

Here's an example that shows how to use these controls.

----
image::logo.png[Logo,link=https://example.org,window=_blank,opts=nofollow]
----

"#
    );

    let output =
        convert("image::logo.png[Logo,link=https://example.org,window=_blank,opts=nofollow]");
    // `_blank` sets the target and auto-adds `noopener`; `nofollow` is also set.
    assert!(output.contains(
        r#"<a class="image" href="https://example.org" target="_blank" rel="nofollow noopener"><img src="logo.png" alt="Logo"></a>"#
    ));
}

non_normative!(
    r#"
Refer to the xref:link-macro-attribute-parsing.adoc#target-a-separate-window[Target a separate window] section in the link macro documentation for more information about how these link controls work.
"#
);
