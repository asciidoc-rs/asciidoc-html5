//! Coverage of the AsciiDoc language description's *Options Attribute* page.
//!
//! The `options` attribute has no strict schema — "any options which are not
//! recognized are ignored" — so the sidebar examples that assign made-up
//! options (`%option`, `opts=option`) produce no distinct rendered output and
//! are tracked as non-normative here; the parsing of the shorthand (`%`) and
//! formal (`opts=`) syntax is verified in `asciidoc-parser`. The examples that
//! assign *recognized* options are verified through `convert`: the built-in
//! table options `header`, `footer`, and `autowidth` produce a `<thead>`, a
//! `<tfoot>`, and the `fit-content` class; and the combined style/role/options
//! example renders a horizontal description list carrying the assigned role.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/options.adoc");

non_normative!(
    r#"
= Options Attribute

The `options` attribute (often abbreviated as `opts`) is a versatile xref:positional-and-named-attributes.adoc#named[named attribute] that can be assigned one or more values.
It can be defined globally as document attribute as well as a block attribute on an individual block.

There is no strict schema for options.
Any options which are not recognized are ignored.

"#
);

mod assign_options_to_blocks {
    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Assign options to blocks

You can assign one or more options to a block using the shorthand or formal syntax for the `options` attribute.

"#
    );

    // Shorthand `%option` on a sidebar: the option is unrecognized and therefore
    // ignored, producing no distinct rendered output. The shorthand parsing is
    // verified in `asciidoc-parser`.
    non_normative!(
        r#"
=== Shorthand options syntax for blocks

To assign an option to a block, prefix the value with a percent sign (`%`) in an attribute list.
The percent sign implicitly sets the `options` attribute.

.Sidebar block with an option assigned using the shorthand dot
[source#ex-block]
----
[%option]
****
This is a sidebar with an option assigned to it, named option.
****
----

"#
    );

    // Multiple unrecognized shorthand options: still no rendered effect (parsing
    // verified in `asciidoc-parser`).
    non_normative!(
        r#"
You can assign multiple options to a block by prefixing each value with a percent sign (`%`).

.Sidebar with two options assigned using the shorthand dot
[source#ex-two-options]
----
[%option1%option2]
****
This is a sidebar with two options assigned to it, named option1 and option2.
****
----

"#
    );

    #[test]
    fn table_with_three_options() {
        verifies!(
            r#"
For instance, consider a table with the three built-in option values, `header`, `footer`, and `autowidth`, assigned to it.
<<ex-table-short>> shows how the values are assigned using the shorthand notation.

.Table with three options assigned using the shorthand syntax
[source#ex-table-short]
----
[%header%footer%autowidth,cols=2*~]
|===
|Cell A1 |Cell B1

|Cell A2 |Cell B2

|Cell A3 |Cell B3
|===
----

"#
        );

        let output = convert(
            "[%header%footer%autowidth,cols=2*~]\n|===\n|Cell A1 |Cell B1\n\n|Cell A2 |Cell B2\n\n|Cell A3 |Cell B3\n|===",
        );

        // `header` promotes the first row to a `<thead>`, `footer` moves the last
        // row to a `<tfoot>`, and `autowidth` renders the table with the
        // `fit-content` class instead of a fixed width.
        assert_css(&output, "table.tableblock.fit-content", 1);
        assert_css(&output, "table > thead > tr > th", 2);
        assert_css(&output, "table > tfoot", 1);
    }

    // Formal `opts=option` on a sidebar: unrecognized option, no rendered effect
    // (parsing verified in `asciidoc-parser`).
    non_normative!(
        r#"
=== Formal options syntax for blocks

Explicitly set `options` or `opts`, followed by the equals sign (`=`), and then the value in an attribute list.

.Sidebar block with an option assigned using the formal syntax
[source#ex-block-formal]
----
[opts=option]
****
This is a sidebar with an option assigned to it, named option.
****
----

"#
    );

    // Multiple formal options via a comma-separated value: still unrecognized,
    // still no rendered effect (parsing verified in `asciidoc-parser`).
    non_normative!(
        r#"
Separate multiple option values with commas (`,`).

.Sidebar with three options assigned using the formal syntax
[source#ex-three-roles-formal]
----
[opts="option1,option2"]
****
This is a sidebar with two options assigned to it, option1 and option2.
****
----

"#
    );

    #[test]
    fn table_with_three_options_formal() {
        verifies!(
            r#"
Let's revisit the table in <<ex-table-short>> that has the three built-in option values, `header`, `footer`, and `autowidth`, assigned to it using the shorthand notation (`%`).
Instead of using the shorthand notation, <<ex-table-formal>> shows how the values are assigned using the formal syntax.

.Table with three options assigned using the formal syntax
[source#ex-table-formal]
----
[cols=2*~,opts="header,footer,autowidth"]
|===
|Cell A1 |Cell B1

|Cell A2 |Cell B2

|Cell A3 |Cell B3
|===
----

"#
        );

        let output = convert(
            "[cols=2*~,opts=\"header,footer,autowidth\"]\n|===\n|Cell A1 |Cell B1\n\n|Cell A2 |Cell B2\n\n|Cell A3 |Cell B3\n|===",
        );

        // The formal syntax assigns the same three built-in options, producing
        // the identical rendered structure as the shorthand form.
        assert_css(&output, "table.tableblock.fit-content", 1);
        assert_css(&output, "table > thead > tr > th", 2);
        assert_css(&output, "table > tfoot", 1);
    }
}

mod using_options_with_other_attributes {
    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Using options with other attributes

Let's consider `options` when combined with other attributes.
"#
    );

    #[test]
    fn style_role_and_options() {
        verifies!(
            r#"
The following example shows how to structure an attribute list when you have style, role, and options attributes.

.Shorthand
[source]
----
[horizontal.properties%step] <.> <.> <.>
property 1:: does stuff
property 2:: does different stuff
----
<.> xref:blocks:styles.adoc[The block style attribute], declared as `horizontal` in this example, is a positional attribute.
A block style value is always placed at the start of the attribute list.
<.> `properties` is prefixed with a dot (`.`), signifying that it's assigned to the xref:role.adoc[role attribute].
The role and options attributes can be set in either order, i.e., `[horizontal%step.properties]`.
<.> The percent sign (`%`) sets the `options` attribute and assigns the `step` value to it.

"#
        );

        let output = convert(
            "[horizontal.properties%step]\nproperty 1:: does stuff\nproperty 2:: does different stuff",
        );

        // The `horizontal` style renders the description list as a table, and the
        // dotted `properties` role becomes a class on the enclosing element. The
        // `step` option is not recognized on a description list and has no
        // rendered effect.
        assert_css(&output, "div.hdlist.properties", 1);
        assert_css(&output, "div.hdlist > table", 1);
    }

    #[test]
    fn style_role_and_options_formal() {
        verifies!(
            r#"
When you use the formal syntax, the positional and named attributes are separated by commas (`,`).

.Formal
[source]
----
[horizontal,role=properties,opts=step] <.>
property 1:: does stuff
property 2:: does different stuff
----
<.> Like in the shorthand example, named attributes such as `role` and `options` can be set in any order in the attribute list once any xref:positional-and-named-attributes.adoc#positional[positional attributes] are set.
"#
        );

        let output = convert(
            "[horizontal,role=properties,opts=step]\nproperty 1:: does stuff\nproperty 2:: does different stuff",
        );

        // The formal syntax assigns the same style and role, producing the same
        // rendered structure as the shorthand form.
        assert_css(&output, "div.hdlist.properties", 1);
        assert_css(&output, "div.hdlist > table", 1);
    }
}
