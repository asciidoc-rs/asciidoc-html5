//! Coverage of the AsciiDoc language description's *Revision Information* page.
//!
//! Revision information lives in three built-in attributes: `revnumber`,
//! `revdate`, and `revremark`. Matching Asciidoctor 2.0.26, this crate drops
//! any characters preceding the number of a revision-line `revnumber` (but
//! keeps an attribute-entry value verbatim), reads the date after the comma,
//! and reads the remark after the colon. Those rules are verified through
//! `convert`; the descriptive definitions are otherwise non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/revision-information.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the section heading. Descriptive
// prose.
non_normative!(
    r#"
= Revision Information

A document's revision information is assigned to three built-in attributes: `revnumber`, `revdate` and `revremark`.
These optional attributes can be set and assigned values xref:revision-line.adoc[using the revision line] or xref:revision-attribute-entries.adoc[using attribute entries] in a document header.

== Revision attributes

"#
);

// A revision-line `revnumber` requires a number, begins with `v` when not
// followed by a date or remark, and drops any preceding letters/symbols; an
// attribute-entry `revnumber` need not contain a number and is displayed whole.
#[test]
fn revnumber_rules() {
    verifies!(
        r#"
revnumber::
The document's revision number or version is assigned to the built-in `revnumber` attribute.
When assigned using the revision line, the version must contain at least one number, and, if it isn't followed by a date or remark, it must begin with the letter *v* (e.g., `v7.0.6`).
Any letters or symbols preceding the number, including *v*, are dropped when the document is rendered.
If `revnumber` is set with an attribute entry, it doesn't have to contain a number and the entire value is displayed in the rendered document.

"#
    );

    // Only a number on the revision line, so it must begin with `v`; the `v` is
    // then dropped in the byline.
    let output = convert("= T\nA\nv7.0.6\n\nBody.\n");
    assert!(output.contains(r#"<span id="revnumber">version 7.0.6</span>"#));

    // Letters preceding the number (here `LPR`) are dropped.
    let output = convert("= T\nA\nLPR55, 2020-10-10\n\nBody.\n");
    assert!(output.contains(r#"<span id="revnumber">version 55,</span>"#));

    // Set with an attribute entry, a non-numeric value is displayed whole.
    let output = convert("= T\nA\n:revnumber: draft\n\nBody.\n");
    assert!(output.contains(r#"<span id="revnumber">version draft</span>"#));
}

// On the revision line the date is separated from the version by a comma.
#[test]
fn revdate_rules() {
    verifies!(
        r#"
revdate::
The date the revision was completed is assigned to the built-in `revdate` attribute.
If the date is assigned using the revision line, it must be separated from the version by a comma (e.g., `78.1, 2020-10-10`).
The date can contain letters, numbers, symbols, and attribute references.
//From @graphitefriction: This statement isn't true according to my tests: "When the version or revision remarks are assigned using the revision line, but the revision date isn't, then `revdate` will be assigned the `docdate` value." Instead, `revdate` is left unset

"#
    );

    let output = convert("= T\nA\n78.1, 2020-10-10\n\nBody.\n");
    assert!(output.contains(r#"<span id="revdate">2020-10-10</span>"#));
}

// On the revision line the remark is separated from the version or date by a
// colon.
#[test]
fn revremark_rules() {
    verifies!(
        r#"
revremark::
Remarks about the revision of the document are assigned to the built-in `revremark` attribute.
The remark must be separated by a colon (`:`) from the version or revision date when assigned using the revision line.
"#
    );

    let output = convert("= T\nA\nv1.0: A new analysis\n\nBody.\n");
    assert!(output.contains(r#"<span id="revremark">A new analysis</span>"#));
}
