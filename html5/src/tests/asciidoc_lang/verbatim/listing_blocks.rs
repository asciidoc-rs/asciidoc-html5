//! Coverage of the AsciiDoc language description's *Listing Blocks* page.
//!
//! A listing block (the `listing` style on a block/paragraph, or a `----`
//! delimited block) renders its content as preformatted text inside
//! `pre` — a fixed-width font with endlines preserved — and applies only the
//! verbatim substitution group, so special characters are escaped and inline
//! formatting marks pass through literally. This crate matches Asciidoctor
//! 2.0.26; the page's three examples and the claims about them are verified
//! through `convert`.
//!
//! The page draws its example content with `include::example$listing.adoc`
//! directives, which Antora resolves against the module's `examples` family.
//! The tests reproduce the relevant tagged snippets inline and convert them.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/listing-blocks.adoc");

// The document title and an attribute entry that a later example
// (`ex-subs`) references; nothing renders yet.
non_normative!(
    r#"
= Listing Blocks
:replace-me: I've been replaced!

"#
);

// A paragraph (or block) assigned the `listing` style renders as a
// `div.listingblock` wrapping a `pre` element. The content is preformatted:
// endlines are preserved and inline formatting marks (here the backticks of a
// monospace span) pass through literally because only the verbatim group is
// applied. A separate conversion confirms that special characters are still
// escaped, the one substitution the verbatim group does perform.
#[test]
fn listing_style_renders_verbatim_preformatted() {
    verifies!(
        r#"
Blocks and paragraphs assigned the `listing` style display their rendered content exactly as you see it in the source.
Listing content is converted to preformatted text (i.e., `<pre>`).
The content is presented in a fixed-width font and endlines are preserved.
Only xref:subs:special-characters.adoc[special characters] and callouts are replaced when the document is converted.

The listing style can be applied to content using one of the following methods:

* setting the `listing` style on a block or paragraph using an attribute list, or
* enclosing the content within a pair of listing block delimiters (`----`).

== Listing style syntax

The block style `listing` can be applied to a block or paragraph, by setting the attribute `listing` using an attribute list.

.Listing style syntax
[#ex-style]
----
include::example$listing.adoc[tag=style]
----

The result of <<ex-style>> is rendered below.

include::example$listing.adoc[tag=style]

"#
    );

    // The `tag=style` snippet from `example$listing.adoc`.
    let output = convert(
        "[listing]\nThis is an example of a paragraph assigned\nthe `listing` style in an attribute list.\nNotice that the monospace marks are\npreserved in the output.\n",
    );

    assert_css(&output, "div.listingblock > div.content > pre", 1);

    // Endlines are preserved and the backtick marks pass through literally —
    // the content is shown exactly as written.
    assert!(output.contains(
        "<pre>This is an example of a paragraph assigned\nthe `listing` style in an attribute list.\nNotice that the monospace marks are\npreserved in the output.</pre>"
    ));

    // Special characters, the one substitution the verbatim group performs, are
    // still escaped.
    let escaped = convert("[listing]\ntags <p> & <pre>\n");
    assert!(escaped.contains("tags &lt;p&gt; &amp; &lt;pre&gt;"));
}

// A delimited listing block is fenced by lines of four hyphens (`----`), which
// lets the content contain empty lines. The `<pre>` in the content is escaped
// (displayed verbatim), endlines are preserved, and the underscores around
// `_delimited listing block_` are shown literally rather than italicizing the
// phrase — none of the inline formatting substitutions run.
#[test]
fn delimited_listing_block_is_verbatim() {
    verifies!(
        r#"
== Delimited listing block

A delimited listing block is surrounded by lines composed of four hyphens (`----`).
This method is useful when the content contains empty lines.

.Delimited listing block syntax
[#ex-block]
------
include::example$listing.adoc[tag=block]
------

Here's how the block in <<ex-block>> appears when rendered.

// The listing style must be added above the rendered block so it is rendered correctly because we're setting `source-language` in the component descriptor which automatically promotes unstyled listing blocks to source blocks.
[listing]
include::example$listing.adoc[tag=block]

You should notice a few things about how the content is processed.

* The HTML element `<pre>` is escaped, that is, it's displayed verbatim, not interpreted.
* The endlines are preserved.
* The phrase _delimited listing block_ isn't italicized, despite having the underscore formatting marks around it.

"#
    );

    // The `tag=block` snippet from `example$listing.adoc`, which contains an
    // empty line.
    let output = convert(
        "----\nThis is a _delimited listing block_.\n\nThe content inside is displayed as <pre> text.\n----\n",
    );

    assert_css(&output, "div.listingblock > div.content > pre", 1);

    // `<pre>` is escaped, the empty line (endline) is preserved, and the
    // underscores stay literal — no `<em>` is produced.
    assert!(output.contains(
        "<pre>This is a _delimited listing block_.\n\nThe content inside is displayed as &lt;pre&gt; text.</pre>"
    ));
    assert_xpath(&output, "//pre/em", 0);
}

// The narrative ties listing blocks to source blocks; tracked as a
// cross-reference with no rendered behavior of its own.
non_normative!(
    r#"
Listing blocks are good for displaying snippets of raw source code, especially when used in tandem with the `source` style and `source-highlighter` attribute.
See xref:source-blocks.adoc[] to learn more about `source` and `source-highlighter`.

"#
);

// Listing content is subject to the verbatim substitution group. The `subs`
// attribute can extend that group: `subs="+attributes"` adds attribute
// references on top of the verbatim group, so an attribute reference such as
// `{replace-me}` is resolved to its value, while the inline formatting marks
// (underscores, backticks) still pass through literally.
#[test]
fn listing_subs_attribute_adds_substitutions() {
    verifies!(
        r#"
== Listing substitutions

Content that is assigned the `listing` style, either via the explicit block style or the listing delimiters is subject to the xref:subs:index.adoc#verbatim-group[verbatim substitution group].
Only xref:subs:special-characters.adoc[special characters] and callouts are replaced automatically in listing content.

You can control the substitutions applied to a listing block using the `subs` attribute.

.Delimited listing block with custom substitutions syntax
[#ex-subs]
------
[subs="+attributes"]
----
This is a _delimited listing block_
with the `subs` attribute assigned
the incremental value `+attributes`.
This attribute reference:

{replace-me}

will be replaced with the attribute's
value when rendered.
----
------

The result of <<ex-subs>> is rendered below.

// The listing style must be added above the rendered block so it is rendered correctly because we're setting `source-language` in the component descriptor which automatically promotes unstyled listing blocks to source blocks.
[listing,subs="+attributes"]
----
This is a _delimited listing block_
with the `subs` attribute assigned
the incremental value `+attributes`.
This attribute reference:

{replace-me}

will be replaced with the attribute's
value when rendered.
----

"#
    );

    let output = convert(
        ":replace-me: I've been replaced!\n\n[listing,subs=\"+attributes\"]\n----\nThis is a _delimited listing block_\nwith the `subs` attribute assigned\nthe incremental value `+attributes`.\nThis attribute reference:\n\n{replace-me}\n\nwill be replaced with the attribute's\nvalue when rendered.\n----\n",
    );

    assert_css(&output, "div.listingblock > div.content > pre", 1);

    // The attribute reference is resolved by the added `attributes` substitution.
    assert!(output.contains("I've been replaced!"));
    assert!(!output.contains("{replace-me}"));

    // The formatting marks still pass through literally: no `<em>` and the
    // backticks remain in the text.
    assert_xpath(&output, "//pre/em", 0);
    assert!(output.contains("This is a _delimited listing block_"));
    assert!(output.contains("the incremental value `+attributes`."));
}

// A cross-reference to the substitutions guide; no rendered behavior.
non_normative!(
    r#"
See xref:subs:apply-subs-to-blocks.adoc[] to learn more about the `subs` attribute and how to apply incremental substitutions to listing content.
"#
);
