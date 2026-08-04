//! Coverage of the AsciiDoc language description's *Paragraphs* page.
//!
//! The page documents how consecutive lines form a paragraph and how to work
//! around the conflict between a paragraph's first line and a block attribute
//! line. This crate renders both, so the examples are verified through
//! `convert`: consecutive lines separated by a blank line become two
//! paragraphs, and the `{empty}` / `[normal]` workarounds keep a line that
//! looks like a block attribute list rendering as a paragraph. The title, the
//! introduction, and the section headings are non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/paragraphs.adoc");

non_normative!(
    r#"
= Paragraphs

The primary block type in most documents is the paragraph.
That's why in AsciiDoc, you don't need to use any special markup or attributes to create paragraphs.
You can just start typing sentences and that content becomes a paragraph.

This page introduces you to the paragraph in AsciiDoc and explains how to set it apart from other paragraphs.

== Create a paragraph

"#
);

// Adjacent lines of text form one paragraph; a blank line starts the next.
// The two-paragraph example thus renders as two `<div class="paragraph">`
// wrappers.
#[test]
fn consecutive_lines_form_paragraphs_separated_by_blank_lines() {
    verifies!(
        r#"
Adjacent or consecutive lines of text form a paragraph element.
To start a new paragraph after another element, such as a section title or table, hit the kbd:[RETURN] key twice to insert an empty line, and then continue typing your content.

.Two paragraphs in an AsciiDoc document
[#ex-paragraph]
----
include::example$paragraph.adoc[tag=para]
----

The result of <<ex-paragraph>> is displayed below.

====
include::example$paragraph.adoc[tag=para]
====

"#
    );

    let output = convert(
        "Paragraphs don't require any special markup in AsciiDoc.\n\
         A paragraph is just one or more lines of consecutive text.\n\n\
         To begin a new paragraph, separate it by at least one empty line from the previous paragraph or block.\n",
    );
    assert_css(&output, "div.paragraph", 2);
}

non_normative!(
    r#"
[#block-attribute-line-conflict]
== Conflict with a block attribute line

"#
);

// A first line that starts with `[` and ends with `]` is otherwise consumed as
// a block attribute line. Prefixing it with `{empty}` (which resolves to
// nothing) or writing it as a `[normal]` literal paragraph disrupts that match,
// so the line renders as an ordinary paragraph — here one carrying a footnote.
#[test]
fn empty_and_normal_workarounds_keep_the_line_as_a_paragraph() {
    verifies!(
        r#"
The first line of a paragraph cannot start with `[` and end with `]`.
That's because it will be treated as block metadata, specifically a block attribute line.

There are two ways to work around this issue.
First, you can add any text in front of the `[` that will otherwise not be rendered to disrupt the syntax match.
For example, you can start the paragraph with the attribute reference `+{empty}+`.

----
{empty}[.rolename]*main text*footnote:[The footnote.]
----

Another way to solve the problem is to write the paragraph as a literal paragraph, then override the implicit style using the explicit style normal.

----
[normal]
 [.rolename]*main text*footnote:[The footnote.]
----

The normal style often comes in handy when you want to prevent the parser from matching beginning-of-line syntax.
"#
    );

    let empty_workaround = convert("{empty}[.rolename]*main text*footnote:[The footnote.]\n");
    assert_css(&empty_workaround, "div.paragraph", 1);
    assert_css(&empty_workaround, "div.paragraph sup.footnote", 1);

    let normal_workaround = convert("[normal]\n [.rolename]*main text*footnote:[The footnote.]\n");
    assert_css(&normal_workaround, "div.paragraph", 1);
    assert_css(&normal_workaround, "div.paragraph sup.footnote", 1);
}
