//! Coverage of the AsciiDoc language description's *Abstract (Section)* page.
//!
//! The `abstract` section style is a book special section. In the `article`
//! document type this crate models (see issue #188), setting `[abstract]` on
//! the first section has no rendering effect: the section is emitted as an
//! ordinary `.sect1`/`<h2>` section, matching Asciidoctor 2.0.26. That plain
//! rendering is verified here. The `include::example$abstract.adoc[]` listing
//! is reproduced through its literal equivalent, and the see-also xref plus the
//! commented-out book-abstract block are tracked non-normatively.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/abstract.adoc");

// Page title and descriptive introduction; the abstract section style is a book
// concept, and this prose merely frames it.
non_normative!(
    r#"
= Abstract (Section)

An abstract is a concise overview of an article.
The abstract section style can be used in the `article` document type.

== Abstract section syntax

"#
);

// The `abstract` section style is set on the first section of the article. With
// no book-doctype structure to react to it, the section renders as an ordinary
// level-1 section: a `.sect1` wrapper with an `<h2 id="_abstract">` heading and
// its paragraph body, byte-for-byte matching Asciidoctor 2.0.26. The listing
// shows a build-time include of example$abstract.adoc; the test drives that
// example's literal content through `convert`.
#[test]
fn abstract_section_renders_as_a_plain_section() {
    verifies!(
        r#"
The `abstract` section style must be set on the first section of the article, as seen in the example below:

[source]
----
include::example$abstract.adoc[]
----

"#
    );

    let html = convert(
        "= Article Title\n\n[abstract]\n== Abstract\n\nDocumentation is a distillation of many long adventures.\n\n== Section Title",
    );

    assert_css(&html, "div.sect1", 2);
    assert_xpath(&html, r#"//div[@class="sect1"]/h2[@id="_abstract"]"#, 1);
    assert!(html.contains(r#"<h2 id="_abstract">Abstract</h2>"#));
    assert!(html.contains("<p>Documentation is a distillation of many long adventures.</p>"));
}

// The see-also xref points to the abstract block style page, and the commented
// `////` … `////` block describes the book-doctype abstract syntax (parts and
// chapters), which is out of scope for 1.0 (article is the only structural
// doctype modeled – see issue #188). Both are non-normative.
non_normative!(
    r#"
If you want to style a paragraph or an open block as an abstract, instead of a whole section, see the xref:abstract-block.adoc[abstract block style] documentation.

////
There's some sort of funkiness with book and abstract so we're putting this section on hold.

== Book abstract syntax

When the document type is book, the `abstract` section style must be placed on the first section _inside_ the chapter section.
The section must be one section level below the chapter, that is, the chapter is marked up as level 1 (`==`) so its abstract must be marked up as level 2 (`===`).
An abstract may not be used _before_ a part or chapter in a book.

[source]
----
= Book Title
:doctype: book

== Chapter Title

[abstract]
=== Summary

Documentation is a distillation of many long adventures.

=== Section
----
////
"#
);
