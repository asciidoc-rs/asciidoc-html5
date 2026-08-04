//! Coverage of the AsciiDoc language description's *Paragraph Alignment* page.
//!
//! The page documents the built-in `text-<alignment>` roles. This crate renders
//! such a role as an extra class on the paragraph's wrapper div, so the example
//! it shows is verified through `convert`. The title and the closing caveat
//! about other converters are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/paragraph-alignment.adoc");

non_normative!(
    r#"
= Paragraph Alignment

"#
);

// A `text-<alignment>` role on a paragraph — assigned with the `[.role]`
// shorthand — becomes an extra class on the paragraph's wrapper div, alongside
// the base `paragraph` class: `[.text-center]` yields `<div class="paragraph
// text-center">`.
#[test]
fn text_alignment_role_becomes_a_paragraph_class() {
    verifies!(
        r#"
AsciiDoc provides built-in roles to align the text of a paragraph.
The name of the role follows the pattern `text-<alignment>`, where `<alignment>` is one of left, center, right, or justify (e.g., `text-center`).

In <<ex-center>>, the paragraph is assigned the `text-center` role in an attribute list.

.Assign text-center to a paragraph
[#ex-center]
----
[.text-center]
This text is centered, so it must be important.
----

"#
    );

    let output = convert("[.text-center]\nThis text is centered, so it must be important.\n");
    assert_css(&output, "div.paragraph.text-center", 1);
    assert_css(&output, "div.paragraph.text-center > p", 1);
}

// The caveat that these roles may not be honored by every converter is an aside
// about output formats other than the HTML this crate produces.
non_normative!(
    r#"
These built-in text alignment roles may not be honored by all converters.
Though, you can expect them to be supported when the output format is either HTML or PDF.
"#
);
