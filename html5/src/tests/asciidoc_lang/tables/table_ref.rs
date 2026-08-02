//! Coverage of the AsciiDoc language description's *Table Syntax and Attribute
//! Reference* page.
//!
//! The page is a single reference table listing every table attribute, its
//! values, and notes. The table exercises several table features at once –
//! computed column widths from the `cols` spec, an implicit header row, and
//! cells that span multiple rows via the `.N+` row-span shorthand – so the
//! whole table is verified through `convert`. The page title and navtitle
//! attribute carry no rendering rule and are tracked as non-normative.

use std::path::Path;

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/table-ref.adoc");

// Page title and navtitle attribute.
non_normative!(
    r#"
= Table Syntax and Attribute Reference
:navtitle: Table Reference

"#
);

#[test]
fn attribute_reference_table() {
    verifies!(
        r#"
[cols="1m,2,1m,2,2"]
|===
|Attribute |Description |Value |Description |Notes

|caption
|defines the title label on a single table
d|user-defined
|
|

|cols
|comma-separated list of column specifiers
d|specifiers
|Specifies the number of columns and the distribution ratio and default formatting for each column. See xref:add-columns.adoc[] for details
|

.4+|format
.4+|data format of the table's contents
|psv
|cells are delimited by `separator` (default `{vbar}`) (aka prefix-separated values)
.4+|

|dsv
|cells are delimited by a colon (`:`) (aka delimiter-separated values)

|csv
|cells are delimited by a comma (`,`) (aka comma-separated values)

|tsv
|cells are delimited by a tab character (aka tab-separated values)

.3+|separator
.3+|character used to separate cells
|{vbar}
|default for top-level tables
.3+|
|!
|default for nested tables
d|user-defined
|any single character or `\t` for tab (e.g., `{brvbar}` or `%`).
_Ideally a character not found in the cell content._

.4+|frame
.4+|draws a border around the table
|all
|border on all sides (default)
.4+|

|ends
|border on top and bottom ends

|none
|no borders

|sides
|border on left and right sides

.4+|grid
.4+|draws boundary lines between rows and columns
|all
|draws boundary lines around each cell (default)
.4+|

|cols
|draws boundary lines between columns

|rows
|draws boundary lines between rows

|none
|no boundary lines

.5+|stripes
.5+|controls row shading (via background color)
|none
|no rows are shaded (default)
.5+|

|even
|even rows are shaded

|odd
|odd rows are shaded

|hover
|row under the mouse cursor is shaded (HTML only)

|all
|all rows are shaded

.3+|align
.3+|horizontally aligns a table with restricted width
|left
|aligns to left side of page (default)
.3+|Not recognized by Asciidoctor.
To align the table, use an alignment role (e.g., `[role=center]` or `[.center]`).
The alignment roles work for both HTML and PDF output.
Alignment roles and the `float` attributes are mutually exclusive.

|right
|aligns to right side of page

|center
|horizontally aligns to center of page

.2+|float
.2+|floats the table to the specified side of the page
|left
|floats the table to the left side of the page (default)
.2+|Applies to HTML output only.
Must be used in conjunction with the table's `width` attribute to take effect.
The `float` and `align` attributes are mutually exclusive.

|right
|floats the table to the right side of the page

.3+|halign
.3+|horizontally aligns all of the cell contents in a table
|left
|aligns the contents of the cells to the left (default)
.3+|Not recognized by Asciidoctor.
Define instead using column or cell specifiers (e.g., `3*>`), which take precedence over this value.

|right
|aligns the contents of the cells to the right

|center
|aligns the contents to the cell centers

.3+|valign
.3+|vertically aligns all of the cell contents in a table
|top
|aligns the cell contents to the top of the cell (default)
.3+|Not recognized by Asciidoctor.
Define instead using column or cell specifiers (e.g., `3*.>`), which take precedence over this value.

|bottom
|aligns the cell contents to the bottom of the cell

|middle
|aligns the cell contents to the middle of the cell

|orientation
|rotates the table
|landscape
|rotated 90 degrees counterclockwise
|Equivalent to setting the rotate option, which is preferred.
DocBook only.

.6+|options
.6+|comma separated list of option names
|header
|promotes first row to the table header
.2+d|header and footer rows are omitted by default

|footer
|promotes last row to the table footer

|breakable
|allows the table to split across a page (default)
.2+d|Mutually exclusive.
DocBook only (specifically for generating PDF output).

|unbreakable
|prevents the table from being split across a page

|autowidth
|disables explicit column widths (ignores distribution ratios in `cols` attribute)
|

|rotate
|Prints the table in landscape
d|Equivalent to setting the orientation to landscape.
DocBook only.

.4+|role
.4+|comma-separated list of role names
|left
|floats the table to the left margin
.3+d|The role is the preferred way to specify the alignment of a table with restricted width.
May be specified using role shorthand (e.g., `[.center]`).

|right
|floats the table to the right

|center
|aligns the table to center

|stretch
|stretches an autowidth table to the width of the page
|

|width
|the table width relative to the available page width
d|user defined value
|a percentage value between 0% and 100%
|
|===
"#
    );

    // Read and render the reference page itself, so the assertions run against
    // exactly the table reproduced above.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ref/asciidoc-lang/docs/modules/tables/pages/table-ref.adoc");

    let input = std::fs::read_to_string(path).expect("read table-ref.adoc");

    let output = convert(&input);

    // The whole page is one table whose `cols="1m,2,1m,2,2"` spec yields five
    // columns whose widths sum to 100%: two at 12.5% and three at 25%.
    assert_css(&output, "table", 1);
    assert_css(&output, "colgroup > col", 5);
    assert_css(&output, r#"col[style="width: 12.5%;"]"#, 2);
    assert_css(&output, r#"col[style="width: 25%;"]"#, 3);

    // The first row, entered on the line after the delimiter, becomes an implicit
    // header of five cells.
    assert_css(&output, "table > thead > tr > th", 5);
    assert_xpath(&output, r#"//thead//th[text()="Attribute"]"#, 1);
    assert_xpath(&output, r#"//thead//th[text()="Notes"]"#, 1);
    assert_xpath(&output, r#"//thead//th[text()="Description"]"#, 2);

    // The `.N+|` shorthand spans a cell down N rows: e.g. `.4+|format` renders a
    // body cell carrying `rowspan="4"`.
    assert_css(&output, r#"td[rowspan="4"]"#, 11);
    assert_xpath(&output, r#"//td[@rowspan="4"]/p/code[text()="format"]"#, 1);
}
