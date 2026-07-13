use asciidoc_parser::blocks::{Block, BlockSelector, Descend, FindBlocks, IsBlock};

use crate::{load, tests::sdd::*};

track_file!("docs/modules/api/pages/find-blocks.adoc");

// This crate's "Find Blocks" API page, adapted from Asciidoctor's page of the
// same name. The prose is tracked as non-normative; each Rust snippet the page
// shows is verified by an ordinary test in this module that runs the same call
// against the sample document the page loads.

// The document the page loads in its first snippet: an intro paragraph (wrapped
// in a preamble because sections follow), a "Get Started" section holding a
// source listing (id `install`, role `primary`), a paragraph, and a sidebar,
// then an empty "API" section.
const SAMPLE: &str = "\
= Reference Guide

Read this first.

== Get Started

[#install.primary]
[source,console]
----
$ npm install
----

Follow the steps above.

****
Need help? Ask on the forum.
****

== API
";

non_normative!(
    r#"
= Find Blocks in a Loaded Document Using the API
:navtitle: Find Blocks
:description: How to search a loaded document's block tree with the FindBlocks API, the asciidoc-parser counterpart to Asciidoctor's find_by.

Once you have loaded a document, you can traverse its block tree to find block
nodes. `asciidoc-html5` returns an
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/document/struct.Document.html[`asciidoc_parser::Document`]
from `load`, and
https://crates.io/crates/asciidoc-parser[`asciidoc-parser`] provides the search
API for that document: the
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/blocks/trait.FindBlocks.html[`FindBlocks`]
trait, its Rust-native counterpart to Asciidoctor's `find_by`. There are two ways
to look for blocks. One is to walk the tree yourself starting from the document.
A quicker way is to let `FindBlocks` do the walking for you, so we start there.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

"#
);

// Bringing `FindBlocks` into scope makes its search methods callable on the
// loaded `Document`.
#[test]
fn find_blocks_trait_is_brought_into_scope() {
    verifies!(
        r#"
`FindBlocks` is a trait implemented for both `Document` and `Block`, so bring it
into scope to call its methods on either one:

[,rust]
----
use asciidoc_parser::blocks::FindBlocks;
----

"#
    );

    let doc = load(SAMPLE);
    assert!(doc.descendant_blocks().next().is_some());
}

non_normative!(
    r#"
[IMPORTANT]
====
The search finds block nodes only. It does not find inline nodes.
====

"#
);

non_normative!(
    r#"
== Find every descendant block

`descendant_blocks` is the counterpart to Asciidoctor's `find_by` with no
arguments: it returns an iterator over every descendant block, depth-first, in
document order. Call it on the document to reach every block in the document, or
on a single block to search only that block's subtree.

Suppose you load a document with a couple of sections, the first of which holds a
source block and a sidebar:

"#
);

// Loading the sample document with `load`.
#[test]
fn load_parses_the_sample_document() {
    verifies!(
        r#"
[,rust]
----
let doc = asciidoc_html5::load(
    "= Reference Guide\n\
     \n\
     Read this first.\n\
     \n\
     == Get Started\n\
     \n\
     [#install.primary]\n\
     [source,console]\n\
     ----\n\
     $ npm install\n\
     ----\n\
     \n\
     Follow the steps above.\n\
     \n\
     ****\n\
     Need help? Ask on the forum.\n\
     ****\n\
     \n\
     == API\n",
);
----

"#
    );

    let doc = load(SAMPLE);
    assert!(doc.descendant_blocks().next().is_some());
}

non_normative!(
    r#"
Because `descendant_blocks` is an ordinary `Iterator` that yields `&Block`, the
idiomatic way to inspect the result is to compose it with the standard
combinators. Collecting each block's
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/blocks/trait.IsBlock.html#tymethod.resolved_context[resolved context]
shows the full walk, in document order:

"#
);

// `descendant_blocks` walks the whole tree in document order; collecting each
// block's resolved context reproduces the exact list the page shows.
#[test]
fn descendant_blocks_yields_the_full_walk() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{FindBlocks, IsBlock};

let contexts: Vec<_> = doc
    .descendant_blocks()
    .map(|block| block.resolved_context().to_string())
    .collect();

assert_eq!(
    contexts,
    ["preamble", "paragraph", "section", "listing", "paragraph", "sidebar", "paragraph", "section"],
);
----

"#
    );

    let doc = load(SAMPLE);
    let contexts: Vec<_> = doc
        .descendant_blocks()
        .map(|block| block.resolved_context().to_string())
        .collect();
    assert_eq!(
        contexts,
        [
            "preamble",
            "paragraph",
            "section",
            "listing",
            "paragraph",
            "sidebar",
            "paragraph",
            "section",
        ]
    );
}

non_normative!(
    r#"
The content before the first section becomes a `preamble` wrapping its paragraph;
the first `section` holds the `listing`, a `paragraph`, and a `sidebar` (which
in turn holds its own paragraph); and the second `section` closes the document.

"#
);

// Chaining a block ahead of its descendants to include the receiver — the
// pattern the page shows for recovering Asciidoctor's "starts with the
// receiver" behavior.
#[test]
fn chaining_includes_the_starting_block() {
    verifies!(
        r#"
[IMPORTANT]
====
Unlike Asciidoctor's `find_by`, these iterators never yield the block you start
from -- they visit its descendants only. A `Document` is not a `Block`, so
"descendants only" is the natural reading on either receiver. To include a
starting block in the results, chain it yourself:

[,rust]
----
use asciidoc_parser::blocks::FindBlocks;

let with_self = std::iter::once(block).chain(block.descendant_blocks());
----
====

"#
    );

    let doc = load(SAMPLE);
    let block = doc.descendant_blocks().next().unwrap();
    let with_self: Vec<_> = std::iter::once(block)
        .chain(block.descendant_blocks())
        .collect();

    // The chained iterator leads with the block itself.
    assert_eq!(with_self.first().copied(), Some(block));
}

non_normative!(
    r#"
Since it is a plain iterator, you reach for `filter`, `find`, or `count` rather
than slicing an array. For example, count the top-level (level 1) sections:

"#
);

// Composing the iterator with `filter`/`count` to find top-level sections.
#[test]
fn counting_top_level_sections() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{Block, FindBlocks};

let top_level_sections = doc
    .descendant_blocks()
    .filter(|block| matches!(block, Block::Section(s) if s.level() == 1))
    .count();

assert_eq!(top_level_sections, 2);
----

"#
    );

    let doc = load(SAMPLE);
    let top_level_sections = doc
        .descendant_blocks()
        .filter(|block| matches!(block, Block::Section(s) if s.level() == 1))
        .count();
    assert_eq!(top_level_sections, 2);
}

non_normative!(
    r#"
To search only part of the document, call `descendant_blocks` on the relevant
block instead of the document. Searching the first section's subtree reaches its
listing, paragraph, and sidebar (and the paragraph inside the sidebar), but not
the second section:

"#
);

// Searching a single block's subtree by calling `descendant_blocks` on it.
#[test]
fn searching_a_single_section_subtree() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{Block, FindBlocks, IsBlock};

let first_section = doc
    .descendant_blocks()
    .find(|block| matches!(block, Block::Section(_)))
    .unwrap();

let contexts: Vec<_> = first_section
    .descendant_blocks()
    .map(|block| block.resolved_context().to_string())
    .collect();

assert_eq!(contexts, ["listing", "paragraph", "sidebar", "paragraph"]);
----

"#
    );

    let doc = load(SAMPLE);
    let first_section = doc
        .descendant_blocks()
        .find(|block| matches!(block, Block::Section(_)))
        .unwrap();
    let contexts: Vec<_> = first_section
        .descendant_blocks()
        .map(|block| block.resolved_context().to_string())
        .collect();
    assert_eq!(contexts, ["listing", "paragraph", "sidebar", "paragraph"]);
}

non_normative!(
    r#"
== Filter with a selector

When you are looking for specific blocks, pass a
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/blocks/struct.BlockSelector.html[`BlockSelector`]
to `find_blocks`. The selector is built from up to four fields, each matching a
block property; a field left unset matches any block, and several set fields are
combined with logical AND:

`context`:: a single block context (block name), such as `listing` or `section`.
`style`:: a single block style, such as `source`.
`id`:: an id.
`role`:: a single role.

The traversal still descends through blocks that do not match, so matches at any
depth are found. Match all source listing blocks by combining `context` and
`style`:

"#
);

// Combining the `context` and `style` selector fields to narrow listings to
// source listings.
#[test]
fn selector_combines_context_and_style() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{BlockSelector, FindBlocks};

let source_listings = doc
    .find_blocks(&BlockSelector::new().context("listing").style("source"))
    .count();

assert_eq!(source_listings, 1);
----

"#
    );

    let doc = load(SAMPLE);
    let source_listings = doc
        .find_blocks(&BlockSelector::new().context("listing").style("source"))
        .count();
    assert_eq!(source_listings, 1);
}

non_normative!(
    r#"
Because literal blocks can also be source blocks, drop the `context` field to
find every source block regardless of its context:

"#
);

// Matching every source block with the `style` field alone.
#[test]
fn selector_by_style_alone() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{BlockSelector, FindBlocks};

let all_source_blocks = doc.find_blocks(&BlockSelector::new().style("source")).count();

assert_eq!(all_source_blocks, 1);
----

"#
    );

    let doc = load(SAMPLE);
    let all_source_blocks = doc
        .find_blocks(&BlockSelector::new().style("source"))
        .count();
    assert_eq!(all_source_blocks, 1);
}

non_normative!(
    r#"
Find every block carrying a given role with the `role` field:

"#
);

// Matching every block that carries a role with the `role` field.
#[test]
fn selector_by_role() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{BlockSelector, FindBlocks};

let blocks_with_role = doc.find_blocks(&BlockSelector::new().role("primary")).count();

assert_eq!(blocks_with_role, 1);
----

"#
    );

    let doc = load(SAMPLE);
    let blocks_with_role = doc
        .find_blocks(&BlockSelector::new().role("primary"))
        .count();
    assert_eq!(blocks_with_role, 1);
}

non_normative!(
    r#"
Because an id is unique within a document, an `id` selector matches at most one
block. For that common case there is a convenience method, `find_block_by_id`,
which returns an `Option<&Block>` -- the equivalent of Asciidoctor's
`find_by(id: '...').first`:

"#
);

// The `find_block_by_id` convenience, returning the one block with a given id.
#[test]
fn find_block_by_id_returns_the_single_match() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{FindBlocks, IsBlock};

let install = doc.find_block_by_id("install").unwrap();

assert_eq!(install.resolved_context().as_ref(), "listing");
----

"#
    );

    let doc = load(SAMPLE);
    let install = doc.find_block_by_id("install").unwrap();
    assert_eq!(install.resolved_context().as_ref(), "listing");
}

non_normative!(
    r#"
By default, and for backwards compatibility, the search does not descend into
AsciiDoc table cells, which are separate nested documents. To include the blocks
inside those cells, set `traverse_documents` on the selector -- the counterpart
to Asciidoctor's `traverse_documents` selector key:

"#
);

// Opting into table-cell traversal with `traverse_documents`. The sample has no
// table cells, so the call yields the same eight blocks as the default walk;
// the test confirms the call is available and returns that full walk.
#[test]
fn selector_can_traverse_documents() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{BlockSelector, FindBlocks};

let all_blocks = doc.find_blocks(&BlockSelector::new().traverse_documents(true));
----

"#
    );

    let doc = load(SAMPLE);
    let all_blocks = doc.find_blocks(&BlockSelector::new().traverse_documents(true));
    assert_eq!(all_blocks.count(), 8);
}

non_normative!(
    r#"
== Control the traversal

When a selector cannot describe what you are after -- or when you want to steer
the walk itself -- use `traverse_blocks`. It calls a closure once per block, in
document order, and the closure returns a
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/blocks/enum.Descend.html[`Descend`]
value that decides both whether the block is included and whether its children
are visited. The four variants map one-to-one onto Asciidoctor's `find_by` block
filter keywords:

`Descend::Accept`:: include the block and descend into its children (Asciidoctor
`:accept` / `true`).
`Descend::Skip`:: omit the block but still descend into its children (Asciidoctor
`:skip` / `false`).
`Descend::Reject`:: omit the block and skip its children (Asciidoctor `:reject`).
`Descend::Prune`:: include the block but skip its children (Asciidoctor
`:prune`).

Here is an efficient way to match every sidebar that is not nested inside another
sidebar. Pruning at each sidebar includes it but stops the walk there, so a
sidebar nested inside another is never reported:

"#
);

// Controlling the walk with `traverse_blocks` and `Descend::Prune` to match
// only top-level sidebars.
#[test]
fn traverse_blocks_prunes_to_top_level_sidebars() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::{Descend, FindBlocks, IsBlock};

let top_level_sidebars = doc
    .traverse_blocks(|block| {
        if block.resolved_context().as_ref() == "sidebar" {
            Descend::Prune
        } else {
            Descend::Accept
        }
    })
    .filter(|block| block.resolved_context().as_ref() == "sidebar")
    .count();

assert_eq!(top_level_sidebars, 1);
----

"#
    );

    let doc = load(SAMPLE);
    let top_level_sidebars = doc
        .traverse_blocks(|block| {
            if block.resolved_context().as_ref() == "sidebar" {
                Descend::Prune
            } else {
                Descend::Accept
            }
        })
        .filter(|block| block.resolved_context().as_ref() == "sidebar")
        .count();
    assert_eq!(top_level_sidebars, 1);
}

non_normative!(
    r#"
Unlike Asciidoctor, the document is never a candidate here (the closure only ever
sees descendant blocks), so there is no document object to special-case: return
`Descend::Accept` for the branches you want to descend through and reserve
`Descend::Prune` or `Descend::Reject` for the subtrees you want to stop at.

== Walk the tree yourself

Another way to find blocks is to traverse the tree explicitly. Starting from the
document, `nested_blocks` (from the
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/blocks/trait.IsBlock.html[`IsBlock`]
trait) gives you an iterator over a block's direct children, which you can then
recurse into:

"#
);

// Walking the tree by hand with `nested_blocks`, which yields a block's direct
// children.
#[test]
fn nested_blocks_yields_direct_children() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_parser::blocks::IsBlock;

for block in doc.nested_blocks() {
    // inspect each top-level block, and recurse with `block.nested_blocks()`
}
----

"#
    );

    let doc = load(SAMPLE);
    let direct: Vec<_> = doc
        .nested_blocks()
        .map(|block| block.resolved_context().to_string())
        .collect();
    assert_eq!(direct, ["preamble", "section", "section"]);
}

non_normative!(
    r#"
[CAUTION]
====
Not all blocks share the same model. Each item in a description list carries two
nodes, and tables have a very different model from other blocks. These
differences matter when you walk the tree by hand, and `descendant_blocks` also
reaches children of a Markdown-style blockquote that `nested_blocks` cannot.
====

If the block you are after is close at hand or in a known location, a custom
traversal can be the most direct route. But when you are not sure where a block
sits in the tree, `find_blocks` and its companions are the easier way to find it.

[NOTE]
.Differences from Asciidoctor
====
The search API is built on Rust's iterator patterns rather than returning an
array, so the results are computed lazily and composed with the standard iterator
combinators. Two behavioral differences follow from that design, both noted
above: the iterators never yield the receiver (only its descendants), and
`FindBlocks` is a sealed trait implemented just for `Document` and `Block`. As in
Asciidoctor, `traverse_documents` is off by default.
====

That covers finding blocks in a loaded document using the API.
"#
);
