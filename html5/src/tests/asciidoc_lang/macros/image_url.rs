//! Coverage of the AsciiDoc language description's *Insert Images from a URL*
//! page.
//!
//! An image macro's target can be a URL, for both block (`image::`) and inline
//! (`image:`) images; `imagesdir` is ignored for a URL target. Conversely, when
//! `imagesdir` is itself a URL, a relative image target resolves under it. All
//! verified through `convert`, matching Asciidoctor 2.0.26.
//!
//! The examples are pulled in with `include::example$image.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/image-url.adoc");

non_normative!(
    r#"
= Insert Images from a URL

//(i.e., images with a URL target)
You can reference images served from any URL (e.g., your blog, an image hosting service, your server, etc.) and never have to worry about downloading the images and putting them somewhere locally.

== Image URL targets

Here are a few examples of images that have a URL target:

"#
);

// Both a block image and an inline image accept a URL target; `imagesdir` is
// ignored for a URL target.
#[test]
fn url_targets() {
    verifies!(
        r#"
.Block image with a URL target
[source]
----
include::example$image.adoc[tag=url]
----

include::example$image.adoc[tag=url]

.Inline image with a URL target
[source]
----
include::example$image.adoc[tag=in-url]
----

include::example$image.adoc[tag=in-url]

NOTE: The value of `imagesdir` is ignored when the image target is a URL.

"#
    );

    // The `tag=url` block image.
    let block =
        convert("image::https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg[Tux,250,350]");
    assert_css(&block, "div.imageblock img", 1);
    assert!(block.contains(
        r#"<img src="https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg" alt="Tux" width="250" height="350">"#
    ));

    // The `tag=in-url` inline image.
    let inline = convert(
        "You can find image:https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg[Linux,25,35] everywhere these days.",
    );
    assert_css(&inline, "span.image img", 1);
    assert!(inline.contains(
        r#"<img src="https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg" alt="Linux" width="25" height="35">"#
    ));

    // `imagesdir` is ignored for a URL target: the URL is used unchanged.
    assert!(
        convert(":imagesdir: local\n\nimage::https://upload.wikimedia.org/x.svg[X]")
            .contains(r#"<img src="https://upload.wikimedia.org/x.svg" alt="X">"#)
    );
}

// When `imagesdir` is itself a URL, a relative image target resolves under it.
#[test]
fn url_as_base_url() {
    verifies!(
        r#"
If you want to avoid typing the URL prefix for every image, and all the images are located on the same server, you can use the `imagesdir` attribute to set the base URL:

.Using a URL as the base URL for images
[source]
----
include::example$image.adoc[tag=base-url]
----

This time, `imagesdir` is used since the image target is not a URL (the value of `imagesdir` just happens to be one).
"#
    );

    // The `tag=base-url` snippet: with `imagesdir` set to a URL, the relative
    // target `3/35/Tux.svg` resolves under it (the scheme's `//` is preserved).
    let output = convert(
        ":imagesdir: https://upload.wikimedia.org/wikipedia/commons\n\nimage::3/35/Tux.svg[Tux,250,350]",
    );
    assert!(output.contains(
        r#"<img src="https://upload.wikimedia.org/wikipedia/commons/3/35/Tux.svg" alt="Tux" width="250" height="350">"#
    ));
}
