//! Coverage of the AsciiDoc language description's *Text Span and Built-in
//! Roles* page.
//!
//! A hash-delimited span carrying at least one role becomes a plain `<span>`
//! with that role as its class, adding no other formatting; with no role it
//! falls back to highlight. This crate applies each built-in role
//! (`underline`, `overline`, `line-through`, `nobreak`, `nowrap`, `pre-wrap`)
//! as the span's class, so those claims are verified through `convert`. The
//! title, section headings, anchors, and the deprecated-roles note are tracked
//! as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/text-span-built-in-roles.adoc");

non_normative!(
    r#"
= Text Span and Built-in Roles

Instead of applying explicit formatting to text, you can enclose a span of a text in a non-formatting element.
This type of markup is referred to as a text span (formerly known as _unquoted text_).
It's purpose is to allow attributes such as role and ID to be applied to unformatted text.
Though those attributes can still be used to apply styles to the text.

== Text span syntax

"#
);

// A hash-delimited span with a role becomes a bare `<span>` carrying that role
// as its CSS class and no other formatting; the same delimiters with no role
// fall back to a highlight (`<mark>`).
#[test]
fn text_span_syntax() {
    verifies!(
        r#"
When text is enclosed in a pair of single or double hash symbols (`#`) *and* has at least one role, the role(s) will be applied to that text without adding any other implicit formatting.

CAUTION: If no attrlist is present, the formatting pair will be interpreted as xref:highlight.adoc[highlighted text] instead.

.Text span syntax
[#ex-text-span]
----
include::example$text.adoc[tag=text-span]
----

When <<ex-text-span>> is converted to HTML, it translates into the following output.

.Text span HTML output
[,html]
----
include::example$text.adoc[tag=text-span-html]
----

As you can see, it's up to the stylesheet to provide styles for this element.
Typically, this means you'll need to xref:custom-inline-styles.adoc[define custom inline styles] that map to the corresponding class.
In this case, since `underline` is a built-in role, the style is provided for you.

"#
    );

    let span = convert("The text [.underline]#underline me# is underlined.\n");
    assert!(span.contains(r#"The text <span class="underline">underline me</span> is underlined."#));
    // The role adds no other formatting: a plain span, not a mark.
    assert_css(&span, "span.underline", 1);
    assert_css(&span, "mark", 0);

    // With no attrlist, the same delimiters fall back to a highlight.
    let no_role = convert("The text #underline me# is underlined.\n");
    assert_css(&no_role, "mark", 1);
    assert_css(&no_role, "span.underline", 0);
}

non_normative!(
    r#"
[#built-in]
== Built-in roles for text

"#
);

// Each built-in role is applied as the span's CSS class, adding no other
// implicit formatting.
#[test]
fn built_in_roles() {
    verifies!(
        r#"
The AsciiDoc language provides a handful of built-in roles you can use to provide formatting hints for the text.
While these roles are often used with a text span, they can also be used with any other formatted text for which a role is accepted.

WARNING: Not all converters recognize these roles, though you can expect them to at least be supported by the HTML converter.

These roles are as follows:

underline:: Applies an underline decoration to the span of text.
overline:: Applies an overline decoration to the span of text.
line-through:: Applies a line-through (aka strikethrough) decoration to the span of text.
nobreak:: Disables words within the span of text from being broken.
nowrap:: Prevents the span of text from wrapping at all.
pre-wrap:: Prevents sequences of space and space-like characters from being collapsed (i.e., all spaces are preserved).

"#
    );

    for role in [
        "underline",
        "overline",
        "line-through",
        "nobreak",
        "nowrap",
        "pre-wrap",
    ] {
        let out = convert(&format!("[.{role}]#styled#\n"));
        assert!(
            out.contains(&format!(r#"<span class="{role}">styled</span>"#)),
            "role {role} did not render as its own class: {out}"
        );
    }
}

non_normative!(
    r#"
[#deprecated]
=== Deprecated roles

There are several built-in roles that were once supported in AsciiDoc, but have since been deprecated.
These roles include `big`, `small`, named colors (e.g., `aqua`), and named background colors (e.g., `aqua-background`).
You should create your own semantic roles in place of these deprecated roles.
"#
);
