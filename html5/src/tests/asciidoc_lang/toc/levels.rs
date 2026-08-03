//! Coverage of the AsciiDoc language description's *Adjust the TOC Depth* page.
//!
//! The `toclevels` attribute controls how deep the table of contents goes.
//! Matching Asciidoctor 2.0.26, this crate lists section titles up to the
//! configured level (so `toclevels: 4` includes level-4 titles but excludes
//! level-5 ones), and in a document without parts it coerces `toclevels: 0` to
//! `1`. Those two behaviors are verified through `convert`; the definitional
//! prose, the default depth (verified on the *Automatic Table of Contents*
//! page), the multipart-book level-0 behavior (the book doctype is a known
//! limitation here), and the screenshot are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_css, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/toc/pages/levels.adoc");

// Renders a standalone document so the TOC and its nested `ul.sectlevelN` lists
// are present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the page-alias attribute entry, and the introductory
// sentence naming the `toclevels` attribute. Setup and descriptive prose.
non_normative!(
    r#"
= Adjust the TOC Depth
:page-aliases: section-depth.adoc

You can adjust the depth of section levels displayed in the table of contents (TOC) using the `toclevels` attribute.

"#
);

// Definitional prose: the accepted `toclevels` values, what a section level is,
// and the default (`2`, i.e. levels 1 and 2). The default depth is verified on
// the *Automatic Table of Contents* page; the multipart-book level-0 behavior
// relies on the book doctype, a known limitation of this crate, so this block
// is non-normative.
non_normative!(
    r#"
== Set toclevels

The `toclevels` document attribute controls the depth of the TOC.
Accepted values are the integers 0 through 5.
These values represent the section levels.
(A section level is one less than the number of `=` signs the precede the title in the source.)

If the `toclevels` attribute is not specified, it defaults to `2`.
That means the TOC displays level 1 (`==`) and level 2 (`===`) section titles and, in the case of a multipart book, level 0 (`=`) section titles (parts).

"#
);

// Raising `toclevels` deepens the TOC: with `:toclevels: 4`, the TOC lists
// section titles up to level 4 (`====`) and no deeper. The example body reaches
// only level 3, so the test drives a document extended through level 5 to
// exercise the level-4 boundary: `ul.sectlevel4` is present while
// `ul.sectlevel5` is not.
#[test]
fn toclevels_sets_the_toc_depth() {
    verifies!(
        r#"
Let's use the `toclevels` attribute to increase the depth of the TOC from 2 to 4.

.Define toclevels value
[source#ex-levels]
----
include::example$toc.adoc[tag=header]
:toc: <.>
:toclevels: 4 <.>
include::example$toc.adoc[tag=body]
----
<.> The `toc` attribute must be set in order to use `toclevels`.
<.> `toclevels` is set and assigned the value `4` in the document header.
The TOC will list the titles of any sections up to level 4 (i.e., `====`), when the document is rendered.

The result of <<ex-levels>> is displayed below.

"#
    );

    // The example body only reaches level 3, so extend it through level 5 to
    // exercise the level-4 boundary the `toclevels: 4` value sets.
    let source = "\
= The Intrepid Chronicles
Kismet Lee
:toc:
:toclevels: 4

== Level 1

=== Level 2

==== Level 3

===== Level 4

====== Level 5
";

    let output = convert(source);
    assert_css(&output, r#"ul.sectlevel4"#, 1);
    assert_css(&output, r#"ul.sectlevel5"#, 0);
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::toclevels.png[table of contents with the toclevels attribute set,role=screenshot]

"#
);

// The multipart-book case: `toclevels: 0` shows part titles (and level-0
// special sections). This relies on the book doctype, a known limitation here,
// so it is non-normative.
non_normative!(
    r#"
In a multipart book, if you only want to see part titles (as well as any special sections at level 0) in the TOC, set `toclevels` to 0.
"#
);

// In a document without parts (an article), `toclevels: 0` is coerced to `1`:
// the TOC still lists the level-1 section titles rather than rendering empty.
#[test]
fn toclevels_zero_without_parts_is_coerced_to_one() {
    verifies!(
        r#"
If the document does not have parts, and you set `toclevels` to 0, the value is coerced to 1.
"#
    );

    let source = "\
= The Intrepid Chronicles
Kismet Lee
:toc:
:toclevels: 0

== Certain Peril

=== A Recipe for Potion
";

    // Coerced to 1: the level-1 titles still appear (a true `0` in an article
    // would leave the TOC empty).
    let output = convert(source);
    assert_css(&output, r#"ul.sectlevel1"#, 1);
}
