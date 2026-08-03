//! Coverage of this crate's *Table Cell Formatting* documentation page.
//!
//! The page shows how `asciidoc-html5` renders cell specifiers: style letters,
//! horizontal and vertical alignment, column and row spans, cell duplication,
//! and AsciiDoc (`a`) cells. Each shown source-and-output pair is verified by
//! converting the source through `convert`; the surrounding prose is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/table-cell-formatting.adoc");

// Title, description, the intro paragraph, the NOTE admonition, and the first
// section heading are descriptive framing with no rendering rule to verify.
non_normative!(
    r#"
= Table Cell Formatting
:navtitle: Cell Formatting
:description: Cell styles, alignment, spans, and duplication in asciidoc-html5 tables.

A cell specifier – the operators between the `|` and the cell content – controls
a cell's style, alignment, and how many columns or rows it spans.
`asciidoc-html5` renders the same cell markup as Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Cell styles

"#
);

// Each style letter wraps the cell content in the matching element – `<em>`,
// `<strong>`, `<code>`, or a literal `<pre>`.
#[test]
fn cell_styles() {
    verifies!(
        r#"
A style letter before the `|` sets how the cell's content is rendered: `e`
(emphasis), `s` (strong), `m` (monospace), and `l` (literal) are the common ones:

[,asciidoc]
----
|===
|Style |Example

e|emphasis
s|strong

m|monospace
l|literal
|===
----

Each style wraps the content in the matching element – `<em>`, `<strong>`,
`<code>`, or a literal `<pre>`:

[,html]
----
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock"><em>emphasis</em></p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock"><strong>strong</strong></p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock"><code>monospace</code></p></td>
<td class="tableblock halign-left valign-top"><div class="literal"><pre>literal</pre></div></td>
</tr>
</tbody>
----

"#
    );

    let output =
        convert("|===\n|Style |Example\n\ne|emphasis\ns|strong\n\nm|monospace\nl|literal\n|===\n");

    assert_xpath(&output, r#"//td/p/em[text()="emphasis"]"#, 1);
    assert_xpath(&output, r#"//td/p/strong[text()="strong"]"#, 1);
    assert_xpath(&output, r#"//td/p/code[text()="monospace"]"#, 1);
    assert_xpath(
        &output,
        r#"//td/div[@class="literal"]/pre[text()="literal"]"#,
        1,
    );
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Alignment

"#
);

// Horizontal and vertical operators become `halign-` and `valign-` classes on
// the cell.
#[test]
fn alignment() {
    verifies!(
        r#"
Horizontal operators are `^` (center) and `>` (right); a vertical operator is
prefixed with a dot, such as `.^` (middle). They become `halign-` and `valign-`
classes on the cell:

[,asciidoc]
----
[cols="1,1,1"]
|===
^|centered
>|right
.^|middle
|===
----

[,html]
----
<tbody>
<tr>
<td class="tableblock halign-center valign-top"><p class="tableblock">centered</p></td>
<td class="tableblock halign-right valign-top"><p class="tableblock">right</p></td>
<td class="tableblock halign-left valign-middle"><p class="tableblock">middle</p></td>
</tr>
</tbody>
----

"#
    );

    let output = convert("[cols=\"1,1,1\"]\n|===\n^|centered\n>|right\n.^|middle\n|===\n");

    assert_xpath(
        &output,
        r#"//td[@class="tableblock halign-center valign-top"]/p[text()="centered"]"#,
        1,
    );

    assert_xpath(
        &output,
        r#"//td[@class="tableblock halign-right valign-top"]/p[text()="right"]"#,
        1,
    );

    assert_xpath(
        &output,
        r#"//td[@class="tableblock halign-left valign-middle"]/p[text()="middle"]"#,
        1,
    );
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Span columns and rows

"#
);

// Span operators become `colspan` and `rowspan` attributes on the cell.
#[test]
fn span_columns_and_rows() {
    verifies!(
        r#"
A span operator merges a cell across neighbors: `N+` spans `N` columns, and
`.N+` spans `N` rows. Here the first row is an implicit header:

[,asciidoc]
----
|===
|A |B |C

2+|spans two columns
|C1

.2+|spans two rows
|B1
|C2

|B2
|C3
|===
----

Spans become `colspan` and `rowspan` attributes on the cell:

[,html]
----
<tbody>
<tr>
<td class="tableblock halign-left valign-top" colspan="2"><p class="tableblock">spans two columns</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">C1</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top" rowspan="2"><p class="tableblock">spans two rows</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">B1</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">C2</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">B2</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">C3</p></td>
</tr>
</tbody>
----

"#
    );

    let output =
        convert("|===\n|A |B |C\n\n2+|spans two columns\n|C1\n\n.2+|spans two rows\n|B1\n|C2\n\n|B2\n|C3\n|===\n");

    assert_xpath(
        &output,
        r#"//td[@colspan="2"]/p[text()="spans two columns"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//td[@rowspan="2"]/p[text()="spans two rows"]"#,
        1,
    );
    assert_xpath(&output, r#"//tbody//td/p[text()="C3"]"#, 1);
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Duplicate a cell

"#
);

// The `N*` duplication operator repeats a cell's content across `N` consecutive
// cells.
#[test]
fn duplicate_a_cell() {
    verifies!(
        r#"
The duplication operator `N*` repeats a cell's content across `N` consecutive
cells:

[,asciidoc]
----
[cols="1,1,1"]
|===
3*|same in every column
|===
----

[,html]
----
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">same in every column</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">same in every column</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">same in every column</p></td>
</tr>
</tbody>
----

"#
    );

    let output = convert("[cols=\"1,1,1\"]\n|===\n3*|same in every column\n|===\n");

    assert_css(&output, "tbody > tr > td.tableblock", 3);
    assert_xpath(&output, r#"//td/p[text()="same in every column"]"#, 3);
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== AsciiDoc cells

"#
);

// An `a` cell renders its block content as a nested document inside a `<div
// class="content">`.
#[test]
fn asciidoc_cells() {
    verifies!(
        r#"
An `a` cell holds full block content – lists, delimited blocks, and block macros
– rendered as a nested document inside a `<div class="content">`:

[,asciidoc]
----
[cols="1,1"]
|===
|Plain |Rich

|no block content
a|A list:

* one
* two
|===
----

[,html]
----
<td class="tableblock halign-left valign-top"><div class="content"><div class="paragraph">
<p>A list:</p>
</div>
<div class="ulist">
<ul>
<li>
<p>one</p>
</li>
<li>
<p>two</p>
</li>
</ul>
</div></div></td>
----

"#
    );

    let output =
        convert("[cols=\"1,1\"]\n|===\n|Plain |Rich\n\n|no block content\na|A list:\n\n* one\n* two\n|===\n");

    assert_css(&output, "td > div.content", 1);
    assert_xpath(
        &output,
        r#"//td/div[@class="content"]/div[@class="paragraph"]/p[text()="A list:"]"#,
        1,
    );
    assert_css(&output, "td div.content div.ulist > ul > li", 2);
    assert_xpath(
        &output,
        r#"//td//div[@class="ulist"]//li/p[text()="one"]"#,
        1,
    );
}

// Closing summary and a cross-reference to column widths; no rendering rule to
// verify.
non_normative!(
    r#"
== Known limitations

Cell formatting is fully supported: the `a`, `d`, `e`, `h`, `l`, `m`, and `s`
styles, horizontal and vertical alignment, column and row spans, and cell
duplication all match Asciidoctor's output. A style, alignment, or width set on a
whole column through `cols` applies to every cell in that column; see
xref:table-column-widths.adoc[].
"#
);
