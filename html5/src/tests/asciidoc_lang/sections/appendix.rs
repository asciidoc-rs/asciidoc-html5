//! Coverage of the AsciiDoc language description's *Appendix* page.
//!
//! The page describes the `appendix` section style. In an article an
//! `[appendix]` level-1 section renders lettered and captioned even without
//! `sectnums` (for example, `Appendix A: First Appendix`), the label comes from
//! the `appendix-caption` attribute (which can be changed or unset), and those
//! behaviors are verified here through `convert`. The multi-part book examples,
//! the `include::` listings, and the rendered table-of-contents blocks are
//! build-time includes and/or book-doctype content, so they are tracked
//! non-normatively.

use crate::{convert, tests::sdd::*};

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

// The `include::` listing and the rendered article table-of-contents block are
// build-time includes, not literal source that this crate runs through
// `convert`.
non_normative!(
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

// Book-doctype behavior (parts, chapters, part/chapter numbering, book special
// sections) is out of scope for 1.0 (article is the only structural doctype
// modeled – see issue #188), and the listing plus rendered table of contents
// are build-time includes, so this is tracked non-normatively.
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
