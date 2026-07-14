use asciidoc_parser::blocks::{Block, BlockSelector, Descend, FindBlocks, IsBlock};

use crate::{load, tests::sdd::*};

track_file!("ref/asciidoctor/docs/modules/api/pages/find-blocks.adoc");

// Asciidoctor's "Find Blocks" page, tracked from the library crate. It
// documents the Ruby `find_by` block-search API on `AbstractBlock`. This
// project delivers the same capability through `asciidoc-parser`'s `FindBlocks`
// trait, which `load` returns a `Document` ready to search: `descendant_blocks`
// is the no-argument `find_by`, `find_blocks(&BlockSelector)` is
// `find_by(selector)`, `find_block_by_id` is `find_by(id: '…').first`, and
// `traverse_blocks` with the `Descend` enum is the `find_by` block filter
// (`:accept` / `:skip` / `:reject` / `:prune`). The concrete, behavior-bearing
// examples on the page are verified against those methods.
//
// The API leans on Rust's iterator patterns rather than Ruby's arrays, and that
// design difference makes a few spans of the page non-normative here:
//
// * The receiver is never yielded. Asciidoctor's `find_by` includes the block
//   you start from as the first result; these iterators visit descendants only
//   (a `Document` is not a `Block`, so "descendants only" reads the same on
//   either receiver). The "always returns the block you start with" note, and
//   the `slice 1..-1` / `.first` array manipulations built on it, therefore
//   have no analog here — you compose the iterator with std combinators
//   instead.
// * The document is never a candidate in a `traverse_blocks` walk, so the
//   caveat about returning `:skip` for the document object does not apply.
// * The keyword definition list and the selector-key list are descriptive; the
//   behavior they describe is verified by the examples that follow them.

// The sample document searched throughout these tests: an intro paragraph
// (wrapped in a preamble because sections follow), a "Prerequisites" section
// holding a source listing (id `setup`, role `try-it`), a note paragraph, a
// plain listing, and a sidebar, then an empty "Reference" section.
const SAMPLE: &str = "\
= Document Title

Intro paragraph.

== Prerequisites

[#setup.try-it]
[source,ruby]
----
require 'asciidoctor'
----

A note about setup.

----
a plain listing
----

****
A sidebar.
****

== Reference
";

// A single-row table whose one cell is an AsciiDoc (`a|`) cell, i.e. a nested
// document. The paragraph inside it is reached only when the traversal is told
// to descend into table cells.
const TABLE: &str = "|===\na| Cell _text_.\n|===\n";

// A sidebar nested inside a sidebar, followed by a top-level paragraph. Used to
// show pruning: there are two sidebars in all, but only one is top-level.
const NESTED_SIDEBARS: &str = "\
****
Outer.

[.inner]
*****
Inner.
*****
****

After.
";

// Collects the resolved context of every descendant block, in document order.
fn contexts<'a, T: FindBlocks<'a>>(node: &'a T) -> Vec<String> {
    node.descendant_blocks()
        .map(|block| block.resolved_context().as_ref().to_string())
        .collect()
}

non_normative!(
    r#"
= Find Blocks

Once the document has been loaded (or partially loaded), you can traverse the document to find block nodes.
There are two ways to look for block nodes.
One way is to start walking down the tree starting from the Document object.
All blocks can be reached from the Document object.
However, a much quicker way to find blocks is to use the `find_by` method, which does the walking for you.
We'll start there, then look at how to use the custom traversal approach.

== find_by

Every block node (a parsed block), including the Document object, provides the {url-api-gems}/asciidoctor/{release-version}/Asciidoctor/AbstractBlock#find_by-instance_method[find_by] method.
The purpose of this method is to help you quickly find descendant blocks.
Since some blocks have different models, this method can help you navigate the document without having to worry about those nuances.

IMPORTANT: The `find_by` method only finds block nodes.
It does not find inline nodes.

If you want to look for any block in the parsed document, call the `find_by` method on the Document object.
Otherwise, you can look for blocks in a specific area of the document by calling it on the relevant ancestor of those blocks.

"#
);

// No-argument `find_by`: return every descendant block in document order (and
// an empty result when there are none). The analog is `descendant_blocks`, an
// iterator over the descendants in depth-first document order. The one
// difference — Asciidoctor also includes the receiver itself, which these
// iterators never do — is covered by the non-normative note that follows.
#[test]
fn descendant_blocks_walks_the_whole_tree_in_document_order() {
    verifies!(
        r#"
The return value of this method is a flat array of blocks in document order which were matched.
The relationship between those blocks is only preserved by way of their own model.
If no blocks are matched, the method returns an empty array.

=== All blocks

If not called with any arguments, the `find_by` method will return all blocks starting from the block on which it was called.
If called on the Document object, it will return all blocks in the document (except for blocks in AsciiDoc table cells), including the document itself.
Here's an example:

[,ruby]
----
require 'asciidoctor'

doc = Asciidoctor.load_file 'input.adoc', safe: :safe
puts doc.find_by
----

Here's an example of how to find all the blocks in the first section:

[,ruby]
----
doc.sections.first.find_by
----

"#
    );

    // An empty document has no descendants (the "empty array" case).
    assert_eq!(load("").descendant_blocks().count(), 0);

    let doc = load(SAMPLE);

    // Every block, in document order.
    assert_eq!(
        contexts(&doc),
        [
            "preamble",
            "paragraph",
            "section",
            "listing",
            "paragraph",
            "listing",
            "sidebar",
            "paragraph",
            "section",
        ]
    );

    // Searching only the first section reaches just that section's subtree.
    let first_section = doc
        .descendant_blocks()
        .find(|block| matches!(block, Block::Section(_)))
        .unwrap();
    assert_eq!(
        contexts(first_section),
        ["listing", "paragraph", "listing", "sidebar", "paragraph"]
    );
}

// The receiver-inclusion behavior and the Ruby array manipulations that build
// on it. Asciidoctor includes the starting block as the first result, then
// slices it off (`slice 1..-1`) or plucks the first match (`.first`). These
// iterators visit descendants only, so there is no leading receiver to slice
// off; the idiomatic equivalents are `descendant_blocks()` as-is and `.next()`
// / `.find(..)`. Non-normative.
non_normative!(
    r#"
Notice that the `find_by` method always returns the block that you start with as the first result (assuming it also matches the provided selector, covered later).
If you want to exclude that block, slice it off from the results:

[,ruby]
----
puts doc.find_by.slice 1..-1
----

If youre just looking for the first result, you can pluck it from the result array:

[,ruby]
----
puts doc.find_by.first
----

"#
);

// Descending into AsciiDoc table cells, which are separate nested documents.
// Off by default (as in Asciidoctor); opt in with
// `BlockSelector::traverse_documents`, the analog of the `traverse_documents`
// selector key.
#[test]
fn traverse_documents_reaches_into_table_cells() {
    verifies!(
        r#"
By default, and for backwards compatibility, the `find_by` method does not traverse into AsciiDoc table cells.
If you want it to look in these cells for blocks, set the `:traverse_documents` key on the selector Hash to true.

[,ruby]
----
all_blocks = doc.find_by traverse_documents: true
----

"#
    );

    let doc = load(TABLE);

    // By default the walk stops at the table itself.
    assert_eq!(doc.find_blocks(&BlockSelector::new()).count(), 1);

    // Opting in reaches the paragraph inside the AsciiDoc cell.
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().traverse_documents(true))
            .count(),
        2
    );
}

non_normative!(
    r#"
The next section will look at how to filter the blocks that are returned.

=== Filter blocks

"#
);

// The selector description and its four keys. The behavior each key selects on
// is verified by the examples that follow (`id`, `context` + `style`, `style`,
// `role`), so the list itself is descriptive. `BlockSelector` exposes exactly
// these four fields plus `traverse_documents`.
non_normative!(
    r#"
When using the `find_by` method, you're probably looking for specific blocks.
The method accepts an optional selector (a Hash) and an optional block filter (a Ruby proc).
The method will walk the entire tree (including in AsciiDoc table cells if `:traverse_documents` is `true`) to find blocks.
By default, it will descend into a block which does not match, though this behavior can be controlled using the block filter.

The simplest way to match blocks is to use the selector.
The selector is a Hash that accepts four predefined symbol keys:

:context:: A single block xref:convert:contexts-ref.adoc[context] (i.e., block name), such as `:paragraph`.
:style:: A single block style, such as `source`.
:id:: An ID.
:role:: A single role.

"#
);

// The `id` selector matches at most one block, since ids are unique. The direct
// analog is `find_block_by_id`, which returns an `Option<&Block>` (the
// equivalent of `find_by(id: '…').first`); the same match is reachable through
// `find_blocks` with an `id` selector.
#[test]
fn find_by_id_matches_at_most_one_block() {
    verifies!(
        r#"
If an `:id` is specified, the method will never return more than one block since an ID is, by natural, globally unique.
Here's an example of how to find a block by ID using the `:id` selector:

[,ruby]
----
match = (doc.find_by id: 'prerequisites').first
----

"#
    );

    let doc = load(SAMPLE);

    // The source listing carries `[#setup]`.
    let matched = doc.find_block_by_id("setup").unwrap();
    assert_eq!(matched.resolved_context().as_ref(), "listing");

    // The `id` selector finds the same single block.
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().id("setup")).count(),
        1
    );
}

// Combining the `context` and `style` selector fields to narrow listings to
// source listings. The sample has two listings, only one of which is a source
// block, so the combined selector matches just that one.
#[test]
fn combining_context_and_style_narrows_the_match() {
    verifies!(
        r#"
Now let's assume we want to match all listing blocks that are source blocks.
We can do so by combining the `:context` and `:style` selectors:

[,ruby]
----
some_source_blocks = doc.find_by context: :listing, style: 'source'
----

"#
    );

    let doc = load(SAMPLE);

    // Two listings in all...
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().context("listing"))
            .count(),
        2
    );

    // ...but only one of them is a source listing.
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().context("listing").style("source"))
            .count(),
        1
    );
}

// Dropping the `context` field to match every source block regardless of
// context, using the `style` field alone.
#[test]
fn matching_by_style_alone_finds_all_source_blocks() {
    verifies!(
        r#"
Since literal blocks can also be source blocks, if we want all source blocks, we'd need to leave off the `:context` selector:

[,ruby]
----
all_source_blocks = doc.find_by style: 'source'
----

"#
    );

    let doc = load(SAMPLE);

    assert_eq!(
        doc.find_blocks(&BlockSelector::new().style("source"))
            .count(),
        1
    );
}

// Matching every block that carries a given role, using the `role` field.
#[test]
fn matching_by_role() {
    verifies!(
        r#"
If we want all blocks marked with a specific role, we can find them using the `:role` selector:

[,ruby]
----
blocks_with_role = doc.find_by role: 'try-it'
----

"#
    );

    let doc = load(SAMPLE);

    // Only the source listing carries `[.try-it]`.
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().role("try-it"))
            .count(),
        1
    );
}

// The block filter proc: a predicate run on each visited block. Descriptive
// prose; the concrete filter examples that follow are verified.
non_normative!(
    r#"
The selector Hash is intentionally simple to make it easy to find blocks.
If the blocks you're looking for cannot be described using that selector, then you'll want to use a block filter instead.

A block filter is a Ruby proc that runs on each block visited.
It accepts the candidate block as the sole argument (i.e., the candidate block is yielded to the proc).
If the proc returns true, then the candidate is considered matched.

"#
);

// Using a predicate to find all top-level (level 1) sections — the case a
// selector cannot express. Because `descendant_blocks` is an ordinary iterator,
// the predicate is a `filter` closure; the "combine with a selector" refinement
// is `find_blocks(context: section)` followed by the same `filter`.
#[test]
fn a_predicate_finds_all_top_level_sections() {
    verifies!(
        r#"
Here's an example of using the block filter to find all top-level sections:

[,ruby]
----
top_level_sections = doc.find_by {|block| block.context == :section && block.level == 1 }
----

We can make this slightly more efficient by combining it with a selector:

[,ruby]
----
top_level_sections = doc.find_by(context: :section) {|section| section.level == 1 }
----

"#
    );

    let doc = load(SAMPLE);

    // The filter form: every section at level 1.
    let top_level_sections = doc
        .descendant_blocks()
        .filter(|block| matches!(block, Block::Section(s) if s.level() == 1))
        .count();
    assert_eq!(top_level_sections, 2);

    // The refined form: pre-select sections with a selector, then apply the same
    // level filter.
    let combined = doc
        .find_blocks(&BlockSelector::new().context("section"))
        .filter(|block| matches!(block, Block::Section(s) if s.level() == 1))
        .count();
    assert_eq!(combined, 2);
}

// The "supplemental filter" semantics — a block given alongside a selector must
// match both. Here that is just iterator composition
// (`find_blocks(..).filter(..)`), which the previous test already exercises;
// the sentence itself is descriptive.
non_normative!(
    r#"
If a Ruby block is given, it's applied as a supplemental filter to the selector.
In other words, the candidate block must match the selector and the filter.

"#
);

// The keyword definition list for the block filter's return values. Each maps
// one-to-one onto a `Descend` variant (`:accept` → `Descend::Accept`, `:skip` →
// `Descend::Skip`, `:reject` → `Descend::Reject`, `:prune` → `Descend::Prune`);
// the behavior is verified by the pruning example that follows, so the
// definitions are non-normative.
non_normative!(
    r#"
=== Control the traversal

The benefit of the block filter is that it also allows you to control the traversal.
The filter method can return any of the following keywords:

true::
:accept::
The block is accepted and the traversal continues.

false::
:skip::
The block is skipped but its children are traversed.

:reject::
The block is rejected and its children are not traversed.

:prune::
The block is accepted, but its descendants are not traversed.

"#
);

// Controlling the traversal to match only top-level sidebars. `traverse_blocks`
// runs the closure per block; returning `Descend::Prune` on a sidebar includes
// it but stops the descent, so a sidebar nested inside another is never
// reported. The document is never a candidate here, so unlike Asciidoctor there
// is no document object to special-case with `:skip`.
#[test]
fn pruning_matches_only_top_level_sidebars() {
    verifies!(
        r#"
Here's an efficient way to match all sidebars that are not contained within another block.

[,ruby]
----
top_level_sidebars = doc.find_by do |block|
  if block == block.document
    :skip
  elsif block.context == :sidebar
    :prune
  else
    :reject
  end
end
----

"#
    );

    let doc = load(NESTED_SIDEBARS);

    // Two sidebars in all (one nested inside the other)...
    assert_eq!(
        doc.find_blocks(&BlockSelector::new().context("sidebar"))
            .count(),
        2
    );

    // ...but pruning at each sidebar reports only the top-level one.
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

// The Ruby-specific caveats about the block filter: returning `:skip` for the
// document object (the document is never a candidate here), and the reduced
// control when a selector and filter are combined. Neither applies to this
// crate's iterator-based API, so both are non-normative.
non_normative!(
    r#"
The filter has to return `:skip` instead of `:reject` for the document object or else no blocks will be traversed.

If you combine the selector and the block filter, you will have less control over which nodes are traversed.
Therefore, if you're going to be using the block filter to control the traversal, it's best to do all logic in that filter.

"#
);

// Walking the tree by hand instead of searching it. Asciidoctor reaches a
// block's direct children with `blocks`; the analog here is `nested_blocks`
// (from `IsBlock`), an iterator over the direct child blocks that you can
// recurse into yourself.
#[test]
fn custom_traversal_reaches_direct_children() {
    verifies!(
        r#"
== Custom traversal

Another way to find blocks is to traverse the tree explicitly.
Starting at the document object, you can access its children by calling the `blocks` method.

[,ruby]
----
doc.blocks.each do |block|
  puts block
end
----

"#
    );

    let doc = load(SAMPLE);

    // The document's direct children: the preamble and the two top-level
    // sections.
    let direct: Vec<_> = doc
        .nested_blocks()
        .map(|block| block.resolved_context().as_ref().to_string())
        .collect();
    assert_eq!(direct, ["preamble", "section", "section"]);
}

// The caution about differing block models, and the closing guidance on when to
// prefer a custom traversal over `find_by`. Descriptive prose.
non_normative!(
    r#"
CAUTION: Not all blocks have the same model.
For example, each item in a description list is an array of two nodes.
And tables have a very different model from other blocks.
These differences are important to be aware of when traversing the document model.

If the block or blocks you're looking for are close at hand or in a known location, it may be more efficient to use a custom traversal.
However, if you aren't sure where the block is located in the document tree, you'd be much better off using the `find_by` method to locate it.
"#
);
