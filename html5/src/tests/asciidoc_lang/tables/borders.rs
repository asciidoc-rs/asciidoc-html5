//! Coverage of the AsciiDoc language description's `borders` page.
//!
//! The page controls table borders with the `frame` and `grid` attributes. Each
//! value renders as a class on the `<table>`: `frame` yields `frame-all`,
//! `frame-ends`, `frame-sides`, or `frame-none`, and `grid` yields `grid-all`,
//! `grid-rows`, `grid-cols`, or `grid-none`. The non-default values are checked
//! through `convert`; the introduction, the section headings, the attribute
//! descriptions, and the discussion of span interference carry no rule of their
//! own and are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/borders.adoc");

// The base-h table reused by every example on this page, standing in for the
// `include::example$row.adoc[tag=base-h]` directive.
const BASE_H: &str = "|===\n|Column 1, header row |Column 2, header row |Column 3, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|Cell in column 3, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n|Cell in column 3, row 3\n|===";

// Title, document-attribute resets, and the introduction describing that
// borders are controlled by `frame` and `grid`; no single rule to verify here.
non_normative!(
    r#"
= Table Borders
:!table-frame:
:!table-grid:

The borders on a table are controlled using the `frame` and `grid` block attributes.
You can combine these two attributes to achieve a variety of border styles for your tables.

"#
);

// Section heading only.
non_normative!(
    r#"
== Frame

"#
);

// Describes the `frame` attribute, its default of `all`, the `table-frame`
// document attribute, and the set of accepted values; the specific values are
// verified in the examples that follow.
non_normative!(
    r#"
The border around a table is controlled using the `frame` attribute on the table.
The frame attribute defaults to the value `all` (implied value), which draws a border on each side of the table.
This default can be changed by setting the `table-frame` document attribute.
You can override the default value by setting the frame attribute on the table to the value `all`, `ends`, `sides` or `none`.

"#
);

#[test]
fn frame_ends() {
    verifies!(
        r#"
The `ends` value draws a border on the top and bottom of the table.

.Table with frame=ends
[source#ex-ends]
----
[frame=ends]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-ends>>
[frame=ends]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `ends` value renders as the `frame-ends` class on the `<table>`.
    let output = convert(&format!("[frame=ends]\n{BASE_H}"));

    assert_css(&output, "table.frame-ends", 1);
}

#[test]
fn frame_sides() {
    verifies!(
        r#"
The `sides` value draws a border on the right and left side of the table.

.Table with frame=sides
[source#ex-sides]
----
[frame=sides]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-sides>>
[frame=sides]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `sides` value renders as the `frame-sides` class on the `<table>`.
    let output = convert(&format!("[frame=sides]\n{BASE_H}"));

    assert_css(&output, "table.frame-sides", 1);
}

#[test]
fn frame_none() {
    verifies!(
        r#"
The `none` value removes the borders around the table.

.Table with frame=none
[source#ex-noframe]
----
[frame=none]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-noframe>>
[frame=none]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `none` value renders as the `frame-none` class on the `<table>`.
    let output = convert(&format!("[frame=none]\n{BASE_H}"));

    assert_css(&output, "table.frame-none", 1);
}

// Section heading only.
non_normative!(
    r#"
== Grid

"#
);

// Describes the `grid` attribute, its default of `all`, the `table-grid`
// document attribute, and the set of accepted values; the specific values are
// verified in the examples that follow.
non_normative!(
    r#"
The borders between the cells in a table are controlled using the `grid` attribute on the table.
The grid attribute defaults to the value `all` (implied value), which draws a border between all cells.
This default can be changed by setting the `table-grid` document attribute.
You can override the default value by setting the grid attribute on the table to the value `all`, `rows`, `cols` or `none`.

"#
);

#[test]
fn grid_rows() {
    verifies!(
        r#"
The `rows` value draws a border between the rows of the table.

.Table with grid=rows
[source#ex-rows]
----
[grid=rows]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-rows>>
[grid=rows]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `rows` value renders as the `grid-rows` class on the `<table>`.
    let output = convert(&format!("[grid=rows]\n{BASE_H}"));

    assert_css(&output, "table.grid-rows", 1);
}

#[test]
fn grid_cols() {
    verifies!(
        r#"
The `cols` value draws borders between the columns.

.Table with grid=cols
[source#ex-cols]
----
[grid=cols]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-cols>>
[grid=cols]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `cols` value renders as the `grid-cols` class on the `<table>`.
    let output = convert(&format!("[grid=cols]\n{BASE_H}"));

    assert_css(&output, "table.grid-cols", 1);
}

#[test]
fn grid_none() {
    verifies!(
        r#"
The `none` value removes all borders between the cells.

.Table with grid=none
[source#ex-nogrid]
----
[grid=none]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-nogrid>>
[grid=none]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `none` value renders as the `grid-none` class on the `<table>`.
    let output = convert(&format!("[grid=none]\n{BASE_H}"));

    assert_css(&output, "table.grid-none", 1);
}

// The interaction between spans and border placement is a CSS/HTML limitation
// with no rendering rule of its own, plus guidance on congruent frame and grid
// combinations; descriptive content with nothing to verify.
non_normative!(
    r#"
== Interaction with row and column spans

Using row and column spans may interfere with the placement of borders on a table.
This is a limitation of styling HTML using CSS.

When a cell extends into other rows or columns, that cell is not represented in the HTML for the rows or columns it extends into.
This is a problem if the cell reaches the boundary of the table.
The CSS selector only matches the cell where it starts and thus does not detect when it is touching the table boundary.
It therefore cannot add or remove the border as it would for a 1x1 cell (i.e., a cell confined to a single row and column).

The interference with border placement caused by row and column spans does not always happen.
Borders on a table with a rowspan or colspan that reaches the table boundary will always work correctly when the frame and grid are congruent.
In this context, congruent means the frame and grid are contributing borders to the same edges.

Here are those scenarios in which the frame and grid are congruent:

* frame=all, grid=all
* frame=all, grid=none
* frame=all, grid=rows
* frame=all, grid=cols
* frame=ends, grid=rows
* frame=sides, grid=cols
* frame=none, grid=none

If you use row and column spans in a table, you are strongly encouraged to use one of these frame and grid combinations.
"#
);
