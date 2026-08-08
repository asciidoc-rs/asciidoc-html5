use crate::{load, load_with, tests::sdd::*, Options};

track_file!("docs/modules/api/pages/catalog-assets.adoc");

// This crate's "Catalog Assets" page. It documents `Options::catalog_assets`
// and the `Catalog::images`/`links`/`ids`/`get_ref` accessors it gates (or, for
// IDs, does not gate). Each shown Rust snippet is verified here; the
// surrounding prose and the closing "Known limitations" section are
// non-normative.

non_normative!(
    r#"
= Catalog Assets
:navtitle: Catalog Assets
:description: How to enable and read the document catalog's image, link, and reference entries -- the asciidoc_html5 counterpart to Asciidoctor's catalog_assets option.

Every document loaded by `asciidoc_html5` carries a catalog of its referenceable
elements -- anchors, section headings, and footnotes. `Options::catalog_assets`
extends that catalog to also record the images and links referenced in the
document, mirroring Asciidoctor's `:catalog_assets` API option.

[NOTE]
====
The prose on this page is non-normative documentation. The API invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== What gets cataloged

Anchors, section headings, and footnotes are always cataloged, whether or not
`catalog_assets` is enabled. Images and links are cataloged only when it is:

"#
);

// Images and links are cataloged only when `catalog_assets` is enabled.
#[test]
fn images_and_links_are_gated_by_catalog_assets() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_html5::{load, load_with, Options};

// Off by default: images and links are not recorded.
let doc = load("image::screenshot.png[]\n\nSee https://example.org.");
assert!(doc.catalog().images().is_empty());
assert!(doc.catalog().links().is_empty());

// Enabled: both are recorded, in document order.
let doc = load_with(
    "image::screenshot.png[]\n\nSee https://example.org.",
    &Options::new().catalog_assets(true),
);
assert_eq!(doc.catalog().images()[0].target, "screenshot.png");
assert_eq!(doc.catalog().links(), ["https://example.org"]);
----

"#
    );

    // Off by default: images and links are not recorded.
    let doc = load("image::screenshot.png[]\n\nSee https://example.org.");
    assert!(doc.catalog().images().is_empty());
    assert!(doc.catalog().links().is_empty());

    // Enabled: both are recorded, in document order.
    let doc = load_with(
        "image::screenshot.png[]\n\nSee https://example.org.",
        &Options::new().catalog_assets(true),
    );
    assert_eq!(doc.catalog().images()[0].target, "screenshot.png");
    assert_eq!(doc.catalog().links(), ["https://example.org"]);
}

non_normative!(
    r#"
Unlike Asciidoctor -- whose lazy, Ruby-runtime conversion model only catalogs
inline images and links once the document is *converted* -- this crate
substitutes inline macros while *parsing*, so both lists are already populated
on the `Document` that `load`/`load_with` returns; no separate conversion step
is required.

== Why is this opt-in?

Rendering an image or link macro already needs its target (and, for images,
`imagesdir`) whether or not `catalog_assets` is set -- that work happens either
way. What the option gates is recording that data *again* into the catalog: an
extra clone and push per occurrence, for every image and link in the document.
`catalog_assets` lets a caller who never reads the image or link catalog skip
that cost, matching Asciidoctor, which documents this same option as an
opt-in: it "does not attempt to store information about all assets it comes
across while processing the document" unless asked to.

== Read cataloged images

Each cataloged image is an `ImageReference`, pairing the macro's `target` with
the `imagesdir` attribute value in effect where it was referenced:

"#
);

// An `ImageReference` pairs `target` with the `imagesdir` in effect where the
// image was referenced.
#[test]
fn image_references_carry_target_and_imagesdir() {
    verifies!(
        r#"
[,rust]
----
let opts = Options::new().catalog_assets(true);
let doc = load_with(
    "= Doc\n:imagesdir: images\n\nimage::screenshot.png[]",
    &opts,
);
let image = &doc.catalog().images()[0];
assert_eq!(image.target, "screenshot.png");
assert_eq!(image.imagesdir.as_deref(), Some("images"));
----

"#
    );

    let opts = Options::new().catalog_assets(true);
    let doc = load_with(
        "= Doc\n:imagesdir: images\n\nimage::screenshot.png[]",
        &opts,
    );
    let image = &doc.catalog().images()[0];
    assert_eq!(image.target, "screenshot.png");
    assert_eq!(image.imagesdir.as_deref(), Some("images"));
}

non_normative!(
    r#"
== Read cataloged links

Cataloged links are the target of each `link:`/`mailto:` macro and bare
URL/email autolink, in document order:

"#
);

// Cataloged links are macro/autolink targets, in document order.
#[test]
fn links_are_cataloged_in_document_order() {
    verifies!(
        r#"
[,rust]
----
let opts = Options::new().catalog_assets(true);
let doc = load_with(
    "See https://example.org and https://docs.example.org.",
    &opts,
);
assert_eq!(
    doc.catalog().links(),
    ["https://example.org", "https://docs.example.org"]
);
----

"#
    );

    let opts = Options::new().catalog_assets(true);
    let doc = load_with(
        "See https://example.org and https://docs.example.org.",
        &opts,
    );
    assert_eq!(
        doc.catalog().links(),
        ["https://example.org", "https://docs.example.org"]
    );
}

non_normative!(
    r#"
A cross-reference target (`<<id>>`) is never recorded as a link.

== Read referenceable IDs

IDs are always cataloged, so this works with or without `catalog_assets`. Look
one up by ID, or list every registered ID:

"#
);

// IDs are always cataloged, unaffected by `catalog_assets`.
#[test]
fn ids_are_always_cataloged() {
    verifies!(
        r#"
[,rust]
----
let doc = load("[#target]\n== A Section\n\nBody.");
assert!(doc.catalog().contains_id("target"));

let entry = doc.catalog().get_ref("target").unwrap();
assert_eq!(entry.reftext.as_deref(), Some("A Section"));
----

"#
    );

    let doc = load("[#target]\n== A Section\n\nBody.");
    assert!(doc.catalog().contains_id("target"));

    let entry = doc.catalog().get_ref("target").unwrap();
    assert_eq!(entry.reftext.as_deref(), Some("A Section"));
}

non_normative!(
    r#"
== Known limitations

Unlike Asciidoctor's documented `:links` catalog, this crate's `Catalog::links`
does not deduplicate repeated targets: a URL referenced twice appears twice.
And a `Catalog` entry (`RefEntry`) carries a `ref_type` and `reftext` rather
than the full converter node Asciidoctor's `catalog[:refs]` exposes, so there
is no `node.context`/`node.xreftext`-style API to build a `context: id =>
xreftext` summary line the way Asciidoctor's page shows.
"#
);
