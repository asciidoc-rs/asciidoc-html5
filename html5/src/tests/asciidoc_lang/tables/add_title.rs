//! Coverage of the AsciiDoc language description's *Add a Title to a Table*
//! page.
//!
//! The page shows that a table can carry a block title and that the processor
//! renders it inside a `<caption class="title">` element, automatically
//! prefixing the title with the caption label and an incrementing number (for
//! example, `Table 1.`). That rendering rule is verified through `convert`; the
//! page header, the introductory prose, and the cross-references to the label
//! customization pages carry no rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/add-title.adoc");

// Page header, the document attribute that names the caption label, and the
// introductory prose: setup and description with no rendering rule to verify
// here.
non_normative!(
    r#"
= Add a Title to a Table
:navtitle: Add a Title
// TODO/FIX: When soft unset is used from the Antora playbook, and then the attribute is reset in the document, it doesn't use the default value, so "Table" has to be explicitly assigned. Otherwise the label is simply the incremented number (i.e., "1.").
:table-caption: Table

A table can have an optional title (i.e., table caption).
To add a title to a table, use the block title syntax.

"#
);

#[test]
fn add_a_title() {
    verifies!(
        r#"
.Add an optional title to a table
[source#ex-title]
----
.A table with a title <.>
[%autowidth]
|===
|Column 1, header row |Column 2, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|===
----
<.> On the line directly above the table's opening delimiter (or above its optional attribute line, as shown here), enter a dot (`.`) directly followed by the text of the title.

The table from <<ex-title>> is displayed below.

.A table with a title
[%autowidth]
|===
|Column 1, header row |Column 2, header row

|Cell in column 1, row 2
|Cell in column 2, row 2
|===

You'll notice in the above result, that the processor automatically added _Table 1._ in front of the table's title.
"#
    );

    // The block title becomes a `<caption class="title">`, and the processor
    // prefixes it with the caption label and an incrementing number, yielding
    // `Table 1.` for the first titled table.
    let output = convert(
        ".A table with a title\n[%autowidth]\n|===\n|Column 1, header row |Column 2, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|===",
    );

    assert_css(&output, "caption.title", 1);

    assert_xpath(
        &output,
        r#"//caption[@class="title"][text()="Table 1. A table with a title"]"#,
        1,
    );
}

// Cross-references to the pages that customize or turn off the caption label:
// see-also pointers with no rule to verify here.
non_normative!(
    r#"
This title label can be xref:customize-title-label.adoc[customized] or xref:turn-off-title-label.adoc[deactivated].
"#
);
