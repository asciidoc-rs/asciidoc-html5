//! Coverage of the AsciiDoc language description's *Hard Line Breaks* page.
//!
//! The page documents the ways to preserve line breaks in a paragraph: the
//! trailing `+` on a single line, the `hardbreaks` option on a paragraph, and
//! the `hardbreaks-option` document attribute, plus the `{empty}` techniques
//! and the endash-replacement conflict. This crate renders `<br>` for each, so
//! every technique is verified through `convert`. The title, the
//! verbatim-retention tip, and the section headings are non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/hard-line-breaks.adoc");

non_normative!(
    r#"
= Hard Line Breaks

"#
);

// The default: adjacent lines of regular text are joined into a single
// paragraph with no `<br>` — the line break becomes a space.
#[test]
fn adjacent_lines_join_into_one_paragraph() {
    verifies!(
        r#"
Adjacent lines of regular text in AsciiDoc are combined into a single paragraph when converted.
That means you can wrap paragraph text in the source document, either at a specific column or by putting each sentence or phrase on its own line.
The line breaks separating adjacent lines won't appear in the output.
Instead, the line break will be (effectively) converted to a single space.
(In fact, all repeating space characters are reduced to a single space, just like in HTML.)

"#
    );

    let output = convert("line one\nline two\n");
    assert_css(&output, "div.paragraph", 1);
    assert_css(&output, "div.paragraph p br", 0);
}

non_normative!(
    r#"
TIP: Hard line breaks are automatically retained in xref:verbatim:literal-blocks.adoc[literal], xref:verbatim:listing-blocks.adoc[listing], xref:verbatim:source-blocks.adoc[source], and xref:verses.adoc[verse] blocks and paragraphs.

"#
);

// A trailing space-plus (`line +`) ends that line with a hard `<br>`.
#[test]
fn trailing_plus_produces_a_hard_break() {
    verifies!(
        r#"
If you want line breaks in a paragraph to be preserved, there are several techniques you can use.
For any single line, you can terminate it with a space followed by a plus sign.
This syntax signals to the processor to end the line in the output with a hard line break.

[source]
----
line one +
line two
----

"#
    );

    let output = convert("line one +\nline two\n");
    assert_css(&output, "div.paragraph p br", 1);
}

// The `hardbreaks` option on the paragraph applies the hard break to every
// line.
#[test]
fn hardbreaks_option_breaks_every_line() {
    verifies!(
        r#"
To add this behavior to every line in the paragraph, set the `hardbreaks` option on the paragraph instead.

[source]
----
[%hardbreaks]
line one
line two
----

"#
    );

    let output = convert("[%hardbreaks]\nline one\nline two\n");
    assert_css(&output, "div.paragraph p br", 1);
}

// The `hardbreaks-option` document attribute applies the behavior to every
// paragraph in the document.
#[test]
fn hardbreaks_option_attribute_breaks_every_paragraph() {
    verifies!(
        r#"
Alternately, you can tell the processor to preserve all line breaks in every paragraph in the document by setting the `hardbreaks-option` document attribute, though this option should be used wisely.

[source]
----
:hardbreaks-option:

line one
line two
----

"#
    );

    let output = convert(":hardbreaks-option:\n\nline one\nline two\n");
    assert_css(&output, "div.paragraph p br", 1);
}

// A space-plus on a line by itself inserts an empty line in the middle of the
// paragraph — two consecutive `<br>` — without starting a new paragraph.
#[test]
fn space_plus_on_its_own_line_inserts_a_blank_line() {
    verifies!(
        r#"
To insert an empty line in the middle of the paragraph, you can use the hard line break syntax on a line by itself.
This allows you to insert space between lines in the output without introducing separate paragraphs.

[source]
----
line one +
 +
line three
----

"#
    );

    let output = convert("line one +\n +\nline three\n");
    assert_css(&output, "div.paragraph", 1);
    assert_css(&output, "div.paragraph p br", 2);
}

// Starting a paragraph with a hard line break: an `{empty}` reference stands in
// for the otherwise-significant leading space.
#[test]
fn empty_reference_allows_a_leading_hard_break() {
    verifies!(
        r#"
If you want the paragraph to start with a hard line break, you need to place an `\{empty}` attribute reference at the start of the line.
That's because a line that starts with a space has a different meaning.
The `\{empty}` attribute reference allows you to insert nothing at the start of the line.

[source]
----
{empty} +
line two
----

To be consistent, you can always start an empty line with `\{empty}`.

[source]
----
{empty} +
line two +
{empty} +
line four
----

"#
    );

    let output = convert("{empty} +\nline two\n");
    assert_css(&output, "div.paragraph p br", 1);
}

non_normative!(
    r#"
Note that `empty` is a built-in document attribute in AsciiDoc.

"#
);

// The endash-replacement conflict: a trailing `+` before a line that begins
// with `--` fails to break, because the endash replacement consumes the
// newline; an inserted `{empty}` line under `hardbreaks` gives the replacement
// something else to consume so each line of dialogue keeps its break.
#[test]
fn endash_replacement_can_consume_a_hard_break() {
    verifies!(
        r#"
If you're writing a story with dialogue, and you want to prefix the dialogue lines with `--`, be aware that you are going to get a conflict between the hard line break and the automatic endash replacement.
For example:

[source]
----
-- Come here! -- I said. +
-- What is it? -- replied Lance.
----

The second `--` will not only be substituted with an endash, it will also consume the preceding newline.
As a result, both lines in the source will end up appearing on the same line in the output.
To circumvent this problem, you can need to add an extra newline for the endash replacement to consume.

[source]
----
[%hardbreaks]
-- Come here! -- I said.
{empty}
-- What is it? -- replied Lance.
----

Then each dialog will appear on its own line.

"#
    );

    // The conflict: no hard break survives, both lines collapse onto one.
    let conflict = convert("-- Come here! -- I said. +\n-- What is it? -- replied Lance.\n");
    assert_css(&conflict, "div.paragraph p br", 0);

    // The fix: the inserted `{empty}` line restores the break.
    let fixed = convert(
        "[%hardbreaks]\n-- Come here! -- I said.\n{empty}\n-- What is it? -- replied Lance.\n",
    );
    assert_css(&fixed, "div.paragraph p br", 1);
}

non_normative!(
    r#"
[#per-line]
== Inline line break syntax

"#
);

// The inline line-break syntax, shown with its rendered example: `Rubies are
// red, +` breaks before `Topazes are blue.`
#[test]
fn inline_line_break_example_renders_a_break() {
    verifies!(
        r#"
To preserve a line break in a paragraph, insert a space followed by a plus sign (`{plus}`) at the end of the line.
This results in a visible line break (e.g., `<br>`) following the line.

.Line breaks preserved using a space followed by the plus sign ({plus})
[#ex-plus]
----
include::example$paragraph.adoc[tag=hb]
----

The result of <<ex-plus>> is displayed below.

====
include::example$paragraph.adoc[tag=hb]
====

"#
    );

    let output = convert("Rubies are red, +\nTopazes are blue.\n");
    assert_css(&output, "div.paragraph p br", 1);
}

non_normative!(
    r#"
[#per-block]
== hardbreaks option

"#
);

// The `hardbreaks` option example: every line of the paragraph breaks.
#[test]
fn hardbreaks_option_example_breaks_every_line() {
    verifies!(
        r#"
To retain all of the line breaks in an entire paragraph, assign the `hardbreaks` option to the paragraph using an attribute list.

.Line breaks preserved using the hardbreaks option
[#ex-option]
----
include::example$paragraph.adoc[tag=hb-p]
----

The result of <<ex-option>> is displayed below.

====
include::example$paragraph.adoc[tag=hb-p]
====

"#
    );

    let output = convert("[%hardbreaks]\nRuby is red.\nJava is beige.\n");
    assert_css(&output, "div.paragraph p br", 1);
}

non_normative!(
    r#"
[#per-document]
== hardbreaks-option attribute

"#
);

// The `hardbreaks-option` document-attribute example: set in the header, it
// preserves the line breaks of the following paragraph.
#[test]
fn hardbreaks_option_attribute_example_breaks_the_paragraph() {
    verifies!(
        r#"
To preserve line breaks in all paragraphs throughout your entire document, set the `hardbreaks-option` document attribute in the document header.

.Line breaks preserved throughout the document using the hardbreaks-option attribute
[#ex-attribute]
----
include::example$paragraph.adoc[tag=hb-attr]
----
"#
    );

    let output = convert(
        "= Line Break Doc Title\n:hardbreaks-option:\n\nRubies are red,\nTopazes are blue.\n",
    );
    assert_css(&output, "div.paragraph p br", 1);
}
