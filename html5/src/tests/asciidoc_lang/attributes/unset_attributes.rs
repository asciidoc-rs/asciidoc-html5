//! Coverage of the AsciiDoc language description's *Unset Document Attributes*
//! page.
//!
//! Unsetting a built-in attribute deactivates its behavior, and this renderer
//! reflects that: unsetting `sectids` suppresses the auto-generated section
//! IDs, unsetting `example-caption` drops the "Example N." caption prefix, and
//! toggling `sectnums` in the body turns section numbering off and back on.
//! Each is verified through `convert`. The bang (`!`) unset *syntax* itself has
//! no rendered output to show, so it is tracked as non-normative here (its
//! parsing is verified in `asciidoc-parser`).

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/unset-attributes.adoc");

non_normative!(
    r#"
= Unset Document Attributes
:navtitle: Unset Attributes

Document attributes--built-in, boolean, and custom--can be unset in the document header and document body.

"#
);

mod unset_in_header {
    use crate::{
        convert,
        tests::{
            assert_html::{assert_css, assert_xpath},
            sdd::*,
        },
    };

    non_normative!(
        r#"
== Unset a document attribute in the header

"#
    );

    // The `:!name:` / `:name!:` unset syntax produces no rendered output of its
    // own; its parsing is verified in `asciidoc-parser`.
    non_normative!(
        r#"
Document attributes are unset by adding a bang symbol (`!`) directly in front of (preferred) or after the attribute's name.
Like when setting an attribute in a document header, the attribute entry must be on its own line.
Don't add a value to the entry.

[source]
----
= Title
:!name: <.>
:name!: <.>
----
<.> An attribute is unset when a `!` is prefixed to its name (preferred).
<.> An attribute is unset when a `!` is appended to its name.

"#
    );

    #[test]
    fn sectids() {
        verifies!(
            r#"
Let's use an attribute entry to turn off the built-in boolean attribute named `sectids`.
The AsciiDoc processor automatically sets `sectids` at processing time unless you unset it.
The `sectids` attribute xref:sections:auto-ids.adoc[generates an ID for each section] from the section's title.

.Unset a boolean attribute
[source#ex-unset-boolean]
----
= Document Title
:!sectids: <.>
----
<.> On a new line, type a colon (`:`), directly followed by a bang symbol (`!`), the attribute's name, and then another colon (`:`).
After the closing colon, press kbd:[Enter].
The attribute is now unset and its behavior won't be applied to the document.

Once an attribute is unset, its behavior is deactivated.
When `sectids` is unset, the AsciiDoc processor will not generate IDs from section titles at processing time.

"#
        );

        // With `sectids` unset, the section heading carries no auto-generated id.
        let output = convert("= Document Title\n:!sectids:\n\n== First Section\n\nText.");
        assert_css(&output, "h2", 1);
        assert_css(&output, "h2[id]", 0);
    }

    #[test]
    fn example_caption() {
        verifies!(
            r#"
Let's unset the built-in attribute `example-caption`.
This is an attribute that is set and assigned a default value of `Example` automatically by the AsciiDoc processor when you use an example block.

.Unset an automatically declared attribute
[source#ex-unset-built-in]
----
= Title
:!example-caption: <.>
----
<.> Example blocks won't be labeled and numbered, e.g., Example 1, because the attribute controlling that behavior is unset with the leading `!`.

"#
        );

        // With `example-caption` unset, the example block's title is rendered
        // verbatim with no "Example N." caption prefix.
        let output = convert("= Title\n:!example-caption:\n\n.My Example\n====\nBody\n====");
        assert_css(&output, "div.exampleblock", 1);
        assert_xpath(&output, r#"//div[@class="title"][text()="My Example"]"#, 1);
    }
}

mod unset_in_body {
    use crate::{
        convert,
        tests::{assert_html::assert_xpath, sdd::*},
    };

    non_normative!(
        r#"
== Unset a document attribute in the body

Custom document attributes and some built-in document attributes can be turned off in the body of the document using an attribute entry and the bang symbol (`!`) as described in the previous section.
For example, let's say you set the section numbering attribute in the header of your document; however, you don't want the two sections midway through the document to be numbered.
To disable the numbering on these two sections, you'd unset `sectnums` before the first section you didn't want numbered and then reset it when you wanted the numbering to start again.

"#
    );

    #[test]
    fn sectnums_example() {
        verifies!(
            r#"
[source]
----
= Title
:sectnums: <.>

== Section Title

:!sectnums: <.>
== Section Title

=== Section Title

:sectnums: <.>
== Section Title
----
<.> The `sectnums` attribute is set in the header to activate section numbering throughout the document.
<.> `sectnums` is unset by adding a `!` to it's name.
The `!` can be placed either before or after the attribute's name.
The attribute entry must be placed on its own line.
All of the sections below where the attribute is unset will not be numbered.
<.> `sectnums` is set and all subsequent sections will be numbered.
"#
        );

        let output = convert(
            "= Title\n:sectnums:\n\n== Section Title\n\n:!sectnums:\n== Section Title\n\n=== Section Title\n\n:sectnums:\n== Section Title",
        );

        // Numbering is on for the first section, off for the section (and its
        // subsection) after `sectnums` is unset, and on again after it is reset.
        assert_xpath(&output, r#"//h2[text()="1. Section Title"]"#, 1);
        assert_xpath(&output, r#"//h2[text()="Section Title"]"#, 1);
        assert_xpath(&output, r#"//h3[text()="Section Title"]"#, 1);
        assert_xpath(&output, r#"//h2[text()="2. Section Title"]"#, 1);
    }
}
