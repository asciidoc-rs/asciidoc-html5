//! Coverage of the AsciiDoc language description's *Highlight* page.
//!
//! Text enclosed in single or double hash symbols, with no role assigned,
//! renders as a `<mark>` element. This crate produces exactly that, so the
//! example and its HTML output are verified through `convert`; only the title,
//! byline comment, and the section heading are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/highlight.adoc");

non_normative!(
    r#"
= Highlight
//New page, content from text-css.adoc

== Highlight syntax

"#
);

// A pair of single (or double) hash symbols with no role renders the enclosed
// text as a `<mark>` element.
#[test]
fn highlight_syntax() {
    verifies!(
        r#"
When text is enclosed in a pair of single or double hash symbols (`#`), and no role is assigned to it, the text will be rendered as highlighted (aka marked) text for notation purposes.

.Highlighted style syntax
[#ex-highlight]
----
include::example$text.adoc[tag=highlight]
----

When <<ex-highlight>> is converted to HTML, it translates into the following output.

.Highlighted text HTML output
[,html]
----
include::example$text.adoc[tag=highlight-html]
----

The result of <<ex-highlight>> is rendered below.

====
include::example$text.adoc[tag=highlight]
====
"#
    );

    // The single-hash (constrained) form from tag=highlight.
    let single = convert("Mark my words, #automation is essential#.\n");
    assert!(single.contains("Mark my words, <mark>automation is essential</mark>."));
    assert_css(&single, "mark", 1);

    // The double-hash (unconstrained) form the page also mentions.
    let double = convert("##Mark##up refers to text that contains formatting ##mark##s.\n");
    assert!(double.contains(
        "<mark>Mark</mark>up refers to text that contains formatting <mark>mark</mark>s."
    ));
    assert_css(&double, "mark", 2);
}
