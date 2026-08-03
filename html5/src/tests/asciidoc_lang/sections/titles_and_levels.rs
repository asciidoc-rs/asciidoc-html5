//! Coverage of the AsciiDoc language description's *Section Titles and Levels*
//! page.
//!
//! The page's normative HTML rule is that a section title becomes a heading tag
//! whose number is one greater than the section level, so a level 1 section
//! (`==`) renders as `<h2>` and a level 5 section as `<h6>`, and that a
//! Markdown `#`-style heading converts identically to the `=`-style marker.
//! Both rules are verified through `convert`. The introduction, the
//! section-level syntax prose, the nesting rules, and the
//! `include::example$section.adoc` build-time listings (which cannot be run
//! through `convert`) are tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/titles-and-levels.adoc");

// Title, the introductory description of what a section is, the "Section level
// syntax" heading and prose, and the admonition about section titles inside
// non-section blocks: descriptive framing with no distinct HTML rule to drive.
non_normative!(
    r#"
= Section Titles and Levels

Sections partition the document into a content hierarchy.
A section is an implicit enclosure.
Each section begins with a title and ends at the next sibling section, ancestor section, or end of document.
Nested section levels must be sequential.
A section can be a child of a document or another section, but it cannot be the child of any other block (i.e., you cannot put a section inside of a delimited block or list).

== Section level syntax

A section title marks the beginning of a section and also acts as the heading for that section.
The section title must be prefixed with a section marker, which indicates the section level.
The number of equal signs in the marker represents the section level using a 0-based index (e.g., two equal signs represents level 1).
A section marker can range from two to six equal signs and must be followed by a space.

IMPORTANT: The section title line is interpreted as paragraph text if it's found inside of a non-section block unless it marked as a xref:blocks:discrete-headings.adoc[discrete heading].

"#
);

// In the HTML output a section title is a heading tag whose number is one more
// than the section level, so levels 1 through 5 map to `<h2>` through `<h6>`.
// Driving the example page's `base` document confirms each mapping, including
// the id auto-generated from each title.
#[test]
fn heading_tag_is_one_more_than_section_level() {
    verifies!(
        r#"
In the HTML output, the section title is represented by a heading tag.
The number of the heading tag is one more than the section level (e.g., section level 1 becomes an h2 tag).
The section level ranges from 0-5.
This limit was established primarily due to the fact that HTML only provides heading tags from h1 to h6 (making level 5 the upper limit).

"#
    );

    let html = convert(
        "= Document Title (Level 0)\n\n== Level 1 Section Title\n\n=== Level 2 Section Title\n\n==== Level 3 Section Title\n\n===== Level 4 Section Title\n\n====== Level 5 Section Title\n\n== Another Level 1 Section Title",
    );

    assert!(
        html.contains(r#"<h2 id="_level_1_section_title">Level 1 Section Title</h2>"#),
        "{html}"
    );

    assert!(
        html.contains(r#"<h3 id="_level_2_section_title">Level 2 Section Title</h3>"#),
        "{html}"
    );

    assert!(
        html.contains(r#"<h4 id="_level_3_section_title">Level 3 Section Title</h4>"#),
        "{html}"
    );

    assert!(
        html.contains(r#"<h5 id="_level_4_section_title">Level 4 Section Title</h5>"#),
        "{html}"
    );

    assert!(
        html.contains(r#"<h6 id="_level_5_section_title">Level 5 Section Title</h6>"#),
        "{html}"
    );

    assert!(
        html.contains(
            r#"<h2 id="_another_level_1_section_title">Another Level 1 Section Title</h2>"#
        ),
        "{html}"
    );
}

// The `[source]` listing and the `====`-delimited "rendered as" block are
// build-time `include::example$section.adoc[tag=...]` directives, not literal
// source we can run through `convert`, so they are tracked non-normatively (the
// heading-tag rule they illustrate is verified above).
non_normative!(
    r#"
.Section titles available in an article doctype
[source]
----
include::example$section.adoc[tag=base]
----

The section titles are rendered as:

====
include::example$section.adoc[tag=b-base]
====

"#
);

// The section-nesting rules and the illustrative `include::example$` listings
// (the "illegal syntax" and preamble/content examples) are descriptive prose
// and build-time includes with no distinct HTML rule to drive here.
non_normative!(
    r#"
Section levels must be nested logically.
There are two rules you must follow:

. A document can only have multiple level 0 sections if the `doctype` is set to `book`.
 ** The first level 0 section is the document title; subsequent level 0 sections represent parts.
. Section levels cannot be skipped when nesting sections (e.g., you can't nest a level 5 section directly inside a level 3 section; an intermediary level 4 section is required).

For example, the following syntax is illegal:

[source]
----
include::example$section.adoc[tag=bad]
----

Content above the first section title is designated as the document's preamble.
Once the first section title is reached, content is associated with the section it is nested in.

[source]
----
include::example$section.adoc[tag=content]
----

"#
);

// Asciidoctor recognizes the Markdown `#` heading marker, so a Markdown outline
// converts identically to the `=` marker: `## Section` produces the same `<h2>`
// as `== Section`.
#[test]
fn markdown_heading_converts_identically() {
    verifies!(
        r#"
TIP: In addition to the equals sign marker used for defining section titles, Asciidoctor recognizes the hash symbol (`#`) from Markdown.
That means the outline of a Markdown document will be converted just fine as an AsciiDoc document.

"#
    );

    let html = convert("# Doc\n\n## Sect");

    assert!(html.contains(r#"<h2 id="_sect">Sect</h2>"#), "{html}");
}

// Section heading only.
non_normative!(
    r#"
== Titles as HTML headings

"#
);

// The closing rule restated for the `html5` backend: a level 1 section (`==`)
// maps to an `<h2>` element.
#[test]
fn level_1_section_maps_to_h2() {
    verifies!(
        r#"
When the document is converted to HTML 5 (using the built-in `html5` backend), each section title becomes a heading element where the heading level matches the number of equal signs.
For example, a level 1 section (`==`) maps to an `<h2>` element.
"#
    );

    let html = convert("= Document Title\n\n== Level 1 Section Title");

    assert!(
        html.contains(r#"<h2 id="_level_1_section_title">Level 1 Section Title</h2>"#),
        "{html}"
    );
}
