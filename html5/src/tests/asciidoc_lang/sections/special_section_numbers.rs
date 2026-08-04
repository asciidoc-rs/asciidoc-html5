//! Coverage of the AsciiDoc language description's *Number Special Sections*
//! page.
//!
//! Special (styled) sections are not numbered by default, but setting
//! `sectnums` to the value `all` numbers them alongside regular sections.
//! Appendix titles are unaffected, keeping their caption-driven prefix
//! (`Appendix A:`). This crate matches Asciidoctor 2.0.26; the
//! book-parts/`partnums` note is tracked non-normatively.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/special-section-numbers.adoc");

// The page title is structural, not behavioral.
non_normative!(
    r#"
= Number Special Sections

"#
);

// Special sections (those carrying a built-in special style) are unnumbered by
// default; assigning `sectnums` the value `all` numbers regular sections and
// special sections alike. The `[source]` listing shows the header entry you
// type. Driving a `[glossary]` (a special section) followed by a regular
// section confirms both are numbered, `1.` and `2.`.
#[test]
fn sectnums_all_numbers_special_sections() {
    verifies!(
        r#"
Sections that are assigned a built-in special style aren't numbered by default.
To number regular sections as well as special sections, set `sectnums` and assign it a value of `all`.

[source]
----
= Title
:sectnums: all
----

"#
    );

    let html = convert("= T\n:sectnums: all\n\n[glossary]\n== Terms\n\n== Regular");

    assert!(html.contains(">1. Terms<"));
    assert!(html.contains(">2. Regular<"));
}

// Appendix numbering is not affected by `sectnums`: an appendix title keeps its
// caption-driven prefix (`Appendix A:`) even under `sectnums: all`, because the
// prefix is controlled by `appendix-caption` rather than the section number.
#[test]
fn appendix_prefix_unaffected_by_sectnums() {
    verifies!(
        r#"
The assignment of appendix numbers isn't affected by `sectnums` as their section title prefixes are controlled by the attribute xref:appendix.adoc#caption[appendix-caption].
"#
    );

    let html = convert("= T\n:sectnums: all\n\n[appendix]\n== Data");

    assert!(html.contains("Appendix A: Data"));
}

// Book parts and their `partnums` numbering are out of scope for 1.0 (article
// is the only structural doctype modeled – see issue #188), so this is tracked
// non-normatively.
non_normative!(
    r#"
Book parts aren't numbered by `sectnums` either, instead, they're xref:part-numbers-and-labels.adoc[controlled by `partnums`].
"#
);
