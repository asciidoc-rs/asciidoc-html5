//! Coverage of the AsciiDoc language description's *Nesting Tables* page.
//!
//! The page teaches that a cell with the AsciiDoc table style (`a`) may contain
//! a nested table, distinguished from the enclosing table by the `!===`
//! delimiter and a different cell separator. That the nested table renders
//! inside the `a` cell with its own independent format is verified through
//! `convert`; the title, the DocBook-backend NOTE, and the closing
//! recommendation carry no HTML rule of their own and are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/nested.adoc");

// Title and the descriptive syntax prose for nested tables; the rendered
// behavior is verified through the example below.
non_normative!(
    r#"
= Nesting Tables

Table cells marked with the AsciiDoc table style (`a`) support nested tables in addition to normal block content.
To distinguish the inner table from the enclosing one, you need to use `!===` as the table delimiter and a cell separator that differs from that used for the enclosing table.
The default cell separator for a nested table is `!`, though you can choose another character by defining the `separator` attribute on the table.

"#
);

// A note about DocBook 5.0 validity: non-HTML backend behavior with no bearing
// on the HTML output.
non_normative!(
    r#"
NOTE: Although nested tables are not technically valid in DocBook 5.0, the DocBook toolchain processes them anyway.

"#
);

#[test]
fn nested_table_in_an_asciidoc_cell() {
    verifies!(
        r#"
The following example contains a nested table in the last cell.
Notice the nested table has its own format, independent of that of the outer table:

[source]
----
include::example$table.adoc[tag=nested]
----

.Result: A nested table
include::example$table.adoc[tag=nested]

"#
    );

    let output = convert(
        r#"[cols="1,2a"]
|===
| Col 1 | Col 2

| Cell 1.1
| Cell 1.2

| Cell 2.1
| Cell 2.2

[cols="2,1"]
!===
! Col1 ! Col2

! C11
! C12

!===

|==="#,
    );

    // The nested table renders inside the AsciiDoc (`a`) cell's nested document,
    // for two tables in all.
    assert_css(&output, "table", 2);
    assert_css(&output, "td div.content table", 1);

    // The nested table keeps its own format (its own two-column `<colgroup>`),
    // independent of the outer table.
    assert_css(&output, "td table colgroup col", 2);

    // The nested table's own cells render with its `!` separator.
    assert_xpath(&output, "//td//table//td/p[contains(text(), \"C11\")]", 1);
}

// A closing recommendation to use nested tables sparingly; advisory prose with
// no rendering rule.
non_normative!(
    r#"
We recommend using nested tables sparingly.
There's usually a better way to present the information.
"#
);
