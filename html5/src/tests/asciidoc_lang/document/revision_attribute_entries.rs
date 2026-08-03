//! Coverage of the AsciiDoc language description's *Assign Revision Attributes
//! with Attribute Entries* page.
//!
//! The revision fields can be set with `:revnumber:`, `:revdate:`, and
//! `:revremark:` attribute entries instead of a revision line. Matching
//! Asciidoctor 2.0.26, this crate renders them in the byline in the fixed
//! `revnumber`/`revdate`/`revremark` order (regardless of entry order), keeps
//! any non-numeric prefix on an attribute-set `revnumber`, resolves attribute
//! references in `revremark`, and drops the version label when `version-label`
//! is unset. Those behaviors are verified through `convert`; the surrounding
//! prose and the screenshot are non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/revision-attribute-entries.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the list of situations that call
// for explicit revision attributes. Descriptive prose.
non_normative!(
    r#"
= Assign Revision Attributes with Attribute Entries

The revision information attributes can be set and assigned values with attribute entries.

== When should I set revision attributes explicitly?

You should set the revision attributes explicitly when one of the following occurs:

* the document doesn't have an author line,
* the document doesn't have a revision number,
* you want the full value of the revision number--including any letter and symbol prefixes--to be displayed, or
* a revision attribute's value contains characters or elements that conflict with the revision line syntax.

== Set the revision attributes

"#
);

// The attribute entries populate the byline in the fixed
// `revnumber`/`revdate`/`revremark` order even though `revdate` is entered
// first; the non-numeric `LPR` prefix is kept (unlike on the revision line);
// `revremark` resolves the `{doctitle}` reference; and the unset
// `version-label` removes the label word, leaving just the space before the
// number.
#[test]
fn revision_attribute_entries_populate_the_byline() {
    verifies!(
        r#"
The attributes `revdate`, `revnumber` and `revremark` are set and assigned values in <<ex-entries>>.
The order of the attribute entries doesn't affect their order in the byline of a rendered document.

.Set the revision attributes in the document header
[source#ex-entries]
----
= The Intrepid Chronicles
:revdate: April 4, 2022
:revnumber: LPR55 <.>
:revremark: The spring incarnation of {doctitle} <.>
:version-label!: <.>
----
<.> Any non-numeric characters that precede the version number aren't dropped when `revnumber` is set using an attribute entry.
<.> The value of `revremark` can contain attribute references.
<.> The `version-label` attribute is unset so that the word _Version_ isn't displayed in the byline.

The result of <<ex-entries>> is displayed below.

"#
    );

    let source = "\
= The Intrepid Chronicles
:revdate: April 4, 2022
:revnumber: LPR55
:revremark: The spring incarnation of {doctitle}
:version-label!:

Body.
";

    let output = convert(source);
    // Byline order is revnumber, revdate, revremark regardless of entry order.
    // The label is gone (leading space only) and `LPR55` keeps its prefix.
    assert!(output.contains(r#"<span id="revnumber"> LPR55,</span>"#));
    assert!(output.contains(r#"<span id="revdate">April 4, 2022</span>"#));
    // `{doctitle}` resolved inside `revremark`.
    assert!(output.contains(
        r#"<br><span id="revremark">The spring incarnation of The Intrepid Chronicles</span>"#
    ));
}

// The screenshot and the closing note about the absent version label.
// Descriptive prose.
non_normative!(
    r#"
image::revision-attributes.png["Byline containing revision information from the explicitly set revnumber, revdate and revremark attributes",role=screenshot]

The word _Version_ is absent from the rendered document's byline because xref:version-label.adoc[the version-label attribute] was unset.
"#
);
