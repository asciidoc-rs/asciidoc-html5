//! Coverage of the AsciiDoc language description's *Sidebars* page.
//!
//! The page documents both the `[sidebar]` paragraph style and the `****`
//! delimited sidebar (including nested content). This crate renders both forms,
//! so each section's example is verified through `convert`. The title, the
//! upstream-provenance comment, the "on this page" preamble, and the section
//! headings are non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/sidebars.adoc");

non_normative!(
    r#"
= Sidebars
// Moved upstream from the Antora documentation at docs.antora.org

On this page, you'll learn:

* [x] How to mark up a sidebar with AsciiDoc.

A sidebar can contain any type of content, such as quotes, equations, and images.
Normal substitutions are applied to sidebar content.

== Sidebar style syntax

"#
);

// The sidebar paragraph style: `[sidebar]` on a contiguous paragraph renders as
// a `sidebarblock` wrapper, just like the delimited form.
#[test]
fn sidebar_paragraph_style_renders_a_sidebarblock() {
    verifies!(
        r#"
If the sidebar content is contiguous, the block style `sidebar` can be placed directly on top of the text in an attribute list (`[]`).

.Assign sidebar block style to paragraph
[#ex-style]
----
[sidebar]
Sidebars are used to visually separate auxiliary bits of content
that supplement the main text.
----

The result of <<ex-style>> is displayed below.

[sidebar]
Sidebars are used to visually separate auxiliary bits of content that supplement the main text.

"#
    );

    let output = convert(
        "[sidebar]\nSidebars are used to visually separate auxiliary bits of content \
         that supplement the main text.\n",
    );
    assert_css(&output, "div.sidebarblock > div.content", 1);
    assert_xpath(
        &output,
        r#"//div[@class="sidebarblock"]/div[@class="content"][contains(text(), "Sidebars are used")]"#,
        1,
    );
}

non_normative!(
    r#"
== Delimited sidebar syntax

"#
);

// The delimited sidebar: a `****` block needs no style name, may carry an
// optional title (rendered inside the content div), and encloses any content —
// here a paragraph, a TIP admonition, and a nested source block.
#[test]
fn delimited_sidebar_encloses_nested_content() {
    verifies!(
        r#"
A delimited sidebar block is delimited by a pair of four consecutive asterisks (`pass:[****]`).
You don't need to set the style name when you use the sidebar's delimiters.

.Sidebar block delimiter syntax
[#ex-block]
------
.Optional Title
****
Sidebars are used to visually separate auxiliary bits of content
that supplement the main text.

TIP: They can contain any type of content.

.Source code block in a sidebar
[source,js]
----
const { expect, expectCalledWith, heredoc } = require('../test/test-utils')
----
****
------

The result of <<ex-block>> is displayed below.

.Optional Title
****
Sidebars are used to visually separate auxiliary bits of content that supplement the main text.

TIP: They can contain any type of content.

.Source code block in a sidebar
[source,js]
----
const { expect, expectCalledWith, heredoc } = require('../test/test-utils')
----
****
"#
    );

    let output = convert(
        ".Optional Title\n****\n\
         Sidebars are used to visually separate auxiliary bits of content that supplement the main text.\n\n\
         TIP: They can contain any type of content.\n\n\
         .Source code block in a sidebar\n[source,js]\n----\n\
         const { expect, expectCalledWith, heredoc } = require('../test/test-utils')\n----\n****\n",
    );
    assert_css(&output, "div.sidebarblock > div.content", 1);
    assert_xpath(
        &output,
        r#"//div[@class="sidebarblock"]/div[@class="content"]/div[@class="title"][text()="Optional Title"]"#,
        1,
    );
    assert_css(&output, "div.sidebarblock > div.content > div.paragraph", 1);
    assert_css(
        &output,
        "div.sidebarblock > div.content > div.admonitionblock.tip",
        1,
    );
    assert_css(
        &output,
        "div.sidebarblock > div.content > div.listingblock",
        1,
    );
    assert_css(
        &output,
        "div.sidebarblock div.listingblock pre.highlight > code.language-js",
        1,
    );
}
