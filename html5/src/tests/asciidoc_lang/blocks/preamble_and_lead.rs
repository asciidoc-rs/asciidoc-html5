//! Coverage of the AsciiDoc language description's *Preamble and Lead Style*
//! page.
//!
//! The page documents the preamble (content between the header and the first
//! section, wrapped in `<div id="preamble">`) and the `lead` role, plus the way
//! any explicit role on the first preamble paragraph disables the automatic
//! lead styling. This crate emits the preamble wrapper and renders a role as a
//! paragraph class, so those structural claims are verified through `convert`.
//! The automatic lead styling itself is a stylesheet (CSS) effect — the HTML
//! carries no extra class, matching Asciidoctor — so the sentences describing
//! it, along with the screenshot images and section headings, are
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/preamble-and-lead.adoc");

non_normative!(
    r#"
= Preamble and Lead Style

[#preamble-style]
== Preamble

"#
);

// The preamble: content that sits between the document header and the first
// section title is wrapped in `<div id="preamble">` (with an inner
// `sectionbody`).
#[test]
fn preamble_wraps_content_before_the_first_section() {
    verifies!(
        r#"
Content between the end of the xref:document:header.adoc[document header] and the first section title in the document body is called the preamble.
A preamble is optional.

.Preamble
[#ex-preamble]
----
include::example$preamble.adoc[]
----

"#
    );

    let output = convert(
        "= Doc Title\n\nPreamble para one.\n\nPreamble para two.\n\n== First Section\n\nSection body.\n",
    );
    assert_css(&output, "div#preamble > div.sectionbody", 1);
    assert_css(&output, "div#preamble div.paragraph", 2);
}

// The automatic lead styling of the first preamble paragraph is applied by the
// stylesheet, not by an HTML class, so it is not structurally verifiable here;
// the screenshot image stands in for the rendered result.
non_normative!(
    r#"
When using the default Asciidoctor stylesheet, if the first paragraph does not have an explicit role, it is styled as if it has the <<lead-role,lead role>>.
The result of <<ex-preamble>> is displayed below.

image::preamble.png[The preamble of a document,role=screenshot]

[#lead-role]
== Lead role

"#
);

// The `lead` role: applied with the `[.lead]` shorthand, it becomes an extra
// class on the paragraph's wrapper div (`<div class="paragraph lead">`).
#[test]
fn lead_role_becomes_a_paragraph_class() {
    verifies!(
        r#"
Apply the `lead` role to any paragraph, and it will render using a larger font size.
The lead role is assigned to the xref:attributes:role.adoc[role attribute].
You can set `role` using the classic or shorthand method.

.Setting role to lead using the shorthand syntax
[#ex-lead]
----
include::example$paragraph.adoc[tag=lead]
----

"#
    );

    let output = convert(
        "== Section title\n\nThis is a regular paragraph.\n\n\
         [.lead]\nThis is a paragraph styled as a lead paragraph.\n",
    );
    assert_css(&output, "div.paragraph.lead", 1);
}

// The screenshot image standing in for the rendered lead paragraph.
non_normative!(
    r#"
The result of <<ex-lead>> is displayed below.

image::lead.png[Paragraph styled with the lead role value,role=screenshot]

"#
);

// Disabling the automatic lead styling: assigning any role to the first
// preamble paragraph adds that role as a class, which is the mechanism the
// stylesheet keys off of to leave the paragraph unstyled. Here the `[.normal]`
// shorthand yields `<div class="paragraph normal">`.
#[test]
fn assigning_a_role_disables_the_automatic_lead_styling() {
    verifies!(
        r#"
When you convert a document to HTML using the default stylesheet, the first paragraph of the <<preamble-style,preamble>> is automatically styled as a lead paragraph.
To disable this behavior, assign any role to the first paragraph.

.Disabling the automatic lead paragraph styling
----
include::example$paragraph.adoc[tag=no-lead]
----

The presence of the custom role (`normal`) informs the CSS not to style it as a lead paragraph.
"#
    );

    let output = convert(
        "[.normal]\nThis is a normal paragraph, regardless of its position in the document.\n",
    );
    assert_css(&output, "div.paragraph.normal", 1);
    assert_css(&output, "div.paragraph.lead", 0);
}
