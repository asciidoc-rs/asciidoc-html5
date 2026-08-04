//! Coverage of the AsciiDoc language description's *Example Blocks* page.
//!
//! The page documents both the `[example]` paragraph style and the `====`
//! delimited example block. It also unsets `example-caption`, so the titles it
//! shows render without an "Example N." caption prefix; the `verifies!` tests
//! drive `convert` with that same attribute unset to match. The title, the
//! upstream-provenance comment, the "on this page" preamble, the section
//! headings, and the closing cross-reference tip are non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/example-blocks.adoc");

non_normative!(
    r#"
= Example Blocks
// Moved upstream from the Antora documentation at docs.antora.org
:!example-caption:

On this page, you'll learn:

* [x] How to mark up an example block with AsciiDoc.

An example block is useful for visually delineating content that illustrates a concept or showing the result of an operation.

An example can contain any type of content and AsciiDoc syntax.
Normal substitutions are applied to example content.

== Example style syntax

"#
);

// The example paragraph style: `[example]` on a contiguous paragraph renders as
// an `exampleblock`. With `example-caption` unset, its title is rendered plain
// (no "Example N." prefix).
#[test]
fn example_paragraph_style_renders_an_exampleblock() {
    verifies!(
        r#"
If the example content is contiguous, i.e., not interrupted by empty lines, the block style name `example` can be placed directly on top of the text in an attribute list (`[]`).

.Assign example block style to paragraph
[#ex-style]
----
.Optional title
[example]
This is an example of an example block.
----

The result of <<ex-style>> is displayed below.

.Optional title
[example]
This is an example of an example block.

"#
    );

    let output = convert(
        ":!example-caption:\n\n.Optional title\n[example]\n\
         This is an example of an example block.\n",
    );
    assert_css(&output, "div.exampleblock > div.content", 1);
    assert_xpath(
        &output,
        r#"//div[@class="exampleblock"]/div[@class="title"][text()="Optional title"]"#,
        1,
    );
}

non_normative!(
    r#"
[#delimited]
== Delimited example syntax

"#
);

// The delimited example: a `====` block needs no style name and encloses any
// content, here two paragraphs. With `example-caption` unset, its title renders
// plain.
#[test]
fn delimited_example_encloses_multiple_blocks() {
    verifies!(
        r#"
If the example content includes multiple blocks or content separated by empty lines, place the content between delimiter lines consisting of four equals signs (`pass:[====]`).

You don't need to set the block style name when you use the example delimiters.

.Example block delimiter syntax
[#ex-block]
----
.Onomatopoeia
====
The book hit the floor with a *thud*.

He could hear doves *cooing* in the pine trees`' branches.
====
----

The result of <<ex-block>> is displayed below.

.Onomatopoeia
====
The book hit the floor with a *thud*.

He could hear doves *cooing* in the pine trees`' branches.
====

"#
    );

    let output = convert(
        ":!example-caption:\n\n.Onomatopoeia\n====\n\
         The book hit the floor with a *thud*.\n\n\
         He could hear doves *cooing* in the pine trees`' branches.\n====\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="exampleblock"]/div[@class="title"][text()="Onomatopoeia"]"#,
        1,
    );
    assert_css(&output, "div.exampleblock > div.content > div.paragraph", 2);
    assert_css(&output, "div.exampleblock div.paragraph strong", 2);
}

// A cross-reference pointing readers to the admonitions page.
non_normative!(
    r#"
TIP: xref:admonitions.adoc[Complex admonitions] use the delimited example syntax.
"#
);
