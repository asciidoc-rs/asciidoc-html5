//! Coverage of the AsciiDoc language description's `striping` page.
//!
//! The page enables zebra-striping on table rows. The `stripes` block attribute
//! applies the CSS class `stripes-<value>` to the `<table>`, the same class can
//! be applied directly through a role shorthand, and the `table-stripes`
//! document attribute sets a document-wide default. Each of those
//! class-applying rules is checked through `convert`; the introduction, the
//! NOTE that the actual shading depends on the stylesheet, the section
//! headings, and the attribute value list are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/striping.adoc");

// Title and the introduction to the zebra-striping feature.
non_normative!(
    r#"
= Table Striping

You can add zebra-striping to rows of a table.
When this feature is enabled, the specified rows are shaded using a background color to create a zebra striping effect.

"#
);

// The actual shading is performed by CSS in the stylesheet, so the visual
// striping effect itself is stylesheet-dependent and not verifiable here; only
// the class this crate emits is checked, below.
non_normative!(
    r#"
NOTE: In the HTML output, table striping is done using CSS and thus depends on the stylesheet to supply the necessary styles.
The default stylesheet for Asciidoctor includes the necessary styles for table striping.

"#
);

// Section heading only.
non_normative!(
    r#"
== Striping attributes

"#
);

// Describes the `stripes` attribute, its default of `none`, the `table-stripes`
// document attribute, and enumerates the accepted values; the resulting CSS
// class is verified in the examples that follow.
non_normative!(
    r#"
Which rows are striped is controlled using the `stripes` attribute on the table.
The stripes attribute defaults to the value `none` (implied value), which means rows are not striped by default.
This default can be changed by setting the `table-stripes` document attribute.
You can override the default value by setting the stripes attribute on the table.

The `stripes` attribute on a table and the `table-stripes` document attribute accept the following values:

* `none` - no rows are shaded (default)
* `even` - even rows are shaded
* `odd` - odd rows are shaded
* `all` - all rows are shaded
* `hover` - the row under the mouse cursor is shaded (HTML only)

"#
);

// Section heading only.
non_normative!(
    r#"
== stripes block attribute

"#
);

#[test]
fn stripes_block_attribute() {
    verifies!(
        r#"
In the following example, the stripes are enabled for even rows in the table body (the row that contains A2 and B2).

[source]
----
[cols=2*,stripes=even]
|===
|A1
|B1
|A2
|B2
|A3
|B3
|===
----

Under the covers, the stripes attribute applies the CSS class `stripes-<value>` (e.g., `stripes-none`) to the table tag.
As a shorthand, you can simply apply the CSS class to the table directly using a role.

"#
    );

    // Setting `stripes=even` applies the `stripes-even` class to the `<table>`.
    let output = convert("[cols=2*,stripes=even]\n|===\n|A1\n|B1\n|A2\n|B2\n|A3\n|B3\n|===");

    assert_css(&output, "table.stripes-even", 1);
}

#[test]
fn stripes_role_shorthand() {
    verifies!(
        r#"
[source]
----
[.stripes-even,cols=2*]
|===
|A1
|B1
|A2
|B2
|A3
|B3
|===
----

"#
    );

    // Applying the `stripes-even` role directly puts the same class on the
    // `<table>` as the `stripes` attribute does.
    let output = convert("[.stripes-even,cols=2*]\n|===\n|A1\n|B1\n|A2\n|B2\n|A3\n|B3\n|===");

    assert_css(&output, "table.stripes-even", 1);
}

// Section heading only.
non_normative!(
    r#"
== table-stripes attribute

"#
);

#[test]
fn table_stripes_document_attribute() {
    verifies!(
        r#"
If you want to apply stripes to all tables in the document, set the `table-stripes` attribute in the document header.
You can still override this setting per table.

[source]
----
= Document Title
:table-stripes: even

[cols=2*]
|===
|A1
|B1
|A2
|B2
|A3
|B3
|===
----
"#
    );

    // The `table-stripes` document attribute sets a document-wide default, so a
    // table with no `stripes` attribute of its own still gets `stripes-even`.
    let output = convert(
        "= Document Title\n:table-stripes: even\n\n[cols=2*]\n|===\n|A1\n|B1\n|A2\n|B2\n|A3\n|B3\n|===",
    );

    assert_css(&output, "table.stripes-even", 1);
}
