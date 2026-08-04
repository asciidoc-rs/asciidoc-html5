//! Coverage of the AsciiDoc language description's *Link & URL Macro Attribute
//! Parsing* page.
//!
//! When an equals sign appears after the first comma in a link/URL macro's
//! brackets, the content is parsed as an attribute list; otherwise it is the
//! link text. This page covers quoting link text that contains commas/equals,
//! the `window` attribute (and the `^` blank-window shorthand), and the
//! `noopener`/`nofollow` options. Every form is verified through `convert`,
//! matching Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/link-macro-attribute-parsing.adoc");

non_normative!(
    r#"
= Link & URL Macro Attribute Parsing

If named attributes are detected between the square brackets of a link or URL macro, that text is parsed as an attribute list.
This page explains the conditions when this occurs and how to write the link text so it is recognized as a single positional attribute.

"#
);

// The whole bracket content is the link text when it has no attribute-list
// trigger (no `=` after a comma).
#[test]
fn link_text_basics() {
    verifies!(
        r#"
== Link text alongside named attributes

Normally, the whole text between the square brackets of a link macro is treated as the link text (i.e., the first positional attribute).

[source]
----
https://chat.asciidoc.org[Discuss AsciiDoc]
----

However, if the text contains an equals sign (`=`), the text is parsed as an xref:attributes:element-attributes.adoc#attribute-list[attribute list].
The exact rules for attribute list parsing and positional attributes are rather complex, and discussed on xref:attributes:positional-and-named-attributes.adoc[].
To be sure the link text is recognized properly, you can apply these two simple checks:

* contains no comma (`,`) or equals sign (`=`) or
* enclosed in double quotes (`"`)

There are several other situations in which text before the first comma may be recognized as the link text.
Let's consider some examples.
"#
    );

    assert!(convert("https://chat.asciidoc.org[Discuss AsciiDoc]")
        .contains(r#"<a href="https://chat.asciidoc.org">Discuss AsciiDoc</a>"#));
}

// Custom link text alongside named attributes; text containing a comma must be
// double-quoted to be captured as a single positional attribute.
#[test]
fn text_with_comma_and_attributes() {
    verifies!(
        r#"

The following example shows a URL macro with custom link text alongside named attributes.

[source]
----
https://chat.asciidoc.org[Discuss AsciiDoc,role=resource,window=_blank]
----

Let's consider a case where the link text contains a comma and the macro also has named attributes.
In this case, you must enclose the link text in double quotes so that it is capture in its entirety as the first positional attribute.

[source]
----
https://example.org["Google, DuckDuckGo, Ecosia",role=teal]
----

"#
    );

    // Custom text plus a role and window.
    assert!(convert("https://chat.asciidoc.org[Discuss AsciiDoc,role=resource,window=_blank]").contains(
        r#"<a href="https://chat.asciidoc.org" class="resource" target="_blank" rel="noopener">Discuss AsciiDoc</a>"#
    ));

    // Quoted text containing commas stays a single positional attribute.
    assert!(
        convert(r#"https://example.org["Google, DuckDuckGo, Ecosia",role=teal]"#).contains(
            r#"<a href="https://example.org" class="teal">Google, DuckDuckGo, Ecosia</a>"#
        )
    );
}

// Text containing an equals sign is likewise double-quoted; a literal quote in
// the text is backslash-escaped.
#[test]
fn text_with_equals_sign() {
    verifies!(
        r##"
Similarly, if the link text contains an equals sign, you can enclose the link text in double quotes to ensure the parser recognizes it as the first positional attribute.

[source]
----
https://example.org["1=2 posits the problem of inequality"]
----

If the quoted link text itself contains the quote character used to enclose the text, escape the quote character in the text by prefixing it with a backslash.

[source]
----
https://example.org["href=\"#top\" attribute"] creates link to top of page
----

"##
    );

    // Quoted text containing `=` stays the link text.
    assert!(
        convert(r#"https://example.org["1=2 posits the problem of inequality"]"#)
            .contains(r#"<a href="https://example.org">1=2 posits the problem of inequality</a>"#)
    );

    // An escaped quote inside the quoted text is unescaped in the output.
    assert!(convert(
        r##"https://example.org["href=\"#top\" attribute"] creates link to top of page"##
    )
    .contains(r##"<a href="https://example.org">href="#top" attribute</a>"##));
}

non_normative!(
    r#"
The double quote enclosure is not required in all cases when the link text contains an equals sign.
Strictly speaking, the enclosure is only required when the text preceding the equals sign matches a valid attribute name.
However, it's best to use the double quotes just to be safe.

"#
);

// With an empty first positional attribute, the target is used as the link
// text.
#[test]
fn named_attributes_without_link_text() {
    verifies!(
        r#"
Finally, to use named attributes without specifying link text, you simply specify the named attributes.
(In other words, you leave the first positional attribute empty, in which case the target will be used as the link text).

[source]
----
https://chat.asciidoc.org[role=button,window=_blank,opts=nofollow]
----

The link macro recognizes all the common attributes (id, role, and opts).
It also recognizes a handful of attributes that are specific to the link macro.

"#
    );

    // No link text: the target is the text (with the `bare` role), plus the
    // supplied role, window, and options.
    assert!(convert("https://chat.asciidoc.org[role=button,window=_blank,opts=nofollow]").contains(
        r#"<a href="https://chat.asciidoc.org" class="bare button" target="_blank" rel="nofollow noopener">https://chat.asciidoc.org</a>"#
    ));
}

// The `window` attribute sets the link's `target`.
#[test]
fn target_a_separate_window() {
    verifies!(
        r#"
== Target a separate window

By default, the link produced by a link macro will target the current window.
In other words, clicking on it will replace the current page.

You can configure the link to open in a separate window (or tab) using the `window` attribute.

[source]
----
https://asciidoctor.org[Asciidoctor,window=read-later]
----

In the HTML output, the value of the `window` attribute is assigned to the `target` attribute on the `<a>` tag (e.g., `target=read-later`).

"#
    );

    assert!(
        convert("https://asciidoctor.org[Asciidoctor,window=read-later]")
            .contains(r#"<a href="https://asciidoctor.org" target="read-later">Asciidoctor</a>"#)
    );
}

// A `_blank` window sets `target="_blank"` and implicitly adds
// `rel="noopener"`.
#[test]
fn target_a_blank_window() {
    verifies!(
        r#"
=== Target a blank window

Most of the time, you'll use the `window` attribute to target a blank window.
Configuring a link that points to a location outside the current site is common practice to avoid disrupting the reader's flow.
To enable this behavior, you set the `window` attribute to the special value `_blank`.

[source]
----
https://asciidoctor.org[Asciidoctor,window=_blank]
----

In the HTML output, the value of the `window` attribute is assigned to the `target` attribute on the `<a>` tag (e.g., `target=_blank`).
If the target is `_blank`, the processor will automatically add the <<noopener and nofollow,`rel=noopener` attribute>> as well.

CAUTION: The underscore at the start of the value `_blank` can unexpectedly form a constrained formatting pair when another underscore appears somewhere else in the line or paragraph, thus causing the macro to break.
You can avoid this problem either by escaping the underscore at the start of the value (i.e., `+window=\_blank+`) or by using the <<Blank window shorthand>> instead.

"#
    );

    assert!(
        convert("https://asciidoctor.org[Asciidoctor,window=_blank]").contains(
            r#"<a href="https://asciidoctor.org" target="_blank" rel="noopener">Asciidoctor</a>"#
        )
    );
}

// The `noopener` and `nofollow` options add `rel` flags; `_blank` implies
// `noopener`, and `nofollow` combines with it (`rel="nofollow noopener"`).
#[test]
fn noopener_and_nofollow() {
    verifies!(
        r#"
=== noopener and nofollow

The `noopener` option is used to control access to the window opened by a link.
*This option is only available if the `window` attribute is set.*
This option adds the `noopener` flag to the `rel` attribute on the `<a>` element in the HTML output (e.g., `rel="noopener"`).

When the value of the `window` attribute is `_blank`, the AsciiDoc processor implicitly sets the `noopener` option.
Doing so is considered a security best practice.

[source]
----
https://asciidoctor.org[Asciidoctor,window=_blank]
----

If the window is not `_blank`, you need to enable the `noopener` flag explicitly by setting the `noopener` option on the macro:

[source]
----
https://asciidoctor.org[Asciidoctor,window=read-later,opts=noopener]
----

If you don't want the search indexer to follow the link, you can add the `nofollow` option to the macro.
This option adds the `nofollow` flag to the `rel` attribute on the `<a>` element in the HTML output, alongside `noopener` if present (e.g., `rel="nofollow noopener"`).

[source]
----
https://asciidoctor.org[Asciidoctor,window=_blank,opts=nofollow]
----

or

[source]
----
https://asciidoctor.org[Asciidoctor,window=read-later,opts="noopener,nofollow"]
----

To fine tune indexing within the site, you can specify the `nofollow` option even if the link does not target a separate window.

[source]
----
link:post.html[My Post,opts=nofollow]
----

"#
    );

    // A non-blank window with an explicit `noopener` option.
    assert!(convert("https://asciidoctor.org[Asciidoctor,window=read-later,opts=noopener]")
        .contains(r#"<a href="https://asciidoctor.org" target="read-later" rel="noopener">Asciidoctor</a>"#));

    // `nofollow` combines with the implicit `noopener` of a `_blank` window.
    assert!(convert("https://asciidoctor.org[Asciidoctor,window=_blank,opts=nofollow]")
        .contains(r#"<a href="https://asciidoctor.org" target="_blank" rel="nofollow noopener">Asciidoctor</a>"#));

    // `nofollow` applies even without a separate window.
    assert!(convert("link:post.html[My Post,opts=nofollow]")
        .contains(r#"<a href="post.html" rel="nofollow">My Post</a>"#));
}

// The `^` shorthand at the end of the link text is equivalent to
// `window=_blank`.
#[test]
fn blank_window_shorthand() {
    verifies!(
        r#"
=== Blank window shorthand

Configuring an external link to target a blank window is a common practice.
Therefore, AsciiDoc provides a shorthand for it.

In place of the named attribute `+window=_blank+`, you can insert a caret (`+^+`) at the end of the link text.
This syntax has the added benefit of not having to worry about the underscore at the start of the value `+_blank+` unexpectedly forming a constrained formatting pair when another underscore appears in the same line or paragraph.

[source]
----
include::example$url.adoc[tag=linkattrs-s]
----

CAUTION: In rare circumstances, if you use the caret syntax more than once in the same line or paragraph, you may need to escape the first occurrence with a backslash.
However, the processor should try to avoid making this a requirement.

If the attribute list has both link text in double quotes and named attributes, the caret should be placed at the end of the link text, but inside the double quotes.

[source]
----
https://example.org["Google, DuckDuckGo, Ecosia^",role=btn]
----

If no named attributes are present, the link text should not be enclosed in quotes.

[source]
----
https://example.org[Google, DuckDuckGo, Ecosia^]
----
"#
    );

    // The `tag=linkattrs-s` snippet: a `^` caret opens a new window.
    assert!(convert("Let's view the raw HTML of the link:view-source:asciidoctor.org[Asciidoctor homepage^].").contains(
        r#"<a href="view-source:asciidoctor.org" target="_blank" rel="noopener">Asciidoctor homepage</a>"#
    ));

    // Inside double quotes, the caret sits at the end of the text; a role also
    // applies.
    assert!(convert(r#"https://example.org["Google, DuckDuckGo, Ecosia^",role=btn]"#).contains(
        r#"<a href="https://example.org" class="btn" target="_blank" rel="noopener">Google, DuckDuckGo, Ecosia</a>"#
    ));

    // Without named attributes the text is not quoted, and the caret still opens
    // a new window.
    assert!(convert("https://example.org[Google, DuckDuckGo, Ecosia^]").contains(
        r#"<a href="https://example.org" target="_blank" rel="noopener">Google, DuckDuckGo, Ecosia</a>"#
    ));
}
