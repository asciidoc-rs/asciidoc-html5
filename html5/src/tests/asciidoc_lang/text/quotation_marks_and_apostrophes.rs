//! Coverage of the AsciiDoc language description's *Quotation Marks and
//! Apostrophes* page.
//!
//! Straight quotation marks pass through unchanged; a monospace backtick pair
//! placed just inside a quote pair produces curved quotation marks; and an
//! apostrophe bounded by word characters becomes a curved apostrophe during the
//! replacements phase (escapable with a backslash). This crate reproduces all
//! of these, including the possessive-monospace forms, so each example is
//! verified through `convert`. The title, introduction, section headings, the
//! "input the characters directly" advice, and the trailing editorial comment
//! are tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/quotation-marks-and-apostrophes.adoc");

non_normative!(
    r#"
= Quotation Marks and Apostrophes

This page describes how to insert curved quotation marks and apostrophes using the AsciiDoc syntax.
It covers the shorthand syntax, the limitations of that syntax, and when it's necessary to input these characters directly.

== Single and double quotation mark syntax

"#
);

// Single or double quotation marks used as constrained pairs carry no special
// meaning and are emitted exactly as entered (straight quotes).
#[test]
fn straight_quotation_marks() {
    verifies!(
        r#"
AsciiDoc does not assign special meaning to single or double quotation marks when used as constrained formatting pairs (e.g., around a word or phrase).
In this case, the kbd:['] and kbd:["] characters are taken to be straight quotation marks (also known as dumb, vertical, or typewriter quotation marks).
When an AsciiDoc processor encounters straight quotation marks in this context, it outputs them as entered.

.Single and double straight quotation marks syntax
[#ex-straight-quotes]
----
include::example$text.adoc[tag=straight-quotes]
----

The result of <<ex-straight-quotes>> is rendered below.

====
include::example$text.adoc[tag=straight-quotes]
====

"#
    );

    let out = convert(
        "In Ruby, '\\n' represents a backslash followed by the letter n.\nSingle quotes prevent escape sequences from being interpreted.\nIn contrast, \"\\n\" represents a newline.\n",
    );
    assert!(out.contains("In Ruby, '\\n' represents"));
    assert!(out.contains("In contrast, \"\\n\" represents a newline."));
    // The straight quotes are not curved.
    assert!(!out.contains("&#8216;"));
    assert!(!out.contains("&#8220;"));
}

// A monospace backtick pair placed directly inside a quote pair forms a
// constrained pair that renders curved single (&#8216;/&#8217;) and double
// (&#8220;/&#8221;) quotation marks.
#[test]
fn curved_quotation_marks() {
    verifies!(
        r#"
You can instruct the AsciiDoc processor to output curved quotation marks (also known as smart, curly, or typographic quotation marks) by adding a repurposed constrained monospace formatting pair (i.e., a pair of backticks, `{backtick}`) directly inside the pair of quotation marks.
The combination of these two formatting pairs forms a new constrained formatting pair for producing single and double curved quotation marks.

.Single and double curved quotation marks syntax
[#ex-curved-quotes]
----
include::example$text.adoc[tag=c-quote-co]
----
<.> To output double curved quotes, enclose a word or phrase in a pair of double quotes (`{quot}`) and a pair of backticks (`{backtick}`).
<.> To output single curved quotes, enclose a word or phrase in a pair of single quotes (`{apos}`) and a pair of backticks (`{backtick}`).
In this example, the phrase _wormwood and licorice_ should be enclosed in curved single quotes when the document is rendered.

The result of <<ex-curved-quotes>> is rendered below.

====
include::example$text.adoc[tag=c-quote]
====

There's no unconstrained equivalent for producing double and single curved quotation marks.
In that case, it's necessary to input the curved quotation marks directly using the characters kbd:[‘], kbd:[’], kbd:[“], and kbd:[”].

"#
    );

    let out = convert(
        "\"`What kind of charm?`\" Lazarus asked.\n\"`An odoriferous one or a mineral one?`\"\n\nKizmet shrugged.\n\"`The note from Olaf's desk says '`wormwood and licorice,`'\nbut these could be normal groceries for werewolves.`\"\n",
    );
    assert!(out.contains("&#8220;What kind of charm?&#8221;"));
    assert!(out.contains("&#8216;wormwood and licorice,&#8217;"));
}

non_normative!(
    r#"
== Apostrophe syntax

"#
);

// A straight apostrophe bounded by two word characters is rendered as a curved
// apostrophe (&#8217;) during the replacements phase.
#[test]
fn curved_apostrophe_replacement() {
    verifies!(
        r#"
When entered using the kbd:['] key, the AsciiDoc processor translates a straight apostrophe directly preceded and followed by a word character, such as in contractions and possessive singular forms, as a curved apostrophe.
This partial support for smart typography without any special syntax is a legacy inherited from AsciiDoc.py.

.Curved apostrophe replacement
[#ex-apostrophe-replacement]
----
Olaf's desk was a mess.
----

The result of <<ex-apostrophe-replacement>> is rendered below.

====
Olaf's desk was a mess.
====

"#
    );

    let out = convert("Olaf's desk was a mess.\n");
    assert!(out.contains("Olaf&#8217;s desk was a mess."));
}

// Preceding the apostrophe with a backslash escapes the replacement, leaving a
// straight apostrophe.
#[test]
fn escape_an_apostrophe() {
    verifies!(
        r#"
If you don't want a straight apostrophe that's bounded by two word characters to be rendered as a curved apostrophe, escape it by preceding it with a backslash (`{backslash}`).

.Escape an apostrophe
[#ex-escape]
----
Olaf\'s desk ...
----

The result of <<ex-escape>> is rendered below.

====
Olaf\'s desk ...
====

"#
    );

    let out = convert("Olaf\\'s desk ...\n");
    assert!(out.contains("Olaf's desk"));
    assert!(!out.contains("Olaf&#8217;s desk"));
}

non_normative!(
    r#"
An apostrophe directly bounded by two word characters is processed during the xref:subs:replacements.adoc[replacements substitution phase], not the inline formatting (quotes) phase.
To learn about additional ways to prevent the replacements substitution, see xref:subs:prevent.adoc[] and xref:pass:pass-macro.adoc[].

"#
);

// An apostrophe followed by a space or punctuation is not curved by default;
// the second half of a single curved quote (`` `' ``) renders it as a curved
// apostrophe.
#[test]
fn curved_apostrophe_when_unbounded() {
    verifies!(
        r#"
An apostrophe directly followed by a space or punctuation, such as the possessive plural form, is not curved by default.
To render a curved apostrophe when not bounded by two word characters, mark it as you would the second half of single curved quote (i.e., `pass:[`']`).
This syntax for a curved apostrophe is effectively unconstrained.

.Curved apostrophe syntax
[#ex-apostrophe]
----
include::example$text.adoc[tag=apos]
----

In the rendered output for <<ex-apostrophe>> below, notice that the plural possessive apostrophe, seen trailing _werewolves_, is curved, as is the omission apostrophe before _00s_.

====
include::example$text.adoc[tag=apos]
====

"#
    );

    let out = convert(
        "Olaf had been with the company since the `'00s.\nHis desk overflowed with heaps of paper, apple cores and squeaky toys.\nWe couldn't find Olaf's keyboard.\nThe state of his desk was replicated, in triplicate, across all of\nthe werewolves`' desks.\n",
    );
    // The omission apostrophe before "00s" is curved.
    assert!(out.contains("since the &#8217;00s."));
    // The plural possessive apostrophe trailing "werewolves" is curved.
    assert!(out.contains("the werewolves&#8217; desks."));
}

non_normative!(
    r#"
=== Possessive monospace

"#
);

// An unconstrained monospace pair followed by `` `' `` renders a possessive
// monospace phrase with a curved apostrophe outside the `<code>` element.
#[test]
fn possessive_monospace_in_word() {
    verifies!(
        r#"
In order to make a possessive, monospaced phrase, you need to switch to unconstrained formatting followed by an explicit typographic apostrophe.

.Use a curved apostrophe with monospace in a word
[#ex-word]
----
``npm```'s job is to manage the dependencies for your application.

A ``std::vector```'s size is the number of items it contains.
----

The result of <<ex-word>> is rendered below.

====
``npm```'s job is to manage the dependencies for your application.

A ``std::vector```'s size is the number of items it contains.
====

"#
    );

    let out = convert(
        "``npm```'s job is to manage the dependencies for your application.\n\nA ``std::vector```'s size is the number of items it contains.\n",
    );
    assert!(out.contains("<code>npm</code>&#8217;s job"));
    assert!(out.contains("A <code>std::vector</code>&#8217;s size"));
}

// The same technique applies when the monospace phrase ends in "s" (plural
// possessive): the curved apostrophe follows the closing `<code>` tag.
#[test]
fn possessive_monospace_at_end_of_word() {
    verifies!(
        r#"
You'll need to use a similar syntax when the last (or only) word in the monospace phrase ends in an "`s`" (i.e., the plural possessive form).

.Use a curved apostrophe with monospace at the end of a word
[#ex-word-end]
----
This ``class```' static methods make it easy to operate on files and directories.
----

The result of <<ex-word-end>> is below.
The word _class_ is rendered in monospace with a curved apostrophe at the end of it.

====
This ``class```' static methods make it easy to operate on files and directories.
====

"#
    );

    let out = convert(
        "This ``class```' static methods make it easy to operate on files and directories.\n",
    );
    assert!(out.contains("This <code>class</code>&#8217; static methods"));
}

// A typographic apostrophe placed immediately after a constrained monospace
// pair yields the same possessive-monospace result without unconstrained marks.
#[test]
fn possessive_monospace_with_typographic_apostrophe() {
    verifies!(
        r#"
You can get the same result by inserting a typographic apostrophe immediately following a constrained formatting pair.
In this case, you're able to leverage the fact that a typographic apostrophe is a punctuation character to avoid the need to resort to unconstrained formatting.

----
The `class`’ static methods make it easy to operate on files and directories.
----

"#
    );

    let out =
        convert("The `class`’ static methods make it easy to operate on files and directories.\n");
    assert!(out.contains("The <code>class</code>’ static methods"));
}

non_normative!(
    r#"
As you can see, it's often simpler to input the curved apostrophe directly using the character kbd:[’].
The shorthand syntax AsciiDoc provides is only meant as a convenience.

////
Add a sidebar describing the history and concerns with smart quotes regarding copy and paste and correct Unicode output.
////
"#
);
