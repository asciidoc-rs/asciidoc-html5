//! Coverage of the AsciiDoc language description's *Using Custom Inline Styles*
//! page.
//!
//! A role assigned to formatted text — built-in or custom — becomes the CSS
//! class on the emitted element. This crate does exactly that, so the built-in
//! and custom role examples are verified through `convert`. The title, the
//! section heading, and the illustrative CSS stylesheet snippet are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/custom-inline-styles.adoc");

non_normative!(
    r#"
= Using Custom Inline Styles

== Custom style syntax

"#
);

// A built-in role assigned as the first positional attribute becomes the CSS
// class on the emitted span.
#[test]
fn custom_style_syntax_with_built_in_role() {
    verifies!(
        r#"
You can assign built-in roles (e.g., `big` or `underline`) or custom roles (e.g., `term` or `required`) to any formatted text.
These roles, in turn, can be used to apply styles to the text.
In HTML, this is done by mapping styles to the role in the stylesheet using a CSS class selector.

.Text with built-in role
[#ex-built-in]
----
include::example$text.adoc[tag=css-co]
----
. The first positional attribute is treated as a role.
You can assign it a custom or built-in CSS class.

The results of <<ex-built-in>> are displayed below.

====
include::example$text.adoc[tag=css]
====

"#
    );

    let small = convert("Do werewolves believe in [.small]#small print#?\n");
    assert!(small.contains(r#"<span class="small">small print</span>"#));

    // The first positional attribute (no leading dot) is treated as the role.
    let big = convert("[big]##O##nce upon an infinite loop.\n");
    assert!(big.contains(r#"<span class="big">O</span>nce upon an infinite loop."#));
    assert_css(&big, "span.big", 1);
}

// A custom role name is emitted verbatim as the span's CSS class, ready for a
// matching CSS class selector in the stylesheet.
#[test]
fn custom_style_syntax_with_custom_role() {
    verifies!(
        r#"
Although xref:text-span-built-in-roles.adoc#built-in[built-in roles] such as `big` and `small` are supported by most AsciiDoc processors, it's really better to define your own semantic role names and map styles to them accordingly.

Here's how you can assign a custom role to text so you can apply your own styles to it.

.Text with custom role
[#ex-custom]
----
include::example$text.adoc[tag=css-custom]
----

When <<ex-custom>> is converted to HTML, the word _asciidoctor_ is enclosed in a `<span>` element and the role `userinput` is used as the element's CSS class.

.HTML output
[,html]
----
include::example$text.adoc[tag=css-custom-html]
----

"#
    );

    let out = convert("Type the word [.userinput]#asciidoctor# into the search bar.\n");
    assert!(out.contains(r#"<span class="userinput">asciidoctor</span>"#));
    assert_css(&out, "span.userinput", 1);
}

non_normative!(
    r#"
The following example shows how you can assign styles to elements that have this role using a CSS class selector.

[,css]
----
.userinput {
  font-family: monospace;
  font-size: 1.1em;
  line-height: calc(1 / 1.1);
}
----
"#
);
