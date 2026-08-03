//! Coverage of the AsciiDoc language description's *Duplicate Cells* page.
//!
//! The page teaches the duplication factor and operator (`<n>*`), the first
//! operator in a cell specifier: it clones a cell's content and properties into
//! `<n>` consecutive cells. That rendering claim is verified through `convert`;
//! the title, headings, and the descriptive definitions of the factor and
//! operator carry no HTML rule of their own and are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/duplicate-cells.adoc");

// Title, headings, and the descriptive definitions of the duplication factor
// and operator, with a specifier schematic; no rendered output to verify.
non_normative!(
    r#"
= Duplicate Cells
// clone, copy, replicate, duplicate

The contents of a cell can be duplicated in consecutive cells.

== Duplication factor and operator

The duplication factor and operator are applied to a xref:add-cells-and-rows.adoc#specifiers[cell's specifier] and allow you to clone a cell's content and properties across consecutive cells.
A duplication is the first operator in a cell specifier.

====
<**duplication factor**><**duplication operator**><horizontal alignment operator><vertical alignment operator><style operator>|<cell's content>
====

The [.term]*duplication factor* is a single integer (`<n>`) that indicates how many times the cell's content should be duplicated.

The [.term]*duplication operator* is an asterisk (`+*+`) placed directly after the duplication factor (`+<n>*+`).
The duplication operator tells the converter to interpret the duplication factor as part of a duplication instead of a span.

== Duplicate a cell and its properties

"#
);

#[test]
fn duplicate_a_cell_and_its_properties() {
    verifies!(
        r#"
To duplicate a cell, enter the duplication factor and duplication operator (`+<n>*+`) in the cell specifier.
Don't insert any spaces between the duplication, any alignment or style operators (if present), and the xref:add-cells-and-rows.adoc#cell-separator[cell's separator] (`|`).

.Duplicate the contents of two cells
[source#ex-clone]
----
include::example$cell.adoc[tag=clone]
----

The table from <<ex-clone>> is displayed below.

.Result of <<ex-clone>>
include::example$cell.adoc[tag=clone]
"#
    );

    let output = convert(
        r#"|===
|Column 1, header row |Column 2, header row |Column 3, header row

2*|This cell is duplicated in columns 1 and 2 because its specifier contains a duplication of `2*`
|Cell in column 3, row 2

|Cell in column 1, row 3
|Cell in column 2, row 3
3*e|This cell specifier contains the duplication `3*` and style operator `e`.

The cell's text is italicized and duplicated in column 3, row 3 and columns 1 and 2 on row 4.

|Cell in column 3, row 4
|==="#,
    );

    // Three columns, a header row, and three body rows of three cells each: the
    // duplicated cells fill real cells rather than spanning.
    assert_css(&output, "thead th.tableblock", 3);
    assert_css(&output, "tbody tr", 3);
    assert_css(&output, "tbody td", 9);

    // The `2*` factor clones the cell's content into two consecutive cells.
    assert_xpath(
        &output,
        "//td/p[contains(text(), \"This cell is duplicated in columns 1 and 2\")]",
        2,
    );

    // The `3*e` factor clones the cell into three consecutive cells, carrying
    // the `e` style operator into each copy (the content is italicized).
    assert_xpath(
        &output,
        "//td/p/em[contains(text(), \"This cell specifier contains the duplication\")]",
        3,
    );
}
