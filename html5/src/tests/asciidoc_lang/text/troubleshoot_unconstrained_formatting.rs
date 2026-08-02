//! Coverage of the AsciiDoc language description's *Troubleshoot Unconstrained
//! Formatting Pairs* page.
//!
//! This page catalogs the situations that require an unconstrained pair and the
//! tricks for escaping unwanted formatting. The edge cases this crate can drive
//! through `convert` are verified: a monospace phrase inside curved quotation
//! marks (three backtick pairs), a monospace phrase inside typewriter quotes, a
//! possessive monospace phrase, and escaping unconstrained marks with an
//! attribute reference or an inline plus macro. The introduction, the
//! decision-question list, the reference table, and the surrounding
//! troubleshooting prose are tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/text/pages/troubleshoot-unconstrained-formatting.adoc");

non_normative!(
    r#"
= Troubleshoot Unconstrained Formatting Pairs

An xref:index.adoc#unconstrained[unconstrained formatting pair] is often used to format just one or a few characters in a word.

[#use-unconstrained]
== When should I use unconstrained formatting?

Consider the following questions:

. Is there a letter, number, or underscore directly outside the opening or closing formatting marks?
. Is there a colon, semicolon, or closing curly bracket directly before the opening formatting mark?
. Is there a space directly inside of a formatting mark?
. Are you writing in a CJK langauge such as Chinese?

If you answered "`yes`" to any of these questions, you need to use an unconstrained pair.

To help you determine whether a particular syntax pattern requires an unconstrained pair versus a xref:index.adoc#constrained[constrained pair], consider the following scenarios:

.Constrained or Unconstrained?
[#constrained-or-unconstrained]
|===
|AsciiDoc |Result |Formatting Pair |Reason

|`+Sara__h__+`
|Sara__h__
|Unconstrained
|The letter `a` is directly adjacent to the opening mark.

|`+**B**old+`
|**B**old
|Unconstrained
|The `o` is directly adjacent to the closing mark.

|`+&ndash;**2016**+`
|&#8211;**2016**
|Unconstrained
|The `;` is directly adjacent to the opening mark.

|`+** bold **+`
|** bold **
|Unconstrained
|There are spaces directly inside the formatting marks.

|`+我喜欢**大**狗+`
|我喜欢**大**狗
|Unconstrained
|There are CJK characters directly outside the formatting marks.

|`+*2016*&ndash;+`
|*2016*&#8211;
|Constrained
|The adjacent `&` is not a letter, number, underscore, colon, or semicolon.

|`+*9*-to-*5*+`
|*9*-to-*5*
|Constrained
|The adjacent hyphen is not a letter, number, underscore, colon, or semicolon.
|===

[#unconstrained-edge-cases]
=== Unconstrained pair edge cases

"#
);

// Backticks first serve to curve the quotation marks, so a monospace phrase
// inside curved quotes needs three backtick pairs: an unconstrained monospace
// pair and a constrained curved-quote pair.
#[test]
fn monospace_phrase_inside_curved_quotes() {
    verifies!(
        r#"
There are cases when it might seem logical to use a constrained pair, but an unconstrained pair is required.
xref:subs:index.adoc[Substitutions] may be applied by the parser before getting to the formatting marks, in which case the characters adjacent to those marks may not be what you see in the original source.

One such example is enclosing a xref:monospace.adoc[monospace phrase] inside xref:quotation-marks-and-apostrophes.adoc[curved quotation marks], such as "```end points```".

You might start with the following syntax:

----
"`end points`"
----

That only gives you "`end points`".
The backticks contribute to making the curved quotation marks, but the word isn't rendered in monospace.

Adding another pair of backticks isn't enough either.

----
"``end points``"
----

The parser ignores the inner pair of backticks and interprets them as literal characters, rendering the phrase as "``end points``".

You have to use an unconstrained pair of monospace formatting marks to render the phrase in monospace and a constrained pair of backticks to render the quotation marks as curved.
That's three pairs of backticks in total.

.A monospace phrase inside curved quotation marks
----
"```end points```"
----

"#
    );

    // A single inner backtick pair only curves the quotes; no monospace.
    let one = convert("\"`end points`\"\n");
    assert!(one.contains("&#8220;end points&#8221;"));

    // A double inner pair is treated as literal backticks.
    let two = convert("\"``end points``\"\n");
    assert!(two.contains("&#8220;`end points`&#8221;"));

    // Three pairs render monospace inside curved quotation marks.
    let three = convert("\"```end points```\"\n");
    assert!(three.contains("&#8220;<code>end points</code>&#8221;"));
}

// To keep typewriter (straight) quotes around a monospace phrase, apply a role
// to the phrase or escape the leading quote so the backticks do not curve it.
#[test]
fn monospace_phrase_inside_typewriter_quotes() {
    verifies!(
        r#"
If, instead, you wanted to surround the monospace phrase with typewriter quotation marks, such as "[.code]``end points``", then you need to interrupt the curved quotation marks by applying a role to the monospace phrase or escaping the typewriter quote.
For example:

.A monospace phrase inside typewriter quotation marks
----
"[.code]``end points``" or \"``end points``"
----

"#
    );

    let out = convert("\"[.code]``end points``\" or \\\"``end points``\"\n");
    assert!(out.contains(r#"<code class="code">end points</code>"#));
    assert!(out.contains(r#"or "<code>end points</code>""#));
}

// A possessive monospace phrase ending in an apostrophe must use an
// unconstrained monospace pair, placing the curved apostrophe outside `<code>`.
#[test]
fn possessive_monospace_phrase() {
    verifies!(
        r#"
Another example is a possessive, monospace phrase that ends in an "`s`".
In this case, you must switch the monospace phrase to unconstrained formatting.

----
The ``class```' static methods make it easy to operate
on files and directories.
----

.Rendered possessive, monospace phrase
====
The ``class```' static methods make it easy to operate on files and directories.
====

"#
    );

    let out = convert(
        "The ``class```' static methods make it easy to operate on files and directories.\n",
    );
    assert!(out.contains("The <code>class</code>&#8217; static methods"));
}

// Encoding the curved apostrophe directly after a constrained monospace pair
// yields the same result without unconstrained marks.
#[test]
fn possessive_monospace_with_encoded_apostrophe() {
    verifies!(
        r#"
Alternately, you could encode the curved apostrophe directly in the AsciiDoc source to get the same result.

----
The `class`’ static methods make it easy to operate on files and directories.
----

"#
    );

    let out = convert(
        "The `class`\u{2019} static methods make it easy to operate on files and directories.\n",
    );
    assert!(out.contains("The <code>class</code>\u{2019} static methods"));
}

non_normative!(
    r#"
This situation is expected to improve in the future when the AsciiDoc language switches to using a parsing expression grammar for inline formatting instead of the current regular expression-based strategy.
For details, follow https://github.com/asciidoctor/asciidoctor/issues/61[Asciidoctor issue #61].

[#escape-unconstrained]
== Escape unconstrained formatting marks

Since unconstrained formatting marks are meant to match anywhere in the text, context free, that means you may catch them formatting text that you don't want styled sometimes.
Admittedly, these symbols are a bit tricky to type literally when the content calls for it.
But being able to do so is just a matter of knowing the tricks, which this section will cover.

Let's assume you are typing the following two lines:

----
The __kernel qualifier can be used with the __attribute__ keyword...

#`CB###2`# and #`CB###3`#
----

In the first sentence, you aren't looking for any text formatting, but you're certainly going to get it.
The processor will interpret the double underscore in front of _++__kernel++_ as an unconstrained formatting mark.
In the second sentence, you might expect _++CB###2++_ and _++CB###3++_ to be highlighted and displayed using a monospace font.
However, what you get is a scrambled mess.
The mix of constrained and unconstrained formatting marks in the line is ambiguous.

There are two reliable solutions for escaping unconstrained formatting marks:

* use an attribute reference to insert the unconstrained formatting mark verbatim, or
* wrap the text you don't want formatted in an inline passthrough.

"#
);

// Assigning the unconstrained mark to an attribute inserts it verbatim, because
// attribute expansion runs after quotes substitution: the marks are no longer
// interpreted as formatting.
#[test]
fn escape_unconstrained_marks_with_attribute_reference() {
    verifies!(
        r#"
The attribute reference is preferred because it's the easiest to read:

----
:scores: __
:hash3: ###

The {scores}kernel qualifier can be used with the {scores}attribute{scores} keyword...

#`CB{hash3}2`# and #`CB{hash3}3`#
----

This works because xref:subs:attributes.adoc[attribute expansion] is performed after text formatting (i.e., xref:subs:quotes.adoc[quotes substitution]) in the normal substitution order.

"#
    );

    let out = convert(
        ":scores: __\n:hash3: ###\n\nThe {scores}kernel qualifier can be used with the {scores}attribute{scores} keyword...\n\n#`CB{hash3}2`# and #`CB{hash3}3`#\n",
    );
    // The `__` marks are inserted verbatim, not interpreted as emphasis.
    assert!(out.contains("The __kernel qualifier can be used with the __attribute__ keyword"));
    assert!(out.contains("<mark><code>CB###2</code></mark> and <mark><code>CB###3</code></mark>"));
}

// Wrapping the text in an inline single-plus passthrough escapes the
// unconstrained marks from interpolation while still applying HTML output
// escaping.
#[test]
fn escape_unconstrained_marks_with_plus_macro() {
    verifies!(
        r#"
Here's how you'd write these lines using the xref:pass:pass-macro.adoc[inline single plus macro] to escape the unconstrained formatting marks instead:

----
The +__kernel+ qualifier can be used with the +__attribute__+ keyword...

#`+CB###2+`# and #`+CB###3+`#
----

Notice the addition of the plus symbols.
Everything between the plus symbols is escaped from interpolation (attribute references, text formatting, etc.).
However, the text still receives proper output escaping for xref:subs:special-characters.adoc[HTML special characters] (e.g., `<` becomes `\&lt;`).

The enclosure `pass:[`+TEXT+`]` (text enclosed in pluses surrounded by backticks) is a special formatting combination in AsciiDoc.
It means to format TEXT as monospace, but don't interpolate formatting marks or attribute references in TEXT.
It's roughly equivalent to Markdown's backticks.
Since AsciiDoc offers more advanced formatting, the double enclosure is necessary.
"#
    );

    let out = convert(
        "The +__kernel+ qualifier can be used with the +__attribute__+ keyword...\n\n#`+CB###2+`# and #`+CB###3+`#\n",
    );
    assert!(out.contains("The __kernel qualifier can be used with the __attribute__ keyword"));
    assert!(out.contains("<mark><code>CB###2</code></mark> and <mark><code>CB###3</code></mark>"));
}
