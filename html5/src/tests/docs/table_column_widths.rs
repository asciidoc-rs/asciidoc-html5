//! Coverage of this crate's *Table Column Widths* documentation page.
//!
//! The page shows how `asciidoc-html5` sizes table columns and the whole table:
//! proportional `cols` widths, a fixed `width`, the `autowidth` option, and the
//! per-column `~` autowidth. Each shown source-and-output pair is verified by
//! converting the source through `convert`; the surrounding prose is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/table-column-widths.adoc");

// Title, description, the intro paragraph, the NOTE admonition, and the first
// section heading are descriptive framing with no rendering rule to verify.
non_normative!(
    r#"
= Table Column Widths
:navtitle: Column Widths
:description: How asciidoc-html5 sizes table columns and the whole table.

Column widths come from the numbers in `cols`, and the table's own width comes
from the `width` attribute or the `autowidth` option. `asciidoc-html5` computes
the same percentages and emits the same `<colgroup>` and width classes as
Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Proportional widths

"#
);

// Each `cols` number is a proportion, so `cols="2,1,1"` becomes per-column
// percentages of 50/25/25 on the `<col>` elements.
#[test]
fn proportional_widths() {
    verifies!(
        r#"
Each number in `cols` is a proportion. A column's width is its share of the sum
of all the numbers, so `cols="2,1,1"` splits the table 50/25/25:

[,asciidoc]
----
[cols="2,1,1"]
|===
|Wide |Narrow |Narrow
|===
----

The proportions become per-column percentages on the `<col>` elements:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<colgroup>
<col style="width: 50%;">
<col style="width: 25%;">
<col style="width: 25%;">
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Wide</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Narrow</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Narrow</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert("[cols=\"2,1,1\"]\n|===\n|Wide |Narrow |Narrow\n|===\n");

    assert_css(&output, "table.tableblock.frame-all.grid-all.stretch", 1);
    assert_css(&output, "colgroup > col", 3);
    assert_xpath(&output, r#"//col[@style="width: 50%;"]"#, 1);
    assert_xpath(&output, r#"//col[@style="width: 25%;"]"#, 2);
    assert_xpath(&output, r#"//tbody//td/p[text()="Wide"]"#, 1);
}

// The trailing note about `stretch` is descriptive; the next heading has no
// rendering rule of its own.
non_normative!(
    r#"
By default the table stretches to the full width of the content area, marked by
the `stretch` class.

== Constrain the table width

"#
);

// Setting `width` to a percentage puts an inline `style` on the table and drops
// the `stretch` class.
#[test]
fn constrain_the_table_width() {
    verifies!(
        r#"
Set `width` to a percentage to constrain the whole table. The value becomes an
inline `style` and the `stretch` class is dropped:

[,asciidoc]
----
[width=50%,cols="1,1"]
|===
|A |B
|===
----

[,html]
----
<table class="tableblock frame-all grid-all" style="width: 50%;">
<colgroup>
<col style="width: 50%;">
<col style="width: 50%;">
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">A</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">B</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert("[width=50%,cols=\"1,1\"]\n|===\n|A |B\n|===\n");

    assert_xpath(&output, r#"//table[@style="width: 50%;"]"#, 1);
    assert_css(&output, "table.tableblock.frame-all.grid-all", 1);
    assert_css(&output, "table.stretch", 0);
    assert_xpath(&output, r#"//col[@style="width: 50%;"]"#, 2);
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Size to content with autowidth

"#
);

// The `autowidth` option gives the table the `fit-content` class and emits each
// `<col>` bare, with no width.
#[test]
fn size_to_content_with_autowidth() {
    verifies!(
        r#"
The `autowidth` option sizes the table and its columns to their content. The
table gains the `fit-content` class and each `<col>` is emitted bare, with no
width:

[,asciidoc]
----
[%autowidth]
|===
|A |B
|1 |2
|===
----

[,html]
----
<table class="tableblock frame-all grid-all fit-content">
<colgroup>
<col>
<col>
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">A</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">B</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">1</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">2</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert("[%autowidth]\n|===\n|A |B\n|1 |2\n|===\n");

    assert_css(
        &output,
        "table.tableblock.frame-all.grid-all.fit-content",
        1,
    );
    assert_css(&output, "colgroup > col", 2);
    assert_xpath(&output, r#"//col[not(@style)]"#, 2);
    assert_css(&output, "tbody > tr", 2);
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Mix fixed and autowidth columns

"#
);

// A `~` column is emitted bare while the fixed columns keep their percentage.
#[test]
fn mix_fixed_and_autowidth_columns() {
    verifies!(
        r#"
Give a single column the special width `~` to size just that column to its
content while the others keep fixed percentages:

[,asciidoc]
----
[cols="25,~,~"]
|===
|small |as big as needed |the rest
|===
----

The fixed column carries its percentage; the `~` columns are emitted bare:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<colgroup>
<col style="width: 25%;">
<col>
<col>
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">small</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">as big as needed</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">the rest</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert("[cols=\"25,~,~\"]\n|===\n|small |as big as needed |the rest\n|===\n");

    assert_css(&output, "colgroup > col", 3);
    assert_xpath(&output, r#"//col[@style="width: 25%;"]"#, 1);
    assert_xpath(&output, r#"//col[not(@style)]"#, 2);
    assert_xpath(&output, r#"//tbody//td/p[text()="as big as needed"]"#, 1);
}

// Closing summary and a cross-reference to cell formatting; no rendering rule
// to verify.
non_normative!(
    r#"
== Known limitations

Column sizing is fully supported: proportional `cols` widths, a fixed table
`width`, the `autowidth` option, and per-column `~` autowidth all match
Asciidoctor's output. To set the columns themselves – their alignment and style –
see xref:table-cell-formatting.adoc[].
"#
);
