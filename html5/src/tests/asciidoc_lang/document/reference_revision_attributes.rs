//! Coverage of the AsciiDoc language description's *Reference the Revision
//! Attributes* page.
//!
//! The revision attributes can be referenced in the body. Matching Asciidoctor
//! 2.0.26, this crate drops any characters preceding the number when
//! `revnumber` is set via the revision line (in both the byline and body
//! references), but keeps the entire value — and the default version label —
//! when `revnumber` is set with an attribute entry. Those behaviors are
//! verified through `convert`; the surrounding prose and the screenshots are
//! non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/reference-revision-attributes.adoc");

// Renders a standalone document so both the byline and the referenced values in
// the body appear in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the section heading. Descriptive
// prose.
non_normative!(
    r#"
= Reference the Revision Attributes

You can reference the revision information attributes in your document regardless of whether they're set via the revision line or attribute entries.

[#reference-revnumber]
== Reference revnumber

"#
);

// A `revnumber` set on the revision line has its `v` prefix dropped both in the
// byline and where `{revnumber}` is referenced in the body.
#[test]
fn revision_line_revnumber_drops_its_prefix_everywhere() {
    verifies!(
        r#"
Remember, when `revnumber` is assigned via the revision line, any characters preceding the version number are dropped.
For instance, the revision number in <<ex-line-references>> is prefixed with a _v_.

.Revision line and revision attribute references
[source#ex-line-references]
----
include::example$reference-revision-line.adoc[]
----

The result of <<ex-line-references>> below shows that the _v_ in the version number has been removed when it's rendered in the byline and referenced in the document.

"#
    );

    // The full contents of `example$reference-revision-line.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee
v8.3, July 29, 2025: Summertime!

== Colophon

[%hardbreaks]
Revision number: {revnumber}
Revision date: {revdate}
Revision notes: {revremark}
";

    let output = convert(source);
    // Byline: the `v` is stripped from the number.
    assert!(output.contains(r#"<span id="revnumber">version 8.3,</span>"#));
    // Body reference: the `v` is likewise stripped.
    assert!(output.contains("Revision number: 8.3"));
}

// The screenshot. Nothing to render.
non_normative!(
    r#"
image::reference-revision-line.png["Revision line and rendered revision references to revnumber, revdate and revremark",role=screenshot]

"#
);

// A `revnumber` set with an attribute entry keeps its entire value — including
// the `v` prefix — both in the byline (with the default `Version` label) and
// where `{revnumber}` is referenced in the body.
#[test]
fn attribute_entry_revnumber_keeps_its_full_value() {
    verifies!(
        r#"
To display the entire value of `revnumber` when it's referenced in the document, you must set and assign it a value using an attribute entry.

.Revision attribute entries and references
[source#ex-references]
----
include::example$reference-revision-attributes.adoc[]
----

The entire value of the `revnumber` from <<ex-references>> is displayed in the byline, including the default `version-label` value _Version_.
When referenced in the document, the entire value of `revnumber` is displayed because it was set with an attribute entry.

"#
    );

    // The full contents of `example$reference-revision-attributes.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee
:revnumber: v8.3
:revdate: July 29, 2025
:revremark: Summertime!

== Colophon

[%hardbreaks]
Revision number: {revnumber}
Revision date: {revdate}
Revision notes: {revremark}
";

    let output = convert(source);
    // Byline: the full `v8.3` value, prefixed by the default `version` label.
    assert!(output.contains(r#"<span id="revnumber">version v8.3,</span>"#));
    // Body reference: the full `v8.3` value.
    assert!(output.contains("Revision number: v8.3"));
}

// The screenshot and the pointer to unsetting `version-label`. Descriptive
// prose.
non_normative!(
    r#"
image::reference-revision-attributes.png["Revision attribute entries and rendered revision references to revnumber, revdate and revremark",role=screenshot]

If you don't want the default version label to be displayed in the byline, xref:version-label.adoc#unset[unset the version-label attribute].
"#
);
