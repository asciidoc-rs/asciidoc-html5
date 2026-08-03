//! Coverage of the AsciiDoc language description's *Position the TOC* page.
//!
//! The `toc` attribute's value chooses where the table of contents lands.
//! Matching Asciidoctor 2.0.26, this crate renders `left`/`right` as a fixed
//! side column (`body.toc2.toc-left`/`toc-right`, `div#toc.toc2`) in standalone
//! output, `preamble` immediately below the preamble, and `macro` at the
//! `toc::[]` block macro. In embeddable output the side column isn't available,
//! so the TOC is always centered (`div#toc.toc`). Those placements are verified
//! through `convert`; the screenshots, the responsive-breakpoint behavior, and
//! the environment/editor descriptions are non-normative.
//!
//! The page draws its example documents with `include::example$toc.adoc`
//! directives; the tests reproduce the relevant tagged snippets inline.

use crate::{
    convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/toc/pages/position.adoc");

// Renders a standalone document, so the `<body>` positional classes and the
// TOC placement are present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// Renders embeddable, body-only output – the mode the page's limitations
// section describes (`header_footer` false).
fn convert_embedded(source: &str) -> String {
    crate::convert(source)
}

// The `tag=header` snippet from `example$toc.adoc`: the document title and its
// three authors.
const HEADER: &str = "\
= The Intrepid Chronicles
Kismet Lee; B. Steppenwolf; Pax Draeke
";

// The `tag=body` snippet: four nested sections (levels 1 through 3).
const BODY: &str = "
== Certain Peril

Daylight trickles across the cobblestones...

=== A Recipe for Potion

We have to harvest the leaves by the light of the teal moons...

==== Searching for Ginseng

Crawling through the twisted understory...

== Dawn on the Plateau

Hanging from...
";

// The document title and the default-placement description, plus the list of
// built-in positional values. The default placement (below the header) is
// verified on the *Automatic Table of Contents* page; this is the value
// enumeration.
non_normative!(
    r#"
= Position the TOC

By default, the table of contents is inserted directly below the document title, author, and revision lines when the `toc` attribute is set and its value is left empty or set to `auto`.
This location can be changed by assigning one of the built-in positional values to the `toc` attribute.
The values are:

* `left`
* `right`
* `preamble`
* `macro`
* `auto` (default)

"#
);

// Assigning `left` or `right` positions the TOC as a fixed side column. In
// standalone output the `<body>` gains `toc2` and `toc-left`/`toc-right`
// classes, and the TOC div carries the `toc2` class (the side-column variant).
#[test]
fn left_or_right_positions_the_toc_as_a_side_column() {
    verifies!(
        r#"
[#side-column]
== Display the TOC as a side column

When converting to HTML, you can position the TOC to the left or right of the main content column by assigning the value `left` or `right` to the `toc` attribute, respectively.
The sidebar column containing the TOC is both fixed and scrollable.

.Assign the left value to toc
[source#ex-left]
----
include::example$toc.adoc[tag=header]
:toc: left
include::example$toc.adoc[tag=body]
----

The result of <<ex-left>> is displayed below.

"#
    );

    // `<<ex-left>>`: the header, `:toc: left`, then the body.
    let left = convert(&format!("{HEADER}:toc: left\n{BODY}"));
    assert_css(&left, r#"body.toc2.toc-left"#, 1);
    assert_css(&left, r#"div#toc.toc2"#, 1);

    // The symmetric `right` value shifts the side column to the other edge.
    let right = convert(&format!("{HEADER}:toc: right\n{BODY}"));
    assert_css(&right, r#"body.toc2.toc-right"#, 1);
    assert_css(&right, r#"div#toc.toc2"#, 1);
}

// The illustrative screenshot and the note that the side-column layout is a
// CSS/stylesheet feature. Descriptive prose.
non_normative!(
    r#"
image::toc-left.png[Display the table of contents as a side column,role=screenshot]

This positioning is achieved using CSS and depends on support from the stylesheet.

"#
);

// The responsive-breakpoint behavior: the side positions need a minimum screen
// width (~768px) and shift back to the center below it. This is runtime CSS
// behavior in the browser, not something the static HTML output expresses, so
// it is non-normative.
non_normative!(
    r#"
[#min-width-requirement]
WARNING: The side positions (left and right) have a width requirement.
These positions are only honored if there's sufficient room on the screen to fit the sidebar column (typically at least 768px).
If sufficient room available is not available (i.e., the screen width falls below the breakpoint), the TOC *automatically shifts back to the center*, appearing directly below the document title.

"#
);

// In embeddable output the TOC is always centered, whatever the `toc` value:
// converting `:toc: left` (or `right`) embedded yields a centered `div#toc.toc`
// rather than the side-column `toc2` variant.
#[test]
fn toc_is_centered_in_embeddable_output() {
    verifies!(
        r#"
NOTE: The TOC is always placed in the center in an embeddable HTML document, regardless of the value of the `toc` attribute.

"#
    );

    let left = convert_embedded(&format!("{HEADER}:toc: left\n{BODY}"));
    assert_css(&left, r#"div#toc.toc"#, 1);

    let right = convert_embedded(&format!("{HEADER}:toc: right\n{BODY}"));
    assert_css(&right, r#"div#toc.toc"#, 1);
}

// Assigning `preamble` places the TOC immediately below the preamble: the TOC
// div is emitted inside the `#preamble` block (after its content) rather than
// in the document header.
#[test]
fn preamble_positions_the_toc_below_the_preamble() {
    verifies!(
        r#"
[#beneath-preamble]
== Display the TOC beneath the preamble

When `toc` is assigned the built-in value `preamble`, the TOC is placed immediately below the xref:blocks:preamble-and-lead.adoc[preamble].

.Assign the preamble value to toc
[source#ex-preamble]
----
include::example$toc.adoc[tags=header;preamble]
----

The result of <<ex-preamble>> is displayed below.

"#
    );

    // `<<ex-preamble>>`: the header, `:toc: preamble`, a two-sentence preamble,
    // then two sections.
    let source = format!(
        "{HEADER}:toc: preamble
\nThis adventure begins on a frigid morning.
We've run out of coffee beans, but leaving our office means venturing into certain peril.

== Certain Peril

Daylight trickles across the cobblestones...

== Dawn on the Plateau

Hanging from...
"
    );

    let output = convert(&source);
    assert_xpath(&output, r#"//div[@id="preamble"]/div[@id="toc"]"#, 1);
    assert_xpath(&output, r#"//div[@id="header"]//div[@id="toc"]"#, 0);
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::toc-preamble.png[Display the table of contents below the preamble,role=screenshot]

"#
);

// The `preamble` placement is conditional: a document with no preamble produces
// no TOC at all. (The remediation – leave the value empty or use `auto` – is
// covered by the default placement verified on the *Automatic Table of
// Contents* page.)
#[test]
fn preamble_toc_is_absent_without_a_preamble() {
    verifies!(
        r#"
CAUTION: When using the `preamble` value, the TOC will not appear if your document does not have a preamble.
To fix this problem, set the `toc` attribute to an empty value (i.e., leave the value empty) or assign it the value `auto`.

"#
    );

    // No preamble: the first block is a section, so `:toc: preamble` renders no
    // TOC.
    let source = format!("{HEADER}:toc: preamble\n{BODY}");
    let output = convert(&source);
    assert_css(&output, r#"div#toc"#, 0);
}

// Assigning `macro` places the TOC wherever the `toc::[]` block macro appears –
// here inside the first section, after its title. The TOC macro is honored only
// when `toc` is set to `macro`: otherwise the macro is ignored entirely.
#[test]
fn macro_positions_the_toc_at_the_toc_macro() {
    verifies!(
        r#"
[#at-macro]
== Use the TOC macro to position the TOC

To place the TOC in specific location in the document, assign the `macro` value to the `toc` attribute.
Then, enter the table of contents block macro (i.e., TOC macro) on the line in your document where you want the TOC to appear.
The TOC macro should only be used once in a document.

If the `toc` document attribute isn't assigned the value `macro`, any TOC macro in the document will be ignored.

.Assign the macro value to toc
[source#ex-macro]
----
include::example$toc.adoc[tags=header;macro]
----
<.> The `toc` attribute must be set to `macro` to enable the use of the TOC macro.
<.> In this example, the TOC macro is placed below the first section's title, indicating that this is the location where the TOC will be displayed once the document is rendered.

The result of <<ex-macro>> is displayed below.

"#
    );

    // `<<ex-macro>>`: the header, `:toc: macro`, then a section whose body
    // leads with the `toc::[]` macro.
    let source = format!(
        "{HEADER}:toc: macro

== Certain Peril

toc::[]

Daylight trickles across the cobblestones...

== Dawn on the Plateau

Hanging from...
"
    );

    let output = convert(&source);

    // The TOC lands inside the first section's body (where `toc::[]` sits),
    // not in the header.
    assert_xpath(
        &output,
        r#"//div[@class="sect1"]/div[@class="sectionbody"]/div[@id="toc"]"#,
        1,
    );
    assert_xpath(&output, r#"//div[@id="header"]//div[@id="toc"]"#, 0);

    // Without `:toc: macro`, the same `toc::[]` macro is ignored: no TOC is
    // rendered.
    let ignored = convert(&format!(
        "{HEADER}
== Certain Peril

toc::[]

Daylight trickles across the cobblestones...
"
    ));
    assert_css(&ignored, r#"div#toc"#, 0);
}

// The illustrative screenshot. Nothing to render.
non_normative!(
    r#"
image::toc-macro.png[Display the table of contents using the TOC macro,role=screenshot]

"#
);

// The list of embeddable-HTML environments and the enumeration of the three
// values valid there. The valid-values constraint is descriptive; the concrete
// consequence (no side column) is verified below.
non_normative!(
    r#"
[#limitations]
== Embeddable HTML, editor and previewer limitations

When AsciiDoc is converted to embeddable HTML (i.e., the `header_footer` option is `false`), there are only three valid values for the `toc` attribute:

* `auto`
* `preamble`
* `macro`

All of the following environments convert AsciiDoc to embeddable HTML:

* the file viewer on GitHub and GitLab
* the AsciiDoc preview in an editor like Atom, Brackets or AsciidocFX
* the Asciidoctor browser extensions

"#
);

// The side-column placement isn't available in embeddable output: converting
// `:toc: left` embedded produces no `toc2` (side-column) markup at all.
#[test]
fn side_column_placement_is_unavailable_in_embeddable_output() {
    verifies!(
        r#"
IMPORTANT: The side column placement (left or right) isn't available in this mode.
That's because the embeddable HTML doesn't have the outer framing (or the CSS) necessary to support a side column TOC.
"#
    );

    let output = convert_embedded(&format!("{HEADER}:toc: left\n{BODY}"));
    assert_css(&output, r#"div.toc2"#, 0);
    assert_css(&output, r#"div#toc.toc"#, 1);
}
