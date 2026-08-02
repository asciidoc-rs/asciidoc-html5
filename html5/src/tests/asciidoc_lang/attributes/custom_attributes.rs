//! Coverage of the AsciiDoc language description's *Declare Custom Document
//! Attributes* page.
//!
//! A custom attribute is set with an attribute entry and referenced by name.
//! Two of the page's claims produce observable output and are verified through
//! `convert`: an attribute name is lower-cased before it is stored (so `URL` is
//! referenced as `{url}`), and a long soft-wrapped value folds into a single
//! line when referenced. The detailed name-grammar rules and the value-content
//! summary describe parser behavior with no distinct rendering and are tracked
//! as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/custom-attributes.adoc");

/// Returns the rendered content of the first paragraph in `output`.
fn first_paragraph(output: &str) -> &str {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner)
        .unwrap_or_default()
}

non_normative!(
    r#"
= Declare Custom Document Attributes
:navtitle: Declare Custom Attributes
// [#set-user-defined]

When you find yourself typing the same text repeatedly, or text that often needs to be updated, consider creating your own attribute.

"#
);

mod user_defined_names {
    use super::*;
    use crate::{convert, tests::assert_html::assert_css};

    non_normative!(
        r#"
[#user-defined-names]
== User-defined attribute names and values

A user-defined attribute must have a name and explicitly assigned value.

The attribute's name must:

* be at least one character long,
* begin with a word character (A-Z, a-z, 0-9, or _), and
* only contain word characters and hyphens.

The name cannot contain dots or spaces.

"#
    );

    #[test]
    fn name_is_lowercased() {
        verifies!(
            r#"
Although uppercase characters are permitted in an attribute name, the name is converted to lowercase before being stored.
For example, `URL` and `Url` are treated as `url`.
A best practice is to only use lowercase letters in the name and avoid starting the name with a number.

"#
        );

        // The attribute is set as `URL` but stored as `url`, so the lowercase
        // `{url}` reference resolves to its value.
        let output = convert(":URL: https://example.com\n\n{url}");
        assert_css(&output, r#"a[href="https://example.com"]"#, 1);
    }

    non_normative!(
        r#"
[[user-values]]Attribute values can:

* be any inline content, and
* contain line breaks, but only if an xref:wrap-values.adoc#hard[explicit line continuation] (`+`) is used.

"#
    );
}

mod create_a_custom_attribute_and_value {
    use super::*;
    use crate::convert;

    #[test]
    fn create_a_custom_attribute_and_value() {
        verifies!(
            r#"
== Create a custom attribute and value

A prime use case for attribute entries is to promote frequently used text and URLs to the top of the document.

.Create a user-defined attribute and value
[source#ex-user-set]
----
:disclaimer: Don't pet the wild Wolpertingers. If you let them into your system, we're \ <.>
not responsible for any loss of hair, chocolate, or purple socks.
:url-repo: https://github.com/asciidoctor/asciidoctor
----
<.> Long values can be xref:wrap-values.adoc[soft wrapped] using a backslash (`\`).

Now, you can xref:reference-attributes.adoc#reference-custom[reference these attributes] throughout the document.
"#
        );

        // The soft-wrapped `disclaimer` value folds into a single line when it is
        // referenced.
        let output = convert(
            ":disclaimer: Don't pet the wild Wolpertingers. If you let them into your system, we're \\\nnot responsible for any loss of hair, chocolate, or purple socks.\n:url-repo: https://github.com/asciidoctor/asciidoctor\n\n{disclaimer}",
        );
        assert_eq!(
            first_paragraph(&output),
            "Don&#8217;t pet the wild Wolpertingers. If you let them into your system, we&#8217;re not responsible for any loss of hair, chocolate, or purple socks.",
        );
    }
}
