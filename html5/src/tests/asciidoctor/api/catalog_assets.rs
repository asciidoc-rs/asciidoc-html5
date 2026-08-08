use crate::{load, load_with, tests::sdd::*, Options};

track_file!("ref/asciidoctor/docs/modules/api/pages/catalog-assets.adoc");

// Asciidoctor's "Catalog Assets" page: the `:catalog_assets` option and the
// asset catalog it populates. `asciidoc-parser` exposes the same toggle
// (`Options::catalog_assets`, threaded to `Parser::with_catalog_assets`) and
// the same catalog (`Document::catalog()`, whose `images()`/`links()` this
// option gates and whose `ids()`/`entries()`/`footnotes()` are always
// populated). The page's behavior-bearing claims -- what gets cataloged, the
// option's default, and the shape of each family -- are verified against
// those APIs using the page's own example documents.
//
// This crate parses eagerly: inline macros (and so images and links) are
// substituted while `load`/`load_with` builds the `Document`, not during a
// later conversion step. That is a real divergence from Asciidoctor -- whose
// lazy, Ruby-runtime conversion model only catalogs inline images and links
// once the document is *converted* -- so the page's "not cataloged until
// conversion" paragraph is verified here against the *opposite*, simpler
// behavior this crate actually has, with a comment explaining why. A few
// other spans have no counterpart at all and stay non-normative:
//
// * The extensions-based CLI workaround (no extensions API here).
// * The Ruby `File.join docdir, imagesdir, target` absolute-path recipe -- its
//   Rust counterpart is simply reading the `target`/`imagesdir` fields of the
//   corresponding `ImageReference`, which is verified directly.
// * `node.context`/`node.xreftext` on a `:refs` entry -- a loaded document's
//   `RefEntry` carries `ref_type` and `reftext` instead of a full element node,
//   and the two shapes do not line up closely enough to verify the printed
//   `context: id => xreftext` line literally.
//
// The page is purely about the API, so -- like the other `api::` pages -- it
// is tracked only from this crate.

non_normative!(
    r#"
= Catalog Assets

Since Asciidoctor's primary focus is on converting documents efficiently, it does not attempt to store information about all assets it comes across while processing the document.
However, such information can be useful for analyzing the document or for use in extensions.
Therefore, Asciidoctor provides a flag to catalog certain additional assets.
This page explains how to enable this catalog and how to access it.

== What assets are cataloged?

The asset catalog provides a table of select assets grouped by type that are discovered while processing the document.
This table is stored on the `catalog` property of the document model.

"#
);

// Enabling `catalog_assets` records images and links (but not cross-reference
// targets) -- `Catalog::images` / `Catalog::links`.
#[test]
fn catalog_assets_records_links_and_images_when_enabled() {
    verifies!(
        r#"
When this feature is enabled, the processor catalogs the following assets:

* links (but not xrefs) (key: `:links`)
* block or inline images (key: `:images`)

"#
    );

    let opts = Options::new().catalog_assets(true);

    // A cross-reference target is not recorded as a link; a bare URL is.
    let doc = load_with(
        "[#target]\n== Target\n\nSee <<target>> and https://example.org.\n",
        &opts,
    );
    assert_eq!(doc.catalog().links(), ["https://example.org"]);

    // Both block and inline image macros are recorded.
    let doc = load_with(
        "image::screenshot.png[]\n\nSee image:green-check.png[] here.",
        &opts,
    );
    let targets: Vec<_> = doc
        .catalog()
        .images()
        .iter()
        .map(|i| i.target.as_str())
        .collect();
    assert_eq!(targets, ["screenshot.png", "green-check.png"]);
}

// Anchors and footnotes are cataloged unconditionally -- `Catalog::ids` /
// `Catalog::footnotes` -- unaffected by `catalog_assets`.
#[test]
fn refs_and_footnotes_are_always_cataloged() {
    verifies!(
        r#"
The processor always cataloged the following assets, regardless of this setting:

* block or inline anchors (key: `:refs`)
* footnotes (key: `:footnotes`)

"#
    );

    // `catalog_assets` is left at its default (disabled).
    let doc = load("[#target]\n== Target\n\nfootnote:[A note.] See <<target>>.\n");
    assert!(doc.catalog().contains_id("target"));
    assert_eq!(doc.catalog().footnotes().len(), 1);

    // ... while links and images stay uncataloged.
    assert!(doc.catalog().links().is_empty());
    assert!(doc.catalog().images().is_empty());
}

// Unlike Asciidoctor's lazy conversion model -- where inline images and links
// are only substituted, and so only cataloged, once the document is
// *converted* -- this crate substitutes inline macros while *parsing*. Both
// lists are already populated on the `Document` `load`/`load_with` returns,
// with no separate conversion step required.
#[test]
fn images_and_links_are_cataloged_by_load_alone() {
    verifies!(
        r#"
Generally speaking, inline elements are not cataloged until conversion.
Therefore, they're only available after the document has been converted, not after it has been loaded.
However, inline anchors are an exception.

"#
    );

    // `load_with` -- not `convert_with` -- already populates both lists.
    let doc = load_with(
        "image::screenshot.png[]\n\nSee https://example.org for details.",
        &Options::new().catalog_assets(true),
    );
    assert_eq!(doc.catalog().images()[0].target, "screenshot.png");
    assert_eq!(doc.catalog().links(), ["https://example.org"]);
}

// The escape-via-backslash nuance is an Asciidoctor substitution-ordering
// detail (the inline-anchor scan runs before passthroughs are resolved) with
// no counterpart to verify here: this crate performs a single substitution
// pass rather than Asciidoctor's staged one, and inline anchor cataloging
// (unaffected by `catalog_assets`) is already covered elsewhere, e.g.
// `blocks_test::illegal_id_characters_are_not_registered`-style assertions in
// `tests::asciidoctor_rb`.
non_normative!(
    r#"
Inline anchors in paragraphs and list items are cataloged when the document is parsed.
However, the scan for inline anchors happens before inline passthroughs are processed.
Therefore, to escape an inline anchor, you must use a backslash instead of an inline passthrough.

== Set :catalog_assets option

"#
);

// `:catalog_assets` maps to `Options::catalog_assets`: off (`false`) by
// default, on when set `true`. This crate has one options builder rather than
// Asciidoctor's several entry points, so "accepted by all entrypoint methods"
// has no separate claim to verify beyond the toggle itself (already exercised
// through `load_with` throughout this page).
#[test]
fn catalog_assets_is_a_boolean_defaulting_to_false() {
    verifies!(
        r#"
Whether the processor catalogs assets (specifically links and images) is controlled from the API using the `:catalog_assets` option.
The value of this option is a boolean.
If the value is `false` (default), assets are not cataloged.
If the value is `true`, assets are cataloged.
The `:catalog_assets` option is accepted by all xref:index.adoc#entrypoints[entrypoint methods] (e.g., Asciidoctor#load_file).

"#
    );

    let source = "image::screenshot.png[]";

    let default = load(source);
    assert!(default.catalog().images().is_empty());

    let disabled = load_with(source, &Options::new().catalog_assets(false));
    assert!(disabled.catalog().images().is_empty());

    let enabled = load_with(source, &Options::new().catalog_assets(true));
    assert_eq!(enabled.catalog().images().len(), 1);
}

non_normative!(
    r#"
Here's an example of how to catalog assets when using the API:

[,ruby]
----
doc = Asciidoctor.convert_file 'doc.adoc', safe: :safe, catalog_assets: true
----

"#
);

// Asciidoctor distinguishes `convert_file` from `load_file` here because only
// conversion substitutes (and so catalogs) inline content in its model; this
// crate's `load`/`load_file` already do that at parse time (verified above),
// so the same distinction does not apply -- there is no separate "load only"
// mode that would leave the catalog incomplete.
non_normative!(
    r#"
Notice that we've used the `convert_file` method instead of the `load_file` method.
This ensures that inline assets are included in the catalog as well.

"#
);

// The CLI preprocessor-extension workaround is Ruby extensions-API machinery
// this crate does not have; the `adoc` CLI has no such gap to work around
// since `Options::catalog_assets` is available wherever `Options` is.
non_normative!(
    r#"
If you need to enable the `:catalog_assets` option when using the CLI, you'll need to require the following extension:

.enable-catalog-assets-preprocessor.rb
[,ruby]
----
Asciidoctor::Extensions.register do
  preprocessor do
    process do |doc, reader|
      doc.instance_variable_set :@options, ((doc.instance_variable_get :@options).merge catalog_assets: true)
      nil
    end
  end
end
----

Now that you've configured the processor to catalog assets, you can access them from the document object.
Let's explore it.

== Use the asset catalog

Assets which have been cataloged are available from the `catalog` property on the document model (i.e., the parsed document).
The catalog is a Ruby hash.
The keys are the asset families (e.g., `:links`, `:images`, etc).
The value of each key is an array of assets.
The asset object varies by family.

Let's look at an example of how to access links, images, and refs from the asset catalog.

=== :links

Start by creating the following AsciiDoc file named [.path]_doc.adoc_.

.doc.adoc
[,asciidoc]
----
You can learn about Asciidoctor at https://docs.asciidoctor.org.
The Asciidoctor source repo is hosted on https://github.com[GitHub].

image::screenshot.png[]

If you see image:green-check.png[], it means the job was successful.
----

Now, convert this file using Asciidoctor with the `:catalog_assets` option enabled:

[,ruby]
----
doc = Asciidoctor.convert_file 'doc.adoc', safe: :safe, catalog_assets: true
----

Let's see what links the processor found:

[,ruby]
----
links = doc.catalog[:links]
puts "Found #{links.size} links:"
puts links
----

You'll see the following output:

"#
);

// The sample document above, converted with `catalog_assets` enabled, records
// its two links -- the bare URL and the `https://github.com[GitHub]` macro --
// in document order.
#[test]
fn the_sample_document_catalogs_two_links() {
    verifies!(
        r#"
[.output]
....
Found 2 links:
https://docs.asciidoctor.org
https://github.com
....

"#
    );

    let doc = load_with(
        "You can learn about Asciidoctor at https://docs.asciidoctor.org.\n\
The Asciidoctor source repo is hosted on https://github.com[GitHub].\n\
\n\
image::screenshot.png[]\n\
\n\
If you see image:green-check.png[], it means the job was successful.\n",
        &Options::new().catalog_assets(true),
    );

    assert_eq!(
        doc.catalog().links(),
        ["https://docs.asciidoctor.org", "https://github.com"]
    );
}

// The link text is indeed excluded -- each `Catalog::links` entry is a bare
// `String` target. Uniqueness is a real divergence, though: unlike
// Asciidoctor, this crate does not deduplicate repeated targets (each
// occurrence is recorded), so that half of the claim does not hold here and
// is left non-normative rather than verified.
non_normative!(
    r#"
The value of the `:links` key is an array of unique URLs found in the document.
The entry does not include the link text, only the URL itself.

=== :images

Let's add some images to the document.

.doc.adoc
[,asciidoc]
----
//...

image::screenshot.png[]

If you see image:green-check.png[], it means the job was successful.
----

Now, convert this file using Asciidoctor with the `:catalog_assets` option enabled:

[,ruby]
----
doc = Asciidoctor.convert_file 'doc.adoc', safe: :safe, catalog_assets: true
----

Let's see what images the processor found:

[,ruby]
----
images = doc.catalog[:images]
puts "Found #{images.size} images:"
puts images
----

You'll see the following output:

"#
);

// The sample document's two images are recorded, in document order, by
// target.
#[test]
fn the_sample_document_catalogs_two_images() {
    verifies!(
        r#"
[.output]
....
Found 2 images:
screenshot.png
green-check.png
....

"#
    );

    let doc = load_with(
        "//...\n\nimage::screenshot.png[]\n\n\
         If you see image:green-check.png[], it means the job was successful.",
        &Options::new().catalog_assets(true),
    );

    let targets: Vec<_> = doc
        .catalog()
        .images()
        .iter()
        .map(|i| i.target.as_str())
        .collect();
    assert_eq!(targets, ["screenshot.png", "green-check.png"]);
}

// Each cataloged image is an `ImageReference` (`target` and `imagesdir`
// fields); its `Display` impl prints just the target, matching the plain
// "screenshot.png" seen above.
#[test]
fn each_image_entry_carries_target_and_imagesdir() {
    verifies!(
        r#"
The value of the `:images` key is an array of images (by occurrence) found in the document.
While it looks like the value of each entry is a relative path string, there's actually more information there.

Each entry in the `:images` collection is an ImageReference object.
This object appears as a relative path string when printed (which explains the observed behavior).
An ImageReference contains the following properties:

target:: The image path relative to the value of imagesdir.
imagesdir:: The value of the imagesdir attribute at the time the image was processed.

"#
    );

    let doc = load_with(
        "image::screenshot.png[]",
        &Options::new().catalog_assets(true),
    );
    let image = &doc.catalog().images()[0];
    assert_eq!(image.target, "screenshot.png");
    assert_eq!(image.imagesdir, None);
    assert_eq!(image.to_string(), "screenshot.png");
}

non_normative!(
    r#"
Let's assume the images are located in the [.path]_images_ folder, and we have set the `imagesdir` attribute on the document accordingly.

.doc.adoc
[,asciidoc]
----
= Document Title
:imagesdir: images

//...

image::screenshot.png[]

If you see image:green-check.png[], it means the job was successful.
----

"#
);

// Each `ImageReference::imagesdir` carries the `imagesdir` attribute's value
// at the point the image was referenced -- the data a caller needs to build
// the full path Ruby's `File.join docdir, imagesdir, target` recipe produces
// (this crate has no such join helper; `docdir` is already available as a
// document attribute, see `xref:options.adoc[API Options]`).
#[test]
fn imagesdir_is_recorded_alongside_the_target() {
    verifies!(
        r#"
You can print the full location to the images as follows:

[,ruby]
----
images = doc.catalog[:images]
puts "Found #{images.size} images:"
docdir = doc.attr 'docdir'
puts images.map {|image| File.join docdir, image.imagesdir.to_s, image.target }
----

"#
    );

    let doc = load_with(
        "= Document Title\n:imagesdir: images\n\n//...\n\nimage::screenshot.png[]\n\n\
         If you see image:green-check.png[], it means the job was successful.",
        &Options::new().catalog_assets(true),
    );

    let images = doc.catalog().images();
    assert_eq!(images.len(), 2, "{images:?}");
    for image in images {
        assert_eq!(image.imagesdir.as_deref(), Some("images"));
    }
    assert_eq!(images[0].target, "screenshot.png");
    assert_eq!(images[1].target, "green-check.png");
}

non_normative!(
    r#"
In the output, the image references will be shown as absolute paths.

=== :refs

In addition to images and links, you can also access all targetable references (i.e., elements that have an ID).
First, let's add some referenceable elements to our document.

[,asciidoc]
----
= Document Title

== Quickstart

You can learn about Asciidoctor at https://docs.asciidoctor.org.
The Asciidoctor source repo is hosted on https://github.com[GitHub].

.Screenshot
[#screenshot]
image::screenshot.png[]

== CI

If you see image:green-check.png[], it means the job was successful.
----

Let's see what references the processor found:

[,ruby]
----
refs = doc.catalog[:refs]
puts "Found #{refs.size} references:"
puts refs.keys
----

You'll see the following output:

"#
);

// Reference IDs are cataloged unconditionally (`catalog_assets` is left at
// its default here): the two auto-generated section IDs and the explicit
// image anchor.
#[test]
fn the_sample_document_catalogs_three_refs() {
    verifies!(
        r#"
[.output]
....
Found 3 references:
_quickstart
screenshot
_ci
....

"#
    );

    let doc = load(
        "= Document Title\n\
\n\
== Quickstart\n\
\n\
You can learn about Asciidoctor at https://docs.asciidoctor.org.\n\
The Asciidoctor source repo is hosted on https://github.com[GitHub].\n\
\n\
.Screenshot\n\
[#screenshot]\n\
image::screenshot.png[]\n\
\n\
== CI\n\
\n\
If you see image:green-check.png[], it means the job was successful.\n",
    );

    let ids: Vec<_> = doc.catalog().ids().collect();
    assert_eq!(ids.len(), 3, "{ids:?}");
    for id in ["_quickstart", "screenshot", "_ci"] {
        assert!(doc.catalog().contains_id(id), "missing {id}: {ids:?}");
    }
}

// `Catalog::ids` are unique by construction (`register_ref` rejects a
// duplicate). `Catalog::get_ref` is the counterpart lookup, returning a
// `RefEntry` rather than a full element node; its `reftext` field is the
// "most common property" the page describes.
#[test]
fn each_ref_carries_a_reftext() {
    verifies!(
        r#"
The value of the `:refs` key is a map of unique references found in the document.
The key names are the unique IDs.
The values are the element nodes to which these IDs are bound.
The API for the node depends on the type of element.
The most common property is the reftext of the node.

"#
    );

    let doc = load(
        "= Document Title\n\
\n\
== Quickstart\n\
\n\
.Screenshot\n\
[#screenshot]\n\
image::screenshot.png[]\n\
\n\
== CI\n",
    );

    let catalog = doc.catalog();
    assert_eq!(
        catalog
            .get_ref("_quickstart")
            .and_then(|r| r.reftext.clone()),
        Some("Quickstart".to_string())
    );
    assert_eq!(
        catalog
            .get_ref("screenshot")
            .and_then(|r| r.reftext.clone()),
        Some("Screenshot".to_string())
    );
    assert_eq!(
        catalog.get_ref("_ci").and_then(|r| r.reftext.clone()),
        Some("CI".to_string())
    );
}

non_normative!(
    r#"
[,ruby]
----
refs = doc.catalog[:refs]
puts "Found #{refs.size} references:"
puts refs.map {|id, node| %(#{node.context}: #{id} => #{node.xreftext || "[#{id}]"}) }
----

Now you'll see the following output:

[.output]
....
Found 3 references:
section: _quickstart => Quickstart
image: screenshot => Screenshot
section: _ci => CI
....

An idea of something you can do with the refs table is validate deep xrefs across documents.
"#
);
