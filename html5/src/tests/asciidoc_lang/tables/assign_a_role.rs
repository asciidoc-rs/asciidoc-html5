//! Coverage of the AsciiDoc language description's *Assign a Role to a Table*
//! page.
//!
//! The page shows that a table's `role` attribute becomes a CSS class on the
//! rendered `<table>`, and that the preferred shorthand is the leading `.role`
//! form in the block attribute list. That rendering rule is verified through
//! `convert`; the page title carries no rule of its own and is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/assign-a-role.adoc");

// Page title only.
non_normative!(
    r#"
= Assign a Role to a Table

"#
);

#[test]
fn assign_a_role() {
    verifies!(
        r#"
Like with all blocks, you can add a role to a table using the `role` attribute.
The role attribute becomes a CSS class when converted to HTML.
The preferred shorthand for assigning the role attribute is to put the role name in the first position of the block attribute list prefixed with a `.` character, as shown here:

[source]
----
[.name-of-role]
include::example$table.adoc[tag=base]
----
"#
    );

    // The `.name-of-role` shorthand assigns the `role` attribute, which the HTML
    // backend renders as an extra CSS class on the `<table>` element. The
    // `include::` directive above resolves to the `base` table snippet.
    let output = convert(
        "[.name-of-role]\n|===\n|Cell in column 1, row 1\n|Cell in column 2, row 1\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|===",
    );

    assert_css(&output, "table.tableblock.name-of-role", 1);
}
