//! Coverage of the AsciiDoc language description's *Delimited Blocks* page.
//!
//! The page is mostly a conceptual overview of structural containers, their
//! delimiters, content models, and the reference table of built-in containers —
//! all tracked as non-normative. The concrete syntax examples it shows (a
//! literal container, an example container, and the two nesting forms) are
//! verified through `convert`: nesting different containers just requires
//! enclosure, while nesting the same container requires varying the delimiter
//! length.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/delimited.adoc");

non_normative!(
    r#"
= Delimited Blocks

In AsciiDoc, a delimited block is a region of content that's bounded on either side by a pair of congruent linewise delimiters.
A delimited block is used either to enclose other blocks (e.g.., multiple paragraphs) or set the content model of the content (e.g., verbatim).
Delimited blocks are a subset of all block types in AsciiDoc.

== Overview

Delimited blocks are defined using structural containers, which are the fixed set of recognized block enclosures in the AsciiDoc syntax.
Here's the structural container for a literal block:

"#
);

// The literal structural container: a `....` delimited region renders as a
// verbatim `literalblock`.
#[test]
fn literal_container_renders_verbatim_content() {
    verifies!(
        r#"
----
....
This text will be treated as verbatim content.
....
----

"#
    );

    let output = convert("....\nThis text will be treated as verbatim content.\n....\n");
    assert_css(&output, "div.literalblock > div.content > pre", 1);
}

non_normative!(
    r#"
A structural container has an opening delimiter and a closing delimiter.
The opening delimiter follows the block metadata, if present.
Leading and trailing empty lines in a structure container are not considered significant and are automatically removed.
The remaining lines define a block's content.

These enclosures not only define the boundaries of a block's content, but also imply a content model (e.g., verbatim content or a subtree).
In certain cases, they provide a mechanism for nesting blocks.
However, delimited blocks cannot be interleaved.

A delimited block has the unique ability of being able to be repurposed by the AsciiDoc syntax, through both built-in mappings and mappings for custom blocks defined by an extension.
To understand how delimited blocks work, it's important to understand structural containers, their linewise delimiters, their default contexts, and their expected content models, as well as block nesting and masquerading.

== Linewise delimiters

A delimited block is characterized by a pair of congruent linewise delimiters.
The opening and closing delimiter must match exactly, both in length and in sequence of characters.
These delimiters, sometimes referred to as fences, enclose the content and explicitly mark its boundaries.
Within the boundaries of a delimited block, you can enter any content or empty lines.
The block doesn't end until the ending delimiter is found.
The block metadata (block attribute and anchor lines) goes above the opening delimiter (thus outside the delimited region).

Here's an example of a delimited example block:

"#
);

// The example structural container: a `====` delimited region renders as a
// compound `exampleblock`.
#[test]
fn example_container_renders_a_compound_block() {
    verifies!(
        r#"
----
====
This is an example of an example block.
That's so meta.
====
----

"#
    );

    let output = convert("====\nThis is an example of an example block.\nThat's so meta.\n====\n");
    assert_css(&output, "div.exampleblock > div.content > div.paragraph", 1);
}

non_normative!(
    r#"
Typically, the delimiter is written using the minimum allowable length (4 characters, with the exception of the open block, which currently has a fixed length of 2 characters).
The length of the delimiter lines can be varied to accommodate nested blocks.

The valid set of delimiters for defining a delimited block, and the meaning they have, is defined by the available structural containers, covered next.

== Structural containers

Structural containers are the fixed set of recognized block enclosures (delimited regions) defined by the AsciiDoc language.
These enclosures provide a reusable building block in the AsciiDoc syntax.
By evaluating the structural container and the block metadata, the processor will determine which kind of block to make.

Each structural container has an expected content model.
For built-in blocks, it's the context of the block that determines the content model, though most built-in blocks adhere to the expected content model.
Custom blocks have the ability to designate the content model.
Even in these cases, though, the content model should be chosen to honor the semantics of the structural container.
//This allows a text editor to understand how to parse the block and provide a reasonable fallback when the extension is not loaded.

Some structural containers are reused for different purposes, such as the structural container for the quote block being used for a verse block.

=== Summary of structural containers

The table below lists the structural containers, documenting the name, default context, and delimiter line for each.

.Structural containers in AsciiDoc
[#table-of-structural-containers,cols="1,2m,2,2l"]
|===
|Type |Default context |Content model (expected) |Minimum delimiter

|comment
|_n/a_
|_n/a_
|////

|example
|:example
|compound
|====

|listing
|:listing
|verbatim
|----

|literal
|:literal
|verbatim
|....

|open
|:open
|compound
|--

|sidebar
|:sidebar
|compound
|****

|table
|:table
|table
|\|===
,===
:===
!===

|pass
|:pass
|raw
|++++

|quote
|:quote
|compound
|____
|===

You may notice the absence of the source block.
That's because source is not a container type.
Rather, it's a specialization of the listing (or literal) container as designated by the block style.
The verse and admonition blocks are also noticeably absent.
That's because they repurpose the structural containers for quote and example blocks, respectively.

The default context is assumed when an explicit block style is not present.

Currently, table is a specialized structural container that cannot be enlisted as a custom block.

Unlike other structural containers, a comment block is not preserved in the parsed document and therefore doesn't have a context or content model.

TIP: When creating a custom block, it's important to choose a structural container that provides the right content model.
This allows a text editor to understand how to parse the block and provide a reasonable fallback when the extension is not loaded.

Structural containers are used to define delimited blocks.
The structural container provides a default context and expected content model, but the actual context and content model is determined after considering the metadata on the block (specifically the declared block content).

[#nesting]
== Nesting blocks

Using delimited blocks, you can nest blocks inside of one other.
(Blocks can also be nested inside sections, list items, and table cells, which is a separate topic).

First, the parent block must have the compound content model.
The compound content model means that the block's content is a sequence of zero or more blocks.

When nesting a block that uses a different structural container from the parent, it's enough to ensure that the child block is entirely inside the parent block.
Delimited blocks cannot be interleaved.

"#
);

// Nesting different containers: a `----` listing nested inside a `====` example
// only needs to be entirely enclosed — the child listing renders inside the
// parent example's content.
#[test]
fn nesting_a_different_container_only_requires_enclosure() {
    verifies!(
        r#"
[source]
....
====
Here's a sample AsciiDoc document:

----
= Document Title
Author Name

Content goes here.
----

The document header is useful, but not required.
====
....

"#
    );

    let output = convert(
        "====\nHere's a sample AsciiDoc document:\n\n----\n= Document Title\nAuthor Name\n\n\
         Content goes here.\n----\n\nThe document header is useful, but not required.\n====\n",
    );
    assert_css(
        &output,
        "div.exampleblock > div.content > div.listingblock",
        1,
    );
}

non_normative!(
    r#"
When nesting a delimited block that uses the same structural container, it's necessary to vary the length of the delimiter lines (i.e., make the length of the delimiter lines for the child block different than the length of the delimiter lines for the parent block).
Varying the delimiter line length allows the parser to distinguish one block from another.

"#
);

// Nesting the same container: the child delimiters must differ in length from
// the parent's. Here `======` collapsible examples nest inside a `====`
// example.
#[test]
fn nesting_the_same_container_requires_varied_delimiter_length() {
    verifies!(
        r#"
----
====
Here are your options:

.Red Pill
[%collapsible]
======
Escape into the real world.
======

.Blue Pill
[%collapsible]
======
Live within the simulated reality without want or fear.
======
====
----

"#
    );

    let output = convert(
        "====\nHere are your options:\n\n.Red Pill\n[%collapsible]\n======\n\
         Escape into the real world.\n======\n\n.Blue Pill\n[%collapsible]\n======\n\
         Live within the simulated reality without want or fear.\n======\n====\n",
    );
    assert_css(&output, "div.exampleblock > div.content > details", 2);
}

non_normative!(
    r#"
The delimiter length for the nested structural container can either be shorter or longer than the parent.
That's a personal style choice.
"#
);
