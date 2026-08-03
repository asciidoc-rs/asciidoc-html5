//! Coverage of the AsciiDoc language description's `width` page.
//!
//! The page shows how to constrain a table's overall width. A `width` attribute
//! becomes an inline `style="width: N%;"` on the `<table>`, the `autowidth`
//! option applies the `fit-content` class and leaves each `<col>` bare, the
//! `stretch` role (or `width=100%`) adds the `stretch` class alongside
//! `fit-content`, and the `~` column specifier mixes a fixed column with
//! autowidth columns. Each rendering rule is checked through `convert`; the
//! introduction, section headings, and the DocBook WARNING are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/width.adoc");

// The base-h table reused by every example on this page, standing in for the
// `include::example$row.adoc[tag=base-h]` directive.
const BASE_H: &str = "|===\n|Column 1, header row |Column 2, header row |Column 3, header row\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|Cell in column 3, row 2\n\n|Cell in column 1, row 3\n|Cell in column 2, row 3\n|Cell in column 3, row 3\n|===";

// Title and the introductory statement that a table spans the content area by
// default; the width-constraining behavior is exercised by the examples below.
non_normative!(
    r#"
= Table Width

By default, a table will span the width of the content area.

"#
);

// Section heading only.
non_normative!(
    r#"
== Fixed width

"#
);

#[test]
fn fixed_width() {
    verifies!(
        r#"
To constrain the width of the table to a fixed value, set the `width` attribute in the table's attribute list.
The width is an integer percentage value ranging from 1 to 100.
The `%` sign is optional.

.Table with width set to 75%
[source#ex-fixed]
----
[width=75%]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-fixed>>
[width=75%]
include::example$row.adoc[tag=base-h]

"#
    );

    // A `width` attribute becomes an inline width style on the `<table>` itself.
    let output = convert(&format!("[width=75%]\n{BASE_H}"));

    assert_css(&output, "table[style=\"width: 75%;\"]", 1);
}

// Section heading only.
non_normative!(
    r#"
== Autowidth

"#
);

#[test]
fn autowidth() {
    verifies!(
        r#"
Alternately, you can make the width fit the content by setting the `autowidth` option.
The columns inherit this setting, so individual columns will also be sized according to the content.

.Table using autowidth
[source#ex-auto]
----
[%autowidth]
include::example$row.adoc[tag=base-h]
----

.Result of <<ex-auto>>
[%autowidth]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `autowidth` option applies the `fit-content` class to the table, and
    // the columns inherit it: every `<col>` is emitted bare, with no width style.
    let output = convert(&format!("[%autowidth]\n{BASE_H}"));

    assert_css(&output, "table.fit-content", 1);
    assert_css(&output, "table.stretch", 0);
    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "col[style]", 0);
}

#[test]
fn autowidth_stretch() {
    verifies!(
        r#"
If you want each column to have an automatic width, but want the table to span the width of the content area, add the `stretch` role to the table.
(Alternatively, you can set the `width` attribute to `100%`.)

.Full-width table with autowidth columns
[source#ex-stretch]
----
[%autowidth.stretch]
include::example$row.adoc[tag=base-h]
----

The table from <<ex-stretch>> is rendered below.
The columns are sized to the content, but the table spans the width of the page.

.Result of <<ex-stretch>>
[%autowidth.stretch]
include::example$row.adoc[tag=base-h]

"#
    );

    // The `stretch` role keeps autowidth columns but makes the table span the
    // page, so both `fit-content` and `stretch` appear on the `<table>`.
    let output = convert(&format!("[%autowidth.stretch]\n{BASE_H}"));

    assert_css(&output, "table.fit-content", 1);
    assert_css(&output, "table.stretch", 1);

    // Setting `width` to 100% is the stated alternative and likewise stretches
    // the table across the page.
    let alternative = convert(&format!("[width=100%]\n{BASE_H}"));

    assert_css(&alternative, "table.stretch", 1);
}

// The autowidth caveat concerns the DocBook converter, a backend this crate
// does not target, so there is no HTML rendering rule to verify.
non_normative!(
    r#"
WARNING: The `autowidth` option is not recognized by the DocBook converter.

"#
);

// Section heading only.
non_normative!(
    r#"
== Mix fixed and autowidth columns

"#
);

#[test]
fn mix_fixed_and_autowidth_columns() {
    verifies!(
        r#"
If you want to apply `autowidth` only to certain columns, use the special value `~` as the width of the column.
In this case, width values are assumed to be a percentage value (i.e., 100-based).

.Table with fixed and autowidth columns
[source#ex-mix]
----
[cols="25h,~,~"]
|===
|small |as big as the column needs to be |the rest
|===
----

.Result of <<ex-mix>>
[cols="25h,~,~"]
|===
|small |as big as the column needs to be |the rest
|===
"#
    );

    // The `25h` specifier fixes the first column at 25%, while each `~` column is
    // autowidth and emits a bare `<col>` with no width style.
    let output = convert(
        "[cols=\"25h,~,~\"]\n|===\n|small |as big as the column needs to be |the rest\n|===",
    );

    assert_css(&output, "colgroup > col", 3);
    assert_css(&output, "col[style=\"width: 25%;\"]", 1);
    assert_css(&output, "col[style]", 1);
}
