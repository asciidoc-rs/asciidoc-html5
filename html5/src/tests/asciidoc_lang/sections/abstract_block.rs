//! Coverage of the AsciiDoc language description's *Abstract (Block)* page.
//!
//! The `abstract` block style can be set on a paragraph or an open block. In
//! both cases this crate wraps the content in `<div class="quoteblock
//! abstract">` around a `<blockquote>` (with an optional `.Abstract` title
//! becoming `<div class="title">`), matching Asciidoctor 2.0.26. Both
//! `[source]` examples show literal "what you type" markup and are verified
//! through `convert`. The book-oriented TIP and the see-also xref are
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/abstract-block.adoc");

// Editorial REVIEW note carried in the source page; not a behavioral claim.
non_normative!(
    r#"
// REVIEW we may want to rethink where this page is located
"#
);

// The `abstract` block style set on a paragraph (optionally with a title)
// renders as `<div class="quoteblock abstract">` wrapping a `<blockquote>`; the
// block title `.Abstract` becomes `<div class="title">Abstract</div>`. The
// `[source]` listing is literal markup, driven here through `convert`.
#[test]
fn abstract_style_on_a_paragraph() {
    verifies!(
        r#"
= Abstract (Block)

An abstract is a concise overview of a document.
The `abstract` block style can be placed on an open block or paragraph.
Here's an example of the abstract block style set on a paragraph:

[source]
----
= Document Title

[abstract]
.Abstract
Documentation is a distillation of many long adventures.

== First Section
----

"#
    );

    let html = convert(
        "= Document Title\n\n[abstract]\n.Abstract\nDocumentation is a distillation of many long adventures.\n\n== First Section",
    );

    assert_css(&html, "div.quoteblock.abstract", 1);
    assert!(html.contains(r#"<div class="quoteblock abstract">"#));
    assert!(html.contains(r#"<div class="title">Abstract</div>"#));
    assert_xpath(
        &html,
        r#"//div[@class="quoteblock abstract"]/blockquote"#,
        1,
    );
    assert!(html.contains("Documentation is a distillation of many long adventures."));
}

// The `abstract` block style set on an open block needs neither a title nor a
// following section to terminate it; it renders as the same `<div
// class="quoteblock abstract">` wrapping a `<blockquote>` that contains the
// open block's child paragraphs. The `[source]` listing is literal markup,
// driven here through `convert`.
#[test]
fn abstract_style_on_an_open_block() {
    verifies!(
        r#"
The abstract block style does not require the open block or paragraph to have a title, and it does not depend on a subsequent section to terminate it.

[source]
----
= Document Title

[abstract]
--
This article will take you on a wonderful adventure of knowledge.

You'll start with the basics.
Beyond that, where you go is up to you.
--

Your journey begins here.
----

"#
    );

    let html = convert(
        "= Document Title\n\n[abstract]\n--\nThis article will take you on a wonderful adventure of knowledge.\n\nYou'll start with the basics.\nBeyond that, where you go is up to you.\n--\n\nYour journey begins here.",
    );

    assert_css(&html, "div.quoteblock.abstract", 1);
    assert_xpath(
        &html,
        r#"//div[@class="quoteblock abstract"]/blockquote/div[@class="paragraph"]"#,
        2,
    );
    assert!(html.contains("This article will take you on a wonderful adventure of knowledge."));
    assert!(html.contains("<p>Your journey begins here.</p>"));
}

// The TIP describes wrapping a quote at the start of a book chapter, which is
// book-doctype behavior out of scope for 1.0 (article is the only structural
// doctype modeled – see issue #188). The final line is a see-also xref to the
// abstract section style page. Both are non-normative.
non_normative!(
    r#"
TIP: To include a quote at the beginning of a chapter in a book, wrap a quote block inside an abstract block.

There's also an xref:abstract.adoc[abstract section style].
"#
);
