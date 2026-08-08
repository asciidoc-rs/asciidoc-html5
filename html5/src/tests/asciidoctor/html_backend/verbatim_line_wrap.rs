use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/verbatim-line-wrap.adoc");

// Asciidoctor's "Verbatim Block Line Wrapping" page. It documents the default
// stylesheet's line-wrapping CSS for listing/literal blocks, and the two ways
// to opt a block (or a whole document) out of it: the `nowrap` block option,
// and unsetting the document's `prewrap` attribute.
//
// Both mechanisms are implemented in full (`Renderer::is_nowrap`/`nowrap_class`
// in `renderer.rs`): a block declaring `nowrap`, or a document with `prewrap`
// unset, adds the `nowrap` class to the verbatim `<pre>`. The opening
// paragraphs describing the *default* stylesheet's CSS rules (which selectors
// apply which `white-space`/`word-wrap` values) are checked once, by the
// default-stylesheet page's own coverage
// (`html5/src/tests/asciidoctor/html_backend/default_stylesheet.rs`), so they
// stay non-normative here to avoid duplicating that assertion.
non_normative!(
    r#"
= Verbatim Block Line Wrapping

The default Asciidoctor stylesheet wraps long lines in verbatim blocks by applying the CSS `white-space: pre-wrap` and `word-wrap: break-word`.
The lines are wrapped at word boundaries, similar to how most text editors wrap lines.
This prevents horizontal scrolling which some users considered a greater readability problem than line wrapping.

However, this behavior is configurable because there are times when you don't want the lines in listing and literal blocks to wrap.

== Prevent line wrapping

There are two ways to prevent lines from wrapping so that horizontal scrolling is used instead:

* `nowrap` block option
* unset the `prewrap` document attribute (on by default)

You can use the `nowrap` option on literal or listing blocks to prevent lines from being wrapped in the HTML.

"#
);

// The `nowrap` block option adds the `nowrap` class to the block's `<pre>`
// element.
#[test]
fn the_nowrap_option_adds_the_nowrap_class() {
    verifies!(
        r#"
.Listing block with nowrap option syntax
[source,asciidoc]
....
include::example$wrap.adoc[tag=nowrap]
....

When the nowrap option is used, the `nowrap` class is added to the `<pre>` element.
This class changes the CSS to `white-space: pre` and `word-wrap: normal`.

"#
    );

    // The example's listing block, `[%nowrap,java]` over a `----` block: no
    // declared style plus a language in the second position promotes it to a
    // source-highlighted `<pre>`, so the `nowrap` class lands alongside
    // `highlight`.
    let html = crate::convert(
        "[%nowrap,java]\n----\npublic class ApplicationConfigurationProvider {\n}\n----\n",
    );
    assert!(html.contains("<pre class=\"highlight nowrap\">"), "{html}");

    non_normative!(
        r#"
.Result: Listing block with nowrap option applied
include::example$wrap.adoc[tag=nowrap]

"#
    );
}

// Unsetting the document's `prewrap` attribute adds the `nowrap` class to
// every verbatim `<pre>` in the document, without needing the block-level
// option.
#[test]
fn unsetting_prewrap_adds_nowrap_to_every_pre() {
    verifies!(
        r#"
To prevent lines from wrapping globally, unset the `prewrap` attribute on the document.

.Disable prewrap globally (thus, enabling nowrap)
[,asciidoc]
----
:prewrap!:
----

When the prewrap attribute is unset, the `nowrap` class is added to any `<pre>` elements.

"#
    );

    let html = crate::convert(":prewrap!:\n\n----\nplain text\n----\n");
    assert!(html.contains("<pre class=\"nowrap\">"), "{html}");
}

non_normative!(
    r#"
Now, you can use the line wrapping strategy that works best for you and your readers.
"#
);
