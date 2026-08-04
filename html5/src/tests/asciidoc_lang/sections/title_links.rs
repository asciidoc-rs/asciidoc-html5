//! Coverage of the AsciiDoc language description's *Activate Section Title
//! Links* page.
//!
//! The page documents two document attributes: `sectlinks`, which turns each
//! section title into a self link, and `sectanchors`, which inserts an empty
//! anchor before each section title. Both rendering effects are verified
//! through `convert`; the sentences describing the default stylesheet's
//! appearance (link color, the floating section entity) are CSS presentation
//! rather than output structure and are tracked non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/title-links.adoc");

// Page title only.
non_normative!(
    r#"
= Activate Section Title Links

"#
);

// Section heading and its explicit ID carry no rule of their own.
non_normative!(
    r#"
[#link]
== Turn section titles into links

"#
);

// Enabling `sectlinks` wraps the section title text in an `<a class="link">`
// that links to the section's own ID.
#[test]
fn sectlinks_turns_title_into_link() {
    verifies!(
        r#"
To turn section titles into links, enable the `sectlinks` attribute.
"#
    );

    let html = convert("= D\n:sectlinks:\n\n== First");

    assert_css(&html, r##"h2#_first a.link[href="#_first"]"##, 1);
    assert!(html.contains(r##"<a class="link" href="#_first">First</a>"##));
}

// The linked-title styling is a default-stylesheet appearance, not part of the
// converted HTML structure.
non_normative!(
    r#"
The default Asciidoctor stylesheet displays linked section titles with the same color and styles as unlinked section titles.

"#
);

// Section heading and its explicit ID carry no rule of their own.
non_normative!(
    r#"
[#anchor]
== Add &#167; to section titles

"#
);

// Enabling `sectanchors` inserts an empty `<a class="anchor">` before the
// section title text, linking to the section's own ID.
#[test]
fn sectanchors_adds_anchor_before_title() {
    verifies!(
        r#"
When the `sectanchors` attribute is enabled on a document, an anchor (empty link) is added before the section title.
"#
    );

    let html = convert("= D\n:sectanchors:\n\n== First");

    assert_css(&html, r##"h2#_first a.anchor[href="#_first"]"##, 1);
    assert!(html.contains(r##"<a class="anchor" href="#_first"></a>First"##));
}

// The floating section-entity rendering of the anchor is a default-stylesheet
// appearance, not part of the converted HTML structure.
non_normative!(
    r#"
The default Asciidoctor stylesheet renders the anchor as a section entity (*&#167;*) that floats to the left of the section title.
"#
);
