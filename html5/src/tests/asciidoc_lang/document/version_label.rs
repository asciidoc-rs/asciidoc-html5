//! Coverage of the AsciiDoc language description's *Version Label Attribute*
//! page.
//!
//! The `version-label` attribute controls the label the byline prints before
//! the revision number. Matching Asciidoctor 2.0.26, this crate emits the
//! lowercased label followed by the revision number in the `<span
//! id="revnumber">`; unsetting `version-label` drops the label, leaving just
//! the (space-prefixed) number. Both behaviors are verified through `convert`;
//! the screenshots are non-normative.
//!
//! The page draws its examples with `include::example$version-label.adoc`
//! directives; the tests reproduce the referenced tagged snippets inline.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/version-label.adoc");

// Renders a standalone document so the header byline (and its `revnumber` span)
// is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the introductory sentence describing the attribute.
non_normative!(
    r#"
= Version Label Attribute

The `version-label` attribute controls the version label displayed before the revision number in the byline.

"#
);

// Assigning `version-label` a new value replaces the default label in the
// byline: the revnumber span prints the *lowercased* label (here `edition`)
// before the number. The `v` prefix on the revision line is still stripped.
#[test]
fn version_label_sets_the_byline_label() {
    verifies!(
        r#"
[#set]
== Change the version label in the byline

By default, `version-label` is assigned the value `Version`.
This label can be changed by setting `version-label` and assigning it a new value in the document header.

.Assign a new label to version-label
[source#ex-label]
----
include::example$version-label.adoc[tag=set]
----

The result of <<ex-label>> is displayed below.

"#
    );

    // The `tag=set` snippet from `example$version-label.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee
v3: An icy winter incarnation
:version-label: Edition

Body.
";

    let output = convert(source);
    assert_xpath(&output, r#"//span[@id="revnumber"][text()="edition 3"]"#, 1);
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::set-version-label.png["Byline with custom version label",role=screenshot]

Notice that when `revnumber` is implicitly set using the revision line, any preceding letters are still removed even though `version-label` is explicitly assigned a value.

"#
);

// Unsetting `version-label` removes the label entirely: the revnumber span
// prints only the number (with the leading space where the empty label would
// go).
#[test]
fn unset_version_label_drops_the_byline_label() {
    verifies!(
        r#"
[#unset]
== Unset the version label

You can remove the default version label from the byline by unsetting the `version-label` attribute.
In an attribute entry, add a bang (`!`) to the attribute's name.

.Unset version-label
[source#ex-unset-label]
----
include::example$version-label.adoc[tag=unset]
----

The result of <<ex-unset-label>> is displayed below.

"#
    );

    // The `tag=unset` snippet from `example$version-label.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet Lee
v3: An icy winter incarnation
:!version-label:

Body.
";

    let output = convert(source);
    assert_xpath(&output, r#"//span[@id="revnumber"][text()=" 3"]"#, 1);
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::unset-version-label.png["Byline without a version label",role=screenshot]
"#
);
