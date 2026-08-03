//! Coverage of the AsciiDoc language description's *Format Content by Cell*
//! page.
//!
//! The page teaches the cell style operators. A cell specifier ends with a
//! style operator (`a`, `d`, `e`, `h`, `l`, `m`, `s`); the operator selects how
//! the cell's content is rendered, a cell operator overrides the column's
//! operator, and inline markup applies in addition to the operator. Each of
//! those rendering claims is verified through `convert`; the headings, the
//! `include`d operator table, and the prose about the nested-document semantics
//! of an AsciiDoc (`a`) cell carry no HTML rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/format-cell-content.adoc");

// Title and section heading only.
non_normative!(
    r#"
= Format Content by Cell

== Cell styles and their operators

"#
);

// Descriptive prose plus the `include` of the style-operators reference table
// from a partial; the table is not expanded here and each operator is verified
// through the examples further down the page.
non_normative!(
    r#"
You can style all of the content in an individual cell by adding a style operator to the xref:add-cells-and-rows.adoc#specifiers[cell's specifier].

include::partial$style-operators.adoc[]

When a style operator isn't explicitly assigned to a cell specifier (or xref:format-column-content.adoc[column specifier]), the cell falls back to the default (`d`) style and is processed as regular paragraph text.
(The explicit `d` style is only needed if you want to revert the cell style based to a normal (default) cell when a style is applied to the column.)

"#
);

// Section heading only.
non_normative!(
    r#"
== Apply a style to a table cell

"#
);

#[test]
fn apply_a_style_to_a_cell() {
    verifies!(
        r#"
The style operator is always entered last in a xref:add-cells-and-rows.adoc#specifiers[cell specifier].
Don't insert any spaces between the `|` and the operator.

====
<factor><span or duplication operator><horizontal alignment operator><vertical alignment operator><**style operator**>|<cell's content>
====

Let's apply a style operator to each cell in <<ex-cell-styles>>.

.Apply a style operator to a cell
[source#ex-cell-styles]
----
include::example$cell.adoc[tag=styles]
----

The table from <<ex-cell-styles>> is rendered below.

.Result of <<ex-cell-styles>>
include::example$cell.adoc[tag=styles]

"#
    );

    let output = convert(
        r#"|===
|Column 1 |Column 2

2*>m|This content is duplicated across two columns (2*) and aligned to the right side of the cell (>).

It's rendered using a monospace font (m).

.3+^.>s|This cell spans 3 rows (`3+`).
The content is centered horizontally (`+^+`), vertically aligned to the bottom of the cell (`.>`), and styled as strong (`s`).
e|This content is italicized (`e`).

m|This content is rendered using a monospace font (m).

s|This content is bold (`s`).
|==="#,
    );

    // The first row entered on the line after the delimiter becomes the header.
    assert_css(&output, "th.tableblock", 2);

    // The `2*>m` specifier duplicates the cell into two right-aligned columns,
    // each paragraph rendered in a monospace `<code>`.
    assert_css(&output, "td.halign-right", 2);
    assert_css(&output, "td.halign-right > p.tableblock > code", 4);

    // The `.3+^.>s` specifier spans three rows, centers horizontally, aligns to
    // the bottom, and renders the content strong.
    assert_css(&output, "td[rowspan=\"3\"]", 1);
    assert_css(&output, "td.halign-center.valign-bottom", 1);

    // The `e` operator italicizes; the two `s` operators (the spanning cell and
    // the last cell) render strong.
    assert_css(&output, "em", 1);
    assert_css(&output, "strong", 2);
}

// Section heading (with an anchor) only.
non_normative!(
    r#"
[#override-column-style]
== Override the column style on a cell

"#
);

#[test]
fn override_the_column_style_on_a_cell() {
    verifies!(
        r#"
When you assign a style operator to a cell, it overrides the xref:format-column-content.adoc[column's style operator].
In <<ex-override>>, the style operator assigned to the first column is overridden on two cells.
The header row also overrides style operators.
However, inline formatting markup is applied in addition to the style specified by an operator.

.Override the column style using a cell style operator
[source#ex-override]
----
[cols="m,m"] <.>
|===
|Column 1, header row |Column 2, header row <.>

|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.

s|This content is rendered as bold paragraph text because the `s` operator in the cell's specifier overrides the style operator in the column specifier. <.>
|*This content is rendered using a monospace font because the column's specifier includes the `m` operator.
It's also bold because it's marked up with the inline syntax for bold formatting.* <.>

d|This content is rendered as regular paragraph text because the `d` operator in the cell's specifier overrides the style operator in the column specifier. <.>
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|===
----
<.> The monospace operator (`m`) is assigned to both columns.
<.> The header row ignores any style operators assigned via column or cell specifiers.
<.> The strong operator (`s`) is assigned to this cell's specifier, overriding the column's monospace style.
<.> Inline formatting is applied in addition to the style assigned via a column specifier.
<.> The default operator (`d`) is assigned to this cell's specifier, resetting it to the default text style.

The table from <<ex-override>> is displayed below.

.Result of <<ex-override>>
[cols="m,m"]
|===
|Column 1, header row |Column 2, header row

|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.

s|This content is rendered as bold paragraph text because the `s` operator in the cell's specifier overrides the style operator in the column specifier.
|*This content is rendered using a monospace font because the column's specifier includes the `m` operator.
It's also bold because it's marked up with the inline syntax for bold formatting.*

d|This content is rendered as regular paragraph text because the `d` operator in the cell's specifier overrides the style operator in the column specifier.
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|===

"#
    );

    let output = convert(
        r#"[cols="m,m"]
|===
|Column 1, header row |Column 2, header row

|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.

s|This content is rendered as bold paragraph text because the `s` operator in the cell's specifier overrides the style operator in the column specifier.
|*This content is rendered using a monospace font because the column's specifier includes the `m` operator.
It's also bold because it's marked up with the inline syntax for bold formatting.*

d|This content is rendered as regular paragraph text because the `d` operator in the cell's specifier overrides the style operator in the column specifier.
|This content is rendered using a monospace font because the column's specifier includes the `m` operator.
|==="#,
    );

    // The header row ignores the column's `m` operator: its cells are plain
    // header cells with no monospace wrapper.
    assert_css(&output, "thead th.tableblock", 2);
    assert_css(&output, "thead th code", 0);

    // Cells that don't override the column render in the column's monospace
    // style: the whole content is wrapped in a block-level `<code>`.
    assert_xpath(
        &output,
        "//td/p/code[contains(text(), \"rendered using a monospace font\")]",
        3,
    );

    // The `s` operator on a cell overrides the column's `m`, rendering the
    // content strong (not monospace).
    assert_xpath(
        &output,
        "//td/p/strong[contains(text(), \"bold paragraph text\")]",
        1,
    );

    // Inline bold markup applies in addition to the column's `m` operator:
    // `<strong>` nests inside the monospace `<code>`.
    assert_css(&output, "td > p.tableblock > code > strong", 1);

    // The `d` operator resets the cell to the default paragraph style: its text
    // is a direct child of the paragraph, not wrapped in `<code>`.
    assert_xpath(
        &output,
        "//td/p[contains(text(), \"regular paragraph text\")]",
        1,
    );
}

// Section heading (with an anchor) only.
non_normative!(
    r#"
[#a-operator]
== Use AsciiDoc block elements in a table cell

"#
);

#[test]
fn use_asciidoc_block_elements_in_a_cell() {
    verifies!(
        r#"
To use AsciiDoc block elements, such as delimited source blocks and lists, in a cell, place the `a` operator directly in front of the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).
Don't insert any spaces between the `|` and the operator.
The `a` can also be specified on the column in the `cols` attribute on the table.

.Apply the AsciiDoc block style operator to two cells
[source#ex-asciidoc]
....
include::example$cell.adoc[tag=adoc]
....

The table from <<ex-asciidoc>> is rendered below.

.Result of <<ex-asciidoc>>
include::example$cell.adoc[tag=adoc]

"#
    );

    let output = convert(
        r#"|===
|Normal Style |AsciiDoc Style

|This cell isn't prefixed with an `a`, so the processor doesn't interpret the following lines as an AsciiDoc list.

* List item 1
* List item 2
* List item 3

a|This cell is prefixed with an `a`, so the processor interprets the following lines as an AsciiDoc list.

* List item 1
* List item 2
* List item 3

|This cell isn't prefixed with an `a`, so the processor doesn't interpret the listing block delimiters or the `source` style.

[source,python]
----
import os
print ("%s" %(os.uname()))
----

a|This cell is prefixed with an `a`, so the listing block is processed and rendered according to the `source` style rules.

[source,python]
----
import os
print "%s" %(os.uname())
----

|==="#,
    );

    // The two `a` cells each create a nested document rendered inside a
    // `<div class="content">`.
    assert_css(&output, "td > div.content", 2);

    // The `a` list cell interprets the AsciiDoc list; the `a` source cell
    // interprets the listing block.
    assert_css(&output, "td div.content ul li", 3);
    assert_css(&output, "td div.content pre", 1);

    // The cells without the `a` operator keep the list and listing markup as
    // literal paragraph text, so only the `a` cells produce a list and a
    // listing block.
    assert_css(&output, "ul", 1);
    assert_css(&output, "pre", 1);
}

// Descriptive prose about the nested-document semantics of an AsciiDoc (`a`)
// cell: attribute inheritance and scoping, and where a preprocessor directive
// may appear. No HTML rendering rule to verify here (and line 111 carries an
// author's editorial note).
non_normative!(
    r#"
An AsciiDoc table cell effectively creates a nested document.
As such, it inherits attributes from the parent document.
If an attribute is set or explicitly unset in the parent document, it _cannot_ be modified in the AsciiDoc table cell.
There are a handful of exceptions to this rule, which includes doctype, toc, notitle (and its complement, showtitle), and compat-mode.
Any newly defined attributes in the AsciiDoc table do not impact the attributes in the parent document.
Instead, these attributes are scoped to the table cell.

If the AsciiDoc table cell starts with a preprocessor directive, that directive should be placed on the line after the cell separator.
While it can be placed on the same line as the cell separator, that style is not recommended.
That's because the preprocessor directive that starts after the cell separator must be treated with special handling and is thus limited to a single line (for example, a multiline preprocessor conditional is not allow in this case).
By starting the contents of the AsciiDoc table cell on the line after the cell separator, the contents will be parsed as normal.
"#
);
