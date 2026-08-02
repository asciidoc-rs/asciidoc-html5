//! Coverage of the AsciiDoc language description's *Literal Blocks* page.
//!
//! A literal block renders its content as preformatted text (`div.literalblock`
//! wrapping a `pre`) with only the verbatim substitution group applied. The
//! page documents three ways to create one — indenting a line, the `literal`
//! block style, or the `....` delimiters — and this crate matches Asciidoctor
//! 2.0.26 for each. The examples and the claims about them are verified through
//! `convert`.
//!
//! The page draws its example content with `include::example$literal.adoc`
//! directives; the tests reproduce the relevant tagged snippets inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/literal-blocks.adoc");

// The overview claims — literal blocks display text verbatim as preformatted
// text with endlines preserved and only special characters/callouts replaced —
// are each demonstrated by the concrete examples that follow and verified
// there. A minimal indented line stands in for the general statement: it
// becomes a `div.literalblock` whose `pre` preserves the text and escapes the
// one special character present.
#[test]
fn literal_blocks_display_text_verbatim() {
    verifies!(
        r#"
= Literal Blocks

Literal blocks display the text you write exactly as you see it in the source.
Literal text is treated as preformatted text.
The text is presented in a fixed-width font and endlines are preserved.
Only xref:subs:special-characters.adoc[special characters] and callouts are replaced when the document is converted.

The literal style can be applied to content using any of the following methods:

* indenting the first line of a paragraph by one or more spaces,
* setting the `literal` style on a block using an attribute list, or
* enclosing the content within a pair of literal block delimiters (`\....` ).

"#
    );

    let output = convert(" a < b is literal\n");

    assert_css(&output, "div.literalblock > div.content > pre", 1);
    assert!(output.contains("<pre>a &lt; b is literal</pre>"));
}

// The indent method: when a line begins with one or more spaces it is displayed
// as a literal block. Asciidoctor strips the common leading indentation, so the
// single space that marks the line does not appear in the rendered `pre`.
#[test]
fn indent_method_creates_a_literal_block() {
    verifies!(
        r#"
== Indent method

When a line begins with one or more spaces it is displayed as a literal block.
This method is an easy way to insert simple code snippets.

.Indicate literal text using an indent
[source#ex-indent]
----
include::example$literal.adoc[tag=indent]
----

The result of <<ex-indent>> is rendered below.

include::example$literal.adoc[tag=indent]

"#
    );

    // The `tag=indent` snippet: a single line indented by one space.
    let output = convert(" ~/secure/vault/defops\n");

    assert_css(&output, "div.literalblock > div.content > pre", 1);
    assert!(output.contains("<pre>~/secure/vault/defops</pre>"));
}

// The literal style: setting `literal` on a block (here a paragraph) via an
// attribute list renders it as a literal block, preserving its endlines.
#[test]
fn literal_style_creates_a_literal_block() {
    verifies!(
        r#"
=== literal style syntax

The literal style can be applied to a block, such as a paragraph, by setting the style attribute `literal` on the block using an attribute list.

.Literal style syntax
[source#ex-style]
----
include::example$literal.adoc[tag=style]
----

The result of <<ex-style>> is rendered below.

include::example$literal.adoc[tag=style]

"#
    );

    // The `tag=style` snippet from `example$literal.adoc`.
    let output = convert(
        "[literal]\nerror: 1954 Forbidden search\nabsolutely fatal: operation lost in the dodecahedron of doom\nWould you like to try again? y/n\n",
    );

    assert_css(&output, "div.literalblock > div.content > pre", 1);
    assert!(output.contains(
        "<pre>error: 1954 Forbidden search\nabsolutely fatal: operation lost in the dodecahedron of doom\nWould you like to try again? y/n</pre>"
    ));
}

// The delimited literal block (`....`) can hold empty lines, and — as the
// closing note states — its content is verbatim: the `*bold*` marks do not
// produce `<strong>` and the three consecutive periods are not replaced by the
// ellipsis character.
#[test]
fn delimited_literal_block_is_verbatim() {
    verifies!(
        r#"
=== Delimited literal block

Finally, you can surround the content you want rendered as literal by enclosing it in a pair of literal block delimiters (`\....`).
This method is useful when the content contains empty lines.

.Delimited literal block syntax
[source#ex-block]
----
include::example$literal.adoc[tag=block]
----

The result of <<ex-block>> is rendered below.

include::example$literal.adoc[tag=block]

Notice in the output that the bold text formatting is not applied to the text nor are the three consecutive periods replaced by the ellipsis Unicode character.
"#
    );

    // The `tag=block` snippet from `example$literal.adoc`, which contains empty
    // lines, bold marks, and three consecutive periods.
    let output = convert(
        "....\nKismet: Where is the *defensive operations manual*?\n\nComputer: Calculating ...\nCan not locate object.\nYou are not authorized to know it exists.\n\nKismet: Did the werewolves tell you to say that?\n\nComputer: Calculating ...\n....\n",
    );

    assert_css(&output, "div.literalblock > div.content > pre", 1);

    // The empty lines are preserved.
    assert!(output.contains("You are not authorized to know it exists.\n\nKismet:"));

    // Bold marks are not interpreted: no `<strong>`, and the asterisks remain.
    assert_xpath(&output, "//pre/strong", 0);
    assert!(output.contains("*defensive operations manual*"));

    // The three periods are not replaced by the ellipsis entity (`&#8230;`).
    assert!(output.contains("Computer: Calculating ..."));
    assert!(!output.contains("&#8230;"));
}
