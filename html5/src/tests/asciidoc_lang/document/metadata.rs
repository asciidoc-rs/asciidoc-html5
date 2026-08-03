//! Coverage of the AsciiDoc language description's *Document Metadata* page.
//!
//! Document metadata attributes become `<meta>` elements in the HTML `<head>`.
//! Matching Asciidoctor 2.0.26, this crate emits `<meta name="description">`
//! from the `description` attribute (joining a backslash-continued value) and
//! `<meta name="keywords">` from the `keywords` attribute. Those are verified
//! through `convert`; the surrounding prose and the docinfo pointer are
//! non-normative.
//!
//! The page draws its examples with `include::example$metadata.adoc`
//! directives; the tests reproduce the referenced tagged snippets inline.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/metadata.adoc");

// Renders a standalone document so the `<head>` metadata is present in the
// output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the Description section heading.
// Descriptive prose.
non_normative!(
    r#"
= Document Metadata

Document metadata, such as a description of the document, keywords, and custom information, can be assigned to attributes in the header.
When converted to HTML, the values of these attributes will correspond to elements contained in the `<head>` section of an HTML document.

[#description]
== Description

"#
);

// The `description` attribute (whose value may be split across several
// backslash-continued lines) becomes a `<meta name="description">` element.
#[test]
fn description_becomes_a_meta_element() {
    verifies!(
        r#"
You can include a description of the document using the `description` attribute.

[source]
----
include::example$metadata.adoc[tag=desc-base]
----
<.> If the document's description is long, you can break the attribute's value across several lines by ending each line with a backslash `\` that is preceded by a space.

When converted to HTML, the document description value is assigned to the HTML `<meta>` element.

.HTML output
include::example$metadata.adoc[tag=desc-html]

"#
    );

    // The `tag=desc-base` snippet from `example$metadata.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee; Lazarus Draeke
:description: A story chronicling the inexplicable \\
hazards and unique challenges a team must vanquish \\
on their journey to finding an open source \\
project's true power.

This journey begins on a bleary Monday morning.
";

    let output = convert(source);
    assert!(output.contains(
        r#"<meta name="description" content="A story chronicling the inexplicable hazards and unique challenges a team must vanquish on their journey to finding an open source project's true power.">"#
    ));
}

// The attribute-continuation TODO comment and the Keywords section heading.
// Descriptive prose.
non_normative!(
    r#"
// TODO: Add attribute continuation documentation to attributes section

[#keywords]
== Keywords

"#
);

// The `keywords` attribute becomes a `<meta name="keywords">` element holding
// its comma-separated list.
#[test]
fn keywords_become_a_meta_element() {
    verifies!(
        r#"
The `keywords` attribute contains a list of comma separated values that are assigned to the HTML `<meta>` element.

[source]
----
include::example$metadata.adoc[tag=key-base]
----

.HTML output
include::example$metadata.adoc[tag=key-html]

"#
    );

    // The `tag=key-base` snippet from `example$metadata.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee; Lazarus Draeke
:keywords: team, obstacles, journey, victory

This journey begins on a bleary Monday morning.
";

    let output = convert(source);
    assert!(
        output.contains(r#"<meta name="keywords" content="team, obstacles, journey, victory">"#)
    );
}

// The custom-metadata section, which points to docinfo files. Descriptive
// prose.
non_normative!(
    r#"
== Custom metadata, styles, and functions

You can add content, such as custom metadata, stylesheet, and script information, to the header of the output document using docinfo (document information) files.
The xref:docinfo:index.adoc[docinfo file] section details what these files can contain and how to use them.
"#
);
