//! Coverage of the AsciiDoc language description's *Section Titles and Levels*
//! page.
//!
//! The page's normative HTML rule is that a section title becomes a heading tag
//! whose number is one greater than the section level, so a level 1 section
//! (`==`) renders as `<h2>` and a level 5 section as `<h6>`, and that a
//! Markdown `#`-style heading converts identically to the `=`-style marker.
//! Both rules are verified through `convert`. The page's example documents
//! (`base`, `b-base`, and `content`, shown on the page via `include::`
//! directives) are also driven directly, since their content is ordinary
//! article source. The introduction, the section-level syntax prose, the
//! nesting rules, and the level-skipping anti-example are tracked as
//! non-normative.

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

/// The page's `base` example (section titles spanning levels 0–5) and its
/// `b-base` "rendered as" counterpart (the same titles as standalone `[float]`
/// headings): each level maps to its heading tag, and the float form keeps that
/// tag with `class="float"`.
#[test]
fn section_titles_example_renders_as_headings() {
    verifies!(
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

    // The `base` tag: a document whose section titles span levels 0 through 5.
    let base = convert(
        "= Document Title (Level 0)\n\n== Level 1 Section Title\n\n=== Level 2 Section Title\n\n==== Level 3 Section Title\n\n===== Level 4 Section Title\n\n====== Level 5 Section Title\n\n== Another Level 1 Section Title",
    );

    assert!(
        base.contains(r#"<h2 id="_level_1_section_title">Level 1 Section Title</h2>"#),
        "{base}"
    );
    assert!(
        base.contains(r#"<h6 id="_level_5_section_title">Level 5 Section Title</h6>"#),
        "{base}"
    );

    // The `b-base` tag renders those same titles as standalone `[float]`
    // headings, so each keeps the heading tag for its level and gains
    // `class="float"`.
    let b_base = convert(
        "[float]\n= Document Title (Level 0)\n\n[float]\n== Level 1 Section Title\n\n[float]\n=== Level 2 Section Title\n\n[float]\n==== Level 3 Section Title\n\n[float]\n===== Level 4 Section Title\n\n[float]\n====== Level 5 Section Title\n\n[float]\n== Another Level 1 Section Title",
    );

    assert!(
        b_base.contains(
            r#"<h1 id="_document_title_level_0" class="float">Document Title (Level 0)</h1>"#
        ),
        "{b_base}"
    );
    assert!(
        b_base.contains(
            r#"<h6 id="_level_5_section_title" class="float">Level 5 Section Title</h6>"#
        ),
        "{b_base}"
    );
}

// The section-nesting rules are descriptive framing for the anti-example that
// follows.
non_normative!(
    r#"
Section levels must be nested logically.
There are two rules you must follow:

. A document can only have multiple level 0 sections if the `doctype` is set to `book`.
 ** The first level 0 section is the document title; subsequent level 0 sections represent parts.
. Section levels cannot be skipped when nesting sections (e.g., you can't nest a level 5 section directly inside a level 3 section; an intermediary level 4 section is required).

"#
);

/// The page's `bad` anti-example (a second level-0 section and a level-skipping
/// nested section): the renderer handles this rule-violating input exactly as
/// Asciidoctor does, demoting the extra `=` to `<h1 class="sect0">` and
/// rendering the skipped `====` as a `sect3`/`<h4>` (with no intervening
/// `sect2`).
#[test]
fn illegal_nesting_matches_asciidoctor() {
    verifies!(
        r#"
For example, the following syntax is illegal:

[source]
----
include::example$section.adoc[tag=bad]
----

"#
    );

    let html = convert(
        "= Document Title\n\n= Illegal Level 0 Section (violates rule #1)\n\n== First Section\n\n==== Illegal Nested Section (violates rule #2)",
    );

    // The second level-0 title is demoted to a standalone `sect0` heading.
    assert!(
        html.contains(
            r#"<h1 id="_illegal_level_0_section_violates_rule_1" class="sect0">Illegal Level 0 Section (violates rule #1)</h1>"#
        ),
        "{html}"
    );

    // The level-skipping nested section still renders, as a `sect3`/`<h4>`.
    assert!(
        html.contains(
            "<div class=\"sect3\">\n<h4 id=\"_illegal_nested_section_violates_rule_2\">Illegal Nested Section (violates rule #2)</h4>"
        ),
        "{html}"
    );
}

/// Content before the first section title is the document's preamble, and
/// content after a section title is nested inside that section — verified by
/// driving the page's `content` example (and a preamble variant).
#[test]
fn content_is_associated_with_its_section() {
    verifies!(
        r#"
Content above the first section title is designated as the document's preamble.
Once the first section title is reached, content is associated with the section it is nested in.

[source]
----
include::example$section.adoc[tag=content]
----

"#
    );

    // The `content` tag: each paragraph lands inside the section it follows.
    let html = convert(
        "== First Section\n\nContent of first section\n\n=== Nested Section\n\nContent of nested section\n\n== Second Section\n\nContent of second section",
    );

    assert!(
        html.contains(
            "<h2 id=\"_first_section\">First Section</h2>\n<div class=\"sectionbody\">\n\
             <div class=\"paragraph\">\n<p>Content of first section</p>"
        ),
        "{html}"
    );
    assert!(
        html.contains(
            "<h3 id=\"_nested_section\">Nested Section</h3>\n\
             <div class=\"paragraph\">\n<p>Content of nested section</p>"
        ),
        "{html}"
    );

    // Content above the first section title is instead placed in the preamble.
    let with_preamble = convert("= Doc\n\nPreamble paragraph.\n\n== First Section\n\nBody.");

    assert!(
        with_preamble.contains(r#"<div id="preamble">"#),
        "{with_preamble}"
    );
}

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
