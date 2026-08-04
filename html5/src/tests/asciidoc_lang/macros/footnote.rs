//! Coverage of the AsciiDoc language description's *Footnotes* page.
//!
//! The `footnote:[…]` / `footnote:id[text]` macro is resolved by
//! `asciidoc-parser` and rendered by this crate exactly as Asciidoctor 2.0.26
//! does: an inline `<sup class="footnote">` marker at the reference (or
//! `<sup class="footnoteref">` for a reuse), plus a `#footnotes` definition
//! list at the end of the document. Both are verified through `convert`.
//!
//! The page draws its examples with `include::example$footnote.adoc[tag=…]`;
//! the tests reproduce those tagged snippets inline (without the source-block
//! callout annotations).

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/footnote.adoc");

non_normative!(
    r#"
= Footnotes

AsciiDoc provides the `footnote` macro for adding footnotes to your document.
A footnote is a reference to an item in a footnote list.
The footnote is defined in AsciiDoc at the location of the reference, but the text is extracted to an item in the footnote list.
You can refer to the same footnote in multiple locations by assigning an ID to the first occurrence and referencing that ID in subsequent occurrences.

NOTE: All AsciiDoc processors, including Asciidoctor, currently implement footnotes as endnotes.
The placement and numbering of footnotes can be customized using a custom converter.

== Footnote macro syntax

"#
);

// The footnote macro leaves an inline marker and extracts its text to the
// document's footnote list. An ID lets the same footnote be reused with empty
// text; a later definition reusing an existing ID has its text ignored.
#[test]
fn macro_syntax() {
    verifies!(
        r#"
You can insert footnotes into your document using the footnote macro.
The text of the footnote is defined between the square brackets of the footnote macro (`+footnote:[text]+`).
The footnote macro accepts an optional ID using the target of the macro (`+footnote:id[text]+`).
Specifying an ID allows you to refer to that same footnote from multiple locations in the document.
To make a reference to a previously defined footnote, you specify the ID in the target without specifying text (`+footnote:id[]+`).

"#
    );

    // The text between the brackets defines the footnote: an inline marker is
    // left in the flow, and the text is emitted in the `#footnotes` block.
    let doc = convert("The footnote.footnote:[The text.]");
    assert!(doc.contains(
        r##"The footnote.<sup class="footnote">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup>"##
    ));
    assert_css(&doc, "sup.footnote", 1);
    assert!(doc.contains(r##"<a href="#_footnoteref_1">1</a>. The text."##));

    // An ID in the target lets the same footnote be referenced again with empty
    // text (`footnote:id[]`); the reuse renders a `footnoteref` marker and adds
    // no new definition.
    let doc = convert("First.footnote:fn1[Shared text.] Again.footnote:fn1[]");
    assert_css(&doc, "sup.footnote", 1);
    assert_css(&doc, "sup.footnoteref", 1);
    assert_css(&doc, "div.footnote", 1);
    assert!(doc.contains(r##"<a href="#_footnoteref_1">1</a>. Shared text."##));

    // If both an ID and text are given but the ID was already defined, the later
    // text is ignored: the original text is kept.
    let doc = convert("First.footnote:fn1[Original.] Again.footnote:fn1[Ignored.]");
    assert_css(&doc, "div.footnote", 1);
    assert!(doc.contains("Original."));
    assert!(!doc.contains("Ignored."));
}

// The full syntax example (the `base-c` snippet): an anonymous footnote after a
// word, a footnote assigning a reusable ID, and a reference to it with empty
// text.
#[test]
fn footnote_syntax() {
    verifies!(
        r#"
.Footnote syntax
[source#ex-footnote]
----
include::example$footnote.adoc[tag=base-c]
----
<.> Insert the footnote macro directly after any punctuation.
Note that the footnote macro only uses a single colon (`:`).
<.> Insert the footnote's content within the square brackets (`+[]+`).
The text may span several lines.
<.> If you plan to reuse a footnote, specify a unique ID in the target position.
<.> To reference an existing footnote, you only need to specify the ID of the footnote in the target slot.
The text between the square brackets should be empty.
If both the ID and text are specified, and the ID has already been defined by an earlier footnote, the text is ignored.

"#
    );

    // The `base-c` region, rendered without the source-block callout markers.
    let doc = convert(
        "The hail-and-rainbow protocol can be initiated at five levels:\n\n\
         . doublefootnote:[The double hail-and-rainbow level makes my toes tingle.]\n\
         . tertiary\n. supernumerary\n. supermassive\n. apocalyptic\n\n\
         A bold statement!footnote:disclaimer[Opinions are my own.]\n\n\
         Another outrageous statement.footnote:disclaimer[]",
    );

    // Two footnotes are defined; the third occurrence only references the second.
    assert_css(&doc, "div.footnote", 2);
    assert_css(&doc, "sup.footnote", 2);
    assert_css(&doc, "sup.footnoteref", 1);
    assert!(doc.contains(
        r##"<a href="#_footnoteref_1">1</a>. The double hail-and-rainbow level makes my toes tingle."##
    ));
    assert!(doc.contains(r##"<a href="#_footnoteref_2">2</a>. Opinions are my own."##));
}

// An `{empty}` reference separates the macro from the preceding word in the
// source without emitting anything, so the marker still renders adjacent to it.
#[test]
fn empty_attribute_reference_separates_word_from_macro() {
    verifies!(
        r#"
TIP: If you find that having to put the footnote macro directly adjacent to a word makes it difficult to read, you can insert an attribute reference in between that resolves to an empty string (e.g., `+word{empty}footnote:[text]+`).

"#
    );

    let doc = convert("A word{empty}footnote:[The footnote text.] follows.");
    assert!(doc.contains(
        r##"A word<sup class="footnote">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup> follows."##
    ));
    assert!(doc.contains(r##"<a href="#_footnoteref_1">1</a>. The footnote text."##));
}

// Footnotes are numbered consecutively in document order.
#[test]
fn numbered_consecutively() {
    verifies!(
        r#"
The footnotes are numbered consecutively throughout the article.

"#
    );

    let doc = convert("One.footnote:[First.] Two.footnote:[Second.] Three.footnote:[Third.]");
    assert_css(&doc, "div.footnote", 3);
    assert!(doc.contains(r##"id="_footnotedef_1">"##));
    assert!(doc.contains(r##"id="_footnotedef_2">"##));
    assert!(doc.contains(r##"id="_footnotedef_3">"##));
}

non_normative!(
    r#"
The results of <<ex-footnote>> are displayed below.

[.unstyled]
|===
a|
include::example$footnote.adoc[tag=base-x]
|===

Just like normal paragraph text, you can use text formatting markup in the text of the footnote.

== Externalizing a footnote

"#
);

// A footnote can be externalized to a document attribute and inserted via an
// attribute reference, because attribute references are expanded before
// footnotes are parsed.
#[test]
fn externalized_footnote() {
    verifies!(
        r#"
Since footnotes are defined using an inline macro, the footnote content must be inserted alongside the text it's annotating.
This requirement can make the text harder to read.
You can solve this problem by externalizing your footnotes using document attributes.

When defining a document attribute that holds a footnote, you can name the document attributes whatever you want.
A common practice is to name the attribute using the `fn-` prefix.
The name of the attribute can be as verbose (`fn-disclaimer`) or concise (`fn-1`) as you prefer.

Here's the previous example with the footnotes defined in document attributes and inserted using attribute references.

.Externalized footnote
[source]
----
include::example$footnote.adoc[tag=externalized]
----

"#
    );

    // The `externalized` region: the footnote macros live in header attributes
    // and are inserted with plain attribute references, still producing footnotes.
    let doc = convert(
        ":fn-hail-and-rainbow: footnote:[The double hail-and-rainbow level makes my toes tingle.]\n\
         :fn-disclaimer: footnote:disclaimer[Opinions are my own.]\n\n\
         The hail-and-rainbow protocol can be initiated at five levels:\n\n\
         . double{fn-hail-and-rainbow}\n\
         . tertiary\n. supernumerary\n. supermassive\n. apocalyptic\n\n\
         A bold statement!{fn-disclaimer}\n\n\
         Another outrageous statement.{fn-disclaimer}",
    );

    assert_css(&doc, "div.footnote", 2);
    assert_css(&doc, "sup.footnote", 2);
    assert_css(&doc, "sup.footnoteref", 1);
    assert!(doc.contains(
        r##"<a href="#_footnoteref_1">1</a>. The double hail-and-rainbow level makes my toes tingle."##
    ));
    assert!(doc.contains(r##"<a href="#_footnoteref_2">2</a>. Opinions are my own."##));
}

non_normative!(
    r#"
Notice you still get the benefit of seeing where the footnote is placed without all the noise.
And since the footnotes are now defined in the document header, they could be further externalized to an include file.

"#
);

// To honor text formatting in an externalized footnote, wrap the attribute
// value in `pass:c,q[…]` so the special-characters and quotes substitutions are
// applied up front.
#[test]
fn externalized_footnote_with_text_formatting() {
    verifies!(
        r#"
This approach works since attribute references are expanded before footnotes are parsed.
However, this technique does not work if you have text formatting markup in the text of the footnote (e.g., `+*bold*+`).
That markup will not be interpreted.
That's because the attributes substitution (which replaces attribute references) is applied _after_ the quotes substitution (which interprets text formatting markup).
In order to use text formatting markup in the text of the footnote, you need to configure the substitutions on the value of the attribute entry using the `\pass:[]` macro.

The following example demonstrates how to configure the substitutions applied to the text of an externalized footnote so that text formatting markup is honored.

.Externalized footnote with text formatting
[source]
----
include::example$footnote.adoc[tag=externalized-format]
----

The `c,q` target on the pass macro instructs the processor to apply the special characters substitution followed by the quotes substitution.
That means the text formatting in the footnote text will already be applied when the footnote is inserted using an attribute reference.

"#
    );

    // The `externalized-format` region: `pass:c,q[…]` applies the quotes
    // substitution to the footnote text up front, so `_mine_` and `*alone*`
    // render as `<em>`/`<strong>` even though the value is inserted via an
    // attribute reference.
    let doc = convert(
        ":fn-disclaimer: pass:c,q[footnote:disclaimer[Opinions are _mine_, and mine *alone*.]]\n\n\
         A bold statement!{fn-disclaimer}\n\n\
         Another outrageous statement.{fn-disclaimer}",
    );

    assert_css(&doc, "div.footnote", 1);
    assert!(doc.contains(
        r##"<a href="#_footnoteref_1">1</a>. Opinions are <em>mine</em>, and mine <strong>alone</strong>."##
    ));
}

// This crate deliberately diverges from Asciidoctor for footnotes in headings
// (see the non-normative note below and asciidoc-parser#594): rather than
// converting section titles eagerly and out of document order, it applies a
// title's substitutions before parsing the section body, so a heading's
// footnote is numbered in straightforward document order.
#[test]
fn footnotes_in_headings_are_numbered_in_document_order() {
    let doc = convert(concat!(
        "== Section 1\n\n",
        "para.footnote:[first footnote]\n\n",
        "== Section 2footnote:[second footnote]\n\n",
        "para.footnote:[third footnote]\n",
    ));

    // The heading's footnote (number 2) is registered between the two body
    // footnotes — document order — rather than deferred or numbered eagerly.
    assert_css(&doc, "div.footnote", 3);
    assert!(doc.contains(r##"<a href="#_footnoteref_1">1</a>. first footnote"##));
    assert!(doc.contains(r##"<a href="#_footnoteref_2">2</a>. second footnote"##));
    assert!(doc.contains(r##"<a href="#_footnoteref_3">3</a>. third footnote"##));
}

// The "Footnotes in headings" section is non-normative: it documents
// Asciidoctor's out-of-order default and an explicit-ID-plus-reftext
// workaround, neither of which this crate models. This crate always numbers
// footnotes in heading in document order (verified by
// `footnotes_in_headings_are_numbered_in_document_order` above), achieving
// unconditionally what the prose says Asciidoctor achieves only via the
// workaround. Tracked by asciidoc-parser#594.
non_normative!(
    r#"
== Footnotes in headings

Footnotes are *not officially supported in headings* (section titles and discrete headings) in pre-spec AsciiDoc.
While the footnote gets parsed, there's no guarantee that it will work properly and may require workarounds.
This limitation may be lifted once the AsciiDoc Language is defined by the specification.

If you use a footnote in a heading, you'll likely find that the footnote index is wrong (either not incremented or out of order).
That's because headings (section titles and discrete headings) get converted out of document order for the purpose of generating IDs, populating up cross references, and eagerly resolving attribute references.

The only way to workaround this limitation is by assigning an explicit ID *and* reftext to any heading that contains a footnote.
For example:

[source]
----
See <<heading>>.

[[heading,Heading]]
== Headingfootnote:[This is a heading with a footnote]
----

Assigning an explicit ID and reftext to a heading will prevent the heading from being converted eagerly (thus deferring the footnote substitution) until the heading is rendered.
As a result, the footnote macro in the heading will be processed in document order.

This workaround will also prevent the footnote number from reappearing in the text of an xref.

Even with this workaround, you still have to avoid using attribute references in the heading as those also causes the heading to be converted eagerly (which forces substitutions to be applied).
If you use an attribute reference in the heading, the footnotes will be processed out of document order.
"#
);
