//! Coverage of the AsciiDoc language description's *Table Data Formats* page.
//!
//! The page describes the `format` attribute – the default prefix-separated
//! values (PSV) format plus the comma-separated (CSV), tab-separated (TSV), and
//! delimiter-separated (DSV) formats – along with escaping the cell separator,
//! overriding it with the `separator` attribute, and the `,===` / `:===`
//! shorthand block delimiters. Each rendered example is verified through
//! `convert`: PSV escaping, the CSV and DSV tables (built from the shared
//! `data.adoc` snippets), custom separators, and the shorthand delimiters. The
//! overview sections, the parsing-rule lists, the terminology sidebar, and the
//! external-file include example carry no verifiable rendering rule and are
//! tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/data-format.adoc");

// Page title and the document attribute definitions used later in the page.
non_normative!(
    r#"
= Table Data Formats
:navtitle: CSV, TSV and DSV Data
:url-dsv: https://en.wikipedia.org/wiki/Delimiter-separated_values
:url-rfc-4180: https://tools.ietf.org/html/rfc4180

"#
);

// Overview of the default table syntax; step-by-step creation of a first table
// is covered on the build-a-basic-table page, and there is no example here.
non_normative!(
    r#"
== Default table syntax

A table is delimited by a vertical bar and three equal signs (`|===`).
It contains cells that are arranged into rows according to the number of columns the table is assigned.
The number of columns a table contains can be specified implicitly using the number of cells in the table's first row or by setting the `cols` attribute.
Each cell is specified by a vertical bar (`|`).

If you're new to AsciiDoc tables, xref:build-a-basic-table.adoc[] provides step by step directions for creating your first table.

"#
);

// Overview of the styling and layout capabilities, with pointers to the
// dedicated pages; no example to verify.
non_normative!(
    r#"
== Style and layout options

Table content can be:

* styled and aligned by column or cell,
* aligned by row,
* duplicated across multiple rows, and
* marked up by any AsciiDoc syntax.

Table cells can span rows and columns.

You can adjust a table's:

* width,
* orientation, and
* border style.

You can also specify each column's width and designate header and footer rows.

"#
);

// Names the supported data formats (PSV, CSV, TSV, DSV); no example to verify.
non_normative!(
    r#"
== Supported data formats

The default table data format is prefix-separated values (PSV); that means the processor creates a new cell each time it encounters a vertical bar (`|`).
AsciiDoc also supports comma-separated values (CSV), tab-separated values (TSV), and delimited data values (DSV).

"#
);

// Describes the three ways to escape the cell separator; the rendered examples
// that demonstrate each way are verified below.
non_normative!(
    r#"
== Escape the cell separator

The parser scans for the cell separator to partition cells _before_ it processes the cell text.
So even if you try to hide the cell separator using an inline passthrough, the parser will see it.
If the cell contain contains the cell separator, you must escape that character.
There are three ways to escape it:

* Prefix the character with a leading backslash (i.e., `\|`), which will be removed from the output.
* Use the `\{vbar}` attribute reference in place of `|` in content.
* Change the cell separator used by the table.

Unless you do one of these things, the cell separator will be interpreted as a cell boundary.

"#
);

#[test]
fn escape_with_backslash() {
    verifies!(
        r#"
Consider the following example, which escapes the cell separator using a leading backslash:

[source]
----
[cols=2*]
|===
|The default separator in PSV tables is the \| character.
|The \| character is often referred to as a "`pipe`".
|===
----

This table will render as follows:

.Result: Converted PSV table that contains pipe characters
[cols=2*]
|===
|The default separator in PSV tables is the \| character.
|The \| character is often referred to as a "`pipe`".
|===

Notice that the pipe character appears without the leading backslash (i.e., unescaped) in the rendered result.

"#
    );

    // The escaped separator `\|` is emitted as a plain pipe in the cell content:
    // the table has one row of two cells, each containing an unescaped `|`.
    let output = convert(
        "[cols=2*]\n|===\n|The default separator in PSV tables is the \\| character.\n|The \\| character is often referred to as a \"`pipe`\".\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 2);

    assert_xpath(&output, r#"//td/p[contains(text(), "| character")]"#, 2);
}

#[test]
fn escape_with_vbar_attribute() {
    verifies!(
        r#"
An alternative is to use the `\{vbar}` attribute reference as a substitute.
This approach produces the same result as the previous example.

[source]
----
[cols=2*]
|===
|The default separator in PSV tables is the {vbar} character.
|The {vbar} character is often referred to as a "`pipe`".
|===
----

"#
    );

    // The `{vbar}` attribute reference resolves to a pipe, producing the same
    // rendered cells as the backslash-escaped example.
    let output = convert(
        "[cols=2*]\n|===\n|The default separator in PSV tables is the {vbar} character.\n|The {vbar} character is often referred to as a \"`pipe`\".\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > tbody > tr > td", 2);

    assert_xpath(&output, r#"//td/p[contains(text(), "| character")]"#, 2);
}

#[test]
fn custom_separator() {
    verifies!(
        r#"
Escaping each cell separator character that appears in the content of a cell can be tedious.
There are also times when you can't or don't want to modify the cell content (perhaps because it is being included from another file).
To address these cases, AsciiDoc allows you to override the cell separator.

The cell separator is controlled using the `separator` attribute on the table block.
You'll want to select any single character that is not found in the content.
A good candidate is the broken bar, or `¦`.

Here's the previous example rewritten using a custom separator.

[source]
----
[cols=2*,separator=¦]
|===
¦The default separator in PSV tables is the | character.
¦The | character is often referred to as a "`pipe`".
|===
----

Notice that it's no longer necessary to escape the pipe character in the content of the table cells.
You can safely use the original cell separator in the cell content and not worry about it being interpreted as the boundary of a cell.

"#
    );

    // With the broken bar as the separator, an unescaped `|` in the cell content
    // is treated as ordinary text rather than a cell boundary.
    let output = convert(
        "[cols=2*,separator=¦]\n|===\n¦The default separator in PSV tables is the | character.\n¦The | character is often referred to as a \"`pipe`\".\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 2);

    assert_xpath(&output, r#"//td/p[contains(text(), "| character")]"#, 2);
}

// Introduces the delimiter-separated value family and the `format` and
// `separator` attributes that control interpretation; no example here.
non_normative!(
    r#"
[#delimiter-separated-values]
== Delimiter-separated values

Tables can also be populated from data formatted as delimiter-separated values (i.e., data tables).
In contrast with the PSV format, in which the delimiter is placed in front of each cell value, the delimiter in a delimiter-separated format (CSV, TSV, DSV) is placed between the cell values (called a _separator_) and does not accept a cell formatting spec.
Each line of data is assumed to represent a single row, though you'll learn that's not a strict rule.
How the table data gets interpreted is controlled by the `format` and `separator` attributes on the table.

"#
);

// Explanatory sidebar defining the delimiter-separated-values terminology; no
// table rendering to verify.
non_normative!(
    r#"
.What the delimiter?
****
Aren't comma-separated values a subset of {url-dsv}[delimiter-separated values^]?
It really depends on who you consult.

The term "`delimiter-separated values`" in this text refers to the family of data formats that use a delimiter, including comma-separated values (CSV), tab-separated values (TSV) and delimited data (DSV), all of which are supported in AsciiDoc tables.
CSV is the data format used most often.

"`Comma-separated values`" is really a misleading term since CSV can use delimiters other than `,` as the field separator (which, in this context, separates cells).
What we're really talking about is how the data is interpreted.

CSV and TSV both use a delimiter and an optional enclosing character, loosely based on {url-rfc-4180}[RFC 4180^].
DSV (i.e., delimited data) only uses a delimiter, which can be escaped using a backslash; an enclosing character is not recognized.
These parsing rules are described in detail in <<data-table-formats>>.
****

"#
);

#[test]
fn csv_format() {
    verifies!(
        r#"
Let's consider an example of using comma-separated values (CSV) to populate an AsciiDoc table with data.
To instruct the processor to read the data as CSV, set the value of the `format` attribute on the table to `csv`.
When the `format` attribute is set to `csv`, the default data separator is a comma (`,`), as seen in the table below.

[source]
----
include::example$data.adoc[tag=csv]
----

.Result: Rendered CSV table
[width=90%]
include::example$data.adoc[tag=csv]

"#
    );

    // The `csv` snippet (with the header option) yields a three-column header row
    // and two body rows of three comma-separated cells each.
    let output = convert(
        "[%header,format=csv]\n|===\nArtist,Track,Genre\nBaauer,Harlem Shake,Hip Hop\nThe Lumineers,Ho Hey,Folk Rock\n|===",
    );

    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);

    assert_xpath(&output, r#"//thead//th[text()="Artist"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Baauer"]"#, 1);
}

// Shows populating a table from an external file via the include directive (a
// file-reading example with no fixture here) and mentions the tsv format;
// neither is verifiable in this test.
non_normative!(
    r#"
This feature is particularly useful when you want to populate a table in your manuscript from data stored in a separate file.
You can do so using the xref:directives:include.adoc[include directive] between the table delimiters, as shown here:

[source]
----
[%header,format=csv]
|===
\include::tracks.csv[]
|===
----

If your data is separated by tabs instead of commas, set the `format` to `tsv` (tab-separated values) instead.

"#
);

#[test]
fn dsv_format() {
    verifies!(
        r#"
Now let's consider an example of using delimited data (DSV) to populate an AsciiDoc table with data.
To instruct the processor to read the data as DSV, set the value of the `format` attribute on the table to `dsv`.
When the `format` attribute is set to `dsv`, the default data separator is a colon (`:`), as seen in the table below.

[source]
----
include::example$data.adoc[tag=dsv]
----

.Result: Rendered DSV table
[width=90%]
include::example$data.adoc[tag=dsv]

"#
    );

    // The `dsv` snippet parses colon-separated cells into a three-column header
    // row and two body rows of three cells each.
    let output = convert(
        "[%header,format=dsv]\n|===\nArtist:Track:Genre\nRobyn:Indestructible:Dance\nThe Piano Guys:Code Name Vivaldi:Classical\n|===",
    );

    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);

    assert_xpath(&output, r#"//thead//th[text()="Artist"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Robyn"]"#, 1);
}

// Describes the parsing rules for the CSV/TSV and DSV data formats; these
// govern parser-level behavior and the page shows no rendering example for
// them.
non_normative!(
    r#"
== Data table formats

The CSV and TSV data formats are parsed differently from the DSV data format.
The following two sections outline those differences.

=== CSV and TSV

Table data in either CSV or TSV format is parsed according to the following rules, loosely based on {url-rfc-4180}[RFC 4180^]:

* The default delimiter for CSV is a comma (`,`) while the default delimiter for TSV is a tab character.
* Empty lines are skipped (unless enclosed in a quoted value).
* Whitespace surrounding each value is stripped.
* Values can be enclosed in double quotes (`"`).
 ** A quoted value may contain zero or more separator or newline characters.
 ** A newline begins a new row unless the newline is enclosed in double quotes.
 ** A quoted value may include the double quote character if escaped using another double quote (`""`).
 ** Newlines in quoted values are retained.
* If rows do not have the same number of cells ("`ragged`" tables), cells are shuffled to fully fill the rows.
 ** This is different behavior than Excel, which pads short rows with empty cells.
 ** Extra cells at the end of the last row get dropped.
 ** As a rule of thumb, data for a single row should be on the same line.

=== DSV

Table data in DSV format is parsed according to the following rules:

* The default delimiter for DSV is a colon (`:`).
* Empty lines are skipped.
* Whitespace surrounding each value is stripped.
* The delimiter character can be included in the value if escaped using a single backslash (`\:`).
* If rows do not have the same number of cells ("`ragged`" tables), cells are shuffled to fully fill the rows.

"#
);

// Section heading.
non_normative!(
    r#"
== Custom delimiters

"#
);

#[test]
fn custom_dsv_delimiter() {
    verifies!(
        r#"
Each data format has a default separator associated with it (csv = comma, tsv = tab, dsv = colon), but the separator can be changed to any character (or even a string of characters) by setting the `separator` attribute on the table.

Here's an example of a DSV table that uses a custom separator character (i.e., delimiter):

.A DSV table with a custom separator
[source]
----
[format=dsv,separator=;]
|===
a;b;c
d;e;f
|===
----

"#
    );

    // With `format=dsv` and a semicolon separator, each line becomes a row of
    // three cells split on `;`.
    let output = convert("[format=dsv,separator=;]\n|===\na;b;c\nd;e;f\n|===");

    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > td", 6);

    assert_xpath(&output, r#"//tbody//td/p[text()="a"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="f"]"#, 1);
}

// A TIP about emulating TSV via csv plus `\t`, and a note that the separator is
// independent of the format's parsing rules; no rendering example to verify.
non_normative!(
    r#"
TIP: To make a TSV table, you can set the `format` attribute to `csv` and the separator to `\t`.
Though the `tsv` format is preferred.

The separator is independent of the processing rules for the format.
If you set `format=dsv` and `separator=,`, the data will be processed using the DSV rules, even though the data looks like CSV.

"#
);

// Introduces the shorthand block-delimiter notation; the shorthand examples are
// verified below.
non_normative!(
    r#"
== Shorthand notation for data tables

AsciiDoc provides shorthand notation for specifying the data format of a table.
The first position of the table block delimiter (i.e., `|===`) can be replaced by a built-in delimiter to set the table format (e.g., `,===` for CSV).

"#
);

#[test]
fn csv_shorthand_delimiter() {
    verifies!(
        r#"
To make a CSV table, you can use `,===` as the table block delimiter:

[source]
----
include::example$data.adoc[tag=s-csv]
----

.Result: Rendered CSV table using shorthand syntax
[width=90%]
include::example$data.adoc[tag=s-csv]

"#
    );

    // The `,===` shorthand implies the CSV format: the first line becomes a
    // header row and the line after the blank becomes the single body row.
    let output = convert(",===\nArtist,Track,Genre\n\nBaauer,Harlem Shake,Hip Hop\n,===");

    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 3);

    assert_xpath(&output, r#"//thead//th[text()="Artist"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Baauer"]"#, 1);
}

#[test]
fn dsv_shorthand_delimiter() {
    verifies!(
        r#"
To make a DSV table, you can use `:===` as the table block delimiter:

[source]
----
include::example$data.adoc[tag=s-dsv]
----

.Result: Rendered DSV table using shorthand syntax
[width=90%]
include::example$data.adoc[tag=s-dsv]

"#
    );

    // The `:===` shorthand implies the DSV format: colon-separated values with
    // an implicit header row followed by one body row.
    let output = convert(":===\nArtist:Track:Genre\n\nRobyn:Indestructible:Dance\n:===");

    assert_css(&output, "table > thead > tr > th", 3);
    assert_css(&output, "table > tbody > tr", 1);
    assert_css(&output, "table > tbody > tr > td", 3);

    assert_xpath(&output, r#"//thead//th[text()="Artist"]"#, 1);
    assert_xpath(&output, r#"//tbody//td/p[text()="Robyn"]"#, 1);
}

// Notes that the shorthand implies the format and that TSV has no shorthand
// delimiter; no additional rendering example to verify.
non_normative!(
    r#"
When using either the CSV or DSV shorthand, you do not need to set the `format` attribute as it's implied.

To make a TSV table, you can set the `format` attribute to `tsv` instead of having to set the `format` to `csv` and the separator to `\t`.
In this case, you can use either `|===` or `,===` as the table block delimiter.
There is no special delimited block notation for a TSV table.

"#
);

// Section heading.
non_normative!(
    r#"
== Formatting cells in a data table

"#
);

#[test]
fn cols_spec_formats_data_cells() {
    verifies!(
        r#"
The delimited formats do not provide a way to express formatting of individual table cells.
Instead, you can apply cell formatting to all cells in a given column using the `cols` spec on the table:

[source]
----
[format=csv,cols="1h,1a"]
|===
Sky,image::sky.jpg[]
Forest,image::forest.jpg[]
|===
----

"#
    );

    // The `cols` spec applies per-column formatting to the data cells: the `h`
    // style makes the first column header cells (`<th>`) and the `a` style parses
    // the second column as AsciiDoc, rendering the image macros as image blocks.
    let output = convert(
        "[format=csv,cols=\"1h,1a\"]\n|===\nSky,image::sky.jpg[]\nForest,image::forest.jpg[]\n|===",
    );

    assert_css(&output, "colgroup > col", 2);
    assert_css(&output, "table > tbody > tr", 2);
    assert_css(&output, "table > tbody > tr > th", 2);
    assert_css(&output, "table > tbody > tr > td .imageblock img", 2);

    assert_xpath(&output, r#"//th/p[text()="Sky"]"#, 1);
}

// Describes that data tables cannot express cells spanning rows or columns, and
// advises using PSV for that; no rendering example to verify.
non_normative!(
    r#"
Data tables do not support cells that span multiple rows or columns, since that information can only be expressed at the cell level.
You are advised to use the PSV format if you need that functionality.
"#
);
