//! Coverage of the AsciiDoc language description's *Section Numbers* page.
//!
//! Setting the `sectnums` attribute prefixes level 1 through level 3 section
//! titles with arabic numbers (`1.`, `1.1.`, …). The attribute is a flexible
//! attribute, so it can be toggled off (`sectnums!`) and back on midstream, and
//! numbering does not increment across the regions where it is off. The
//! `sectnumlevels` attribute limits how deep the numbering goes. This crate
//! matches Asciidoctor 2.0.26 for the article doctype; the CLI/API
//! order-of-precedence prose and the book-doctype notes are tracked
//! non-normatively.
//!
//! The page draws two of its examples with `include::example$section.adoc`
//! directives; the tests reproduce the referenced tagged snippets inline and
//! drive them through `convert`.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/numbers.adoc");

// The page title and the first section heading are structural, not behavioral.
non_normative!(
    r#"
= Section Numbers
// New page, content taken from sections.adoc

== Turn on section numbers

"#
);

// Sections are unnumbered by default; setting `sectnums` in the header prefixes
// level 1 (`==`) through level 3 (`====`) titles with arabic numbers in the
// form `1.`, `1.1.`, etc. The `[source]` listing shows the header entry you
// type to enable it.
#[test]
fn sectnums_prefixes_titles_with_arabic_numbers() {
    verifies!(
        r#"
Sections aren't numbered by default.
However, you can enable this feature by setting the attribute `sectnums`.

[source]
----
= Title
:sectnums:
----

When `sectnums` is set, level 1 (`==`) through level 3 (`====`) section titles are prefixed with arabic numbers in the form of _1._, _1.1._, etc.
"#
    );

    let html = convert("= D\n:sectnums:\n\n== First\n\n=== Nested\n\n== Second");

    assert!(html.contains(">1. First<"));
    assert!(html.contains(">1.1. Nested<"));
    assert!(html.contains(">2. Second<"));
}

// This prose points at where `sectnums` can be set (header, CLI, API) and
// forward-references the per-section toggle and `sectnumlevels` features
// covered below; the CLI/API surfaces are not exercised through `convert`.
non_normative!(
    r#"
Section numbers can be set and unset via the document header, CLI, and API.
Once you've set `sectnums`, you can reduce or increase the section levels that get numbered in the whole document with the <<numlevels,sectnumlevels attribute>>.
You can also control whether a section is numbered on a section by section basis.

"#
);

// The subsection heading is structural, not behavioral.
non_normative!(
    r#"
=== Toggle section numbers on or off per section

"#
);

// `sectnums` is a flexible attribute: unsetting it (`sectnums!`) above a
// section stops numbering, and setting it again resumes numbering midstream.
// Driving the `num-off` example confirms the first section is numbered `1.`,
// the sections in the off region carry no number, and numbering resumes
// afterward.
#[test]
fn sectnums_can_be_toggled_per_section() {
    verifies!(
        r#"
The `sectnums` attribute is a unique attribute.
It's a [.term]*flexible attribute*, which means it can be set and unset midstream in a document, even if it is enabled through the API or CLI.
This allows you to toggle numbering on and off throughout a document.

//You can also use `sectnums` above any section title in the document to toggle the auto-numbering setting.
To turn off numbering for one or more sections, insert the attribute above the section where you want numbering to cease and unset it by adding an exclamation point to the end of its name.
To turn section numbering back on midstream, reset the attribute above the section where numbering should resume.

"#
    );

    let html = convert(NUM_OFF);

    assert!(html.contains(">1. Numbered Section<"));
    assert!(html.contains(">Unnumbered Section<"));
    assert!(html.contains(">2. Numbered Section<"));
}

// The `num-off` listing is a build-time `include::` of the example file, and
// the `num-out` `....` block is a literal illustration of the resulting
// numbering; neither is convertible source. The non-increment rule they
// illustrate is verified below.
non_normative!(
    r#"
[source]
----
include::example$section.adoc[tag=num-off]
----

For regions of the document where section numbering is turned off, the section numbering will not be incremented.
Given the above example, the sections will be numbered as follows:

....
include::example$section.adoc[tag=num-out]
....

"#
);

// Within an off region the section number is not incremented, so after
// numbering resumes the next numbered section is `2.` (not `4.`), even though
// three sections appeared while numbering was off.
#[test]
fn number_does_not_increment_in_off_regions() {
    verifies!(
        r#"
The section number does not increment in regions of the document where section numbers are turned off.

"#
    );

    let html = convert(NUM_OFF);

    assert!(html.contains(">2. Numbered Section<"));
    assert!(!html.contains(">4. Numbered Section<"));
}

// The order-of-precedence rules concern how a CLI/API `sectnums` value
// interacts with the header value; CLI/API precedence is not exercised through
// `convert`.
non_normative!(
    r#"
=== sectnums order of precedence

If `sectnums` is set on the command line or API, it overrides the value set in the document header, but it does not prevent the document from toggling the value for regions of the document.

If it is unset (`sectnums!`) on the command line or API, then the numbers are disabled regardless of the setting within the document.

"#
);

// The anchor and section heading are structural, not behavioral.
non_normative!(
    r#"
[#numlevels]
== Specify the section levels that are numbered

"#
);

// With `sectnums` set, levels 1 through 3 are numbered by default; setting
// `sectnumlevels` raises or lowers that limit. Driving `sectnumlevels: 2`
// confirms a level-2 title is still numbered (`1.1.`).
#[test]
fn sectnumlevels_sets_the_numbered_level_limit() {
    verifies!(
        r#"
When `sectnums` is set, level 1 (`==`) through level 3 (`====`) section titles are numbered by default.
You can increase or reduce the section level limit by setting the `sectnumlevels` attribute and assigning it the section level you want it to number.
The `sectnumlevels` attribute accepts a value of 0 through 5, and it can only be set in the document header.

"#
    );

    let html = convert("= D\n:sectnums:\n:sectnumlevels: 2\n\n== One\n\n=== Two\n\n==== Three");

    assert!(html.contains(">1.1. Two<"));
}

// The `sectnuml` listing is a build-time `include::` of the example file, not
// convertible source; the level-limit rule it introduces is verified below.
non_normative!(
    r#"
[source]
----
include::example$section.adoc[tag=sectnuml]
----
"#
);

// With `sectnumlevels` set to `2`, level 3 through 5 section titles are not
// numbered: a level-3 title renders with no numeric prefix.
#[test]
fn sectnumlevels_two_leaves_deeper_titles_unnumbered() {
    verifies!(
        r#"
<.> When the `sectnumlevels` attribute is assigned a value of `2`, level 3 through 5 section titles are not numbered.
// (i.e., not prefixed with a number).

"#
    );

    let html = convert("= D\n:sectnums:\n:sectnumlevels: 2\n\n== One\n\n=== Two\n\n==== Three");

    assert!(html.contains("<h4 id=\"_three\">Three</h4>"));
}

// Book-doctype behavior (level 1 sections becoming chapters, the
// `sectnumlevels` 4→3-levels-per-chapter translation) and the multi-part
// `sectnumlevels -1` / `partnums` case are out of scope for 1.0 (article is the
// only structural doctype modeled – see issue #188), so this is tracked
// non-normatively.
non_normative!(
    r#"
When the `doctype` is `book`, level 1 sections become xref:chapters.adoc[chapters].
Therefore, a `sectnumlevels` of `4` translates to 3 levels of numbered sections inside each chapter.

Assigning `sectnumlevels` a value of `0` is effectively the same as disabling section numbering (`sectnums!`).
However, if your document is a xref:parts.adoc[multi-part book] with xref:part-numbers-and-labels.adoc#partnums[part numbering enabled], then you'd have to set `sectnumlevels` to `-1` to disable part numbering too (the equivalent of `partnums!`).
"#
);

// The `num-off` tagged example from `example$section.adoc`: numbering is
// enabled, turned off for a run of sections, then resumed.
const NUM_OFF: &str = "= Title
:sectnums:

== Numbered Section

:sectnums!:

== Unnumbered Section

== Unnumbered Section

=== Unnumbered Section

:sectnums:

== Numbered Section";
