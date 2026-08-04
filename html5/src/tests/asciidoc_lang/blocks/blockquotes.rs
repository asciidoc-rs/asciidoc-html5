//! Coverage of the AsciiDoc language description's *Blockquotes* page.
//!
//! The page documents every blockquote form: the `[quote]` paragraph, the
//! `____` delimited quote block, the "quoted paragraph" (double-quoted text
//! with a `--` attribution), the `excerpt` role, and Markdown-style blockquotes
//! (including nested block content). This crate renders them all, so each
//! section's example is verified through `convert`. The title, the
//! introduction, the section headings, and the closing caveats are
//! non-normative.
//!
//! Each `verifies!` block drives the exact source of the `quote.adoc` example
//! tag the page includes in that section (from
//! `ref/asciidoc-lang/docs/modules/blocks/examples/quote.adoc`).

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/blockquotes.adoc");

non_normative!(
    r#"
= Blockquotes

Prose excerpts, quotes and verses share the same syntax structure, including:

* block name, either `quote` or `verse`
* name of who the content is attributed to
* bibliographical information of the book, speech, play, poem, etc., where the content was drawn from
* excerpt text

== Basic quote syntax

// tag::basic[]
"#
);

// The basic quote syntax: `[quote,attribution,citation]` over a paragraph
// renders as a `quoteblock` whose attribution div carries the author and a
// `<cite>` citation.
#[test]
fn basic_quote_paragraph_carries_attribution_and_citation() {
    verifies!(
        r#"
For content that doesn't require the preservation of line breaks, set the `quote` attribute in the first position of the attribute list.
Next, set the attribution and relevant citation information.
These positional attributes are all optional.

.Anatomy of a basic quote
[#ex-basic]
----
[quote,attribution,citation title and information]
Quote or excerpt text
----

You can include an optional space after the comma that separates each positional attribute.
If an attribute value includes a comma, enclose the value in double or single quotes.

"#
    );

    let output =
        convert("[quote,attribution,citation title and information]\nQuote or excerpt text\n");
    assert_css(&output, "div.quoteblock > blockquote", 1);
    assert_xpath(
        &output,
        r#"//div[@class="attribution"]/cite[text()="citation title and information"]"#,
        1,
    );
}

// A titled quote paragraph: a block title becomes the quote's title (rendered
// above the blockquote), with the attribution and citation following.
#[test]
fn quote_paragraph_with_title_renders_title_and_attribution() {
    verifies!(
        r#"
If the quote is a single line or paragraph (i.e., a styled paragraph), you can place the attribute list directly on top of the text.

.Quote paragraph syntax
[#ex-style]
----
include::example$quote.adoc[tag=para2-c]
----
<.> Mark lead-in text explaining the context or setting of the quote using a period (`.`). (optional)
<.> For content that doesn't require the preservation of line breaks, set `quote` in the first position of the attribute list.
<.> The second position contains who the excerpt is attributed to. (optional)
<.> Enter additional citation information in the third position. (optional)
<.> Enter the excerpt or quote text on the line immediately following the attribute list.

The result of <<ex-style>> is displayed below.

include::example$quote.adoc[tag=para2]

"#
    );

    let output = convert(
        ".After landing the cloaked Klingon bird of prey in Golden Gate park:\n\
         [quote,Captain James T. Kirk,Star Trek IV: The Voyage Home]\n\
         Everybody remember where we parked.\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="quoteblock"]/div[@class="title"][text()="After landing the cloaked Klingon bird of prey in Golden Gate park:"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//div[@class="attribution"]/cite[text()="Star Trek IV: The Voyage Home"]"#,
        1,
    );
}

non_normative!(
    r#"
== Quoted block

"#
);

// A delimited quote block: a `____` block holds multiple paragraphs, all
// wrapped in a single `<blockquote>`, with the attribution below. Here only the
// author is given, so there is no `<cite>`.
#[test]
fn delimited_quote_block_holds_multiple_paragraphs() {
    verifies!(
        r#"
If the quote or excerpt is more than one paragraph, place the text between delimiter lines consisting of four underscores (`+____+`).

.Quote block syntax
[#ex-block]
----
include::example$quote.adoc[tag=comp]
----

The result of <<ex-block>> is displayed below.

include::example$quote.adoc[tag=comp]
// end::basic[]

"#
    );

    let output = convert(
        "[quote,Monty Python and the Holy Grail]\n____\n\
         Dennis: Come and see the violence inherent in the system. Help! Help! I'm being repressed!\n\n\
         King Arthur: Bloody peasant!\n____\n",
    );
    assert_css(&output, "div.quoteblock > blockquote > div.paragraph", 2);
    assert_xpath(
        &output,
        r#"//div[@class="attribution"][contains(text(), "Monty Python and the Holy Grail")]"#,
        1,
    );
    assert_css(&output, "div.quoteblock cite", 0);
}

non_normative!(
    r#"
== Quoted paragraph

"#
);

// The quoted-paragraph shorthand: a paragraph wrapped in double quotes with a
// `--` attribution line renders as a `quoteblock`, with the quotes stripped and
// the attribution split into author and citation.
#[test]
fn quoted_paragraph_shorthand_renders_a_quoteblock() {
    verifies!(
        r#"
You can turn a single paragraph into a blockquote by:

. surrounding it with double quotes
. adding an optional attribution (prefixed with two dashes) below the quoted text

.Quoted paragraph syntax
[#ex-quoted]
----
include::example$quote.adoc[tag=abbr]
----

The result of <<ex-quoted>> is displayed below.

include::example$quote.adoc[tag=abbr]

"#
    );

    let output = convert(
        "\"I hold it that a little rebellion now and then is a good thing,\n\
         and as necessary in the political world as storms in the physical.\"\n\
         -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11\n",
    );
    assert_css(&output, "div.quoteblock > blockquote", 1);
    assert_xpath(
        &output,
        r#"//div[@class="attribution"]/cite[text()="Papers of Thomas Jefferson: Volume 11"]"#,
        1,
    );
}

non_normative!(
    r#"
== Excerpt

"#
);

// The `excerpt` role: adding `[.excerpt]` to a quote block puts an `excerpt`
// class alongside `quoteblock` on the wrapper; the visual decoration is left to
// the stylesheet.
#[test]
fn excerpt_role_adds_a_class_to_the_quoteblock() {
    verifies!(
        r#"
The quote block can be designated as an excerpt by adding the `excerpt` role.
The exceptation is that this role makes the quote block appear with the quote decoration.

[source]
----
[.excerpt]
____
This text is an excerpt from the referenced literature.
____
----

The impact of this role is strictly a presentation concern and is thus handled by the styling system, such as the stylesheet for HTML.

"#
    );

    let output = convert(
        "[.excerpt]\n____\nThis text is an excerpt from the referenced literature.\n____\n",
    );
    assert_css(&output, "div.quoteblock.excerpt > blockquote", 1);
}

non_normative!(
    r#"
== Markdown-style blockquotes

"#
);

// Markdown-style blockquotes: lines prefixed with `>` render as a `quoteblock`,
// with a `--` line becoming the attribution. Nested `>` and block content
// (paragraphs, lists, nested blockquotes) are supported, producing nested
// `quoteblock`s and a `ulist`.
#[test]
fn markdown_style_blockquotes_render_quoteblocks() {
    verifies!(
        r#"
Asciidoctor supports Markdown-style blockquotes.
This syntax was adopted both to ease the transition from Markdown and because it's the most common method of quoting in email messages.

.Markdown-style blockquote syntax
[source#ex-md]
----
include::example$quote.adoc[tag=md]
----

The result of <<ex-md>> is displayed below.

include::example$quote.adoc[tag=md]

Like Markdown, Asciidoctor supports some block content inside the blockquote, including paragraphs, lists, and nested blockquotes.

.Markdown-style blockquote containing block content
[source#ex-md-block]
....
include::example$quote.adoc[tag=md-alt]
....

Here's how the conversation from <<ex-md-block>> is rendered.

include::example$quote.adoc[tag=md-alt]

"#
    );

    let simple = convert(
        "> I hold it that a little rebellion now and then is a good thing,\n\
         > and as necessary in the political world as storms in the physical.\n\
         > -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11\n",
    );
    assert_css(&simple, "div.quoteblock > blockquote", 1);
    assert_xpath(
        &simple,
        r#"//div[@class="attribution"]/cite[text()="Papers of Thomas Jefferson: Volume 11"]"#,
        1,
    );

    let nested = convert(
        "> > What's new?\n>\n> I've got Markdown in my AsciiDoc!\n>\n> > Like what?\n>\n\
         > * Blockquotes\n> * Headings\n>\n> > Is there more?\n>\n> Yep.\n",
    );
    // The outer blockquote encloses nested quoteblocks and a list.
    assert_css(&nested, "div.quoteblock div.quoteblock", 3);
    assert_css(&nested, "div.quoteblock div.ulist", 1);
}

// Caveats: a description list is not permitted inside a Markdown-style
// blockquote, and the syntax should be reserved for simple/migration cases.
non_normative!(
    r#"
Be aware that not all AsciiDoc block elements are supported inside a Markdown-style blockquote.
In particular, a description list is not permitted.
The parser looks for the Markdown-style blockquote only after looking for a description list, meaning the description list takes precedence.
Since the quote marker is a valid prefix for a description list term, the Markdown-style blockquote is not recognized in this case.
If you need to put a description list inside a blockquote, you should use the AsciiDoc syntax for a blockquote instead.

The Markdown-style blockquote should only be used in simple cases and when migrating from Markdown.
The AsciiDoc syntax should always be preferred, if possible.
"#
);
