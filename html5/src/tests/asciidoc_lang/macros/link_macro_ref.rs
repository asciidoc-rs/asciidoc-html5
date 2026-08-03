//! Coverage of the AsciiDoc language description's *Link, URL, and Mailto Macro
//! Attributes Reference* page.
//!
//! This reference table lists the attributes shared by the link, URL, and
//! mailto macros. Each row's example syntax is verified through `convert`: `id`
//! and `role` add `id`/`class`, `title` adds a `title`, `window` (and its `^`
//! shorthand) sets `target`/`rel`, and `opts` adds link options such as
//! `nofollow`. The rendered output matches Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/link-macro-ref.adoc");

non_normative!(
    r#"
= Link, URL, and Mailto Macro Attributes Reference

These attributes apply to the link, URL, and mailto (email) macros.

[%autowidth]
|===
|Attribute |Value(s) |Example Syntax |Comments

"#
);

// `id` sets the link element's `id`.
#[test]
fn id() {
    verifies!(
        r#"
|`id`
|Unique identifier for element in output
|+https://asciidoctor.org[Home,id=home]+
|

"#
    );

    assert!(convert("https://asciidoctor.org[Home,id=home]")
        .contains(r#"<a href="https://asciidoctor.org" id="home">Home</a>"#));
}

// `role` adds a CSS class to the link.
#[test]
fn role() {
    verifies!(
        r#"
|`role`
|CSS classes available to inline elements
|+https://chat.asciidoc.org[Discuss AsciiDoc,role=teal]+
|

"#
    );

    assert!(
        convert("https://chat.asciidoc.org[Discuss AsciiDoc,role=teal]")
            .contains(r#"<a href="https://chat.asciidoc.org" class="teal">Discuss AsciiDoc</a>"#)
    );
}

// `title` adds a `title` attribute (a tooltip).
#[test]
fn title() {
    verifies!(
        r#"
|`title`
|Description of link, often show as tooltip.
|+https://asciidoctor.org[Home,title=Project home page]+
|

"#
    );

    assert!(
        convert("https://asciidoctor.org[Home,title=Project home page]")
            .contains(r#"<a href="https://asciidoctor.org" title="Project home page">Home</a>"#)
    );
}

// `window` sets the link's `target` (and adds `rel="noopener"`).
#[test]
fn window() {
    verifies!(
        r#"
|`window`
|any
|+https://chat.asciidoc.org[Discuss AsciiDoc,window=_blank]+
|The blank window target can also be specified using `^` at the end of the link text.

"#
    );

    assert!(convert("https://chat.asciidoc.org[Discuss AsciiDoc,window=_blank]").contains(
        r#"<a href="https://chat.asciidoc.org" target="_blank" rel="noopener">Discuss AsciiDoc</a>"#
    ));
}

// The `^` shorthand at the end of the link text is equivalent to
// `window=_blank`.
#[test]
fn window_shorthand() {
    verifies!(
        r#"
|`window` +
(shorthand)
|`^`
|+https://example.org[Google, DuckDuckGo, Ecosia^]+ +
+https://chat.asciidoc.org[Discuss AsciiDoc^]+
|

"#
    );

    // The `^` at the end lets the whole comma-containing text remain the link
    // text while still opening a new window.
    assert!(convert("https://example.org[Google, DuckDuckGo, Ecosia^]").contains(
        r#"<a href="https://example.org" target="_blank" rel="noopener">Google, DuckDuckGo, Ecosia</a>"#
    ));
    assert!(convert("https://chat.asciidoc.org[Discuss AsciiDoc^]").contains(
        r#"<a href="https://chat.asciidoc.org" target="_blank" rel="noopener">Discuss AsciiDoc</a>"#
    ));
}

// `opts` adds link options such as `nofollow` (emitted as a `rel` value).
#[test]
fn opts() {
    verifies!(
        r#"
|`opts`
|Additional options for link creation.
|+https://asciidoctor.org[Home,opts=nofollow]+
|Option names include: `nofollow`, `noopener`
"#
    );

    assert!(convert("https://asciidoctor.org[Home,opts=nofollow]")
        .contains(r#"<a href="https://asciidoctor.org" rel="nofollow">Home</a>"#));
}

non_normative!(
    r#"
|===
"#
);
