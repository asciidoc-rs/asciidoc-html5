//! Coverage of Asciidoctor's *Syntax Highlighting* overview page.
//!
//! The page introduces source blocks and the two families of syntax
//! highlighter — client-side and build-time. This crate implements the
//! *client-side* family, so its concrete claims are verified through `convert`:
//! a source block is defined by a language (the `source` style is then
//! implied), and a client-side highlighter passes the block's contents through
//! unchanged while tagging the element with its language.
//!
//! The *build-time* family (CodeRay, Pygments, Rouge) tokenizes source during
//! conversion by invoking an external library — a settled non-goal here (the
//! library depends only on `asciidoc-parser`, and no in-process highlighter can
//! match the parity oracle byte for byte), so the paragraphs describing it, the
//! support matrix, the DocBook note, the "explore the integrations" cross
//! references, and the custom-adapter guidance are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoctor/docs/modules/syntax-highlighting/pages/index.adoc");

// The page title.
non_normative!(
    r#"
= Syntax Highlighting

"#
);

// A source block is a listing block carrying a source language. The page's
// opening example is a source display: an AsciiDoc snippet shown verbatim
// inside a listing. Converting it yields a listing block whose `<pre>` shows
// the literal source-block markup.
#[test]
fn a_source_block_is_a_listing_with_a_language() {
    verifies!(
        r#"
AsciiDoc defines a style of listing block known as a xref:asciidoc:verbatim:source-blocks.adoc[source block] for adding source code snippets intended to be colorized by a syntax highlighter to a document.
Here's a simple example of a source block:

[source,asciidoc]
....
[source,ruby]
----
puts "Hello, World!"
----
....

"#
    );

    let html = convert(
        "[source,asciidoc]\n....\n[source,ruby]\n----\nputs \"Hello, World!\"\n----\n....\n",
    );
    assert_css(
        &html,
        "div.listingblock pre.highlight > code.language-asciidoc",
        1,
    );
    assert!(html.contains("[source,ruby]"));
    assert!(html.contains("puts \"Hello, World!\""));
}

// When a language is set on a listing block, the `source` style is implied, so
// `[source]` may be dropped from the block attributes. Both the `[,ruby]`
// display and the rendered result below confirm it: converting the source
// block yields a `pre.highlight > code.language-ruby`, the source shape, with
// no explicit `source` style given.
#[test]
fn a_language_implies_the_source_style() {
    verifies!(
        r#"
If a language is specified on a listing block, the source style is implied.
Therefore, it can be excluded in this case.

[source,asciidoc]
....
[,ruby]
----
puts "Hello, World!"
----
....

Here's how we expect it to appear:

[,ruby]
----
puts "Hello, World!"
----

"#
    );

    let html = convert("[,ruby]\n----\nputs \"Hello, World!\"\n----\n");
    assert_css(
        &html,
        "div.listingblock pre.highlight > code.language-ruby",
        1,
    );
    assert!(html.contains("data-lang=\"ruby\""));
    assert!(html.contains("puts \"Hello, World!\""));
}

// Descriptive: an AsciiDoc processor applies the highlighting, via built-in or
// custom adapters. The concrete client-side behavior is verified below.
non_normative!(
    r#"
It's up to an AsciiDoc processor such as Asciidoctor to apply the syntax highlighting to this source block, a process referred to as xref:asciidoc:verbatim:source-highlighter.adoc[source highlighting].
Asciidoctor provides adapters that integrate with several popular syntax highlighter libraries to perform this task.
It also provides an interface for implementing a custom syntax highlighter adapter.

"#
);

// Conceptual framing of the two highlighter families; each is described in the
// paragraphs that follow.
non_normative!(
    r#"
== Syntax highlighter types

Asciidoctor supports two types of syntax highlighters: client-side and build-time.
Let's explore each type and how they work.

"#
);

// The client-side family — the one this crate implements. Its defining claims
// are observable: the converter passes the source block's contents through
// unchanged ("as is") and tags the element with metadata (the language) so the
// browser highlighter can act on it. It does not hide callout numbers from the
// highlighter, so a conum is left inline in the highlighted `<code>` (rather
// than stripped out the way a build-time highlighter would).
#[test]
fn a_client_side_highlighter_passes_the_source_through_with_metadata() {
    verifies!(
        r#"
A [.term]*client-side syntax highlighter* performs syntax highlighting in the browser as the page is loading.
Asciidoctor does not invoke the syntax highlighter itself.
Instead, it focuses on adding the assets to the generated HTML so the browser can load the syntax highlighter and run it.
For this type of syntax highlighter, Asciidoctor passes the contents of the source block through to the output as is.
It also adds metadata to the element so that the syntax highlighter knows to highlight it and which language it is.
Unfortunately, Asciidoctor does not process callout numbers in the source block in this case, so they may cause the syntax highlighter to get tripped up.

"#
    );

    // highlight.js is client-side, so the source is passed through as is and
    // the element carries the language metadata. A document-set
    // `:source-highlighter:` is honored below `Server`, so run under `Safe`.
    let html = crate::convert_with(
        "= Doc\n:source-highlighter: highlight.js\n\n[,ruby]\n----\nputs \"Hello\" <1>\n----\n<1> Prints it.\n",
        &crate::Options::new().safe_mode(crate::SafeMode::Safe),
    );

    // Metadata added to the element: the language and the `hljs` marker.
    assert_css(
        &html,
        "pre.highlightjs.highlight > code.language-ruby.hljs",
        1,
    );

    // The source text passes through unchanged, and the callout number is left
    // inline within the highlighted `<code>` (not hidden from the highlighter).
    assert!(html.contains("puts \"Hello\" <b class=\"conum\">(1)</b></code>"));
}

// The build-time family (CodeRay, Pygments, Rouge) invokes an external library
// during conversion to emit tokenized `<span>` markup. This crate does not
// implement build-time highlighting — it is a settled non-goal — so these
// claims are not verifiable here.
non_normative!(
    r#"
A [.term]*build-time syntax highlighter* performs syntax highlighting during AsciiDoc conversion.
Asciidoctor does invoke the syntax highlighter in this case.
It also takes care of hiding the callout numbers from the syntax highlighter, ensuring they are put back in the proper place afterwards.
What Asciidoctor emits into the output is the result produced by the syntax highlighter, which are the tokens enclosed in `<span>` elements to apply color and other formatting (either inline or via CSS classes).
These syntax highlighters tend to support more features because Asciidoctor has greater control over the process.

"#
);

// A comparison of the trade-offs between the two families — advisory prose with
// no rendered behavior. The build-time half is a non-goal here regardless.
non_normative!(
    r#"
=== Client-side vs build-time

There are benefits and drawbacks of each type.
The benefit of a client-side syntax highlighter is that does not require installing any additional libraries.
It also makes conversion faster and makes it produce smaller output since the syntax highlighting is deferred until page load.
The main drawback is that callouts in the source block can be mangled by the syntax highlighter or confuse it.
The benefits of a build-time syntax highlighter are that you have more control over syntax highlighting and can enable additional features such as line numbers and line highlighting.
The main drawback is that it requires installing an extra library, it slows down conversion, and causes the output to be larger.

You should try each syntax highlighter and find the one that works best for you.

"#
);

// The support matrix and the DocBook note. The matrix lists one client-side
// adapter (Highlight.js, implemented here) and three build-time adapters
// (non-goals); DocBook is a backend this crate does not render. Descriptive.
non_normative!(
    r#"
== Built-in syntax highlighter adapters

[%autowidth]
|===
|Type |Syntax Highlighter |Required Gem |Compatible Converters

h|Client-side

|Highlight.js
|_n/a_
|HTML, Reveal.js

.3+h|Build-time

|CodeRay
|coderay (>= 1.1)
|HTML, PDF, EPUB3, Reveal.js

|Pygments
|pygments.rb (>= 1.2)
|HTML, PDF, EPUB3, Reveal.js

|Rouge
|rouge (>= 2)
|HTML, PDF, EPUB3, Reveal.js
|===

Asciidoctor does not apply syntax highlighting when generating DocBook.
The assumption is that this is a task the DocBook toolchain can handle.
The DocBook converter generates a `<programlisting>` element for the source block and passes through the source language as specified in the AsciiDoc.
It's then up to the DocBook toolchain to apply syntax highlighting to the contents of that tag.

"#
);

// Cross references to the per-highlighter pages and the custom-adapter page.
non_normative!(
    r#"
You can explore these integrations in depth on the xref:highlightjs.adoc[], xref:rouge.adoc[], xref:pygments.adoc[], and xref:coderay.adoc[] pages.

You can also create your own integration by making a xref:custom.adoc[].

"#
);

// Advisory guidance on not mixing text-formatting substitutions with syntax
// highlighting. The stated failure — the highlighter being confused by the
// injected markup — happens in the browser (client-side) or another toolchain,
// so it is not observable in this crate's output; the guidance is descriptive.
non_normative!(
    r#"
== Custom subs on source blocks

You should not mix syntax highlighting with AsciiDoc text formatting (i.e., the `quotes` and `macros` substitutions).
In many converters, these two operations are mutually exclusive.

.Improper use of custom substitutions on a source block
[source,asciidoc]
....
[,java,subs=+quotes]
----
interface OrderRepository extends CrudRepository<Order,Long> {

  *List<Order>* findByCategory(String category);

  Order findById(long id);
}
----
....

The additional markup introduced by AsciiDoc text formatting may confuse the syntax highlighter and lead to unexpected results.
While it may work in some syntax highlighters in the HTML backend (which perhaps know how to work around the formatting tags), it will most certainly fail when converting to other formats such as PDF.

If you're going to customize the substitutions on a source block using the `subs` attribute, you should limit those substitutions to attribute replacements (`attributes`).

.Valid use of custom substitutions on a source block
[source,asciidoc]
....
[,java,subs=attributes+]
----
interface {model}Repository extends CrudRepository<{model},Long> {

  {model} findById(long id);
}
----
....

If you still need to emphasize certain tokens in the block of code, you should do so by creating a custom lexer or formatter for the syntax highlighting library that understands these additional semantics and perhaps even hints.
In other words, work through the syntax highlighter so it's added during the highlighting process, giving the syntax highlighter full knowledge as to what's going on.
Another option is to use a xref:custom.adoc[custom syntax highlighter adapter].

Before trying to get the syntax highlighter to recognize new tokens, make sure it doesn't already recognize them.
If it does, it may just be a matter of customizing the syntax highlighter theme to apply different formatting to those tokens.
"#
);
