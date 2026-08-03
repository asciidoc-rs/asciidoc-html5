//! Coverage of the AsciiDoc language description's *Build a Basic Table* page.
//!
//! The page teaches the table basics: a `cols` list of equal specifiers yields
//! equal-width columns, cells arranged into rows (whether one per line or
//! several on a line) render as a `<tbody>`, and a first row entered entirely
//! on the line after the opening delimiter becomes an implicit `<thead>`. Each
//! of those rendering claims is verified through `convert`; the section
//! headings, the learning-objectives list, and the cross-references to other
//! table pages carry no rule of their own and are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/build-a-basic-table.adoc");

// Title, page alias, and the learning-objectives list: an introduction to the
// page with no rendering rule to verify.
non_normative!(
    r#"
= Build a Basic Table
:page-aliases: index.adoc

A table is a delimited block that can have optional customizations, such as an ID and a title, as well as table-specific attributes, options, and roles.
However, at its most basic, a table only needs columns and rows.

On this page, you'll learn:

* [x] How to set up an AsciiDoc table block and its attribute list.
* [x] How to add columns to a table using the `cols` attribute.
* [x] How to add cells to a table and arrange them into rows.
* [x] How to designate a row as the table's header row.

"#
);

// Section heading only.
non_normative!(
    r#"
== Create a table with two columns and three rows

"#
);

#[test]
fn two_columns_and_three_rows() {
    verifies!(
        r#"
In <<ex-cols>>, we'll assign the `cols` attribute a list of column specifiers.
A column specifier represents a column.

.Set up a table with two columns
[source#ex-cols]
----
[cols="1,1"] <.> <.>
|=== <.>
----
<.> On a new line, create an attribute list.
Set the `cols` attribute, followed by an equals sign (`=`).
<.> Assign a list of comma-separated column specifiers enclosed in double quotation marks (`"`) to `cols`.
Each column specifier represents a column.
<.> On the line directly after the attribute list, enter the opening table delimiter.
A table delimiter is one vertical bar followed by three equals signs (`|===`).
This delimiter starts the table block.

The table in <<ex-cols>> will contain two columns because there are two comma-separated entries in the list assigned to `cols`.
Each entry in the list is called a column specifier.
A [.term]*column specifier* represents a column and the width, alignment, and style properties assigned to that column.
When each column specifier is the same number, in this case the integer `1`, all of the columns`' widths will be identical.
Each column in <<ex-cols>> will be the same width regardless of how much content they contain.

Next, let's add three rows to the table.
Each row has the same number of cells.
Since the table in <<ex-rows>> has two columns, each row will contain two cells.
A cell starts with a vertical bar (`|`).

.Add three rows to the table
[source#ex-rows]
----
[cols="1,1"]
|===
|Cell in column 1, row 1 <.>
|Cell in column 2, row 1 <.>
<.>
|Cell in column 1, row 2
|Cell in column 2, row 2

|Cell in column 1, row 3
|Cell in column 2, row 3 <.>
|=== <.>
----
<.> To create a new cell, press kbd:[Shift+|].
After the vertical bar (`|`), enter the content you want displayed in that cell.
<.> On a new line, start another cell with a `|`.
Each consecutive cell is placed in a separate, consecutive column in a row.
<.> Rows are separated by one or more empty lines.
<.> When you finish adding cells to your table, press kbd:[Enter] to go to a new line.
<.> Enter the closing delimiter (`|===`) to end the table block.

TIP: The suggestion to start each cell on its own line and to separate rows by empty lines is merely a stylistic choice.
You can enter xref:add-cells-and-rows.adoc[more than one cell or all of the cells in a row on the same line] since the processor creates a new cell each time it encounters a vertical bar (`|`).

The table from <<ex-rows>> is displayed below.
It contains two columns and three rows of text positioned and styled using the default alignment, style, border, and width attribute values.

[cols="1,1"]
|===
|Cell in column 1, row 1
|Cell in column 2, row 1

|Cell in column 1, row 2 |Cell in column 2, row 2
|Cell in column 1, row 3 |Cell in column 2, row 3
|===

"#
    );

    // Two equal column specifiers produce two columns whose widths are
    // identical: each `<col>` carries a 50% width.
    let output = convert(
        "[cols=\"1,1\"]\n|===\n|Cell in column 1, row 1\n|Cell in column 2, row 1\n\n|Cell in column 1, row 2 |Cell in column 2, row 2\n|Cell in column 1, row 3 |Cell in column 2, row 3\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "col[style=\"width: 50%;\"]", 2);

    // The cells arrange into three body rows of two cells each; with no header
    // row identified, every row is an ordinary `<tbody>` row.
    assert_css(&output, "table > tbody > tr", 3);
    assert_css(&output, "table > tbody > tr > td", 6);
    assert_css(&output, "thead", 0);

    // Entering the cells one per line and entering both cells of a row on the
    // same line produce the identical result: the processor starts a new cell at
    // each vertical bar.
    let same_line = convert(
        "[cols=\"1,1\"]\n|===\n|Cell in column 1, row 1 |Cell in column 2, row 1\n|Cell in column 1, row 2 |Cell in column 2, row 2\n|Cell in column 1, row 3 |Cell in column 2, row 3\n|===",
    );
    assert_css(&same_line, "table > tbody > tr", 3);
    assert_css(&same_line, "table > tbody > tr > td", 6);
}

// Cross-references to the alternative ways to declare columns and to the pages
// that customize width, alignment, and style: descriptive pointers with no rule
// to verify here.
non_normative!(
    r#"
In addition to the xref:add-columns.adoc[cols attribute], you can identify the number of columns using a xref:add-columns.adoc#column-multiplier[column multiplier] or xref:add-columns.adoc#implicit-cols[the table's first row].
However, the `cols` attribute is required to customize the xref:adjust-column-widths.adoc[width], xref:align-by-column.adoc[alignment], or xref:format-column-content.adoc[style] of a column.

"#
);

#[test]
fn implicit_header_row() {
    verifies!(
        r#"
=== Add a header row to the table

Let's add a header row to the table in <<ex-header>>.
You can implicitly identify the first row of a table as a header row by entering all of the first row's cells on the line directly after the opening table delimiter.

.Add a header row to the table
[source#ex-header]
----
[cols="1,1"]
|===
|Cell in column 1, header row |Cell in column 2, header row <.>
<.>
|Cell in column 1, row 2
|Cell in column 2, row 2

|Cell in column 1, row 3
|Cell in column 2, row 3

|Cell in column 1, row 4
|Cell in column 2, row 4
|===
----
<.> On the line directly after the opening delimiter (`|===`), enter all of the first row's cells on a single line.
<.> Leave the line directly after the header row empty.

The table from <<ex-header>> is displayed below.

[cols="1,1"]
|===
|Cell in column 1, header row |Cell in column 2, header row

|Cell in column 1, row 2
|Cell in column 2, row 2

|Cell in column 1, row 3
|Cell in column 2, row 3

|Cell in column 1, row 4
|Cell in column 2, row 4
|===

"#
    );

    let output = convert(
        "[cols=\"1,1\"]\n|===\n|Cell in column 1, header row |Cell in column 2, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n\n|Cell in column 1, row 4\n|Cell in column 2, row 4\n|===",
    );

    // The first row, entered entirely on the line after the delimiter, becomes
    // an implicit header: a `<thead>` of two `<th>` cells, leaving the three
    // remaining rows in the `<tbody>`.
    assert_css(&output, "table > thead > tr", 1);
    assert_css(&output, "table > thead > tr > th", 2);
    assert_css(&output, "table > tbody > tr", 3);
    assert_css(&output, "table > tbody > tr > td", 6);
}

// Cross-reference to the explicit way to declare a header row (the `header`
// option), covered on its own page.
non_normative!(
    r#"
A header row can also be identified by assigning xref:add-header-row.adoc[header to the options attribute].
"#
);
