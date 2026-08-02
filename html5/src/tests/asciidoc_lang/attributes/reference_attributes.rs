//! Coverage of the AsciiDoc language description's *Reference Document
//! Attributes* page.
//!
//! An attribute reference (`{name}`) is replaced with the attribute's value
//! when the document is processed. This page's examples — referencing a custom
//! attribute in a link and an admonition, referencing a built-in character
//! replacement attribute, and the two escape mechanisms (a leading backslash
//! and an inline passthrough) — all produce observable output, so each is
//! verified by converting the shown source and checking the substituted (or
//! preserved) text. The introduction, section headings, and the closing prose
//! on alternative escape mechanisms are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/reference-attributes.adoc");

/// Returns the rendered content of the first paragraph in `output`. Used by the
/// escape tests, whose expected text contains braces and backslashes that are
/// awkward to match through the CSS/XPath assertions.
fn first_paragraph(output: &str) -> &str {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner)
        .expect("output should contain a paragraph")
}

non_normative!(
    r#"
= Reference Document Attributes
:navtitle: Reference Attributes
:disclaimer: Don't pet the wild Wolpertingers. We're not responsible for any loss \
of hair, chocolate, or purple socks.
:url-repo: https://github.com/asciidoctor/asciidoctor

You'll likely want to insert the value of a user-defined or built-in document attribute in various locations throughout a document.
To reference a document attribute for insertion, enclose the attribute's name in curly brackets (e.g., `+{name-of-attribute}+`).
This inline element is called an *attribute reference*.
The AsciiDoc processor replaces the attribute reference with the value of the attribute.
To prevent this replacement, you can prefix the element with a backslash (e.g., `+\{name-of-attribute}+`).

[#reference-custom]
== Reference a custom attribute

Before you can reference a custom (i.e., user-defined) attribute in a document, it must first be declared using an attribute entry in the document header.
"#
);

#[test]
fn reference_custom() {
    verifies!(
        r#"
In <<ex-set-custom>>, we declare two user-defined attributes that we'll later be able to reference.

.Custom attributes set in the document header
[source#ex-set-custom,subs=+attributes]
----
= Ops Manual
:disclaimer: {disclaimer}
:url-repo: {url-repo}
----

Once you've set and assigned a value to a document attribute, you can reference that attribute throughout your document.
In <<ex-reference>>, the attribute `url-repo` is referenced twice and `disclaimer` is referenced once.

.Custom attributes referenced in the document body
[source#ex-reference]
----
Asciidoctor is {url-repo}[open source]. <.>

WARNING: {disclaimer} <.>
If you're missing a lime colored sock, file a ticket in
the {url-repo}/issues[Asciidoctor issue tracker]. <.>
(Actually, please don't).
----
<.> Attribute references can be used in macros.
<.> Attribute references can be used in blocks, such as xref:blocks:admonitions.adoc[admonitions], and inline.
Since there isn't an empty line between the `disclaimer` reference and the next sentence, the sentence will be directly appended to the end of the attribute's value when it's processed.
<.> The reference to the `url-repo` attribute is inserted to build the complete URL address, which is interpreted as a xref:macros:url-macro.adoc[URL macro].

As you can see below, the attribute references are replaced with the corresponding attribute value when the document is processed.

====
Asciidoctor is {url-repo}[open source].

WARNING: {disclaimer}
If you're missing a lime colored sock, file a ticket in the {url-repo}/issues[Asciidoctor issue tracker].
Actually, please don't.
====

"#
    );

    let output = convert(
        "= Ops Manual\n:disclaimer: Don't pet the wild Wolpertingers. We're not responsible for any loss \\\nof hair, chocolate, or purple socks.\n:url-repo: https://github.com/asciidoctor/asciidoctor\n\n\
         Asciidoctor is {url-repo}[open source].\n\n\
         WARNING: {disclaimer}\nIf you're missing a lime colored sock, file a ticket in the {url-repo}/issues[Asciidoctor issue tracker].\nActually, please don't.",
    );

    // The `url-repo` reference builds both link targets, and the `disclaimer`
    // reference is substituted verbatim into the admonition's content.
    assert_css(
        &output,
        r#"a[href="https://github.com/asciidoctor/asciidoctor"]"#,
        1,
    );
    assert_css(
        &output,
        r#"a[href="https://github.com/asciidoctor/asciidoctor/issues"]"#,
        1,
    );
    assert_css(&output, "div.admonitionblock.warning", 1);
    assert!(
        output.contains("Don&#8217;t pet the wild Wolpertingers."),
        "the disclaimer attribute should be substituted into the admonition",
    );
}

#[test]
fn reference_built_in() {
    verifies!(
        r#"
[#reference-built-in]
== Reference a built-in attribute

A built-in document attribute (i.e., a document attribute which is automatically set by the processor) is referenced the same way as a custom (i.e., user-defined) document attribute.
For instance, an AsciiDoc processor automatically sets these supported xref:character-replacement-ref.adoc[character replacement attributes].
That means that you can reference them throughout your document without having to create an attribute entry in its header.

[source]
----
TIP: Wolpertingers don't like temperatures above 100{deg}C. <.>
Our servers don't like them either.
----
<.> Reference the character replacement attribute `deg` by enclosing its name in a pair of curly brackets (`{` and `}`).

As you can see below, the attribute reference is replaced with the attribute's value when the document is processed.

TIP: Wolpertingers don't like temperatures above 100{deg}C.
Our servers don't like them either.

"#
    );

    let output = convert(
        "TIP: Wolpertingers don't like temperatures above 100{deg}C.\nOur servers don't like them either.",
    );

    // The built-in `deg` attribute is referenced with no header entry and
    // resolves to its numeric character reference inside the admonition.
    assert_css(&output, "div.admonitionblock.tip", 1);
    assert!(
        output.contains("100&#176;C"),
        "the built-in `deg` attribute should resolve to its replacement value",
    );
}

mod escape_attribute_reference {
    use crate::{convert, tests::sdd::*};

    non_normative!(
        r#"
== Escape an attribute reference

You may have a situation where a sequence of characters occurs in your content that matches the syntax of an AsciiDoc attribute reference, but is not, in fact, an AsciiDoc attribute reference.
For example, if you're documenting path templating, you may need to reference a replaceable section of a URL path, which is also enclosed in curly braces (e.g., /items/\{id}).
In this case, you need a way to escape the attribute reference so the AsciiDoc processor knows to skip over it.
Otherwise, the processor could warn about a missing attribute reference or perform an unexpected replacement.
AsciiDoc provides several ways to escape an attribute reference.

"#
    );

    #[test]
    fn prefix_with_backslash() {
        verifies!(
            r#"
=== Prefix with a backslash

You can escape an attribute reference by prefixing it with a backslash.
When the processor encounters this syntax, it will remove the backslash and pass through the remainder of what looks to be an attribute reference as written.

In <<ex-backslash-escape>>, the attribute reference is escaped using a backslash.

.An attribute reference escaped using a backslash
[#ex-backslash-escape]
----
In the path /items/\{id}, id is a path parameter.
----

In the output of <<ex-backslash-escape>>, we can see that the `\{id}` expression in the path is preserved.

====
In the path /items/\{id}, id is a path parameter.
====

"#
        );

        let output = convert("In the path /items/\\{id}, id is a path parameter.");

        // The backslash is removed and the `{id}` expression is passed through as
        // written rather than treated as an attribute reference.
        assert_eq!(
            super::first_paragraph(&output),
            "In the path /items/{id}, id is a path parameter.",
        );
    }

    #[test]
    fn backslash_remains_when_braces_do_not_hold_a_valid_attribute_name() {
        verifies!(
            r#"
Keep in mind that the backslash will only be recognized if the text between the curly braces is a valid attribute name.
If the syntax that follows the backslash does not match an attribute reference, the backslash will not be removed during processing.

"#
        );

        // The braces enclose text with a space, which is not a valid attribute
        // name, so the sequence is not an attribute reference and the backslash
        // is left in place.
        let output = convert("The pattern \\{id here} is not an attribute reference.");

        assert_eq!(
            super::first_paragraph(&output),
            "The pattern \\{id here} is not an attribute reference.",
        );
    }

    #[test]
    fn enclose_in_passthrough() {
        verifies!(
            r#"
=== Enclose in a passthrough

You can also escape an attribute reference by enclosing it in an inline passthrough.
In this case, the processor uses the normal substitution rules for the passthrough type you have chosen.

In <<ex-passthrough-escape>>, the attribute reference is escaped by enclosing it in an inline passthrough.

.An attribute reference escaped by enclosing it in an inline passthrough
[#ex-passthrough-escape]
----
In the path +/items/{id}+, id is a path parameter.
----

In the output of <<ex-passthrough-escape>>, we can see that the `\{id}` expression in the path is preserved.

====
In the path +/items/{id}+, id is a path parameter.
====

"#
        );

        let output = convert("In the path +/items/{id}+, id is a path parameter.");

        // The `+…+` passthrough passes the braces through unchanged.
        assert_eq!(
            super::first_paragraph(&output),
            "In the path /items/{id}, id is a path parameter.",
        );
    }

    #[test]
    fn enclose_in_passthrough_passes_through_unknown_attribute() {
        verifies!(
            r#"
When using an inline passthrough, you don't have to worry whether the curly braces form an attribute reference or not.
All the text between the passthrough enclosure will get passed through to the output.

"#
        );

        // Even when the braces do name a (missing) attribute, the passthrough
        // hands the text to the output without attempting substitution.
        let output = convert("In the value +{no-such-attr}+ passes through.");

        assert_eq!(
            super::first_paragraph(&output),
            "In the value {no-such-attr} passes through.",
        );
    }

    non_normative!(
        r#"
=== Alternative escape mechanisms

Attribute references are replaced by the xref:subs:attributes.adoc[attributes substitution].
Therefore, wherever you can control substitutions, you can prevent attribute references from being replaced.
This includes the inline pass macro as well as the subs attribute on a block.
See xref:subs:prevent.adoc#passthroughs[using passthroughs to prevent substitutions] for more details.
"#
    );
}
