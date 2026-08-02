//! Coverage of this crate's *Table Titles and Captions* documentation page.
//!
//! The page shows how `asciidoc-html5` renders a table title as a `<caption>`:
//! the automatic `Table N.` label, a custom `caption` value, and turning the
//! label off. Each shown source-and-output pair is verified by converting the
//! source through `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/table-titles.adoc");

// Title, description, the intro paragraph, the NOTE admonition, and the first
// section heading are descriptive framing with no rendering rule to verify.
non_normative!(
    r#"
= Table Titles and Captions
:navtitle: Titles and Captions
:description: Adding a title to a table and controlling its caption label.

Give a table a title with the block-title syntax – a line beginning with `.`
directly above the table. The processor renders it as a `<caption>` and, by
default, prefixes it with an automatic label such as `Table 1.`, matching
Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Add a title

"#
);

// A block title becomes a `<caption class="title">` with the automatic `Table
// 1.` label prepended.
#[test]
fn add_a_title() {
    verifies!(
        r#"
Put the title on the line above the table (or above its attribute line):

[,asciidoc]
----
.Quarterly results
[cols="1,1"]
|===
|Q |Revenue
|Q1 |100
|===
----

The title becomes a `<caption class="title">` with the automatic `Table 1.`
label prepended:

[,html]
----
<table class="tableblock frame-all grid-all stretch">
<caption class="title">Table 1. Quarterly results</caption>
<colgroup>
<col style="width: 50%;">
<col style="width: 50%;">
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Q</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">Revenue</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">Q1</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">100</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert(".Quarterly results\n[cols=\"1,1\"]\n|===\n|Q |Revenue\n|Q1 |100\n|===\n");

    assert_css(&output, "table.tableblock.frame-all.grid-all.stretch", 1);
    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Table 1. Quarterly results"]"#,
        1,
    );
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Customize the label

"#
);

// A custom `caption` value replaces the automatic label, kept verbatim before
// the title text.
#[test]
fn customize_the_label() {
    verifies!(
        r#"
Set the `caption` attribute on the table to replace the automatic label with
your own text (include the trailing space and punctuation you want):

[,asciidoc]
----
[caption="Dataset 7. "]
.Quarterly results
[cols="1,1"]
|===
|Q |Revenue
|===
----

[,html]
----
<caption class="title">Dataset 7. Quarterly results</caption>
----

"#
    );

    let output = convert(
        "[caption=\"Dataset 7. \"]\n.Quarterly results\n[cols=\"1,1\"]\n|===\n|Q |Revenue\n|===\n",
    );

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Dataset 7. Quarterly results"]"#,
        1,
    );
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Turn off the label

"#
);

// An empty `caption` value drops the label, leaving just the title text.
#[test]
fn turn_off_the_label() {
    verifies!(
        r#"
Assign an empty value to `caption` to drop the label on a single table, leaving
just the title text:

[,asciidoc]
----
[caption=]
.Quarterly results
[cols="1,1"]
|===
|Q |Revenue
|===
----

[,html]
----
<caption class="title">Quarterly results</caption>
----

"#
    );

    let output =
        convert("[caption=]\n.Quarterly results\n[cols=\"1,1\"]\n|===\n|Q |Revenue\n|===\n");

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Quarterly results"]"#,
        1,
    );
}

// The document-wide `table-caption` note, closing summary, and cross-references
// have no rendering rule to verify.
non_normative!(
    r#"
To drop the label for every table in a document, unset the `table-caption`
document attribute (`:table-caption!:`) in the header instead.

== Known limitations

Table titles and captions are fully supported: the automatic `Table N.` label, a
custom `caption` value, and turning the label off (per table or document-wide)
all match Asciidoctor's output.
"#
);
