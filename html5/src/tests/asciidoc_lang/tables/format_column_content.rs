//! Coverage of the AsciiDoc language description's *Format Content by Column*
//! page.
//!
//! The page teaches the column style operators. A style operator placed last on
//! a column specifier styles every cell in that column (`h` header, `m`
//! monospace, `s` strong, `e` emphasis, `a` AsciiDoc block content); the header
//! row is unaffected and inline markup still applies. Those rendering claims
//! are verified through `convert`; the headings, the `include`d operator table,
//! and the cross-references carry no HTML rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/format-column-content.adoc");

// Title and intro prose.
non_normative!(
    r#"
= Format Content by Column

A column style operator is applied to a column specifier and xref:add-columns.adoc#cols-attribute[assigned to the cols attribute].

"#
);

// Section heading (with an anchor) only.
non_normative!(
    r#"
[#cols-style]
== Column styles and their operators

"#
);

// Descriptive prose plus the `include` of the style-operators reference table
// from a partial; the table is not expanded here and each operator is verified
// through the examples further down the page.
non_normative!(
    r#"
You can style all of the content in a column by adding a style operator to a column's specifier.

include::partial$style-operators.adoc[]

When a style operator isn't explicitly applied to a column specifier, the `d` style is assigned automatically and the column is processed as paragraph text.

"#
);

// Section heading only.
non_normative!(
    r#"
== Apply a style operator to a column

"#
);

#[test]
fn apply_a_style_operator_to_a_column() {
    verifies!(
        r#"
A style operator is always placed in the last position on a column's specifier or multiplier.

* `[cols=">pass:q[#e#],.^3pass:q[#s#]"]` A style operator is placed directly after any other operators and the column width in the column's specifier.
* `[cols="pass:q[#h#],pass:q[#e#]"]` When a column width isn't specified, the style operator can represent both the column and the column's content style.
* `[cols="3*.>pass:q[#m#]"]` When a multiplier is present, the style operator is placed after any horizontal and vertical alignment operators.

Let's apply a different style to each column in <<ex-style>>.

.Add a style operator to each column
[source#ex-style]
----
[cols="h,m,s,e"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).
|===
----

The table from <<ex-style>> is displayed below.
Note that the style applied to each column doesn't affect the xref:add-header-row.adoc[header row] or override any inline formatting.

.Result of <<ex-style>>
[cols="h,m,s,e"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).
|===

"#
    );

    let output = convert(
        r#"[cols="h,m,s,e"]
|===
|Column 1 |Column 2 |Column 3 |Column 4

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).

|This column's content and borders are rendered using the table header (`h`) styles.
|This column's content is rendered using a monospace font (m).
|This column's content is bold (`s`).
|This column's content is italicized (`e`).
|==="#,
    );

    // Four equal-width columns and a four-cell header row.
    assert_css(&output, "colgroup col", 4);
    assert_css(&output, "col[style=\"width: 25%;\"]", 4);
    assert_css(&output, "thead th.tableblock", 4);

    // The `h` column applies header semantics to its body cells, rendering them
    // as `<th>` inside the `<tbody>`.
    assert_css(&output, "tbody th", 2);

    // The `m`, `s`, and `e` columns render their body cells as monospace, bold,
    // and italic respectively.
    assert_css(&output, "tbody td > p.tableblock > code", 2);
    assert_css(&output, "tbody td > p.tableblock > strong", 2);
    assert_css(&output, "tbody td > p.tableblock > em", 2);

    // The column style doesn't override inline formatting: the inline monospace
    // markup on the `e` column's `e` still renders as `<code>` within the italic
    // paragraph.
    assert_css(&output, "tbody td > p.tableblock > em > code", 2);
}

// Cross-reference to the cell-level override, covered on the
// format-cell-content page.
non_normative!(
    r#"
Additionally, if a xref:format-cell-content.adoc#override-column-style[cell specifier contains a style operator], that style will override a column's style operator.

"#
);

#[test]
fn use_asciidoc_block_elements_in_a_column() {
    verifies!(
        r#"
== Use AsciiDoc block elements in a column

To use AsciiDoc block elements, such as delimited source blocks and lists, in a column, place the lowercase letter `a` on the column specifier.

.Apply the AsciiDoc block style operator to the first column
[source#ex-asciidoc]
....
[cols="2a,2"]
|===
|Column with the `a` style operator applied to its specifier |Column using the default style

|
* List item 1
* List item 2
* List item 3
|
* List item 1
* List item 2
* List item 3

|
[source,python]
----
import os
print "%s" %(os.uname())
----
|
[source,python]
----
import os
print ("%s" %(os.uname()))
----
|===
....

The AsciiDoc block style effectively creates a nested, standalone AsciiDoc document in each cell in the column.
The parent document's implicit attributes, such as `doctitle`, are shadowed and custom attributes are inherited.
// what does "shadowed" actually mean???

.Result of <<ex-asciidoc>>
[cols="2a,2"]
|===
|Column with the `a` style operator applied to its specifier |Column using the default style

|
* List item 1
* List item 2
* List item 3
|
* List item 1
* List item 2
* List item 3

|
[source,python]
----
import os
print "%s" %(os.uname())
----

|
[source,python]
----
import os
print ("%s" %(os.uname()))
----
|===

"#
    );

    let output = convert(
        r#"[cols="2a,2"]
|===
|Column with the `a` style operator applied to its specifier |Column using the default style

|
* List item 1
* List item 2
* List item 3
|
* List item 1
* List item 2
* List item 3

|
[source,python]
----
import os
print "%s" %(os.uname())
----

|
[source,python]
----
import os
print ("%s" %(os.uname()))
----
|==="#,
    );

    // Two columns and a two-cell header row.
    assert_css(&output, "colgroup col", 2);
    assert_css(&output, "thead th.tableblock", 2);

    // Each body cell of the first column (the `a` column) creates a nested
    // document rendered inside a `<div class="content">`.
    assert_css(&output, "td > div.content", 2);

    // The `a` column interprets the AsciiDoc list and the source listing.
    assert_css(&output, "td div.content ul li", 3);
    assert_css(&output, "td div.content pre", 1);

    // The default column keeps the list and listing markup as literal paragraph
    // text, so only the `a` column produces a list and a listing block.
    assert_css(&output, "ul", 1);
    assert_css(&output, "pre", 1);
}

// Cross-reference to the cell-level AsciiDoc block operator, covered on the
// format-cell-content page.
non_normative!(
    r#"
You can also apply the xref:format-cell-content.adoc#a-operator[AsciiDoc block style operator to individual cells].
"#
);
