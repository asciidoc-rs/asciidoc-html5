use std::fs;

use crate::{tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/manage-images.adoc");

// Asciidoctor's "Manage Images" page. It documents the `data-uri` attribute,
// which embeds referenced images as base64 `data:` URIs instead of linking
// them, and the `allow-uri-read` attribute, which additionally permits
// embedding a *remote* image.
//
// The verifiable claim is `data-uri` embedding a local image — this crate
// implements it in full (see the `data_uri` test module in `renderer.rs`,
// which this page's test reuses the same GIF fixture from). `allow-uri-read`
// has no counterpart: embedding a remote image would require a network
// fetch, which is a deliberate non-goal for this crate (issue #39 — this
// crate does no network reads), so that section is non-normative.
//
// Tracked from the library crate only: every verifiable claim is about the
// `<img>` markup `asciidoc_html5::convert`/`convert_with` emits. Where the
// page drives behavior with a CLI `-a` attribute, we supply the same
// attribute through `Options`; the `adoc -a` option is a thin forwarder onto
// that API, covered by the CLI crate's own tests.

/// A one-pixel transparent GIF — the smallest real image to embed. Its
/// strict-base64 encoding is [`GIF_B64`].
const GIF: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];
const GIF_B64: &str = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==";

/// Creates a fresh temp directory holding `my-images/photo.gif`, so a
/// `:imagesdir: my-images` document can find it.
fn scratch_with_image() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "adoc-html-backend-manage-images-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("my-images")).expect("create scratch dir");
    fs::write(dir.join("my-images/photo.gif"), GIF).expect("write scratch image");
    dir.canonicalize().expect("canonicalize scratch dir")
}

non_normative!(
    r#"
= Manage Images

Images are not embedded in the HTML output by default.
If you have image references in your document, you'll have to save the image files in the same directory as your converted document.

== Embed images with the data-uri attribute

As an alternative, you can embed the images directly into the document by setting the `data-uri` document attribute.

"#
);

// Setting `data-uri` (below `Secure`, where embedding is enabled) embeds a
// locally readable, `imagesdir`-resolved image as a base64 `data:` URI
// instead of linking it.
#[test]
fn data_uri_embeds_the_image_as_a_base64_data_uri() {
    verifies!(
        r#"
.data-uri attribute set in document header
[,asciidoc]
----
include::example$my-document.adoc[tag=title]
:imagesdir: my-images
:data-uri:
include::example$my-document.adoc[tags=body;image]
----

You can also set `data-uri` using the API or CLI (shown here):

 $ asciidoctor -a data-uri my-document.adoc

"#
    );

    // The page sets `data-uri` and `imagesdir` from the document header /
    // CLI; we supply them the same way — as external attributes — through
    // `Options`, the API the `adoc -a` option feeds into.
    let dir = scratch_with_image();
    let html = crate::convert_with(
        ":imagesdir: my-images\n\nimage::photo.gif[A photo]",
        &Options::new()
            .safe_mode(SafeMode::Unsafe)
            .base_dir(dir.clone())
            .set("data-uri"),
    );
    let _ = fs::remove_dir_all(&dir);

    assert!(
        html.contains(&format!(
            "<img src=\"data:image/gif;base64,{GIF_B64}\" alt=\"A photo\">"
        )),
        "{html}"
    );

    non_normative!(
        r#"
When you view the HTML file in your browser, you should see the image displayed in the page.

image::my-document-with-data-uri.png[]

"#
    );
}

// Fetching a *remote* image to embed it under `allow-uri-read` requires a
// network read this crate does not perform — a deliberate non-goal (issue
// #39), so this section carries no rule to verify.
non_normative!(
    r#"
== allow-uri-read attribute

If the target of one or more images in the document is a URI, you must also set the `allow-uri-read` attribute securely and run Asciidoctor in `SECURE` mode or less.

 $ asciidoctor -a data-uri -a allow-uri-read my-document.adoc

The same is true when converting the document to PDF using Asciidoctor PDF, regardless of whether the `data-uri` attribute is set since this behavior is implicit.
"#
);
