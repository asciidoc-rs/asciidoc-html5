//! Coverage of the AsciiDoc language description's *Add Columns to a Table*
//! page.
//!
//! The page teaches the three ways a table's column count and widths are
//! established: a `cols` list of specifiers, a column multiplier (`<n>*`,
//! optionally mixed with specifiers), and the implicit count derived from the
//! first row's cells. Each of those rendering claims is verified through
//! `convert`; the section headings and the alignment/style operator
//! cross-references, documented on their own pages, are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/add-columns.adoc");

// Title and an overview of the two ways to set the column count, each verified
// in its own section below.
non_normative!(
    r#"
= Add Columns to a Table

The number of columns in a table is specified by the `cols` attribute or <<implicit-cols,by the number of cells found in the first non-empty line>> after the opening table delimiter (`|===`).

"#
);

// Section anchor and heading only.
non_normative!(
    r#"
[#cols-attribute]
== Specify the number of columns with the cols attribute

"#
);

#[test]
fn cols_attribute() {
    verifies!(
        r#"
The `cols` attribute is set in the attribute list on a table block.
It accepts a comma-separated list of column specifiers.
[[col-specifier]]Each [.term]*column specifier* represents a column and the width, alignment, and style properties assigned to that column.
A column specifier is commonly represented by a number, but in some cases, can be represented by a symbol or letter.
In <<ex-cols>>, `cols` is assigned a list of four numeric column specifiers.

.Assign column specifiers to the cols attribute
[source#ex-cols]
----
[cols="1,1,1,1"]
----

In <<ex-cols>>, the value assigned to `cols`  contains four column specifiers.
The number of entries in the value's list determines the number of columns in the table.
That means the table in the above example will contain four columns.
When the specifier is a number, such as `1` or `50`, the integer represents the xref:adjust-column-widths.adoc[width of the column in proportion to the other columns in the table].
In <<ex-cols>>, each column will be the same width because the integer in each specifier is the same.
Let's look at the column specifiers in <<ex-cols-alt>> and compare it to <<ex-cols>>.

.Assign column specifiers to the cols attribute
[source#ex-cols-alt]
----
[cols="3,3,3,3"]
----

Both <<ex-cols>> and <<ex-cols-alt>> will produce tables with four columns of equal width.
Let's use the `cols` value in <<ex-cols-alt>> to create a table.

.Create a table with four columns of equal width
[source#ex-cols-table]
----
[cols="3,3,3,3"] <.>
|=== <.>
|Column 1 |Column 2 |Column 3 |Column 4 <.>
<.>
|Cell in column 1 <.>
|Cell in column 2
|Cell in column 3
|Cell in column 4
|=== <.>
----
<.> In an attribute list, set the `cols` attribute, followed by an equals sign (`=`), and then a list of comma-separated column specifiers enclosed in double quotation marks (`"`).
<.> On the line directly after the attribute list, enter the opening table delimiter.
A table delimiter is one vertical bar followed by three equals signs (`|===`).
<.> A table cell is specified by a vertical bar (`|`).
Since four consecutive cells are entered on the first line directly after the delimiter, this row is implicitly set as the table's header row.
<.> Insert an empty line after the header row.
<.> The cells for the next row can be entered on a single line or on individual lines.
<.> On a new line after the last cell of the last row, enter another table delimiter (`|===`) to close the table block.

<<ex-cols-table>> creates the table displayed below.

.Result of <<ex-cols-table>>
[cols="3,3,3,3"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|Cell in column 1
|Cell in column 2
|Cell in column 3
|Cell in column 4
|===

As specified, the table includes four columns of equal width, a header row, and a regular row.
Since all of the columns in <<ex-cols-table>> are assigned the same width via their column specifiers (i.e., `3`), the number of columns could be specified with a <<column-multiplier,column multiplier>>.
Or, you could adjust the width of an individual column by xref:adjust-column-widths.adoc[increasing the numerical value of its specifier].

"#
    );

    // Four equal specifiers produce four columns, each 25% wide; the first row
    // on a single line becomes the header, leaving one regular body row.
    let output = convert(
        "[cols=\"3,3,3,3\"]\n|===\n|Column 1 |Column 2 |Column 3 |Column 4\n\n|Cell in column 1\n|Cell in column 2\n|Cell in column 3\n|Cell in column 4\n|===",
    );

    assert_css(&output, "colgroup > col", 4);
    assert_css(&output, "col[style=\"width: 25%;\"]", 4);
    assert_css(&output, "table > thead > tr", 1);
    assert_css(&output, "table > thead > tr > th", 4);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 4);
}

// Section anchor and heading only.
non_normative!(
    r#"
[#column-multiplier]
=== Using a column multiplier

"#
);

#[test]
fn column_multiplier() {
    verifies!(
        r#"
A [.term]*column multiplier* allows you to apply the same width, horizontal alignment, vertical alignment, and content style to multiple, consecutive columns in a table.
A multiplier consists of an integer (`<n>`) and an asterisk (`+*+`).
The integer represents the number of consecutive columns to be added to the table.
The asterisk (`+*+`) is called the [.term]*multiplier operator* and is placed directly after the integer (`+<n>*+`).
The operator tells the converter to interpret the integer as part of a column multiplier instead of a column specifier.

For example, let's rewrite the value of `[cols="5,5,5"]` as a column multiplier.

.Represent [cols="5,5,5"] using a column multiplier
[source]
----
[cols="3*"] <.>
----
<.> Assign an integer to `cols` that represents the number of columns in the table.
Enter the multiplier operator (`+*+`) directly after the integer.

The integer `3`, combined with the `+*+` operator, indicates that the table will contain three columns of equal width.

You can use a multiplier in a comma-separated list with column specifiers, too.
In <<ex-spec-and-multiplier>>, the first column is represented by a column specifier, and the next three columns are represented by a multiplier.

.Assign a column specifier and a column multiplier to cols
[source#ex-spec-and-multiplier]
----
[cols="5,3*"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|Cell in column 1
|Cell in column 2
|Cell in column 3
|Cell in column 4
|===
----

As shown below, <<ex-spec-and-multiplier>> creates a table containing a xref:adjust-column-widths.adoc[wide first column] followed by three columns of equal width.

.Result of <<ex-spec-and-multiplier>>
[cols="5,3*"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|Cell in column 1
|Cell in column 2
|Cell in column 3
|Cell in column 4
|===

"#
    );

    // The `5,3*` value combines one specifier with a three-column multiplier:
    // four columns, the first wide (62.5%) and the next three equal (12.5%).
    let output = convert(
        "[cols=\"5,3*\"]\n|===\n|Column 1 |Column 2 |Column 3 |Column 4\n\n|Cell in column 1\n|Cell in column 2\n|Cell in column 3\n|Cell in column 4\n|===",
    );

    assert_css(&output, "colgroup > col", 4);
    assert_css(&output, "col[style=\"width: 62.5%;\"]", 1);
    assert_css(&output, "col[style=\"width: 12.5%;\"]", 3);
    assert_css(&output, "table > thead > tr > th", 4);
    assert_css(&output, "table > tbody > tr > td", 4);
}

// The alignment- and style-operator section: a heading, a description, and a
// bulleted list of operators each documented on its own page, with no HTML rule
// to verify here.
non_normative!(
    r#"
[#cols-format]
=== Alignment and style column operators

AsciiDoc provides operators that control the positioning and style of column content when the `cols` attribute is set.
A column specifier or multiplier can contain these optional operators for one or more of the following properties:

* xref:align-by-column.adoc#horizontal-operators[horizontal alignment]
* xref:align-by-column.adoc#vertical-operators[vertical alignment]
* xref:format-column-content.adoc[content style]

Many of these operators can be applied to individual cells as well.

"#
);

// Section anchor and heading only.
non_normative!(
    r#"
[#implicit-cols]
== Specify the number of columns using the first row

"#
);

#[test]
fn implicit_columns_from_first_row() {
    verifies!(
        r#"
When all of the columns in a table use the default width, alignment, and style values, you don't need to set the `cols` attribute.
Instead, you can implicitly declare the number of columns by entering all of the first row's cells on the same line.
The processor will derive the number columns from the number of cells in this row.
<<ex-implicit>> uses its first row to indicate that it has three columns.

.Create a table with three columns using its first row
[source#ex-implicit]
----
|===
<.>
|Cell in column 1, row 1 |Cell in column 2, row 1 |Cell in column 3, row 1 <.>

|Cell in column 1, row 2 <.>
|Cell in column 2, row 2
|Cell in column 3, row 2
|===
----
<.> After the opening delimiter, insert an empty line before the first row, unless you want the first row to be treated as header row.
<.> Enter all of the first row's cells on a single line.
Each cell represents one column.
<.> The cells in subsequent rows don't need to be entered on a single line.

The table in <<ex-implicit>> has three columns since its first row contains three cells.

.Result of <<ex-implicit>>
|===

|Cell in column 1, row 1 |Cell in column 2, row 1 |Cell in column 3, row 1

|Cell in column 1, row 2 |Cell in column 2, row 2 |Cell in column 3, row 2
|===
"#
    );

    // With no `cols` attribute and an empty line before the first row, the count
    // is derived from the first row's three cells; no header is designated, so
    // both rows land in the body.
    let output = convert(
        "|===\n\n|Cell in column 1, row 1 |Cell in column 2, row 1 |Cell in column 3, row 1\n\n|Cell in column 1, row 2 |Cell in column 2, row 2 |Cell in column 3, row 2\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "thead", 0);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);
}
