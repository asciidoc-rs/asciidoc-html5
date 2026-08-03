//! Coverage of the AsciiDoc language description's `align-by-column` page.
//!
//! The page teaches the column alignment operators: horizontal operators (`<`,
//! `>`, `^`) and vertical operators (`.<`, `.>`, `.^`) placed on a column
//! specifier in the `cols` attribute, applied to every cell in the column and
//! combinable with a column width and a multiplier. Each rendered example is
//! verified through `convert`, which emits the alignment as
//! `halign-*`/`valign-*` classes on both the header (`<th>`) and body (`<td>`)
//! cells. The section headings, the operator-definition lists, and the
//! operator-order bullet lists carry no rendering rule of their own and are
//! tracked as non-normative.
//!
//! A column's alignment propagates to its header cells as well as its body
//! cells, matching Asciidoctor 2.0.26. This relies on the fix for
//! asciidoc-parser issue #1061 (<https://github.com/asciidoc-rs/asciidoc-parser/issues/1061>),
//! released in `asciidoc-parser` 0.29.8: before that fix a header cell's
//! alignment defaulted to `Left`/`Top` regardless of the column's operator.
//! The multiplier examples below therefore assert the header-cell alignment
//! alongside the body-cell alignment.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/align-by-column.adoc");

// Title, editorial comment, and introductory description: no rendering rule to
// verify.
non_normative!(
    r#"
= Align Content by Column
// Using Wikipedia's names for the operators. For reference, see https://en.wikipedia.org/wiki/Less-than_sign

The alignment operators allow you to horizontally and vertically align a column's content.
They're applied to a column specifier and xref:add-columns.adoc#cols-attribute[assigned to the cols attribute].

"#
);

// The horizontal-operators section heading, the operator definitions, and the
// operator-order bullet list: descriptive definitions with no example to
// render.
non_normative!(
    r#"
[#horizontal-operators]
== Horizontal alignment operators

Content can be horizontally aligned to the left or right side of the column as well as the center of the column.

Flush left operator (<):: The less-than sign (`<`) left aligns the content.
This is the default horizontal alignment.
Flush right operator (>):: The greater-than sign (`>`) right aligns the content.
Center operator (^):: The caret (`+^+`) centers the content.

A horizontal alignment operator is entered in front a <<vertical-operators,vertical alignment operator>> (if present) and in front of a xref:adjust-column-widths.adoc[column's width] (if present).
If the number of columns is assigned using a multiplier (`+<n>*+`), the horizontal alignment operator is placed directly after the multiplier operator (`+*+`).

* `[cols="2,pass:q[#^#]1"]` A horizontal alignment operator is placed in front of the column width.
* `[cols="pass:q[#>#].^1,2"]` A horizontal alignment operator is placed in front of a vertical alignment operator.
* `[cols="pass:q[#>#],pass:q[#^#]"]` When a column width isn't specified, a horizontal alignment operator can represent both the column and the column content's alignment.
* `[cols="3*pass:q[#>#]"]` The horizontal alignment operator is placed directly after a multiplier.

"#
);

#[test]
fn center_horizontally() {
    verifies!(
        r#"
=== Center content horizontally in a column

To horizontally center the content in a column, place the `+^+` operator at the beginning of the xref:add-columns.adoc#col-specifier[column's specifier].

.Center column content horizontally
[source#ex-horizontal]
----
[cols="^4,1"]
|===
|This content is horizontally centered.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|===
----

The table from <<ex-horizontal>> is rendered below.

.Result of <<ex-horizontal>>
[cols="^4,1"]
|===
|This content is horizontally centered.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|===

When the columns are specified using the xref:add-columns.adoc#column-multiplier[multiplier], place the `+^+` operator after the multiplier operator (`+*+`).

.Horizontal alignment and multiplier operator order
[source#ex-horizontal-multiplier]
----
[cols="2*^",options=header]
|===
|Column name
|Column name

|This content is horizontally centered.
|This content is also horizontally centered.
|===
----

The table from <<ex-horizontal-multiplier>> is rendered below.

.Result of <<ex-horizontal-multiplier>>
[cols="2*^",options=header]
|===
|Column name
|Column name

|This content is horizontally centered.
|This content is also horizontally centered.
|===

"#
    );

    // A `^` at the front of a column specifier center-aligns every cell in that
    // column; the second column, with no operator, stays left-aligned.
    let horizontal = convert(
        r#"[cols="^4,1"]
|===
|This content is horizontally centered.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|==="#,
    );

    assert_css(&horizontal, "td.tableblock.halign-center.valign-top", 1);
    assert_css(&horizontal, "td.tableblock.halign-left.valign-top", 1);

    // With the columns declared via a multiplier, `2*^` center-aligns both
    // columns' cells; the alignment reaches the header cells too, not only the
    // body cells.
    let multiplier = convert(
        r#"[cols="2*^",options=header]
|===
|Column name
|Column name

|This content is horizontally centered.
|This content is also horizontally centered.
|==="#,
    );

    assert_css(&multiplier, "th.tableblock.halign-center.valign-top", 2);
    assert_css(&multiplier, "td.tableblock.halign-center.valign-top", 2);
}

#[test]
fn right_align() {
    verifies!(
        r#"
=== Right align content in a column

To align the content in a column to its right side, place the `+>+` operator in front of the column's specifier.

.Right align column content
[source#ex-right]
----
[cols=">4,1"]
|===
|This content is aligned to the right side of the column.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|===
----

The table <<ex-right>> is rendered below.

.Result of <<ex-right>>
[cols=">4,1"]
|===
|This content is aligned to the right side of the column.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|===

When the columns are specified using the xref:add-columns.adoc#column-multiplier[multiplier], place the `+>+` operator after the multiplier operator (`+*+`).

.Right alignment and multiplier operator order
[source#ex-right-multiplier]
----
[cols="2*>",options=header]
|===
|Column name
|Column name

|This content is aligned to the right side of the column.
|This content is also aligned to the right side of the column.
|===
----

The table from <<ex-right-multiplier>> is rendered below.

.Result of <<ex-right-multiplier>>
[cols="2*>",options=header]
|===
|Column name
|Column name

|This content is aligned to the right side of the column.
|This content is also aligned to the right side of the column.
|===

"#
    );

    // A `>` at the front of a column specifier right-aligns every cell in that
    // column; the second column stays left-aligned.
    let right = convert(
        r#"[cols=">4,1"]
|===
|This content is aligned to the right side of the column.
|There isn't a horizontal alignment operator on this column's specifier, so the column falls back to the default horizontal alignment.
Content is left-aligned by default.
|==="#,
    );

    assert_css(&right, "td.tableblock.halign-right.valign-top", 1);
    assert_css(&right, "td.tableblock.halign-left.valign-top", 1);

    // With the columns declared via a multiplier, `2*>` right-aligns both
    // columns' cells, including the header cells.
    let multiplier = convert(
        r#"[cols="2*>",options=header]
|===
|Column name
|Column name

|This content is aligned to the right side of the column.
|This content is also aligned to the right side of the column.
|==="#,
    );

    assert_css(&multiplier, "th.tableblock.halign-right.valign-top", 2);
    assert_css(&multiplier, "td.tableblock.halign-right.valign-top", 2);
}

// The vertical-operators section heading, the operator definitions, and the
// operator-order bullet list: descriptive definitions with no example to
// render.
non_normative!(
    r#"
[#vertical-operators]
== Vertical alignment operators

Content can be vertically aligned to the top or bottom of a column's cells as well as the center of a column.
Vertical alignment operators always begin with a dot (`.`).

Flush top operator (.<):: The dot and less-than sign (`.<`) aligns the content to the top of the column's cells.
This is the default vertical alignment.
Flush bottom operator (.>):: The dot and greater-than sign (`.>`) aligns the content to the bottom of the column's cells.
Center operator (.^):: The dot and caret (`+.^+`) centers the content vertically.

A vertical alignment operator is entered directly after a <<horizontal-operators,horizontal alignment operator>> (if present) and before a xref:adjust-column-widths.adoc[column's width] (if present).
If the number of columns is assigned using a multiplier (`+<n>*+`), the vertical alignment operator is placed directly after the horizontal alignment operator (if present).
Otherwise, it's placed directly after the multiplier operator (`+*+`).

* `[cols="2,pass:q[#.^#]1"]` A vertical alignment operator is placed in front of the column width.
* `[cols=">pass:q[#.^#]1,2"]` The vertical alignment operator is placed after the horizontal alignment operator but before the column width.
* `[cols="pass:q[#.^#],pass:q[#.>#]"]` When a column width doesn't need to be specified, a vertical alignment operator can represent both the column and the column content's alignment.
* `[cols="3*pass:q[#.>#]"]` The vertical alignment operator is placed directly after a multiplier unless there is a horizontal alignment operator.
Then it's placed after the horizontal alignment operator, (e.g., `[cols="3*^pass:q[#.>#]"]`)

"#
);

#[test]
fn align_bottom() {
    verifies!(
        r#"
=== Align content to the bottom of a column's cells

To align the content in a column to the bottom of each cell, place the `+.>+` operator directly in front of the xref:adjust-column-widths.adoc[column's width].

.Bottom align column content
[source#ex-bottom]
----
[cols=".>2,1"]
|===
|This content is vertically aligned to the bottom of the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|===
----

The table from <<ex-bottom>> is rendered below.

.Result of <<ex-bottom>>
[cols=".>2,1"]
|===
|This content is vertically aligned to the bottom of the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|===

"#
    );

    // A `.>` at the front of a column specifier bottom-aligns every cell in that
    // column; the second column stays top-aligned.
    let bottom = convert(
        r#"[cols=".>2,1"]
|===
|This content is vertically aligned to the bottom of the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|==="#,
    );

    assert_css(&bottom, "td.tableblock.halign-left.valign-bottom", 1);
    assert_css(&bottom, "td.tableblock.halign-left.valign-top", 1);
}

#[test]
fn center_vertically() {
    verifies!(
        r#"
=== Center content vertically in a column

To vertically center the content in a column, place the `+.^+` operator directly in front of the xref:adjust-column-widths.adoc[column's width].

.Center column content vertically
[source#ex-vertical]
----
[cols=".^2,1"]
|===
|This content is centered vertically in the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|===
----

The table from <<ex-vertical>> is rendered below.

.Result of <<ex-vertical>>
[cols=".^2,1"]
|===
|This content is centered vertically in the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|===

To vertically align the content to the middle of the cells in all of the columns, enter the  `.^` operator after the xref:add-columns.adoc#column-multiplier[multiplier].

.Vertical alignment and multiplier operator order
[source#ex-vertical-multiplier]
----
[cols="2*.^",options=header]
|===
|Column name
|Column name

|This content is vertically centered.
|This content is also vertically centered.
|===
----

The table from <<ex-vertical-multiplier>> is rendered below.

.Result of <<ex-vertical-multiplier>>
[cols="2*.^",options=header]
|===
|Column name
|Column name

|This content is centered vertically in the cell.
|This content is also centered vertically in the cell.
|===

"#
    );

    // A `.^` in a column specifier vertically centers every cell in that column;
    // the second column stays top-aligned.
    let vertical = convert(
        r#"[cols=".^2,1"]
|===
|This content is centered vertically in the cell.
|There isn't a vertical alignment operator on this column's specifier, so the column falls back to the default vertical alignment.
Content is top-aligned by default.
|==="#,
    );

    assert_css(&vertical, "td.tableblock.halign-left.valign-middle", 1);
    assert_css(&vertical, "td.tableblock.halign-left.valign-top", 1);

    // With the columns declared via a multiplier, `2*.^` vertically centers both
    // columns' cells, header cells included.
    let multiplier = convert(
        r#"[cols="2*.^",options=header]
|===
|Column name
|Column name

|This content is centered vertically in the cell.
|This content is also centered vertically in the cell.
|==="#,
    );

    assert_css(&multiplier, "th.tableblock.halign-left.valign-middle", 2);
    assert_css(&multiplier, "td.tableblock.halign-left.valign-middle", 2);
}

// Operator-order note for combining a horizontal operator with the vertical
// operator on a multiplier: descriptive placement rule with no example to
// render.
non_normative!(
    r#"
When a horizontal alignment operator is also applied to the multiplier, then the vertical alignment operator is placed directly after the horizontal operator (e.g., `[cols="2*>.^"]`).

"#
);

#[test]
fn combine_horizontal_and_vertical() {
    verifies!(
        r#"
== Apply horizontal and vertical alignment operators to the same column

A column can have a vertical and horizontal alignment operator placed on its xref:add-columns.adoc#col-specifier[specifier].
The <<horizontal-operators,horizontal operator>> always precedes the <<vertical-operators,vertical operator>>.
Both operators precede the column width.
When a xref:add-columns.adoc#column-multiplier[multiplier] is used, the operators are placed after the multiplier.

.Horizontally and vertically align column content
[source#ex-center]
----
[cols="^.>2,1,>.^1"]
|===
|Column name |Column name |Column name

|This content is centered horizontally and aligned to the bottom
of the cell.
|There aren't any alignment operators on this column's specifier,
so the column falls back to the default alignments.
The default horizontal alignment is left-aligned.
The default vertical alignment is top-aligned.
|This content is aligned to the right side of the cell and
centered vertically.
|===
----

The table from <<ex-center>> is rendered below.

.Result of <<ex-center>>
[cols="^.>2,1,>.^1"]
|===
|Column name |Column name |Column name

|This content is centered horizontally and aligned to the bottom
of the cell.
|There aren't any alignment operators on this column's specifier,
so the column falls back to the default alignments.
The default horizontal alignment is left-aligned.
The default vertical alignment is top-aligned.
|This content is aligned to the right side of the cell and
centered vertically.
|===

"#
    );

    // Each column specifier carries a horizontal operator before a vertical one:
    // `^.>` centers horizontally and bottom-aligns, the unmarked middle column
    // stays left/top, and `>.^` right-aligns and centers vertically.
    let center = convert(
        r#"[cols="^.>2,1,>.^1"]
|===
|Column name |Column name |Column name

|This content is centered horizontally and aligned to the bottom
of the cell.
|There aren't any alignment operators on this column's specifier,
so the column falls back to the default alignments.
The default horizontal alignment is left-aligned.
The default vertical alignment is top-aligned.
|This content is aligned to the right side of the cell and
centered vertically.
|==="#,
    );

    assert_css(&center, "td.tableblock.halign-center.valign-bottom", 1);
    assert_css(&center, "td.tableblock.halign-left.valign-top", 1);
    assert_css(&center, "td.tableblock.halign-right.valign-middle", 1);
}

// Cross-reference to the cell-level operators, which override a column's
// alignment: a pointer to `align-by-cell.adoc` with no example to render here.
non_normative!(
    r#"
IMPORTANT: If there is an xref:align-by-cell.adoc[alignment operator on a cell's specifier], it will override the column's alignment operator.
"#
);
