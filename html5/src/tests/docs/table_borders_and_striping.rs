//! Coverage of this crate's *Table Borders and Striping* documentation page.
//!
//! The page shows how `asciidoc-html5` renders the `frame`, `grid`, `stripes`,
//! and role attributes as classes on the `<table>` element. Each shown
//! source-and-output pair is verified by converting the source through
//! `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("docs/modules/ROOT/pages/table-borders-and-striping.adoc");

// Title, description, the intro paragraph, the NOTE admonition, and the first
// section heading are descriptive framing with no rendering rule to verify.
non_normative!(
    r#"
= Table Borders and Striping
:navtitle: Borders and Striping
:description: Frame, grid, striping, and roles on asciidoc-html5 tables.

The `frame` and `grid` attributes choose which borders a table draws, the
`stripes` attribute shades alternating rows, and a role adds a CSS class of your
own. Each renders as a class on the `<table>` element, matching Asciidoctor's
`html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed. The borders and stripes themselves are drawn
by the stylesheet from these classes.
====

== Frame and grid

"#
);

// The chosen `frame` and `grid` values become `frame-` and `grid-` classes on
// the table.
#[test]
fn frame_and_grid() {
    verifies!(
        r#"
`frame` controls the outer border (`all`, `ends`, `sides`, `none`) and `grid`
controls the inner lines (`all`, `rows`, `cols`, `none`):

[,asciidoc]
----
[frame=ends,grid=rows]
|===
|A |B
|1 |2
|===
----

The chosen values become `frame-` and `grid-` classes:

[,html]
----
<table class="tableblock frame-ends grid-rows stretch">
<colgroup>
<col style="width: 50%;">
<col style="width: 50%;">
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

Set both to `none` to remove every border:

[,asciidoc]
----
[frame=none,grid=none]
|===
|A |B
|1 |2
|===
----

[,html]
----
<table class="tableblock frame-none grid-none stretch">
----

"#
    );

    let ends_rows = convert("[frame=ends,grid=rows]\n|===\n|A |B\n|1 |2\n|===\n");
    assert_css(
        &ends_rows,
        "table.tableblock.frame-ends.grid-rows.stretch",
        1,
    );

    let none_none = convert("[frame=none,grid=none]\n|===\n|A |B\n|1 |2\n|===\n");
    assert_css(
        &none_none,
        "table.tableblock.frame-none.grid-none.stretch",
        1,
    );
}

// The next section heading has no rendering rule of its own.
non_normative!(
    r#"
== Striping

"#
);

// The `stripes` attribute becomes a `stripes-` class on the table.
#[test]
fn striping() {
    verifies!(
        r#"
The `stripes` attribute shades rows: `even`, `odd`, `all`, `hover`, or `none`. It
becomes a `stripes-` class on the table:

[,asciidoc]
----
[cols="1,1",stripes=even]
|===
|A1 |B1
|A2 |B2
|===
----

[,html]
----
<table class="tableblock frame-all grid-all stripes-even stretch">
<colgroup>
<col style="width: 50%;">
<col style="width: 50%;">
</colgroup>
<tbody>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">A1</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">B1</p></td>
</tr>
<tr>
<td class="tableblock halign-left valign-top"><p class="tableblock">A2</p></td>
<td class="tableblock halign-left valign-top"><p class="tableblock">B2</p></td>
</tr>
</tbody>
</table>
----

"#
    );

    let output = convert("[cols=\"1,1\",stripes=even]\n|===\n|A1 |B1\n|A2 |B2\n|===\n");

    assert_css(
        &output,
        "table.tableblock.frame-all.grid-all.stripes-even.stretch",
        1,
    );
}

// The document-wide `table-stripes` note is descriptive; the next heading has
// no rendering rule of its own.
non_normative!(
    r#"
To stripe every table in a document, set the `table-stripes` document attribute
in the header instead.

== Roles

"#
);

// A role is appended as a class after the built-in table classes.
#[test]
fn roles() {
    verifies!(
        r#"
A role – set with the shorthand dot prefix – adds your own class to the table,
appended after the built-in classes:

[,asciidoc]
----
[.summary,cols="1,1"]
|===
|A |B
|===
----

[,html]
----
<table class="tableblock frame-all grid-all stretch summary">
----

"#
    );

    let output = convert("[.summary,cols=\"1,1\"]\n|===\n|A |B\n|===\n");

    assert_css(
        &output,
        "table.tableblock.frame-all.grid-all.stretch.summary",
        1,
    );
}

// Closing summary and a cross-reference to the default stylesheet; no rendering
// rule to verify.
non_normative!(
    r#"
== Known limitations

Borders, striping, and roles are fully supported: every `frame` and `grid`
value, the `stripes` values (including the document-wide `table-stripes`
attribute), and table roles all match Asciidoctor's output. The visible borders
and shading come from the stylesheet; see
xref:generate-html:default-stylesheet.adoc[].
"#
);
