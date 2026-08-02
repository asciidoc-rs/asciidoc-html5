//! Coverage of this crate's *Tables* documentation page.
//!
//! The page shows how `asciidoc-html5` renders tables: columns and rows, an
//! implicit header, explicit header and footer rows, and the CSV data format.
//! Each shown source-and-output pair is verified by converting the source
//! through `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/tables.adoc");

// Title, description, the intro paragraph, the NOTE admonition, and the first
// section heading are descriptive framing with no rendering rule to verify.
non_normative!(
    r#"
= Tables
:navtitle: Tables
:description: How asciidoc-html5 renders tables as HTML.

A table is a delimited block bounded by `|===`. Inside it, each cell begins with
a vertical bar (`|`), the `cols` attribute describes the columns, and blank lines
group cells into rows. `asciidoc-html5` renders the same `<table
class="tableblock">` markup as Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Columns and rows

"#
);

// A `cols` table with an implicit header: the first row becomes a `<thead>` of
// `<th>` cells and the rest a `<tbody>`; equal specifiers give equal widths.
#[test]
fn columns_and_rows() {
    verifies!(
        r#"
List one column specifier per column in `cols`, then enter the cells. Entering
every cell of the first row on the line right after `|===` promotes that row to a
header:

[,asciidoc]
----
[cols="1,1,1"]
|===
|Name |Role |Location

|Ada |Engineer |London
|Grace |Admiral |New York
|===
----

The header row becomes a `<thead>` of `<th>` cells and the remaining rows a
`<tbody>`; equal specifiers give equal column widths:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<colgroup>
<col style="width: 33.3333%;">
<col style="width: 33.3333%;">
<col style="width: 33.3334%;">
</colgroup>
<thead>
<tr>
<th class="tableblock halign-left valign-top">Name</th>
<th class="tableblock halign-left valign-top">Role</th>
<th class="tableblock halign-left valign-top">Location</th>
</tr>
</thead>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Ada</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Engineer</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">London</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Grace</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Admiral</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">New York</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output =
        convert("[cols=\"1,1,1\"]\n|===\n|Name |Role |Location\n\n|Ada |Engineer |London\n|Grace |Admiral |New York\n|===\n");

    assert_css(&output, "table.tableblock.frame-all.grid-all.stretch", 1);
    assert_css(&output, "colgroup > col", 3);
    assert_xpath(&output, r#"//col[@style="width: 33.3333%;"]"#, 2);
    assert_xpath(&output, r#"//col[@style="width: 33.3334%;"]"#, 1);
    assert_css(&output, "thead > tr > th.tableblock", 3);
    assert_xpath(&output, r#"//thead//th[text()="Name"]"#, 1);
    assert_css(&output, "tbody > tr", 2);
    assert_xpath(&output, r#"//tbody//td/p[text()="New York"]"#, 1);
}

// The trailing note about cell placement is descriptive; the next heading has
// no rendering rule of its own.
non_normative!(
    r#"
Whether you enter each cell on its own line or several on one line makes no
difference: the processor starts a new cell at every `|`.

== Header and footer rows

"#
);

// The `%header%footer` shorthand promotes the first row to `<thead>` and moves
// the last row into a `<tfoot>`.
#[test]
fn header_and_footer_rows() {
    verifies!(
        r#"
The `header` and `footer` options name the first and last rows explicitly. Use
the shorthand (`%header%footer`), placing it before `cols`:

[,asciidoc]
----
[%header%footer,cols="2,1"]
|===
|Item |Qty

|Apples |3
|Pears |5

|Total |8
|===
----

The last row moves into a `<tfoot>`:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<colgroup>
<col style="width: 66.6666%;">
<col style="width: 33.3334%;">
</colgroup>
<thead>
<tr>
<th class="tableblock halign-left valign-top">Item</th>
<th class="tableblock halign-left valign-top">Qty</th>
</tr>
</thead>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Apples</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">3</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Pears</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">5</p></td>
</tr>
</tbody>
<tfoot>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Total</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">8</p></td>
</tr>
</tfoot>
</table>
----

"#
    );

    let output =
        convert("[%header%footer,cols=\"2,1\"]\n|===\n|Item |Qty\n\n|Apples |3\n|Pears |5\n\n|Total |8\n|===\n");

    assert_xpath(&output, r#"//col[@style="width: 66.6666%;"]"#, 1);
    assert_xpath(&output, r#"//col[@style="width: 33.3334%;"]"#, 1);
    assert_css(&output, "thead > tr > th.tableblock", 2);
    assert_css(&output, "tbody > tr", 2);
    assert_css(&output, "tfoot > tr > td.tableblock", 2);
    assert_xpath(&output, r#"//tfoot//td/p[text()="Total"]"#, 1);
    assert_xpath(&output, r#"//tfoot//td/p[text()="8"]"#, 1);
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Data formats

"#
);

// A `format=csv` table splits each comma into a cell, producing the same table
// markup as the bar syntax.
#[test]
fn data_formats() {
    verifies!(
        r#"
Besides the default bar-separated values, a table can hold comma-separated
(`format=csv`) or delimiter-separated (`format=dsv`) data. Enter the rows without
leading bars:

[,asciidoc]
----
[%header,format=csv]
|===
Artist,Track,Genre
Baauer,Harlem Shake,Hip Hop
|===
----

Each comma starts a new cell, producing the same table markup as the bar syntax:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<colgroup>
<col style="width: 33.3333%;">
<col style="width: 33.3333%;">
<col style="width: 33.3334%;">
</colgroup>
<thead>
<tr>
<th class="tableblock halign-left valign-top">Artist</th>
<th class="tableblock halign-left valign-top">Track</th>
<th class="tableblock halign-left valign-top">Genre</th>
</tr>
</thead>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Baauer</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Harlem Shake</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Hip Hop</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert(
        "[%header,format=csv]\n|===\nArtist,Track,Genre\nBaauer,Harlem Shake,Hip Hop\n|===\n",
    );

    assert_css(&output, "table.tableblock.frame-all.grid-all.stretch", 1);
    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "thead > tr > th.tableblock", 3);
    assert_xpath(&output, r#"//thead//th[text()="Artist"]"#, 1);
    assert_css(&output, "tbody > tr", 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Harlem Shake"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Hip Hop"]"#, 1);
}

// Closing summary and cross-references to the other table pages; no rendering
// rule to verify.
non_normative!(
    r#"
== Known limitations

Tables are fully supported: `cols` specifiers, implicit and explicit header
rows, footer rows, and the CSV and DSV data formats all match Asciidoctor's
output. For column widths and autowidth, see
xref:table-column-widths.adoc[]; for cell styles, alignment, and spans, see
xref:table-cell-formatting.adoc[]; for borders and striping, see
xref:table-borders-and-striping.adoc[]; and for titles and caption labels, see
xref:table-titles.adoc[].
"#
);
