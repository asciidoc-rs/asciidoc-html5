//! Coverage of the AsciiDoc language description's `adjust-column-widths` page.
//!
//! The page teaches how the `cols` attribute controls column widths. A list of
//! integer specifiers assigns each column a proportional share of the table's
//! width, which the HTML5 backend emits as a computed percentage on each
//! `<col>`. Percentage specifiers are used verbatim, and omitting the percent
//! sign yields the same result. Each of those computed widths is checked
//! through `convert`; the conceptual introduction and the section headings
//! carry no rule of their own and are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/adjust-column-widths.adoc");

// Title and the conceptual "Column width" section: an introduction describing
// how a width is assigned and that the total table width is backend dependent,
// with no single rendering rule to verify here.
non_normative!(
    r#"
= Adjust Column Widths
// Check "proportional" usage

== Column width

The width of a column is assigned by its xref:add-columns.adoc#col-specifier[column specifier].
The value of a column's width is either an integer or a percentage.
The default column width is `1`.
The integer or percentage represents the width of the column in proportion to the other columns within the total width of the table.
The total width of a table is backend dependent.
When using the HTML5 backend with the default Asciidoctor stylesheet, tables stretch the width of the page body unless the xref:width.adoc[table width attribute] is explicitly set.

"#
);

// Section heading only.
non_normative!(
    r#"
== Assign column widths using integers

"#
);

#[test]
fn integer_widths() {
    verifies!(
        r#"
To assign widths to the columns in a table, set the `cols` attribute and assign it a list of comma-separated column specifiers using integers.

.Assign column widths using integers
[source#ex-int]
----
[cols="2,1,3"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 2
|This column has a proportional width of 1
|This column has a proportional width of 3
|===
----

As seen below, the columns stretch across the width of the page according to their proportional widths.

.Result of <<ex-int>>
[cols="2,1,3"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 2
|This column has a proportional width of 1
|This column has a proportional width of 3
|===

"#
    );

    // The three integer specifiers 2, 1, and 3 divide the table's width into
    // proportional shares, each emitted as a computed percentage on its `<col>`.
    let output = convert(
        "[cols=\"2,1,3\"]\n|===\n|Column 1 |Column 2 |Column 3\n\n|This column has a proportional width of 2\n|This column has a proportional width of 1\n|This column has a proportional width of 3\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "col[style=\"width: 33.3333%;\"]", 1);
    assert_css(&output, "col[style=\"width: 16.6666%;\"]", 1);
    assert_css(&output, "col[style=\"width: 50.0001%;\"]", 1);
}

#[test]
fn increase_a_column_width() {
    verifies!(
        r#"
=== Increase or decrease the width of a column

To increase the width of a column, use a bigger integer in the column's specifier.
Let's make column 1 from <<ex-int>> the largest column in the table by increasing its width from `2` to `6` in <<ex-increase>>.

.Increase the width of a column
[source#ex-increase]
----
[cols="6,1,3"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 6
|This column has a proportional width of 1
|This column has a proportional width of 3
|===
----

Below, the result of <<ex-increase>> shows that column 1 is now much wider than column 3.

.Result of <<ex-increase>>
[cols="6,1,3"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 6
|This column has a proportional width of 1
|This column has a proportional width of 3
|===

"#
    );

    // Raising column 1's specifier from 2 to 6 widens it: the shares 6, 1, and 3
    // sum to 10, so the columns compute to a clean 60%, 10%, and 30%.
    let output = convert(
        "[cols=\"6,1,3\"]\n|===\n|Column 1 |Column 2 |Column 3\n\n|This column has a proportional width of 6\n|This column has a proportional width of 1\n|This column has a proportional width of 3\n|===",
    );

    assert_css(&output, "col[style=\"width: 60%;\"]", 1);
    assert_css(&output, "col[style=\"width: 10%;\"]", 1);
    assert_css(&output, "col[style=\"width: 30%;\"]", 1);
}

#[test]
fn decrease_a_column_width() {
    verifies!(
        r#"
To decrease the width of a column, use a smaller integer in its specifier.
In <<ex-decrease>>, let's make the width of column 3 smaller, but not quite as small as column 2, by decreasing its width from `3` to `2`.

.Decrease the width of a column
[source#ex-decrease]
----
[cols="6,1,2"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 6
|This column has a proportional width of 1
|This column has a proportional width of 2
|===
----

The columns, displayed in the table below, have adjusted across the width of the page according to their proportional widths.

.Result of <<ex-decrease>>
[cols="6,1,2"]
|===
|Column 1 |Column 2 |Column 3

|This column has a proportional width of 6
|This column has a proportional width of 1
|This column has a proportional width of 2
|===

"#
    );

    // Shrinking column 3 from 3 to 2 gives shares 6, 1, and 2 (summing to 9),
    // which compute to the rounded percentages below.
    let output = convert(
        "[cols=\"6,1,2\"]\n|===\n|Column 1 |Column 2 |Column 3\n\n|This column has a proportional width of 6\n|This column has a proportional width of 1\n|This column has a proportional width of 2\n|===",
    );

    assert_css(&output, "col[style=\"width: 66.6666%;\"]", 1);
    assert_css(&output, "col[style=\"width: 11.1111%;\"]", 1);
    assert_css(&output, "col[style=\"width: 22.2223%;\"]", 1);
}

// Section heading only.
non_normative!(
    r#"
== Change column widths using percentage values

"#
);

#[test]
fn percentage_widths() {
    verifies!(
        r#"
Column widths can be assigned using a percentage between `1%` and `100%`.
Like with integer values, set `cols` and assign it a list of comma-separated column specifiers using percentages.

.Assign column widths using percentages
[source#ex-percent]
----
[cols="15%,30%,55%"]
|===
|Column 1 |Column 2 |Column 3

|This column has a width of 15%
|This column has a width of 30%
|This column has a width of 55%
|===
----

As seen in the table below, the columns stretch across the width of the page according to the percentage assigned via their column specifiers.

.Result of <<ex-percent>>
[cols="15%,30%,55%"]
|===
|Column 1 |Column 2 |Column 3

|This column has a width of 15%
|This column has a width of 30%
|This column has a width of 55%
|===

When assigning percentages to `cols`, you don't have to include the percent sign (`%`).
For instance, both `[cols="15%,30%,55%"]` and `[cols="15,30,55"]` are valid.
"#
    );

    // Percentage specifiers are used directly as each column's width.
    let with_percent = convert(
        "[cols=\"15%,30%,55%\"]\n|===\n|Column 1 |Column 2 |Column 3\n\n|This column has a width of 15%\n|This column has a width of 30%\n|This column has a width of 55%\n|===",
    );

    assert_css(&with_percent, "col[style=\"width: 15%;\"]", 1);
    assert_css(&with_percent, "col[style=\"width: 30%;\"]", 1);
    assert_css(&with_percent, "col[style=\"width: 55%;\"]", 1);

    // Omitting the percent sign is equivalent: the same three widths result.
    let without_percent = convert(
        "[cols=\"15,30,55\"]\n|===\n|Column 1 |Column 2 |Column 3\n\n|This column has a width of 15%\n|This column has a width of 30%\n|This column has a width of 55%\n|===",
    );

    assert_css(&without_percent, "col[style=\"width: 15%;\"]", 1);
    assert_css(&without_percent, "col[style=\"width: 30%;\"]", 1);
    assert_css(&without_percent, "col[style=\"width: 55%;\"]", 1);
}
