//! Coverage of the AsciiDoc language description's *Customize the TOC Title*
//! page.
//!
//! The `toc-title` attribute overrides the table-of-contents heading. Matching
//! Asciidoctor 2.0.26, this crate emits its value as the `<div id="toctitle">`
//! text. That behavior is verified through `convert`; the surrounding prose and
//! the screenshot are non-normative.
//!
//! The page draws its example document with `include::example$toc.adoc`
//! directives; the test reproduces the relevant tagged snippets inline.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/toc/pages/title.adoc");

// Renders a standalone document so the TOC and its title are present in the
// output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the introductory sentence naming the `toc-title`
// attribute. Descriptive prose.
non_normative!(
    r#"
= Customize the TOC Title

You can change the title of the table of contents with the `toc-title` attribute.

"#
);

// Setting `toc-title` in the header replaces the default _Table of Contents_
// heading: the `<div id="toctitle">` carries the assigned value (here _Table of
// Adventures_).
#[test]
fn toc_title_sets_the_toc_heading() {
    verifies!(
        r#"
== Set toc-title

To generate a TOC with a custom title, set the `toc-title` attribute in the header and assign it your preferred title.

.Define a custom TOC title
[source#ex-title]
----
include::example$toc.adoc[tags=header;title]
----
<.> The `toc` attribute must be set in order to use `toc-title`.
<.> The `toc-title` is set and assigned the value `Table of Adventures` in the document's header.

The result of <<ex-title>> is displayed below.

"#
    );

    // The `tags=header;title` snippets from `example$toc.adoc`, assembled: the
    // title and authors, then the `toc` and `toc-title` attribute entries, then
    // a section.
    let source = "\
= The Intrepid Chronicles
Kismet Lee; B. Steppenwolf; Pax Draeke
:toc:
:toc-title: Table of Adventures

== Certain Peril

Daylight trickles across the cobblestones...
";

    let output = convert(source);
    assert_xpath(
        &output,
        r#"//div[@id="toctitle"][text()="Table of Adventures"]"#,
        1,
    );
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::toc-title.png[Table of contents with a custom title,role=screenshot]
"#
);
