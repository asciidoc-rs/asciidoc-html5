//! Coverage of the AsciiDoc language description's `align-by-cell` page.
//!
//! The page teaches the cell alignment operators: horizontal operators (`<`,
//! `>`, `^`) and vertical operators (`.<`, `.>`, `.^`) placed on a cell's
//! specifier, applied after any span or duplication operator and combinable on
//! the same cell. Each rendered example is verified through `convert`, which
//! emits the alignment as `halign-*`/`valign-*` classes on the cell. The
//! section headings, the operator-definition lists, and the syntax-order
//! pseudo-blocks carry no rendering rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/align-by-cell.adoc");

// Title and introductory description: no rendering rule to verify.
non_normative!(
    r#"
= Align Content by Cell

The alignment operators are applied to a xref:add-cells-and-rows.adoc#specifiers[cell's specifier] and allow you to horizontally and vertically align a cell's content.

"#
);

// The horizontal-operators section heading, the operator definitions, and the
// syntax-order pseudo-block: descriptive definitions with no example to render.
non_normative!(
    r#"
[#horizontal-operators]
== Horizontal alignment operators

Content can be horizontally aligned to the left or right side of the cell as well as the center of the cell.

Flush left operator (<):: The less-than sign (`<`) aligns the content to the left side of the cell.
This is the default horizontal alignment.
Flush right operator (>):: The greater-than sign (`>`) aligns the content to the right side of the cell.
Center operator (^):: The caret (`+^+`) centers the content horizontally in the cell.

A horizontal alignment operator is entered after a span or duplication operator (if present) and in front a <<vertical-operators,vertical alignment operator>> (if present).

====
<factor><span or duplication operator><**horizontal alignment operator**><vertical alignment operator><style operator>|<cell's content>
====

"#
);

#[test]
fn center_horizontally() {
    verifies!(
        r#"
=== Center content horizontally in a cell

To horizontally center the content in a cell, place the `+^+` operator in front of the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).
Don't insert any spaces between the `|` and the operator.

.Center the content of a cell horizontally
[source#ex-hcenter]
----
include::example$align-cell.adoc[tag=hcenter]
----

The table from <<ex-hcenter>> is rendered below.

.Result of <<ex-hcenter>>
include::example$align-cell.adoc[tag=hcenter]

If the cell specifier includes a span (`<n>+`) or duplication (`+<n>*+`), place the `+^+` directly after the span or duplication operator.

.Center content horizontally in spanned columns and duplicated cells
[source#ex-factor]
----
include::example$align-cell.adoc[tag=factor]
----

The table from <<ex-factor>> is rendered below.

.Result of <<ex-factor>>
include::example$align-cell.adoc[tag=factor]

"#
    );

    // The `^` cell specifier centers that cell's content horizontally; the
    // unmarked cell keeps the default left alignment.
    let hcenter = convert(
        r#"|===
|Column Name |Column Name

^|This content is horizontally centered because the cell specifier includes the `+^+` operator.
|There isn't a horizontal alignment operator on this cell specifier, so the cell falls back to the default horizontal alignment.
Content is aligned to the left side of the cell by default.
|==="#,
    );

    assert_css(&hcenter, "td.tableblock.halign-center.valign-top", 1);
    assert_css(&hcenter, "td.tableblock.halign-left.valign-top", 1);

    // Placing `^` after a span (`2+`) or duplication (`2*`) operator centers the
    // spanned cell and each duplicated cell: three centered body cells in all,
    // one of them spanning two columns.
    let factor = convert(
        r#"|===
|Column Name |Column Name

2+^|This cell spans two columns, and its content is horizontally centered because the cell specifier includes the `+^+` operator.
2*^|This content is duplicated in two adjacent columns.
Its content is horizontally centered because the cell specifier
includes the `+^+` operator.
|==="#,
    );

    assert_css(&factor, "td.tableblock.halign-center.valign-top", 3);
    assert_css(&factor, "td[colspan=\"2\"]", 1);
}

#[test]
fn align_right() {
    verifies!(
        r#"
== Align the content of a cell to the right

To align the content in a cell to its right side, place the `>` operator in front of the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`), but after a span (`<n>+`) or duplication (`+<n>*+`) operator (if present).
Don't insert any spaces between the `|` and the operators.

.Right align the content of a cell
[source#ex-right]
----
include::example$align-cell.adoc[tag=right]
----

The table from <<ex-right>> is rendered below.

.Result of <<ex-right>>
include::example$align-cell.adoc[tag=right]

"#
    );

    // The `>` operator right-aligns a cell's content, and placed after a span
    // (`2+`) it right-aligns the spanning cell as well: two right-aligned body
    // cells, one of them spanning two columns.
    let right = convert(
        r#"|===
|Column Name |Column Name

>|This content is aligned to the right side of the cell because the cell specifier includes the `>` operator.
|There isn't a horizontal alignment operator on this cell specifier, so the cell falls back to the default horizontal alignment.
Content is aligned to the left side of the cell by default.

2+>|This cell spans two columns.

Its content is aligned to the right because the cell specifier includes the `>` operator.
The `>` operator must be placed directly after the span operator (`+`).
|==="#,
    );

    assert_css(&right, "td.tableblock.halign-right.valign-top", 2);
    assert_css(&right, "td.tableblock.halign-left.valign-top", 1);
    assert_css(&right, "td[colspan=\"2\"]", 1);
}

// The vertical-operators section heading, the operator definitions, and the
// syntax-order pseudo-block: descriptive definitions with no example to render.
non_normative!(
    r#"
[#vertical-operators]
== Vertical alignment operators

Content can be vertically aligned to the top or bottom of a cell as well as the center of a cell.
Vertical alignment operators always begin with a dot (`.`).

Flush top operator (.<):: The dot and less-than sign (`.<`) aligns the content to the top of the cell.
This is the default vertical alignment.
Flush bottom operator (.>):: The dot and greater-than sign (`.>`) aligns the content to the bottom of the cell.
Center operator (.^):: The dot and caret (`+.^+`) centers the content vertically.

A vertical alignment operator is entered after a <<horizontal-operators,horizontal alignment operator>> (if present) and in front of a style operator (if present).

====
<factor><span or duplication operator><horizontal alignment operator><**vertical alignment operator**><style operator>|<cell's content>
====

"#
);

#[test]
fn align_bottom() {
    verifies!(
        r#"
=== Align content to the bottom of a cell

To align the content to the bottom of a cell, place the `+.>+` operator in front of the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).
Don't insert any spaces between the `|` and the operator.

.Align content to the bottom of a cell
[source#ex-bottom]
----
include::example$align-cell.adoc[tag=bottom]
----

The table from <<ex-bottom>> is rendered below.

.Result of <<ex-bottom>>
include::example$align-cell.adoc[tag=bottom]

If the cell specifier includes a span (`<n>+`) or duplication (`+<n>*+`), place the `.>` after the span or duplication operator.

.Align content to the bottom of a cell that spans rows
[source#ex-span-vertical]
----
include::example$align-cell.adoc[tag=vspan]
----

The table from <<ex-span-vertical>> is rendered below.

.Result of <<ex-span-vertical>>
include::example$align-cell.adoc[tag=vspan]

"#
    );

    // The `.>` operator aligns the cell's content to the bottom; the unmarked
    // cell keeps the default top alignment.
    let bottom = convert(
        r#"[cols="2,1"]
|===
|Column Name |Column Name

.>|This content is aligned to the bottom of the cell because the cell specifier includes the `.>` operator.
|There isn't a vertical alignment operator on this cell specifier, so the cell falls back to the alignment assigned via the column specifier or the default vertical alignment.
Content is aligned to the top of the cell by default.
|==="#,
    );

    assert_css(&bottom, "td.tableblock.halign-left.valign-bottom", 1);
    assert_css(&bottom, "td.tableblock.halign-left.valign-top", 1);

    // Placing `.>` after a row span (`.2+`) bottom-aligns the spanning cell,
    // which renders with `rowspan="2"`.
    let vspan = convert(
        r#"|===
|Column Name |Column Name

|There isn't a vertical alignment operator on this cell specifier, so the content is aligned to the top of the cell by default.

.2+.>|This cell spans two rows, and its content is aligned to the bottom because the cell specifier includes the `.>` operator.

|This content is aligned to the top of the cell by default.
|==="#,
    );

    assert_css(&vspan, "td.tableblock.halign-left.valign-bottom", 1);
    assert_css(&vspan, "td[rowspan=\"2\"]", 1);
    assert_css(&vspan, "td.tableblock.halign-left.valign-top", 2);
}

#[test]
fn center_vertically() {
    verifies!(
        r#"
=== Center content vertically in a cell

To vertically center the content in a cell, place the `+.^+` operator in front of the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).
Don't insert any spaces between the `|` and the operator.

.Center the content of a cell vertically
[source#ex-vcenter]
----
include::example$align-cell.adoc[tag=vcenter]
----

The table from <<ex-vcenter>> is rendered below.

.Result of <<ex-vcenter>>
include::example$align-cell.adoc[tag=vcenter]

"#
    );

    // The `.^` operator centers the cell's content vertically; the unmarked cell
    // keeps the default top alignment.
    let vcenter = convert(
        r#"|===
|Column Name |Column Name

.^|This content is vertically centered because the cell specifier includes the `+.^+` operator.
|There isn't a vertical alignment operator on this cell specifier, so the cell falls back to the default vertical alignment.
Content is aligned to the top of the cell by default.
|==="#,
    );

    assert_css(&vcenter, "td.tableblock.halign-left.valign-middle", 1);
    assert_css(&vcenter, "td.tableblock.halign-left.valign-top", 1);
}

#[test]
fn combine_horizontal_and_vertical() {
    verifies!(
        r#"
== Apply horizontal and vertical alignment operators to the same cell

A cell can have a vertical and horizontal alignment operator included in its cell specifier.
The <<horizontal-operators,horizontal operator>> always precedes the <<vertical-operators,vertical operator>>.

.Align cells horizontally and vertically
[source#ex-combine]
----
include::example$align-cell.adoc[tag=combine]
----

The table from <<ex-combine>> is rendered below.

.Result <<ex-combine>>
include::example$align-cell.adoc[tag=combine]
"#
    );

    // A cell specifier can carry both a horizontal and a vertical operator (the
    // horizontal one first), and both survive alongside span/duplication: `^.>`
    // is center/bottom, `>.^` is right/middle, the `2.3+^.^` cell spans two
    // columns and three rows centered both ways, and `3*.>` duplicates a
    // bottom-aligned cell into three rows.
    let combine = convert(
        r#"|===
|Column 1 |Column 2 |Column 3

^.>|The specifier for this cell is `^.>`.
The content is centered horizontally and aligned to the bottom of the cell.
|There aren't any alignment operators on this cell's specifier, so the cell falls back to the default alignments.
The default horizontal alignment is the left side of the cell.
The default vertical alignment is the top of the cell.
>.^|The specifier for this cell is `>.^`.
The content is aligned to the right side of the cell and centered vertically.

2.3+^.^|The specifier for this cell is `pass:[2.3+^.^]`.
It spans two columns and three rows.

Its content is centered horizontally and vertically.
3*.>|The specifier for this cell is `3*.>`.
The cell is duplicated in three consecutive rows in the same column.
It's content is aligned to the bottom of the cell.
|==="#,
    );

    assert_css(&combine, "td.tableblock.halign-center.valign-bottom", 1);
    assert_css(&combine, "td.tableblock.halign-right.valign-middle", 1);
    assert_css(&combine, "td.tableblock.halign-center.valign-middle", 1);
    assert_css(&combine, "td[colspan=\"2\"][rowspan=\"3\"]", 1);
    assert_css(&combine, "td.tableblock.halign-left.valign-bottom", 3);
}
