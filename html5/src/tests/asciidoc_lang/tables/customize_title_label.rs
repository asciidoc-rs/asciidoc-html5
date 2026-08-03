//! Coverage of the AsciiDoc language description's *Customize the Title Label*
//! page.
//!
//! The page shows two ways to change the label that the processor prefixes to a
//! table title: the document attribute `table-caption` (which applies to every
//! titled table and is followed by an auto-incrementing number) and the block
//! attribute `caption` (which sets the exact label text for a single table and
//! is not numbered). Those rendering rules are verified through `convert`:
//! setting each attribute, a later table without its own `caption` reverting to
//! the `table-caption` label, and the two caption-only forms that combine
//! `caption`, `title`, and the `{table-caption}`/`{counter:table-number}`
//! references to make a caption of just the label. Only the section headings
//! and the introductory prose carry no rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/customize-title-label.adoc");

// Page header, the introductory prose describing the default label, and the
// cross-reference to the deactivation page: description with no rendering rule
// to verify here.
non_normative!(
    r#"
= Customize the Title Label
:table-caption: Data Set

When you xref:add-title.adoc[add a title to a table], the processor automatically prefixes it with the label _Table <n>._, where _<n>_ is the 1-based index of all of the titled tables in the document.
This label can be modified at the document level or per table.
It can also be xref:turn-off-title-label.adoc[deactivated].

"#
);

// Section heading only.
non_normative!(
    r#"
== Modify the label using table-caption

"#
);

#[test]
fn modify_label_using_table_caption() {
    verifies!(
        r#"
You can change the label for all titled tables using the document attribute `table-caption`.
(Don't let the attribute's name mislead you.
It's the attribute that controls the table title labels at the document level.)

In the document header, set the `table-caption` attribute and assign it your custom label text.

[source]
----
= Document Title
:table-caption: Data Set <.> <.>
----
<.> Set the document attribute `table-caption` and assign it the text you want to precede each table title.
<.> Don't enter a number after the label text.
The processor will automatically insert and increment the number.

In <<ex-label>>, the first and third tables have a title, but the second table doesn't have a title.

.Add two titled tables and one untitled table to a document
[source#ex-label]
----
= Document Title
:table-caption: Data Set

.A table with a title
[cols="2,1"]
|===
|Lots and lots of data |A little data

|834,734 |3
|3,999,271.5601 |5
|===

|===
|Group |Climate |Example

|A
|Tropical
|Suva, Fiji

|B
|Arid
|Lima, Peru
|===

.Another table with a title
|===
|Value |Result |Notes

|Null |A mystery |See Appendix R
|===
----

Since `table-caption` is assigned the value `Data Set`, any table title should be preceded with the label _Data Set <n>._
The three tables from <<ex-label>> are displayed below.

.A table with a title
[cols="2,1"]
|===
|Lots and lots of data |A little data

|834,734 |3
|3,999,271.5601 |5
|===

|===
|Group |Climate |Example

|A
|Tropical
|Suva, Fiji

|B
|Arid
|Lima, Peru
|===

.Another table with a title
|===
|Value |Result |Notes

|Null |A mystery |See Appendix R
|===

Notice that the table that doesn't have a title didn't get a label nor was it counted when the processor incremented the label number.
Therefore, the third table is assigned the label _Data Set 2._

"#
    );

    // Setting `table-caption` to `Data Set` prefixes every titled table with
    // that label. Only the two titled tables receive a caption; the untitled
    // middle table gets none and is not counted, so the second titled table is
    // labeled `Data Set 2.`.
    let output = convert(
        "= Document Title\n:table-caption: Data Set\n\n.A table with a title\n[cols=\"2,1\"]\n|===\n|Lots and lots of data |A little data\n\n|834,734 |3\n|3,999,271.5601 |5\n|===\n\n|===\n|Group |Climate |Example\n\n|A\n|Tropical\n|Suva, Fiji\n\n|B\n|Arid\n|Lima, Peru\n|===\n\n.Another table with a title\n|===\n|Value |Result |Notes\n\n|Null |A mystery |See Appendix R\n|===",
    );

    assert_css(&output, "caption.title", 2);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Data Set 1. A table with a title"]"#,
        1,
    );

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Data Set 2. Another table with a title"]"#,
        1,
    );
}

// Section heading only.
non_normative!(
    r#"
== Modify the label of an individual table using caption

"#
);

#[test]
fn modify_label_using_caption() {
    verifies!(
        r#"
You can customize the label on an individual table by setting the `caption` attribute.
(Don't let the name of the attribute mislead you.
The caption attribute only sets the caption's label, not the whole caption line).
When using `caption`, assign it the exact value you want displayed (including trailing spaces).
Labels assigned using `caption` don't get an automatically incremented number and only apply to the table they are set on.

CAUTION: If you want a space between the label and the title, you must add a trailing space to the value of the caption attribute.

.Modify the label using caption
[source#ex-caption]
----
[caption="Table A. "] <.> <.>
.A table with a custom label
[cols="3*"]
|===
|Null
|A mystery
|See Appendix R
|===
----
<.> Create an attribute list directly above the table's title and set the named attribute `caption`, followed by an equals sign (`=`), and then a value.
<.> Enclose the value in double quotation marks (`"`).
Otherwise the processor will remove any trailing whitespaces, and the title text will start directly after the last character of the label.

The table from <<ex-caption>> is displayed below.

[caption="Table A. "]
.A table with a custom label
[cols="3*"]
|===
|Null
|A mystery
|See Appendix R
|===

"#
    );

    // The `caption` attribute sets the exact label text, including its trailing
    // space, and is not followed by a number, so the caption reads
    // `Table A. A table with a custom label`.
    let output = convert(
        "[caption=\"Table A. \"]\n.A table with a custom label\n[cols=\"3*\"]\n|===\n|Null\n|A mystery\n|See Appendix R\n|===",
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Table A. A table with a custom label"]"#,
        1,
    );
}

#[test]
fn subsequent_table_reverts_to_table_caption() {
    verifies!(
        r#"
If you create any subsequent tables in your document and don't set `caption` on them, the title labels will revert to the value assigned to `table-caption`.

"#
    );

    // The first table sets `caption`, so it shows that exact, unnumbered label.
    // The second table doesn't set `caption`, so its label reverts to the
    // document's `table-caption` value (`Data Set`) followed by the
    // auto-incrementing table number.
    let output = convert(
        "= Document Title\n:table-caption: Data Set\n\n[caption=\"Custom Label. \"]\n.A table with a custom label\n|===\n|Null |A mystery\n|===\n\n.A subsequent table\n|===\n|Group |Climate\n|===",
    );

    assert_css(&output, "caption.title", 2);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Custom Label. A table with a custom label"]"#,
        1,
    );

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Data Set 1. A subsequent table"]"#,
        1,
    );
}

#[test]
fn caption_only_via_title_attribute() {
    verifies!(
        r#"
If you want the caption of the table to only consist of the caption label, use the following syntax:

[source]
----
[caption=,title="{table-caption} {counter:table-number}"]
include::example$table.adoc[tag=b-col-h]
----

"#
    );

    // Emptying `caption` suppresses the automatic label prefix, and setting
    // `title` to `{table-caption} {counter:table-number}` makes the whole caption
    // read as just the label and its number, `Data Set 1`, with no title text.
    let output = convert(
        r#"= Document Title
:table-caption: Data Set

[caption=,title="{table-caption} {counter:table-number}"]
[%header,cols=2*]
|===
|Name of Column 1
|Name of Column 2

|Cell in column 1, row 1
|Cell in column 2, row 1

|Cell in column 1, row 2
|Cell in column 2, row 2
|==="#,
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Data Set 1"]"#,
        1,
    );
}

#[test]
fn caption_only_via_empty_title() {
    verifies!(
        r#"
Alternately, you can write is as follows:

[source]
----
.{empty}
[caption="{table-caption} {counter:table-number}"]
include::example$table.adoc[tag=b-col-h]
----
"#
    );

    // The other way around: an empty block title (`.{empty}`) leaves no title
    // text, and `caption` supplies the label and number directly, so the caption
    // again reads as just `Data Set 1`.
    let output = convert(
        r#"= Document Title
:table-caption: Data Set

.{empty}
[caption="{table-caption} {counter:table-number}"]
[%header,cols=2*]
|===
|Name of Column 1
|Name of Column 2

|Cell in column 1, row 1
|Cell in column 2, row 1

|Cell in column 1, row 2
|Cell in column 2, row 2
|==="#,
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Data Set 1"]"#,
        1,
    );
}
