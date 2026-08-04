//! Coverage of the AsciiDoc language description's *Mailto Macro* page.
//!
//! The `mailto:<address>[…]` macro is a URL-macro specialization for email
//! links: the first positional attribute is the link text, the second the
//! subject, and the third the body (the latter two encoded into the `mailto:`
//! query string). Named attributes such as `role` are also honored. Every
//! documented form is verified through `convert`; `asciidoc-parser` produces
//! the inline HTML and this crate emits it verbatim, matching Asciidoctor
//! 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/mailto-macro.adoc");

non_normative!(
    r#"
= Mailto Macro
:page-aliases: email-macro.adoc

The mailto macro is a specialization of the xref:url-macro.adoc[URL macro] that adds support for defining an email link with text and augmenting it with additional metadata, such as a subject and body.

== Link text and named attributes

"#
);

// The first positional attribute is the link text.
#[test]
fn link_text() {
    verifies!(
        r#"
Using an attribute list, you can specify link text as well as named attributes such as `id` and `role`.
Unlike other URL macros, you must add the `mailto:` prefix in front of the email address in order to append an attribute list.

Here's an example of an email link with explicit link text.

----
mailto:join@discuss.example.org[Subscribe]
----

"#
    );

    assert!(convert("mailto:join@discuss.example.org[Subscribe]")
        .contains(r#"<a href="mailto:join@discuss.example.org">Subscribe</a>"#));
}

// A `role` named attribute is applied as a CSS class on the link.
#[test]
fn add_role() {
    verifies!(
        r#"
If you want to add a role to this link, you can do so by appending the `role` attribute after a comma.

----
mailto:join@discuss.example.org[Subscribe,role=email]
----

"#
    );

    assert!(
        convert("mailto:join@discuss.example.org[Subscribe,role=email]")
            .contains(r#"<a href="mailto:join@discuss.example.org" class="email">Subscribe</a>"#)
    );
}

// Link text containing a comma must be double-quoted so the comma is not parsed
// as an attribute separator.
#[test]
fn link_text_contains_comma() {
    verifies!(
        r#"
If the link text contains a comma, you must enclose the text in double quotes.
Otherwise, the portion of the text that follows the comma will be interpreted as additional attribute list entries.

----
mailto:join@discuss.example.org["Click, subscribe, and participate!"]
----

"#
    );

    assert!(
        convert(r#"mailto:join@discuss.example.org["Click, subscribe, and participate!"]"#).contains(
            r#"<a href="mailto:join@discuss.example.org">Click, subscribe, and participate!</a>"#
        )
    );
}

non_normative!(
    r#"
To learn more about how the attributes are parsed, refer to xref:link-macro-attribute-parsing.adoc[attribute parsing].

== Subject and body

"#
);

// The second positional attribute is encoded as the `subject` query parameter.
#[test]
fn second_arg_is_subject() {
    verifies!(
        r#"
Like with other URL macros, the first positional attribute of the email macro is the link text.
If a comma is present in the text, and the text is not enclosed in quotes, or the comma comes after the closing quote, the next positional attribute is treated as the subject line.

For example, you can configure the email link to populate the subject line when the reader clicks the link as follows:

----
mailto:join@discuss.example.org[Subscribe,Subscribe me]
----

When the reader clicks the link, a conforming email client will fill in the subject line with "`Subscribe me`".

"#
    );

    assert!(
        convert("mailto:join@discuss.example.org[Subscribe,Subscribe me]").contains(
            r#"<a href="mailto:join@discuss.example.org?subject=Subscribe%20me">Subscribe</a>"#
        )
    );
}

// The third positional attribute is encoded as the `body` query parameter
// (joined to the subject with `&amp;`).
#[test]
fn third_arg_is_body() {
    verifies!(
        r#"
If you want the body of the email to also be populated, specify the body text in the third positional argument.

----
mailto:join@discuss.example.org[Subscribe,Subscribe me,I want to participate.]
----

When the reader clicks the link, a conforming email client will fill in the body with "`I want to participate.`"

"#
    );

    assert!(
        convert("mailto:join@discuss.example.org[Subscribe,Subscribe me,I want to participate.]")
            .contains(
                r#"<a href="mailto:join@discuss.example.org?subject=Subscribe%20me&amp;body=I%20want%20to%20participate.">Subscribe</a>"#
            )
    );
}

// Leaving the first positional attribute empty reuses the email address as the
// link text.
#[test]
fn reuse_email_address_as_link_text() {
    verifies!(
        r#"
If you want to reuse the email address as the link text, leave the first positional attribute empty.

----
mailto:join@discuss.example.org[,Subscribe me,I want to participate.]
----

"#
    );

    assert!(
        convert("mailto:join@discuss.example.org[,Subscribe me,I want to participate.]").contains(
            r#"<a href="mailto:join@discuss.example.org?subject=Subscribe%20me&amp;body=I%20want%20to%20participate.">join@discuss.example.org</a>"#
        )
    );
}

// Only a subject (no body): leave off the third attribute.
#[test]
fn only_subject() {
    verifies!(
        r#"
If you only want to specify a subject, leave off the body.

----
mailto:join@discuss.example.org[,Subscribe me]
----

"#
    );

    assert!(convert("mailto:join@discuss.example.org[,Subscribe me]").contains(
        r#"<a href="mailto:join@discuss.example.org?subject=Subscribe%20me">join@discuss.example.org</a>"#
    ));
}

// A subject or body containing a comma must be double-quoted.
#[test]
fn subject_or_body_with_comma() {
    verifies!(
        r#"
If either the subject or body contains a comma, that value must be enclosed in double quotes.

----
mailto:join@discuss.example.org[Subscribe,"I want to participate, so please subscribe me"]
----

"#
    );

    assert!(
        convert(r#"mailto:join@discuss.example.org[Subscribe,"I want to participate, so please subscribe me"]"#)
            .contains(
                r#"<a href="mailto:join@discuss.example.org?subject=I%20want%20to%20participate%2C%20so%20please%20subscribe%20me">Subscribe</a>"#
            )
    );
}

non_normative!(
    r#"
To learn more about how the attributes are parsed, refer to xref:link-macro-attribute-parsing.adoc[attribute parsing].
"#
);
