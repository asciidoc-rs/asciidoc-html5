//! Coverage of the AsciiDoc language description's *Open Blocks* page.
//!
//! The page documents the `--` open block and its ability to masquerade as
//! another block via a declared style. This crate renders the anonymous open
//! block and both masquerades the page demonstrates (sidebar and source), so
//! each section's example is verified through `convert`. Only the page title,
//! the conceptual introduction, and the section heading are non-normative.
//!
//! Each `verifies!` block drives the exact source of the `open.adoc` example
//! tag the page includes in that section (from
//! `ref/asciidoc-lang/docs/modules/blocks/examples/open.adoc`).

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/open-blocks.adoc");

non_normative!(
    r#"
= Open Blocks

The most versatile block of all is the open block.
It allows you to apply block attributes to a chunk of text without giving it any other semantics.
In other words, it provides a generic structural container for enclosing content.
The only notable limitation is that an open block cannot be nested inside of another open block.

== Open block syntax

"#
);

// The anonymous open block: a `--` delimited container renders as `<div
// class="openblock"><div class="content">…</div></div>`, wrapping its content
// without adding any other semantics.
#[test]
fn anonymous_open_block_renders_an_openblock_wrapper() {
    verifies!(
        r#"
.Open block syntax
[#ex-open]
----
include::example$open.adoc[tag=base]
----

The result of <<ex-open>> is displayed below.

include::example$open.adoc[tag=base]

"#
    );

    let output = convert(
        "--\nAn open block can be an anonymous container,\n\
         or it can masquerade as any other block.\n--\n",
    );
    assert_css(&output, "div.openblock > div.content", 1);
    assert_css(&output, "div.openblock > div.content > div.paragraph", 1);
}

// An open block masquerading as a sidebar: the `[sidebar]` style on a `--`
// block changes the wrapper to `sidebarblock`, with the title rendered inside
// the content div (as for a delimited sidebar).
#[test]
fn open_block_can_masquerade_as_a_sidebar() {
    verifies!(
        r#"
An open block can act as any other paragraph or delimited block, with the exception of _pass_ and _table_.
For instance, in <<ex-open-sidebar>> an open block is acting as a sidebar.

.Open block masquerading as a sidebar
[#ex-open-sidebar]
----
include::example$open.adoc[tag=sb]
----

The result of <<ex-open-sidebar>> is displayed below.

include::example$open.adoc[tag=sb]

"#
    );

    let output = convert(
        "[sidebar]\n.Related information\n--\nThis is aside text.\n\n\
         It is used to present information related to the main content.\n--\n",
    );
    assert_css(&output, "div.sidebarblock > div.content", 1);
    assert_css(&output, "div.sidebarblock > div.content > div.title", 1);
    assert_css(&output, "div.sidebarblock > div.content > div.paragraph", 2);
    assert_css(&output, "div.openblock", 0);
}

// An open block masquerading as a source block: the `[source]` style on a `--`
// block changes it to a `listingblock` with the highlighted `<pre>`/`<code>`
// shape of a source block.
#[test]
fn open_block_can_masquerade_as_a_source_block() {
    verifies!(
        r#"
<<ex-open-source>> is an open block acting as a source block.

.Open block masquerading as a source block
[#ex-open-source]
----
include::example$open.adoc[tag=src]
----

The result of <<ex-open-source>> is displayed below.

include::example$open.adoc[tag=src]
"#
    );

    let output = convert("[source]\n--\nputs \"I'm a source block!\"\n--\n");
    assert_css(&output, "div.listingblock > div.content", 1);
    assert_css(&output, "div.listingblock pre.highlight > code", 1);
    assert_css(&output, "div.openblock", 0);
}
