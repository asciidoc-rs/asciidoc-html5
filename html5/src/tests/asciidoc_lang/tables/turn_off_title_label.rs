//! Coverage of the AsciiDoc language description's *Turn Off the Title Label*
//! page.
//!
//! The page shows two ways to remove the label the processor prefixes to a
//! table title: unsetting the document attribute `table-caption` (with a
//! trailing `!`), which drops the label for every table, and assigning an empty
//! value to the block attribute `caption`, which drops it for a single table.
//! In both cases the title still renders inside a `<caption class="title">`,
//! just without a `Table <n>.` prefix. Both rendering rules are verified
//! through `convert`; the page header and the section headings carry no rule of
//! their own and are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/turn-off-title-label.adoc");

// Page header, including the document attribute that unsets the caption label
// for this page itself: setup with no rendering rule to verify here.
non_normative!(
    r#"
= Turn Off the Title Label
:table-caption!:

"#
);

// Section heading only.
non_normative!(
    r#"
== Disable the label using table-caption

"#
);

#[test]
fn disable_using_table_caption() {
    verifies!(
        r#"
You can disable the title label for all of the tables in a document by unsetting the `table-caption` document attribute.

[source]
----
= Title of Document
:table-caption!: <.>

.A table with a title but no label
|===
|Value |Result |Notes

|Null |A mystery |See Appendix R
|===
----
<.> In an attribute entry, enter the name of the attribute, `table-caption`, and append a bang (`!`) to the end of the name.
This unsets the attribute.

When `table-caption` is unset, table titles aren't preceded by a label and label number.

.A table with a title but no label
|===
|Value |Result |Notes

|Null |A mystery |See Appendix R
|===

"#
    );

    // Unsetting `table-caption` with a trailing `!` drops the label entirely, so
    // the caption is just the title text with no `Table <n>.` prefix.
    let output = convert(
        "= Title of Document\n:table-caption!:\n\n.A table with a title but no label\n|===\n|Value |Result |Notes\n\n|Null |A mystery |See Appendix R\n|===",
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="A table with a title but no label"]"#,
        1,
    );
}

// Section heading only.
non_normative!(
    r#"
== Disable the label using caption

"#
);

#[test]
fn disable_using_caption() {
    verifies!(
        r#"
To remove the label on an individual table, assign an empty value to the `caption` attribute.

[source]
----
[caption=] <.>
.A table with a title but no label
[cols="2,1"]
|===
|Lots and lots of data |A little data

|834,734 |3
|3,999,271.5601 |5
|===
----
<.> Enter the attribute's name, `caption`, in an attribute list directly above the table title, followed by an equals sign (`=`).
Don't enter a value after the `=`.

The table from the previous example is displayed below.

[caption=]
.A table with a title but no label
[cols="2,1"]
|===
|Lots and lots of data |A little data

|834,734 |3
|3,999,271.5601 |5
|===
"#
    );

    // Assigning an empty value to `caption` drops the label for this table only,
    // so the caption is just the title text with no `Table <n>.` prefix.
    let output = convert(
        "[caption=]\n.A table with a title but no label\n[cols=\"2,1\"]\n|===\n|Lots and lots of data |A little data\n\n|834,734 |3\n|3,999,271.5601 |5\n|===",
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="A table with a title but no label"]"#,
        1,
    );
}
