//! Coverage of the AsciiDoc language description's *Adjust the TOC Depth* page.
//!
//! The `toclevels` attribute controls how deep the table of contents goes.
//! Matching Asciidoctor 2.0.26, this crate lists section titles up to the
//! configured level across the accepted range 0–5 (so `toclevels: 4` includes
//! level-4 titles but excludes level-5 ones), defaults the depth to `2` when
//! the attribute is unset, and in a document without parts coerces `toclevels:
//! 0` to `1`. Those behaviors are verified through `convert`; the
//! multipart-book level-0 behavior (the book doctype is a known limitation
//! here) and the screenshot are non-normative.

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

non_normative!(
    r#"
== Set toclevels

"#
);

// `toclevels` controls the TOC depth across its accepted range (the integers 0
// through 5), each value naming the deepest section level shown. A section
// level is one less than the number of leading `=` signs, so `==` is level 1
// (listed under `ul.sectlevel1`), `===` is level 2, and so on: at `toclevels:
// 5` all five levels appear, while at `toclevels: 1` only the level-1 titles
// do. (The value `0` is exercised by
// `toclevels_zero_without_parts_is_coerced_to_one`.)
#[test]
fn toclevels_controls_the_depth_across_the_accepted_range() {
    verifies!(
        r#"
The `toclevels` document attribute controls the depth of the TOC.
Accepted values are the integers 0 through 5.
These values represent the section levels.
(A section level is one less than the number of `=` signs the precede the title in the source.)

"#
    );

    // Sections at every level 1 (`==`) through 5 (`======`).
    let deep = "\
= Document Title
:toc:

== Level 1

=== Level 2

==== Level 3

===== Level 4

====== Level 5
";

    // At the maximum, all five section levels are listed. The `==` heading (two
    // `=` signs) is level 1 and lists under `ul.sectlevel1`, confirming the
    // level-is-one-less-than-the-sign-count definition.
    let five = convert(&deep.replace(":toc:", ":toc:\n:toclevels: 5"));
    assert_css(&five, r#"ul.sectlevel1"#, 1);
    assert_css(&five, r#"ul.sectlevel5"#, 1);

    // At `toclevels: 1`, only the level-1 titles appear.
    let one = convert(&deep.replace(":toc:", ":toc:\n:toclevels: 1"));
    assert_css(&one, r#"ul.sectlevel1"#, 1);
    assert_css(&one, r#"ul.sectlevel2"#, 0);
}

// When `toclevels` is unset it defaults to `2`, so the TOC lists level-1 (`==`)
// and level-2 (`===`) titles and stops there – a level-3 (`====`) section is
// omitted. (The line's multipart-book clause, level-0 part titles, relies on
// the book doctype, a known limitation of this crate, and is not exercised
// here.)
#[test]
fn toclevels_defaults_to_two() {
    verifies!(
        r#"
If the `toclevels` attribute is not specified, it defaults to `2`.
That means the TOC displays level 1 (`==`) and level 2 (`===`) section titles and, in the case of a multipart book, level 0 (`=`) section titles (parts).

"#
    );

    // No `toclevels`: a document with sections through level 3.
    let source = "\
= Document Title
:toc:

== Level 1

=== Level 2

==== Level 3
";

    let output = convert(source);
    assert_css(&output, r#"ul.sectlevel1"#, 1);
    assert_css(&output, r#"ul.sectlevel2"#, 1);
    assert_css(&output, r#"ul.sectlevel3"#, 0);
}

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
