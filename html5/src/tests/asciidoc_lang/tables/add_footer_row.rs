//! Coverage of the AsciiDoc language description's *Create a Footer Row* page.
//!
//! The page teaches how the last row of a table becomes a footer row: the
//! `footer` value can be assigned to the `options` attribute using either the
//! shorthand `%footer` or the formal `options="footer"` syntax, promoting the
//! final row to a `<tfoot>`. Each rendering claim is verified through
//! `convert`; the introduction and the section heading carry no rule of their
//! own and are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/add-footer-row.adoc");

// Title and introductory sentence: an introduction with no rendering rule to
// verify.
non_normative!(
    r#"
= Create a Footer Row

The last row of a table is promoted to a footer row if the `footer` value is assigned to the table's `options` attribute.

"#
);

// Section heading only.
non_normative!(
    r#"
== Assign footer to the last row

"#
);

#[test]
fn footer_shorthand() {
    verifies!(
        r#"
The footer row semantics and styles are applied to the last row in a table by assigning `footer` to the `options` attribute.
The `options` attribute is set in the table's attribute list using the shorthand (`%value`) or formal syntax (`options="value"`).

The `options` attribute is represented by the percent sign (`%`) when it's set using the shorthand syntax.
In <<ex-short>>, `footer` is assigned using the shorthand syntax for `options`.

.Table with footer assigned using the shorthand syntax
[source#ex-short]
----
[%header%footer,cols="2,2,1"] <.>
|===
|Column 1, header row
|Column 2, header row
|Column 3, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|Cell in column 3, row 2

|Column 1, footer row
|Column 2, footer row
|Column 3, footer row
|===
----
<.> Values assigned using the shorthand syntax must be entered before the `cols` attribute (or any other named attributes) in a table's attribute list, otherwise the processor will ignore them.

The table from <<ex-short>> is displayed below.

.Result of <<ex-short>>
[%header%footer,cols="2,2,1"]
|===
|Column 1, header row
|Column 2, header row
|Column 3, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|Cell in column 3, row 2

|Column 1, footer row
|Column 2, footer row
|Column 3, footer row
|===

"#
    );

    // The `%header%footer` shorthand promotes the first row to a `<thead>` and
    // the last row to a `<tfoot>`, leaving the middle row in the `<tbody>`: three
    // columns (40%, 40%, 20%), with three cells in each of the head, body, and
    // foot rows.
    let output = convert(
        "[%header%footer,cols=\"2,2,1\"]\n|===\n|Column 1, header row\n|Column 2, header row\n|Column 3, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|Cell in column 3, row 2\n\n|Column 1, footer row\n|Column 2, footer row\n|Column 3, footer row\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 3);
    assert_css(&output, "table > tfoot > tr", 1);
    assert_css(&output, "table > tfoot > tr > td", 3);
}

#[test]
fn footer_formal() {
    verifies!(
        r#"
In <<ex-formal>>, the `options` attribute is set and assigned the `footer` value using the formal syntax.
The `options` attribute accepts a comma-separated list of values.

.Table with footer assigned to the options attribute
[source#ex-formal]
----
include::example$row.adoc[tag=opt-f]
----

The last row of the table in <<ex-formal>> is rendered using the corresponding footer styles.

.Result of <<ex-formal>>
include::example$row.adoc[tag=opt-f]
"#
    );

    // The formal `options="footer"` syntax promotes the last row to a `<tfoot>`.
    // Here the table's layout also implies a header row, so the first row renders
    // as a `<thead>`, the two middle rows as the `<tbody>`, and the final row as
    // the `<tfoot>`.
    let output = convert(
        "[options=\"footer\"]\n|===\n|Column 1, header row |Column 2, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n\n|Column 1, footer row\n|Column 2, footer row\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > thead > tr > th", 2);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 4);
    assert_css(&output, "table > tfoot > tr", 1);
    assert_css(&output, "table > tfoot > tr > td", 2);
}
