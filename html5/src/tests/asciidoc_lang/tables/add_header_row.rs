//! Coverage of the AsciiDoc language description's *Create a Header Row* page.
//!
//! The page teaches how the first row of a table becomes a header row: the
//! `header` value can be assigned to the `options` attribute explicitly (using
//! either the shorthand `%header` or the formal `options="header"` syntax) or a
//! header row can be inferred implicitly from the table's layout. It also shows
//! how to suppress the implicit behavior with `noheader`. Each rendering claim
//! is verified through `convert`; the introduction, the TIP admonition, and the
//! section headings carry no rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/add-header-row.adoc");

// Title, introductory paragraphs, and a TIP admonition describing the header
// row's handling of style and alignment operators: an introduction with no
// rendering rule to verify.
non_normative!(
    r#"
= Create a Header Row

The first row of a table is promoted to a header row if the `header` value is assigned to the table's `options` attribute.
You can assign `header` to a table's first row explicitly or implicitly.

TIP: The header row ignores any style operators assigned via column and cell specifiers.
It also ignores alignment operators assigned to the table's column specifiers; however, any alignment operators assigned to a cell specifier in the header row are applied.

"#
);

// Section heading only.
non_normative!(
    r#"
== Explicitly assign header to the first row

"#
);

#[test]
fn explicit_header_shorthand() {
    verifies!(
        r#"
The header row semantics and styles are explicitly assigned to the first row in a table by assigning `header` to the `options` attribute.
The `options` attribute is set in the table's attribute list using the shorthand (`%value`) or formal syntax (`options="value"`).

The `options` attribute is represented by the percent sign (`%`) when it's set using the shorthand syntax.
In <<ex-short>>, `header` is assigned to using the shorthand syntax for `options`.

.Table with `header` assigned using the shorthand syntax
[source#ex-short]
----
[%header,cols="2,2,1"] <.>
|===
|Column 1, header row
|Column 2, header row
|Column 3, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|Cell in column 3, row 2
|===
----
<.> Values assigned using the shorthand syntax must be entered before the `cols` attribute (or any other named attributes) in a table's attribute list, otherwise the processor will ignore them.

The table from <<ex-short>> is displayed below.

.Result of <<ex-short>>
[%header,cols="2,2,1"]
|===
|Column 1, header row
|Column 2, header row
|Column 3, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|Cell in column 3, row 2
|===

"#
    );

    // The `%header` shorthand promotes the first row to a header: three column
    // specifiers yield three `<col>` elements (40%, 40%, 20%), the first row
    // renders as a `<thead>` of three `<th>` cells, and the remaining row stays
    // in the `<tbody>`.
    let output = convert(
        "[%header,cols=\"2,2,1\"]\n|===\n|Column 1, header row\n|Column 2, header row\n|Column 3, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|Cell in column 3, row 2\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "col[style=\"width: 40%;\"]", 2);
    assert_css(&output, "col[style=\"width: 20%;\"]", 1);
    assert_css(&output, "table > thead > tr", 1);
    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 3);
}

#[test]
fn explicit_header_formal() {
    verifies!(
        r#"
In <<ex-formal>>, the `options` attribute is set and assigned the `header` value using the formal syntax.
The `options` attribute accepts a comma-separated list of values.

.Table with header assigned to the options attribute
[source#ex-formal]
----
include::example$row.adoc[tag=opt-h]
----

The first row of the table in <<ex-formal>> is rendered using the corresponding header styles and semantics.

.Result of <<ex-formal>>
include::example$row.adoc[tag=opt-h]

"#
    );

    // The formal `options="header"` syntax produces the same result as the
    // shorthand: two columns, the first row rendered as a `<thead>` of two
    // `<th>` cells, and the two remaining rows in the `<tbody>`.
    let output = convert(
        "[cols=\"2*\",options=\"header\"]\n|===\n|Column 1, header row\n|Column 2, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > thead > tr > th", 2);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 4);
}

// Section heading only.
non_normative!(
    r#"
== Implicitly assign header to the first row

"#
);

#[test]
fn implicit_header() {
    verifies!(
        r#"
You can implicitly define a header row based on how you layout the table.
The following conventions determine when the first row automatically becomes the header row:

. The first line of content inside the table delimiters is not empty.
. The second line of content inside the table delimiters is empty.

.First row is implicitly assigned header
[source#ex-implicit]
----
include::example$row.adoc[tag=impl-h]
----

As seen in the result below, if all of these rules hold true, then the first row of the table is treated as a header row.

.Result of <<ex-implicit>>
include::example$row.adoc[tag=impl-h]

"#
    );

    // With the first row's cells on the line after the delimiter and the next
    // line empty, the first row is implicitly treated as a header: a `<thead>`
    // of two `<th>` cells, leaving the two remaining rows in the `<tbody>`.
    let output = convert(
        "|===\n|Column 1, header row |Column 2, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n|===",
    );

    assert_css(&output, "table > thead > tr", 1);
    assert_css(&output, "table > thead > tr > th", 2);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 4);
}

// Section heading only.
non_normative!(
    r#"
=== Deactivate the implicit assignment of header

"#
);

#[test]
fn deactivate_implicit_header() {
    verifies!(
        r#"
To suppress the implicit behavior of promoting the first row to a header row, assign the value `noheader` to the `options` attribute using the formal (`options=noheader`) or shorthand (`%noheader`) syntax.
In <<ex-noheader>>, `noheader` is assigned using the shorthand syntax.

.Deactivate implicit header row with noheader
[source#ex-noheader]
----
[%noheader]
|===
|Cell in column 1, row 1 |Cell in column 2, row 1

|Cell in column 1, row 2 |Cell in column 2, row 2
|===
----

The table from <<ex-noheader>> is displayed below.

.Result of <<ex-noheader>>
[%noheader]
|===
|Cell in column 1, row 1 |Cell in column 2, row 1

|Cell in column 1, row 2 |Cell in column 2, row 2
|===

"#
    );

    // With `noheader`, the layout that would otherwise imply a header row is
    // suppressed: there is no `<thead>`, and both rows land in the `<tbody>`.
    let output = convert(
        "[%noheader]\n|===\n|Cell in column 1, row 1 |Cell in column 2, row 1\n\n|Cell in column 1, row 2 |Cell in column 2, row 2\n|===",
    );

    assert_css(&output, "thead", 0);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 4);
}

// Commented-out CAUTION note in the source page: it is an AsciiDoc line comment
// that produces no output, so there is nothing to verify.
non_normative!(
    r#"
//CAUTION: We're considering using a similar convention for enabling the footer in the future.
//Thus, if you rely on this convention to enable the header row, it's advised that you not put all the cells in the last row on the same line unless you intend on making it the footer row.
"#
);
