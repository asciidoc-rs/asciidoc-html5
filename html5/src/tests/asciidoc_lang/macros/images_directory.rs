//! Coverage of the AsciiDoc language description's *Set the Images Directory*
//! page.
//!
//! The `imagesdir` document attribute is prepended to every image macro target,
//! so an image target should be written relative to that directory. An absolute
//! path or URL target ignores `imagesdir`, and its default value is empty
//! (targets resolve relative to the document). All verified through `convert`,
//! matching Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/images-directory.adoc");

non_normative!(
    r#"
= Set the Images Directory

The path to the location of the image catalog is controlled by the `imagesdir` attribute.

== imagesdir attribute syntax

"#
);

// `imagesdir` is prepended to every image target, so the target itself must not
// repeat it (the "Incorrect" example doubles the directory).
#[test]
fn imagesdir_is_prepended_to_the_target() {
    verifies!(
        r#"
`imagesdir` is a document attribute.
Its value is automatically added to the beginning of every image macro target.
The resolved location of a image is: `<value-of-imagesdir> + <image-macro-target>`.
Therefore, you never need to reference this attribute in an image macro.
You only need to set it in your document header.

.Incorrect
[source]
----
image::{imagesdir}/name-of-image.png[]
----

.Correct
[source]
----
image::name-of-image.png[]
----

"#
    );

    // Correct: the bare target is resolved under `imagesdir`.
    assert!(convert(":imagesdir: images\n\nimage::name-of-image.png[]")
        .contains(r#"<img src="images/name-of-image.png" alt="name of image">"#));

    // Incorrect: referencing `{imagesdir}` in the target doubles the directory,
    // because `imagesdir` is prepended again.
    assert!(
        convert(":imagesdir: images\n\nimage::{imagesdir}/name-of-image.png[]")
            .contains(r#"<img src="images/images/name-of-image.png" alt="name of image">"#)
    );
}

// An absolute or URL target ignores `imagesdir`; the default `imagesdir` is
// empty (targets resolve relative to the document).
#[test]
fn absolute_url_and_default() {
    verifies!(
        r#"
The value of `imagesdir` can be an absolute path, relative path or URL.
By default, the `imagesdir` value is empty.
That means the images are resolved relative to the document.
If an image macro's target is an absolute path or URL, the value of `imagesdir` is not added to the target path.

"#
    );

    // The default `imagesdir` is empty: the target is used unchanged.
    assert!(convert("image::name-of-image.png[]")
        .contains(r#"<img src="name-of-image.png" alt="name of image">"#));

    // An absolute-path target ignores a set `imagesdir`.
    assert!(convert(":imagesdir: images\n\nimage::/abs/pic.png[]")
        .contains(r#"<img src="/abs/pic.png" alt="pic">"#));

    // A URL target ignores a set `imagesdir`.
    assert!(
        convert(":imagesdir: images\n\nimage::https://example.org/pic.png[]")
            .contains(r#"<img src="https://example.org/pic.png" alt="pic">"#)
    );
}

non_normative!(
    r#"
The benefit of the processor adding the value of `imagesdir` to the start of all image targets is that you can globally control the folder where images are located per converter.
We refer to this folder as the image catalog.
Since different output formats require the images to be stored in different locations, this attribute makes it possible to accommodate many different scenarios.

We recommend relying on `imagesdir` when defining the target of your image to avoid hard-coding that common path in every single image macro.
Always think about where the image is relative to the image catalog.

TIP: You can set the `imagesdir` attribute in multiple places in your document, as long as it is not locked by the API.
This technique is useful if you store images for different parts, chapters, or sections of your document in different locations.
"#
);
