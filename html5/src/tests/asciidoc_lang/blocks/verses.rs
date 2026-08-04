//! Coverage of the AsciiDoc language description's *Verses* page.
//!
//! The page documents the `[verse]` style on a paragraph and on a `____`
//! delimited excerpt block. This crate renders both as a `verseblock` whose
//! `<pre>` preserves the line breaks, with an attribution line built from the
//! author and citation positional attributes, so each example is verified
//! through `convert`. The title, the introduction, and the section headings are
//! non-normative.
//!
//! Each `verifies!` block drives the exact source of the `verse.adoc` example
//! tag the page includes in that section (from
//! `ref/asciidoc-lang/docs/modules/blocks/examples/verse.adoc`).

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/verses.adoc");

non_normative!(
    r#"
= Verses

When you need to preserve indents and line breaks, use a `verse` block.
Verses are defined by setting `verse` on a paragraph or an excerpt block delimited by four underscores (`+____+`).

== verse style syntax

"#
);

// The verse paragraph style: `[verse,author,citation]` on a paragraph renders
// as a `verseblock` whose `<pre class="content">` preserves the line breaks,
// plus an attribution div carrying the author and a `<cite>` citation.
#[test]
fn verse_paragraph_style_renders_a_verseblock() {
    verifies!(
        r#"
When verse content doesn't contain any empty lines, you can assign the `verse` style using the first position in an attribute list.

.Verse style syntax
[#ex-style]
----
include::example$verse.adoc[tag=para]
----

The result of <<ex-style>> is displayed below.

include::example$verse.adoc[tag=para]

"#
    );

    let output = convert(
        "[verse,Carl Sandburg, two lines from the poem Fog]\n\
         The fog comes\non little cat feet.\n",
    );
    assert_css(&output, "div.verseblock > pre.content", 1);
    assert_css(&output, "div.verseblock > div.attribution", 1);
    assert_xpath(
        &output,
        r#"//div[@class="attribution"]/cite[text()="two lines from the poem Fog"]"#,
        1,
    );
}

non_normative!(
    r#"
== Delimited verse block syntax

"#
);

// The delimited verse block: `[verse]` over a `____` excerpt block preserves
// the empty lines within the verse in its `<pre>`, rendering the same
// `verseblock` shape with an attribution.
#[test]
fn delimited_verse_block_preserves_empty_lines() {
    verifies!(
        r#"
When the verse content includes empty lines, enclose it in a delimited excerpt block.

.Verse delimited block syntax
[#ex-block]
----
include::example$verse.adoc[tag=bl]
----

The delimited verse block from <<ex-block>> is rendered below.

include::example$verse.adoc[tag=bl]
"#
    );

    let output = convert(
        "[verse,Carl Sandburg,Fog]\n____\nThe fog comes\non little cat feet.\n\n\
         It sits looking\nover harbor and city\non silent haunches\nand then moves on.\n____\n",
    );
    assert_css(&output, "div.verseblock > pre.content", 1);
    assert_xpath(
        &output,
        r#"//div[@class="verseblock"]/pre[contains(text(), "and then moves on.")]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//div[@class="attribution"]/cite[text()="Fog"]"#,
        1,
    );
}
