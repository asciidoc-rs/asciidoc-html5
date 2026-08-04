//! Coverage of the AsciiDoc language description's *Appendix* page.
//!
//! The page describes the `appendix` section style. In an article an
//! `[appendix]` level-1 section renders lettered and captioned even without
//! `sectnums` (for example, `Appendix A: First Appendix`), the label comes from
//! the `appendix-caption` attribute (which can be changed or unset), and those
//! behaviors are verified here through `convert`. The page's article example is
//! also driven directly (the page shows it via an `include::` directive): its
//! appendices are lettered and captioned, its regular sections numbered, and
//! its generated table of contents matches the documented illustration. The
//! multi-part book examples require the book doctype, so they are tracked
//! non-normatively.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/appendix.adoc");

// Page title only.
non_normative!(
    r#"
= Appendix

"#
);

// Descriptive introduction to the appendix section style.
non_normative!(
    r#"
The `appendix` section style can be used in books and articles, and it can have subsections.
While the AsciiDoc structure allows appendices to be placed anywhere, it's customary to place them near the end of the document.

"#
);

// Section heading only.
non_normative!(
    r#"
== Appendix section syntax

"#
);

/// In an article an `[appendix]` level-1 section renders lettered and captioned
/// even without `sectnums`; a second appendix advances the letter to `B`.
#[test]
fn appendix_is_lettered_and_captioned() {
    verifies!(
        r#"
For articles, the appendix must be defined as a level 1 section (`==`).
"#
    );

    let html = convert("[appendix]\n== First Appendix\n\n[appendix]\n== Second Appendix");

    assert!(html.contains(r#"<h2 id="_first_appendix">Appendix A: First Appendix</h2>"#));
    assert!(html.contains(r#"<h2 id="_second_appendix">Appendix B: Second Appendix</h2>"#));
}

/// The article example the page includes: every appendix is lettered and
/// captioned (with the `appendix-caption` override `Exhibit`) while the regular
/// sections are numbered, and the generated table of contents lists exactly the
/// entries shown in the `appx-article-out` illustration.
#[test]
fn appendix_example_letters_captions_and_lists_in_the_toc() {
    verifies!(
        r#"
For example:

[source]
----
include::example$appendix.adoc[tag=appx-article]
----

The table of contents will appear as follows:

----
include::example$appendix.adoc[tag=appx-article-out]
----

"#
    );

    // The `appx-article` tag of `examples/appendix.adoc`, run through the
    // renderer directly (the page shows it via an `include::` directive).
    let html = convert(
        "= Article Title\n:appendix-caption: Exhibit\n:sectnums:\n:toc:\n\n== Section\n\n=== Subsection\n\n[appendix]\n== First Appendix\n\n=== First Subsection\n\n=== Second Subsection\n\n[appendix]\n== Second Appendix",
    );

    assert_css(&html, "div#toc.toc", 1);

    // Each line of the `appx-article-out` table-of-contents illustration: the
    // regular sections are numbered, the appendices are lettered and captioned,
    // and the appendix subsections carry the appendix letter.
    for entry in [
        "1. Section",
        "1.1. Subsection",
        "Exhibit A: First Appendix",
        "A.1. First Subsection",
        "A.2. Second Subsection",
        "Exhibit B: Second Appendix",
    ] {
        assert!(html.contains(entry), "missing {entry:?} in {html}");
    }
}

// The book (and multi-part book) appendix examples require the book doctype,
// which is out of scope for 1.0 (article is the only structural doctype modeled
// – see issue #188), so this is tracked non-normatively.
non_normative!(
    r#"
For books, the appendix must be defined as a level 1 section (`==`) if you want the appendix to be a adjacent to the chapters.
In a multi-part book, if you want the appendix to be adjacent to other parts, the appendix must be defined as a level 0 section (`=`).
In either case, the first subsection of the appendix must be a level 2 section (`===`).

The following example shows how to define an appendix for a multi-part book.

[source]
----
include::example$appendix.adoc[tag=appx-book]
----

The table of contents will appear as follows:

----
include::example$appendix.adoc[tag=appx-book-out]
----

"#
);

// Section heading only.
non_normative!(
    r#"
[#caption]
== Appendix label

"#
);

/// A rendered appendix title is built from a label (the `appendix-caption`
/// value), a letter, a colon, and the section title – for example,
/// `Appendix A: Data Access Matrix`.
#[test]
fn appendix_label_components() {
    verifies!(
        r#"
When rendered, the titles of sections marked as `appendix` will include:

* A label, taken from the value of the `appendix-caption` attribute, which defaults to "`Appendix`"
* A letter that represents the order of the appendix within the sequence of appendices (A, B, ...)
* A colon
* The section title

For example:

 Appendix A: Data Access Matrix

"#
    );

    let html = convert("[appendix]\n== Data Access Matrix");

    assert!(html.contains(r#"<h2 id="_data_access_matrix">Appendix A: Data Access Matrix</h2>"#));
}

/// Setting `:appendix-caption:` changes the label prefix, and unsetting it with
/// `:appendix-caption!:` removes the prefix, leaving just the letter (`A.`).
#[test]
fn appendix_caption_can_be_changed() {
    verifies!(
        r#"
The prefix can be modified by setting the `appendix-caption` attribute and overriding the default value with a custom value.

[source]
----
:appendix-caption: Exhibit
----

Unset the attribute to remove the prefix.

[source]
----
:appendix-caption!:
----
"#
    );

    let changed = convert(":appendix-caption: Exhibit\n\n[appendix]\n== First Appendix");

    assert!(changed.contains(r#"<h2 id="_first_appendix">Exhibit A: First Appendix</h2>"#));

    let unset = convert(":appendix-caption!:\n\n[appendix]\n== First Appendix");

    assert!(unset.contains(r#"<h2 id="_first_appendix">A. First Appendix</h2>"#));
}
