//! Coverage of the AsciiDoc language description's *Declare Built-In Document
//! Attributes* page.
//!
//! Setting a built-in attribute with an empty value activates it with its
//! default value; the page's `toc` example is verified through `convert` by
//! confirming the Table of Contents is rendered, and the asset-directory
//! example is verified by confirming an explicit `imagesdir` replaces the
//! default when resolving an image target. The `doctype: book` override has no
//! observable effect in this renderer (the book doctype is not yet
//! implemented), so it is tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/built-in-attributes.adoc");

non_normative!(
    r#"
= Declare Built-In Document Attributes
:navtitle: Declare Built-In Attributes

An AsciiDoc processor has numerous attributes reserved for special purposes.
*Built-in attributes* add, configure, and control common features in a document.
Many built-in attributes only take effect when defined in the document header with an attribute entry.

"#
);

mod use_an_attributes_default_value {
    use super::*;
    use crate::{convert, tests::assert_html::assert_css};

    #[test]
    fn empty_value_activates_the_default() {
        verifies!(
            r#"
== Use an attribute's default value

Many built-in attributes have a default value.
When you want to activate a built-in attribute and assign it its default value, you can leave the value in the attribute entry empty.

For example, to turn on the xref:toc:index.adoc[Table of Contents for a document], you set the `toc` attribute using an attribute entry in the document header.

[source]
----
= Title of Document
:toc:
----

The default value of an activated attribute will be assigned at processing time, if:

. it has a default value, and
. the value in the attribute entry is left empty

In the example above, the default value of `auto` will be assigned to `toc` since the value was left empty in the attribute entry.

"#
        );

        // Setting `toc` with an empty value activates it with its default `auto`
        // value, rendering the Table of Contents with an entry per section.
        let output = convert("= Title of Document\n:toc:\n\n== One\n\nText.\n\n== Two\n\nText.");
        assert_css(&output, "div#toc.toc", 1);
        assert_css(&output, "#toc ul.sectlevel1 > li > a", 2);
    }
}

// The `doctype: book` example has no rendered result to check: this renderer
// does not yet implement the book doctype, so `:doctype: book` produces output
// identical to the default `article`. Its parsing is covered by
// `asciidoc-parser`.
non_normative!(
    r#"
== Override an attribute's default value

You may not want to use the default value of a built-in attribute.
In the next example, we'll override the default value of an attribute that the AsciiDoc processor sets automatically.
The built-in attribute `doctype` is automatically set and assigned a value of `article` at processing time.
However, if you want to use AsciiDoc's book features, the `doctype` attribute needs to be assigned the `book` value.

[source]
----
= Title of My Document
:doctype: book <.>
----
<.> Set `doctype` in the document header and assign it the value `book`.
Explicit values must be offset from the closing colon (`:`) by at least one space.

To override an attribute's default value, you have to explicitly assign a value when you set the attribute.
The value assigned to an attribute in the document header replaces the default value (assuming the attribute is not locked via the CLI or API).

"#
);

mod override_a_default_asset_directory_value {
    use super::*;
    use crate::{convert, tests::assert_html::assert_css};

    #[test]
    fn asset_directory_attribute_replaces_the_default() {
        verifies!(
            r#"
//Change to override a default value with a user-defined value
=== Override a default asset directory value

You can also use the built-in asset directory attributes to customize the base path to images (default: `_empty_`), icons (default: `./images/icons`), stylesheets (default: `./stylesheets`) and JavaScript files (default: `./javascripts`).

.Replace the default values of the built-in asset directory attributes
[source]
----
= My Document
:imagesdir: ./images
:iconsdir: ./icons
:stylesdir: ./styles
:scriptsdir: ./js
----

The four built-in attributes in the example above have default values that are automatically set at processing time.
However, in the example, they're being set and assigned explicit values in the document header.
This explicit user-defined value replaces the default value (assuming the attribute is not locked via the CLI or API).

"#
        );

        // The explicit `imagesdir` value replaces its (empty) default, so it is
        // prepended to the target of an image reference.
        let output = convert(
            "= My Document\n:imagesdir: ./images\n:iconsdir: ./icons\n:stylesdir: ./styles\n:scriptsdir: ./js\n\nimage::a.png[X]",
        );
        assert_css(&output, r#"img[src="./images/a.png"]"#, 1);
    }
}

non_normative!(
    r#"
////
Many built-in attributes have a built-in value that is designated as the default value.
This default value is assigned when the attribute is set and its value is left empty.
For example, the xref:sections:id.adoc#separator[ID word separator attribute] can accept <<user-values,user-defined values>> and it has one default value.
If you set `idseparator` and leave the value empty, the default value will be assigned automatically when the document is processed.

[source]
----
:idseparator: <1>
----
<1> The words in automatically generated IDs will be separated with an underscore (`_`), the attribute's default value, because the value is empty.

To override the default value of an attribute, you have to explicitly assign a new value when you set the attribute.

[source]
----
:idseparator: - <1>
----
<1> The words in automatically generated IDs will be separated with a hyphen (`-`).
The value must be offset from the attribute's name by a space.
////
"#
);
