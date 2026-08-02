//! Coverage of the AsciiDoc language description's *Span Columns and Rows*
//! page.
//!
//! The page teaches how a single table cell can span adjacent columns, rows, or
//! a block of both: a column span factor (`<n>+`) becomes a `colspan`, a row
//! span factor (`.<n>+`) becomes a `rowspan`, and a block span factor
//! (`<n>.<n>+`) produces both. Each rendering claim is verified through
//! `convert`; the title, the introduction, the definitions of the span factor
//! and operator, and the section headings carry no rule of their own and are
//! tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/span-cells.adoc");

// Title, introduction, and the *Span factor and operator* section: descriptive
// definitions of the span factor and span operator syntax, plus an example
// block showing the cell-specifier notation, with no rendering rule to verify.
non_normative!(
    r#"
= Span Columns and Rows

A table cell can span more than one column and row.

== Span factor and operator

With a [.term]*span* a table cell can stretch across adjacent columns, rows, or a block of adjacent columns and rows.
A span consists of a span factor and a span operator.

The [.term]*span factor* indicates the number columns, rows, or columns and rows a cell should span.

[[col-factor]]Column span factor:: A single integer (`<n>`) that represents the number of consecutive columns a cell should span.
[[row-factor]]Row span factor:: A single integer prefixed with a dot (`.<n>`) that represents the number of consecutive rows a cell should span.
[[block-factor]]Block span factor:: Two integers (`<n>.<n>`) that represent a block of adjacent columns and rows a cell should span.
The first integer, `<n>`, is the column span factor.
The second integer, which is prefixed with a dot, `.<n>`, is the row span factor.

The [.term]*span operator* is a plus sign (`\+`) placed directly after the span factor (`<n>.<n>+`).
The span operator tells the converter to interpret the span factor as part of a span instead of a duplication.

A span is the first operator in a xref:add-cells-and-rows.adoc#specifiers[cell specifier].

====
<**span factor**><**span operator**><horizontal alignment operator><vertical alignment operator><style operator>|<cell's content>
====

"#
);

// Section heading only.
non_normative!(
    r#"
== Span multiple columns

"#
);

#[test]
fn span_multiple_columns() {
    verifies!(
        r#"
To have a cell span consecutive columns, enter the <<col-factor,column span factor>> and span operator (`<n>+`) in the cell specifier.
Don't insert any spaces between the span, any alignment or style operators (if present), and the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).

.Span three columns with a cell
[source#ex-span-columns]
----
include::example$cell.adoc[tag=span-cols]
----

The table from <<ex-span-columns>> is displayed below.

.Result of <<ex-span-columns>>
include::example$cell.adoc[tag=span-cols]

"#
    );

    // A column span factor of `3+` renders as a single `<td>` carrying
    // `colspan="3"`; the remaining cells occupy the four-column grid normally.
    let output = convert(
        "|===\n|Column 1, header row |Column 2, header row |Column 3, header row |Column 4, header row\n\n3+|This cell spans columns 1, 2, and 3 because its specifier contains a span of `3+`\n|Cell in column 4, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n|Cell in column 3, row 3\n|Cell in column 4, row 3\n|===",
    );

    assert_css(&output, "colgroup > col", 4);
    assert_css(&output, "table > thead > tr > th", 4);
    assert_css(&output, "td[colspan=\"3\"]", 1);
    assert_css(&output, "table > tbody > tr", 2);
}

// Section heading only.
non_normative!(
    r#"
== Span multiple rows

"#
);

#[test]
fn span_multiple_rows() {
    verifies!(
        r#"
To have a cell span consecutive rows, enter the <<row-factor,row span factor>> and span operator (`.<n>+`) in the cell specifier.
Remember to prefix the span factor with a dot (`.`).
Don't insert any spaces between the span, any alignment or style operators (if present), and the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).

.Span two rows with a cell
[source#ex-span-rows]
----
include::example$cell.adoc[tag=span-rows]
----

The table from <<ex-span-rows>> is displayed below.

.Result of <<ex-span-rows>>
include::example$cell.adoc[tag=span-rows]

"#
    );

    // A row span factor of `.2+` renders as a single `<td>` carrying
    // `rowspan="2"`, stretching the cell down over two body rows.
    let output = convert(
        "|===\n|Column 1, header row |Column 2, header row\n\n.2+|This cell spans rows 2 and 3 because its specifier contains a span of `.2+`\n|Cell in column 2, row 2\n\n|Cell in column 2, row 3\n\n|Cell in column 1, row 4\n|Cell in column 2, row 4\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > thead > tr > th", 2);
    assert_css(&output, "td[rowspan=\"2\"]", 1);
    assert_css(&output, "table > tbody > tr", 3);
}

// Section heading only.
non_normative!(
    r#"
== Span columns and rows

"#
);

#[test]
fn span_columns_and_rows() {
    verifies!(
        r#"
A single cell can span a block of adjacent columns and rows.
Enter the column span factor (`<n>`), followed by the row span factor (`.<n>`), and then the span operator (`+`).

.Span two columns and three rows with a single cell
[source#ex-block]
----
include::example$cell.adoc[tag=span-block]
----

The table from <<ex-block>> is displayed below.

.Result of <<ex-block>>
include::example$cell.adoc[tag=span-block]
"#
    );

    // A block span factor of `2.3+` renders as a single `<td>` carrying both
    // `colspan="2"` and `rowspan="3"`, covering a two-column, three-row block.
    let output = convert(
        "|===\n|Column 1, header row |Column 2, header row |Column 3, header row |Column 4, header row\n\n|Cell in column 1, row 2\n2.3+|This cell spans columns 2 and 3 and rows 2, 3, and 4 because its specifier contains a span of `2.3+`\n|Cell in column 4, row 2\n\n|Cell in column 1, row 3\n|Cell in column 4, row 3\n\n|Cell in column 1, row 4\n|Cell in column 4, row 4\n|===",
    );

    assert_css(&output, "colgroup > col", 4);
    assert_css(&output, "table > thead > tr > th", 4);
    assert_css(&output, "td[colspan=\"2\"]", 1);
    assert_css(&output, "td[rowspan=\"3\"]", 1);
}
