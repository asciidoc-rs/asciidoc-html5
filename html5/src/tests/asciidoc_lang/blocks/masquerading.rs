//! Coverage of the AsciiDoc language description's *Block Masquerading* page.
//!
//! The page documents how a declared block style changes a block's context.
//! This crate renders every masquerade the page demonstrates — a literal block
//! to a listing, a paragraph to a sidebar, and an example block or a paragraph
//! to a NOTE admonition — so each example is verified through `convert`. The
//! title, the conceptual introduction, the resolution-algorithm description,
//! and the reference table of built-in permutations are non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/masquerading.adoc");

non_normative!(
    r#"
= Block Masquerading
:page-aliases: masquerade.adoc

The declared block style (i.e., the first positional attribute of the block attribute list) can be used to modify the context of any paragraph and most structural containers.
This practice is known as block masquerading (meaning to disguise one block as another).

When the context of a paragraph is changed using the declared block style, the block retains the simple content model.
When masquerading the context of a structural container, only contexts that preserve the expected content model are permitted.

== How it works

If the block style declared on a block matches the name of a context, it sets the context of the block to that value and the resolved block style will be left unset.
If the declared block style does not match the name of a context, it will either specialize the context or set the context implicitly and also specialize that context.
How the declared block style is handled for a custom block is up to the extension, though a similar process takes place.

"#
);

// A structural container masquerade: the `[listing]` style on a literal
// (`....`) block changes its context to listing, rendering as a `listingblock`.
#[test]
fn listing_style_changes_a_literal_block_to_a_listing() {
    verifies!(
        r#"
Let's consider the case of using the declared block style to change the context of a structural container.
In this case, we're using the declared block style to change a literal block to a listing block.

----
[listing]
....
a > b
....
----

Even though the default context for the structural container is `:literal`, the declared block style changes it to `:listing`.
The resolved block style of the block remains unset.

"#
    );

    let output = convert("[listing]\n....\na > b\n....\n");
    assert_css(&output, "div.listingblock > div.content > pre", 1);
    assert_css(&output, "div.literalblock", 0);
}

// A paragraph masquerade: the `[sidebar]` style turns a normal paragraph into a
// sidebar, keeping the simple content model.
#[test]
fn sidebar_style_turns_a_paragraph_into_a_sidebar() {
    verifies!(
        r#"
The declared block style can also be used to transform a paragraph into a different kind of block.
The block will still retain the simple content model.
Let's consider the case of turning a normal paragraph into a sidebar.

----
[sidebar]
This sidebar is short, so a styled paragraph will do.
----

"#
    );

    let output = convert("[sidebar]\nThis sidebar is short, so a styled paragraph will do.\n");
    assert_css(&output, "div.sidebarblock > div.content", 1);
}

// A specializing masquerade: the `[NOTE]` style on an example (`====`) block
// transforms it into a NOTE admonition block.
#[test]
fn note_style_changes_an_example_block_to_an_admonition() {
    verifies!(
        r#"
Finally, let's consider an admonition block.
Declaring the NOTE block style on an example structural container transforms it to an admonition block and also sets the style of the block to NOTE.

----
[NOTE]
====
Remember the milk.
====
----

"#
    );

    let output = convert("[NOTE]\n====\nRemember the milk.\n====\n");
    assert_css(&output, "div.admonitionblock.note", 1);
    assert_css(&output, "div.exampleblock", 0);
}

// The same specialization applied to a paragraph: `[NOTE]` over a paragraph
// yields a NOTE admonition block.
#[test]
fn note_style_turns_a_paragraph_into_an_admonition() {
    verifies!(
        r#"
This technique also works for converting a paragraph into an admonition block.

----
[NOTE]
Remember the milk.
----

"#
    );

    let output = convert("[NOTE]\nRemember the milk.\n");
    assert_css(&output, "div.admonitionblock.note", 1);
}

// The reference table of which structural containers can masquerade as which
// contexts, plus the closing note that paragraphs share the open block's
// permutations and add the `normal` style.
non_normative!(
    r#"
Where permitted, the declared block style can be used to specialize the context of the block, change the context of the block, or both.

== Built-in permutations

The table below lists the xref:delimited.adoc#table-of-structural-containers[structural containers] whose context can be altered, and which contexts are valid, using the declared block style.

[cols="1,1,3"]
|===
|Type |Default context |Masquerading contexts

|example
|:example
|admonition (designated by the NOTE, TIP, WARNING, CAUTION, or IMPORTANT style)

|listing
|:listing
|literal

|literal
|:literal
|listing (can be designated using the source style)

|open
|:open
|abstract, admonition (designated by the NOTE, TIP, WARNING, CAUTION, or IMPORTANT style), comment, example, literal, listing (can be designated using the source style), partintro, pass, quote, sidebar, verse

|pass
|:pass
|stem, latexmath, asciimath

|sidebar
|:sidebar
|_n/a_

|quote
|:quote
|verse
|===

All the contexts that can be applied to an open block can also be applied to a paragraph.
A paragraph also access the `normal` style, which can be applied to revert a literal paragraph to a normal paragraph.
"#
);
