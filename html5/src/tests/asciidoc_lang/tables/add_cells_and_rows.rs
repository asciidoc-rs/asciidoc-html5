//! Coverage of the AsciiDoc language description's *Add Cells and Rows to a
//! Table* page.
//!
//! The page teaches how the cell separator (`|`) declares cells, how the
//! processor arranges those cells into rows once the column count is reached,
//! how a cell specifier styles a single cell, and the equivalence of entering a
//! row's cells on one line or on consecutive lines. Each of those rendering
//! claims is verified through `convert`; the section headings, the operator
//! cross-reference lists, and the transitional summaries carry no rule of their
//! own and are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/add-cells-and-rows.adoc");

// Title and an editorial authoring comment: an introduction to the page with no
// rendering rule to verify.
non_normative!(
    r#"
= Add Cells and Rows to a Table
//let's add a tip or xref to something about the "default" cell separator and data format and the alternatives that's not too invasive to the flow

"#
);

// Section heading only.
non_normative!(
    r#"
== Table cells

"#
);

#[test]
fn cell_separator() {
    verifies!(
        r#"
[[cell-separator]]Each new cell in a table is declared with a cell separator.
The default [.term]*cell separator* is a vertical bar (`|`).
All of the content entered after a cell separator is included in that cell until the processor encounters a space followed by another vertical bar (`|`) or a new line that begins with a `|`.

.Creating table cells with the default cell separator
[source#ex-separator]
----
[cols="3,2,3"]
|===
|This content is placed in the first cell of column 1
|This line starts with a vertical bar so this content is placed in a new cell in column 2 |When the processor encounters a whitespace followed by a vertical bar it ends the previous cell and starts a new cell
|===
----

When the processor encounters another `|`, it creates a new cell in the next consecutive column.
Once the processor reaches the xref:add-columns.adoc[last column assigned to the table], the next cell it encounters is placed in a new row.
Taking into account any xref:span-cells.adoc[spans], which are applied via a <<specifiers,cell specifier>>, each row consists of the same number of cells.

"#
    );

    // The three column specifiers produce three columns; the first cell on its
    // own line and the two cells on the following line (the second started at a
    // space-then-bar) fill exactly one body row of three cells.
    let output = convert(
        "[cols=\"3,2,3\"]\n|===\n|This content is placed in the first cell of column 1\n|This line starts with a vertical bar so this content is placed in a new cell in column 2 |When the processor encounters a whitespace followed by a vertical bar it ends the previous cell and starts a new cell\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "col[style=\"width: 37.5%;\"]", 2);
    assert_css(&output, "col[style=\"width: 25%;\"]", 1);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 3);
    assert_css(&output, "thead", 0);
}

// Section anchor and heading only.
non_normative!(
    r#"
[#specifiers]
=== Cell specifiers and operators

"#
);

#[test]
fn cell_specifier() {
    verifies!(
        r#"
A [.term]*cell specifier* is a positional attribute placed directly in front of a cell separator that represents the position and style properties assigned to a cell's content.
In the example below, a horizontal alignment operator and style operator have been assigned to the first cell's specifier.

.Using cell specifiers
[source#ex-specifier]
----
[cols="2*"]
|===
>s|This cell's specifier indicates that this cell's content is right-aligned and bold.
|The cell specifier on this cell hasn't been set explicitly, so the  default properties are applied.
|===
----

"#
    );

    // The `>s` cell specifier right-aligns and bolds only the first cell; the
    // second cell, with no specifier, keeps the default left alignment and no
    // strong styling.
    let output = convert(
        "[cols=\"2*\"]\n|===\n>s|This cell's specifier indicates that this cell's content is right-aligned and bold.\n|The cell specifier on this cell hasn't been set explicitly, so the  default properties are applied.\n|===",
    );

    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 2);
    assert_css(&output, "td.halign-right", 1);
    assert_css(&output, "td.halign-right > p > strong", 1);
    assert_css(&output, "td.halign-left", 1);
    assert_css(&output, "td.halign-left > p > strong", 0);
}

// A bulleted list of cell-property operators, each documented on its own page,
// and prose about a specifier's single-cell scope and its precedence over a
// column specifier: the list points elsewhere and the override behavior is not
// demonstrated here (the single-cell scope itself is shown by the ex-specifier
// test above).
non_normative!(
    r#"
AsciiDoc provides operators to control the following cell properties:

* xref:span-cells.adoc[span]
* xref:duplicate-cells.adoc[duplication]
* xref:align-by-cell.adoc#horizontal-operators[horizontal alignment]
* xref:align-by-cell.adoc#vertical-operators[vertical alignment]
* xref:format-cell-content.adoc[content style]

A cell specifier only applies to the cell it's placed on, not to all of the cells in the same row.
Also, the operator in a cell specifier will override the operator in a xref:add-columns.adoc#col-specifier[column specifier] if they belong to the same property.

"#
);

// Section heading only.
non_normative!(
    r#"
== Create a table cell

"#
);

#[test]
fn two_cells_form_one_row() {
    verifies!(
        r#"
In this section, we'll set up a table and add two rows of cells to it.
First, let's create two cells in <<ex-cells>> and see how they get arranged into a row.

.Add two cells to a table
[source#ex-cells]
----
[cols="1,1"] <.>
|===
|This cell is in column 1, row 1 <.>
|This cell is in column 2, row 1 <.>
|===
----
<.> The table has two columns because two column specifiers are assigned to the `cols` attribute.
<.> The processor places this cell in the first column and row of the table because the vertical bar (`|`) at the beginning of this cell is the first cell separator the processor encounters after the opening table delimiter (`|===`).
<.> This is the second `|` the processor encounters, so this cell is placed in the second column of the first row.

Though the two cells in <<ex-cells>> were entered on separate lines, the processor places them in the same row.

.Result of <<ex-cells>>
[cols="1,1"]
|===
|This cell is in column 1, row 1
|This cell is in column 2, row 1
|===

"#
    );

    // Two column specifiers make two columns; the two cells, though entered on
    // separate lines, complete a single body row of two cells.
    let output = convert(
        "[cols=\"1,1\"]\n|===\n|This cell is in column 1, row 1\n|This cell is in column 2, row 1\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 2);
    assert_css(&output, "thead", 0);
}

#[test]
fn four_cells_form_two_rows() {
    verifies!(
        r#"
Since the number of columns in <<ex-cells>> is set to two by the `cols` attribute, and there are two cells entered in the table, the first row is complete.
Now, let's add two more cells to the table.

.Add two more cells to a table
[source#ex-more-cells]
----
[cols="1,1"]
|===
|This cell is in column 1, row 1
|This cell is in column 2, row 1
<.>
|This cell is in column 1, row 2 <.>
|    This cell is in column 2, row 2 <.>
|===
----
<.> Separate rows by one or more empty lines.
<.> The processor places this cell on the second row because the table has two columns and this is the third cell separator (`|`) it encounters.
<.> Any leading or trailing spaces around the cell content is stripped by the processor.

The table from <<ex-more-cells>> now has four cells arranged into two consecutive rows.

.Result of <<ex-more-cells>>
[cols="1,1"]
|===
|This cell is in column 1, row 1
|This cell is in column 2, row 1

|This cell is in column 1, row 2
|    This cell is in column 2, row 2
|===

"#
    );

    // Four cells in a two-column table arrange into two consecutive body rows.
    let output = convert(
        "[cols=\"1,1\"]\n|===\n|This cell is in column 1, row 1\n|This cell is in column 2, row 1\n\n|This cell is in column 1, row 2\n|    This cell is in column 2, row 2\n|===",
    );

    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 4);
    assert_css(&output, "thead", 0);

    // The leading spaces in front of "This cell is in column 2, row 2" are
    // stripped, so the cell's text carries no leading whitespace.
    assert_xpath(
        &output,
        "//td/p[text()=\"This cell is in column 2, row 2\"]",
        1,
    );
}

// A transitional summary of what the two following sections demonstrate: the
// row a cell lands on depends only on the column count and the order of
// separators.
non_normative!(
    r#"
The cells in a row can be entered on the same line or consecutive lines because the row a cell in placed on is determined by the number of columns in a table and the order in which the processor encounters the cell's separator (`|`).

"#
);

// Section anchor and heading only.
non_normative!(
    r#"
[#same]
== Enter a row's cells on a single line

"#
);

#[test]
fn cells_on_a_single_line() {
    verifies!(
        r#"
You can enter a row's cells on a single line.
//This method is how the number or columns in a table are implicitly assigned and implicitly assign the `header` option to the table's first row.
When entering cells on a single line, *at least one space must be entered between the last character of the previous cell's content and the cell separator (`|`) of the next cell*.

.Cells entered on the same line
[source#ex-single-line]
----
|===
|Column 1 |Column 2 |Column 3 <.> <.>

|Cell in column 1, row 2 |Cell in column 2, row 2 |Cell in column 3, row 2 <.>

|Cell in column 1, row 3 <.>
|Cell in column 2, row 3 |Cell in column 3, row 3
|===
----
<.> Since `cols` is not set, the first row in this table must have the cells entered on a single line in order to implicitly assign three columns to the table.
<.> The first row is entered on the line directly after the opening table delimiter (`|===`) and followed by an empty line.
This automatically assigns the `header` option to it.
<.> When multiple cells are entered on a single line, enter at least one space between the last character of the previous cell's content and the cell separator (`|`) of the next cell.
<.> A row's cells can be entered on a combination of lines as long as the lines are consecutive.

The table created in <<ex-single-line>> contains three columns and three rows.

.Result of <<ex-single-line>>
|===
|Column 1 |Column 2 |Column 3

|Cell in column 1, row 2 |Cell in column 2, row 2 |Cell in column 3, row 2

|Cell in column 1, row 3
|Cell in column 2, row 3 |Cell in column 3, row 3
|===

Any leading and trailing spaces around the cell content are stripped and don't affect the table's layout when rendered.

"#
    );

    // The first row, entered on a single line and followed by an empty line,
    // becomes the implicit header of three columns; the two remaining rows fill
    // the body, whether entered on one line or across consecutive lines.
    let output = convert(
        "|===\n|Column 1 |Column 2 |Column 3\n\n|Cell in column 1, row 2 |Cell in column 2, row 2 |Cell in column 3, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3 |Cell in column 3, row 3\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "table > thead > tr", 1);
    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);

    // The trailing spaces before each cell separator are stripped: the header
    // cells' text is exactly the column label with no surrounding whitespace.
    assert_xpath(&output, "//th[text()=\"Column 1\"]", 1);
    assert_xpath(&output, "//th[text()=\"Column 2\"]", 1);
    assert_xpath(&output, "//th[text()=\"Column 3\"]", 1);
}

// Section anchor and heading only.
non_normative!(
    r#"
[#consecutive]
== Enter a row's cells on consecutive lines

"#
);

#[test]
fn cells_on_consecutive_lines() {
    verifies!(
        r#"
The cells in a row can be entered on consecutive, individual lines.
When using this method, remember to either xref:add-columns.adoc[set the cols attribute] or xref:add-columns.adoc#implicit-cols[enter the first row's cells on a single line].

.Cells on consecutive, individual lines
[source#ex-consecutive-lines]
----
include::example$row.adoc[tag=indv]
----

The `cols` attribute in <<ex-consecutive-lines>> is assigned a xref:add-columns.adoc#column-multiplier[multiplier] of `+3*+`, indicating that this table has three columns.

.Result of <<ex-consecutive-lines>>
include::example$row.adoc[tag=indv]

Entering cells on consecutive lines can improve the readability (and debugging) of your raw AsciiDoc content when you're applying <<specifiers,specifiers to the cells>>, using xref:format-cell-content.adoc#a-operator[AsciiDoc block elements in the cells], or entering a lot of content into the cells.
"#
    );

    // The `3*` multiplier makes three columns; the six cells entered on
    // consecutive lines arrange into two body rows of three cells each.
    let output = convert(
        "[cols=\"3*\"]\n|===\n|Cell in column 1, row 1\n|Cell in column 2, row 1\n|Cell in column 3, row 1\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|Cell in column 3, row 2\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);
    assert_css(&output, "thead", 0);
}
