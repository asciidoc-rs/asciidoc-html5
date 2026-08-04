//! Coverage of the AsciiDoc language description's *Glossary* page.
//!
//! The page describes the `glossary` style. A `[glossary]` level-1 section
//! renders as a plain section, and the `glossary` block style on a description
//! list produces a `<div class="dlist glossary">` wrapper; both are verified
//! here through `convert`. The level-0 whole-book variant is book-doctype
//! behavior and is tracked non-normatively.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/glossary.adoc");

// Page title only.
non_normative!(
    r#"
= Glossary

"#
);

// Descriptive introduction to the glossary style.
non_normative!(
    r#"
You can include a glossary in an article, book, and book part by setting the `glossary` style on a section.

"#
);

// Section heading only.
non_normative!(
    r#"
== Glossary section syntax

"#
);

/// A `[glossary]` level-1 section renders as a plain section (the `glossary`
/// style has no section-level HTML effect).
#[test]
fn glossary_section() {
    verifies!(
        r#"
The glossary section is defined as a level 1 section (`==`) when:

* the doctype is `article`
* the doctype is `book` and the book doesn't contain any parts
* the glossary is for a book part

[source]
----
[glossary]
== Terminology
----

"#
    );

    let html = convert("[glossary]\n== Terminology");

    assert!(html.contains(r#"<h2 id="_terminology">Terminology</h2>"#));
}

// Book-doctype behavior (parts and the whole-book level-0 special section) is
// out of scope for 1.0 (article is the only structural doctype modeled – see
// issue #188), so this level-0 variant is tracked non-normatively.
non_normative!(
    r#"
If the book has parts, and the glossary is for the whole book, the section is defined as a level 0 section (`=`).

[source]
----
[glossary]
= Glossary
----

"#
);

// Section heading only.
non_normative!(
    r#"
== Glossary description list style syntax

"#
);

/// Applying the `glossary` block style to a description list produces a
/// `<div class="dlist glossary">` wrapper.
#[test]
fn glossary_description_list_style() {
    verifies!(
        r#"
In addition to setting `glossary` on the section, the block style `glossary` must be set on the description list in the section.

[source]
----
[glossary]
== Glossary

[glossary]
mud:: wet, cold dirt
rain::
	water falling from the sky
----
"#
    );

    let html = convert("[glossary]\nmud:: wet dirt\nrain:: water");

    assert_css(&html, "div.dlist.glossary", 1);
}
