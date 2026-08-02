//! Coverage of the AsciiDoc language description's *Compound List Items*
//! (continuation) page.
//!
//! A list item's principal text can span contiguous lines (merged into one
//! paragraph), and further blocks are attached with a list continuation — a `+`
//! on its own line — which keeps a list continuous where a bare empty line plus
//! a block would end it. Wrapping several blocks in an open block needs only
//! one continuation, and leading empty lines before the `+` attach the block to
//! an ancestor list item instead of the current one. This crate renders every
//! one of those forms — including dropping the principal text with `{empty}` so
//! an attached block lines up with the marker — so each section's example and
//! claims are verified through `convert`.

use crate::{
    convert, convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/continuation.adoc");

non_normative!(
    r#"
= Compound List Items

This page covers how to create lists that have compound list items.

A [.term]*compound list item* is a list item that has blocks attached to it, including paragraphs, which follow the (optionally empty) principal text.
In other words, the list item contains block content.
This scenario is different from a list item whose principal text merely spans multiple lines, a distinction which is further explained on this page.
The page goes on to explain how to attach a block to a list item in an ancestor list.

In additional to unordered and ordered lists, callout and description lists also support compound list items.
On this page, the term list item refers to any list item in an unordered, ordered, callout, and description list.
For a description list, it refers specifically to the description of the list item (not the list item term).

The main focus of the syntax covered on this page is to keep the list continuous (i.e., to prevent the list from breaking).

== Multiline principal text

"#
);

// A list item's principal text can span any number of contiguous lines — even
// indented ones — which are merged into a single principal paragraph rather
// than attached as separate blocks.
#[test]
fn multiline_principal_text_merges_into_one_paragraph() {
    verifies!(
        r#"
As with regular paragraph text, the principal text in a list item can span any number of lines as long as those lines are contiguous (i.e., adjacent with no empty lines).
Multiple lines are combined into a single paragraph and wrap as regular paragraph text.
This behavior holds even if the lines are indented, as shown in the third bullet in this example:

----
include::example$complex.adoc[tag=indent]
----

====
include::example$complex.adoc[tag=indent]
====

"#
    );

    let output = convert(
        "* The document header in AsciiDoc is optional.\n\
         If present, it must start with a document title.\n\n\
         * Optional author and revision information lines\n\
         immediately follow the document title.\n\n\
         * The document header must be separated from\n  \
         the remainder of the document by one or more\n  \
         empty lines and it cannot contain empty lines.\n",
    );
    assert_css(&output, ".ulist:root > ul > li", 3);
    // The contiguous lines merge into the principal text; nothing is attached
    // as a separate block.
    assert_css(&output, ".ulist:root > ul > li > div.paragraph", 0);
    assert_xpath(
        &output,
        r#"//li/p[contains(text(), "empty lines and it cannot contain empty lines")]"#,
        1,
    );
}

// An empty line between two list items does not break the list.
#[test]
fn an_empty_line_between_items_does_not_break_the_list() {
    verifies!(
        r#"
TIP: When list items contain more than one line of text, leave an empty line between items to make the list easier to read while working in the code.
An empty line between two list items will not break the list.

"#
    );

    let output = convert("* one\n\n* two\n");
    assert_css(&output, "div.ulist", 1);
    assert_css(&output, ".ulist:root > ul > li", 2);
}

non_normative!(
    r#"
=== Empty lines in a list

"#
);

// Empty lines between items keep one continuous list (an ordered list keeps
// numbering), but an empty line followed by a block that is not a list item
// ends the list — a following item then starts a new list.
#[test]
fn a_block_after_an_empty_line_ends_the_list() {
    verifies!(
        r#"
Empty lines between two items in a list (ordered or unordered) will not break the list.
For ordered lists, this means the numbering will be continuous rather than restarting at 1.
(See xref:separating.adoc[] to learn how to force two adjacent lists apart).

If an empty line after a list item is followed by the start of a block, such as a paragraph or delimited block rather than another list item, the list will terminate at this point.
If this happens, you'll notice that a subsequent list item will be placed into a new list.
For ordered lists, that means the numbering will restart (at 1).

To keep the list continuous in those cases--such as when you're documenting complex steps in a procedure--you must use a <<list-continuation,list continuation>> to attach blocks to the list item.
For ordered lists, this will ensure that the numbering continues from one list item to the next rather than being reset.

"#
    );

    // Empty lines between items keep a single, continuous ordered list.
    let continuous = convert(". one\n\n. two\n");
    assert_css(&continuous, "div.olist", 1);
    assert_css(&continuous, ".olist:root > ol > li", 2);

    // A paragraph after an empty line ends the list; the next item starts a
    // fresh list.
    let broken = convert(". one\n\nA paragraph.\n\n. two\n");
    assert_css(&broken, "div.olist", 2);
}

non_normative!(
    r#"
[#list-continuation]
== Attach blocks using a list continuation

"#
);

// A list continuation — a `+` on its own line — attaches a following block to
// the list item as a sibling of its principal paragraph. A `+` at the *end* of
// a line is not a continuation; it is a hard line break.
#[test]
fn a_list_continuation_attaches_a_block() {
    verifies!(
        r#"
In addition to the principal text, a list item may contain block elements, including paragraphs, delimited blocks, and block macros.
To add block elements to a list item, you must "`attach`" them (in a series) using a list continuation.
This technique works for unordered and ordered lists as well as callout and description lists.

A [.term]*list continuation* is a `{plus}` symbol on a line by itself, immediately adjacent to the block being attached.
The attached block must be left-aligned, just like all blocks in AsciiDoc.

NOTE: A `{plus}` at the end of a line, rather than on a line by itself, is not a list continuation.
Instead, it creates a hard line break.

Here's an example of a list item that uses a list continuation:

----
include::example$complex.adoc[tag=cont]
----

====
include::example$complex.adoc[tag=cont]
====

"#
    );

    // A `+` on its own line attaches the following paragraph to the item.
    let attached = convert(
        "* The header in AsciiDoc must start with a document title.\n+\nThe header is optional.\n",
    );
    assert_css(&attached, ".ulist:root > ul > li > p", 1);
    assert_css(&attached, ".ulist:root > ul > li > div.paragraph > p", 1);

    // A `+` at the end of a line is a hard line break, not a continuation.
    let hard_break = convert("text +\nmore\n");
    assert_css(&hard_break, "div.paragraph > p > br", 1);
}

// A chain of continuations attaches several blocks: the first item here carries
// both a listing block and a paragraph.
#[test]
fn a_chain_of_continuations_attaches_several_blocks() {
    verifies!(
        r#"
Using a list continuation, you can attach any number of block elements to a list item.
Unless the block is inside a delimited block which itself has been attached, each block must be preceded by a list continuation to form a chain of blocks.

Here's an example that attaches both a listing block and a paragraph to the first list item:

[source]
....
include::example$complex.adoc[tag=complex]
....

Here's how the source is rendered:

.A list with compound content
====
include::example$complex.adoc[tag=complex]
====

"#
    );

    let output = convert(
        "* The header in AsciiDoc must start with a document title.\n+\n\
         ----\n= Document Title\n----\n+\n\
         Keep in mind that the header is optional.\n\n\
         * Optional author and revision information lines immediately follow the document title.\n+\n\
         ----\n= Document Title\nDoc Writer <doc.writer@asciidoc.org>\nv1.0, 2022-01-01\n----\n",
    );
    assert_css(&output, ".ulist:root > ul > li", 2);
    assert_css(
        &output,
        ".ulist:root > ul > li:first-child > div.listingblock",
        1,
    );
    assert_css(
        &output,
        ".ulist:root > ul > li:first-child > div.paragraph > p",
        1,
    );
}

non_normative!(
    r#"
Notice that we inserted an empty line after the attached paragraph block.
That's because only a sibling list item can interrupt a paragraph.
If the next list item had been a nested list item instead of a sibling, this empty line would have been required.
Otherwise, the nested list marker and text would have just become the next line of the paragraph.
For consistency, a best practice is to always include an empty line at the end of a compound list item.
That way, you never have to remember when it's required.

IMPORTANT: A sibling or nested list item acts as an interrupting line for the principal text of a list item.
Only a sibling list item acts as an interrupting line for an attached block, such as a paragraph.
(The AsciiDoc Language working group has decided that the latter exception will be removed, so it's best not to depend on it.)

"#
);

// Wrapping several blocks in an open block needs only one continuation: the
// open block is attached to the item and its own contents need no further `+`.
#[test]
fn an_open_block_wraps_compound_content() {
    verifies!(
        r#"
If you're attaching more than one block to a list item, you're strongly encouraged to wrap the content inside an open block.
That way, you only need a single list continuation line to attach the open block to the list item.
Within the open block, you write like you normally would, no longer having to worry about adding list continuations between the blocks to keep them attached to the list item.

Here's an example of wrapping compound list content in an open block:

[source]
....
include::example$complex.adoc[tag=complex-o]
....

Here's how that content is rendered:

.A list with compound content wrapped in an open block
====
include::example$complex.adoc[tag=complex-o]
====

"#
    );

    let output = convert(
        "* The header in AsciiDoc must start with a document title.\n+\n\
         --\nHere's an example of a document title:\n\n\
         ----\n= Document Title\n----\n\n\
         NOTE: The header is optional.\n--\n",
    );
    // A single open block is attached to the item and holds the compound
    // content (a listing block and an admonition).
    assert_css(
        &output,
        ".ulist:root > ul > li > div.openblock > div.content",
        1,
    );
    assert_css(&output, "div.openblock div.listingblock", 1);
    assert_css(&output, "div.openblock div.admonitionblock", 1);
}

// An `include::` directive wrapped in an open block attached to a list item
// pulls the shared file's content into the item unmodified. Resolving the
// include needs a base directory, so this drives `convert_with` against a temp
// file under the `Safe` safe mode rather than the plain `convert`.
#[test]
fn an_included_file_can_be_wrapped_in_an_open_block() {
    verifies!(
        r#"
The open block wrapper is also useful if you're including content from a shared file into a list item.
For example:

----
* list item
+
--
\include::shared-content.adoc[]
--
----

By wrapping the include directive in an open block, the content can be used unmodified.

"#
    );

    // A temp directory holding the shared file the include resolves against.
    let dir = std::env::temp_dir().join(format!(
        "asciidoc-html5-lists-continuation-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp include dir");
    std::fs::write(
        dir.join("shared-content.adoc"),
        "Shared content from the included file.\n",
    )
    .expect("write shared include file");

    let output = convert_with(
        "* list item\n+\n--\ninclude::shared-content.adoc[]\n--\n",
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone()),
    );

    let _ = std::fs::remove_dir_all(&dir);

    // The open block is attached to the item and holds the included content
    // (a paragraph) unmodified.
    assert_css(
        &output,
        ".ulist:root > ul > li > div.openblock > div.content > div.paragraph",
        1,
    );
    assert_xpath(
        &output,
        r#"//li/div[@class="openblock"]//p[text()="Shared content from the included file."]"#,
        1,
    );
}

// The nesting limitation (open blocks cannot yet contain another open block) is
// a syntax constraint rather than a rendering behavior to assert here. Tracked
// as non-normative.
non_normative!(
    r#"
The only limitation of this technique is that the content itself may not contain an open block since open blocks cannot (yet) be nested.

"#
);

non_normative!(
    r#"
[#drop-principal-text]
== Drop the principal text

"#
);

// Making the principal text empty with `{empty}` drops its node, so an attached
// block (here a listing block) lines up with the list marker: each item renders
// an empty `<p></p>` followed by its attached block.
#[test]
fn an_empty_principal_text_is_dropped() {
    verifies!(
        r#"
If the principal text of a list item is empty, the node for the principal text is dropped.
This is how you can get the first block (such as a listing block) to line up with the list marker.
You can make the principal text empty by using the `+{empty}+` attribute reference.

Here's an example of a list that has items with _only_ compound content.

[source]
....
include::example$complex.adoc[tag=complex-only]
....

Here's how the source is rendered:

.A list with compound content
====
include::example$complex.adoc[tag=complex-only]
====

"#
    );

    let output = convert(
        ". {empty}\n+\n----\nprint(\"one\")\n----\n. {empty}\n+\n----\nprint(\"one\")\n----\n",
    );
    // Two items, each with an empty principal `<p></p>` (the dropped `{empty}`
    // principal) followed by the attached listing block as its own node, rather
    // than the listing content folded into the principal paragraph.
    assert_css(&output, ".olist:root > ol > li", 2);
    assert_css(&output, ".olist:root > ol > li > p:empty", 2);
    assert_css(&output, ".olist:root > ol > li > div.listingblock", 2);
    assert_css(
        &output,
        ".olist:root > ol > li > div.listingblock > div.content > pre",
        2,
    );
    assert_xpath(&output, r#"//li//pre[contains(text(), "one")]"#, 2);
}

non_normative!(
    r#"
[#attach-to-ancestor-list]
== Attach blocks to an ancestor list

Instead of attaching a block to the current list item, you may need to end that list and attach a block to its ancestor instead.
There are two ways to express this composition in the AsciiDoc syntax.
You can either enclose the child list in an open block, or you can use insert empty lines above the list continuation to first escape from the nesting.
Let's look at enclosing the child list in an open block first, since that is the preferred method.

=== Enclose in open block

"#
);

// The preferred way to attach a block after a nested list is to enclose the
// nested list in an open block: the open block holds the child list, and the
// attached paragraph is a sibling of the open block inside the current item.
#[test]
fn an_open_block_encloses_a_nested_list() {
    verifies!(
        r#"
If you plan to attach blocks to a list item as a sibling of a nested list, the most robust way of creating that structure is to enclose the nested list in an open block.
That way, it's clear where the nested list ends and the current list item continues.

Here's an example of a list item with a nested list followed by an attached paragraph.
The open block makes the boundaries of the nested list clear.

[source]
....
include::example$complex.adoc[tag=complex-enclosed]
....

Here's how the source is rendered:

.A nested list enclosed in an open block
====
include::example$complex.adoc[tag=complex-enclosed]
====

"#
    );

    let output = convert(
        "* grandparent list item\n+\n--\n** parent list item\n*** child list item\n--\n+\n\
         paragraph attached to grandparent list item\n",
    );
    // The nested list is enclosed in the open block; the paragraph is attached
    // as a sibling of that open block within the same item.
    assert_css(
        &output,
        ".ulist:root > ul > li > div.openblock > div.content > div.ulist",
        1,
    );
    assert_css(&output, ".ulist:root > ul > li > div.paragraph > p", 1);
}

non_normative!(
    r#"
The main limitation of this approach is that it can only be used once in the hierarchy (i.e., it can only enclose a single nested list).
That's because the open block itself cannot be nested.
If you require more control, then you must use the ancestor list continuation.

=== Ancestor list continuation

Normally, a list continuation will attach a block to the current list item.
For each empty line you add before the list continuation, the association will move up one level in the nesting.
In other words, an empty line signals to the list continuation to back out of the current list by one level.
As a result, the block will be attached to the current item in an ancestor list.
This syntax is referred to as an [.term]*ancestor list continuation*.

WARNING: The ancestor list continuation is a fragile syntax.
For one, it may not be apparent to new authors that the empty lines before the list continuation are significant.
That's because the AsciiDoc syntax generally ignores repeating empty lines.
There are also scenarios where even these empty lines are collapsed, thus preventing the ancestor list continuation from working as expected.
Use this feature of the syntax with caution.
If possible, enclose the nested list in an open block, as described in the previous section.

"#
);

// One empty line before the continuation attaches the block to the parent list
// item: the paragraph becomes a sibling of the nested list inside the parent
// item, not a child of the deepest item.
#[test]
fn one_empty_line_attaches_to_the_parent_item() {
    verifies!(
        r#"
Here's an example of a paragraph that's attached to the parent list item after the nested list ends.
The empty line above the list continuation indicates that the block should be attached to current list item in the parent list.

[source]
....
include::example$complex.adoc[tag=complex-parent]
....

Here's how the source is rendered:

.A block attached to the parent list item
====
include::example$complex.adoc[tag=complex-parent]
====

"#
    );

    let output = convert(
        "* parent list item\n** child list item\n\n+\nparagraph attached to parent list item\n",
    );
    // The paragraph attaches to the top-level (parent) item, alongside the one
    // nested list.
    assert_css(&output, ".ulist:root > ul > li > div.ulist", 1);
    assert_css(&output, ".ulist:root > ul > li > div.paragraph > p", 1);
    // Only one level of nesting: the parent item, not a deeper one.
    assert_css(
        &output,
        ".ulist:root > ul > li > div.ulist ul li div.ulist",
        0,
    );
}

// Two empty lines before the continuation move up two levels, attaching the
// block to the grandparent item — a sibling of the two-deep nested structure.
#[test]
fn two_empty_lines_attach_to_the_grandparent_item() {
    verifies!(
        r#"
Each empty line that precedes the list continuation signals a move up one level of nesting.
Here's an example that shows how to attach a paragraph to a grandparent list item using two leading empty lines:

[source]
....
include::example$complex.adoc[tag=complex-grandparent]
....

Here's how the source is rendered:

.A block attached to the grandparent list item
====
include::example$complex.adoc[tag=complex-grandparent]
====

"#
    );

    let output = convert(
        "* grandparent list item\n** parent list item\n*** child list item\n\n\n+\n\
         paragraph attached to grandparent list item\n",
    );
    // The paragraph attaches to the top-level (grandparent) item, which nests
    // two levels deep.
    assert_css(&output, ".ulist:root > ul > li > div.paragraph > p", 1);
    assert_css(
        &output,
        ".ulist:root > ul > li > div.ulist > ul > li > div.ulist",
        1,
    );
}

non_normative!(
    r#"
== Summary

On this page, you learned that the principal text of a list item can span multiple contiguous lines, and that those lines can be indented for readability without affecting the output.
You learned that you can attach any type of block content to a list item using the list continuation.
You also learned that using this feature in combination with the open block makes it easier to create list items with compound content, to attach blocks to a parent list, or to drop the principal text.
Finally, you learned that you can use the ancestor list continuation to attach blocks to the current item in an ancestor list, and the risks with doing so.
"#
);
