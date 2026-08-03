//! Coverage of the AsciiDoc language description's *Character Replacement
//! Substitutions* page.
//!
//! The replacements step turns textual symbols (marks, arrows, dashes) into
//! their Unicode code points and passes character references through to the
//! output. This crate performs both through `convert`, so the symbol
//! replacements, the character-reference handling, and the
//! `replacements`/`r` substitution values are verified. The "Anatomy of a
//! character reference" sidebar is definitional, and the applicability table
//! and advisory notes are descriptive, so they are tracked as non-normative.

use super::applicability::ran;
use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/replacements.adoc");

non_normative!(
    r#"
= Character Replacement Substitutions
:navtitle: Character Replacements
:table-caption: Table
:y: Yes
//icon:check[role="green"]
:n: No
//icon:times[role="red"]
:url-html-ref: https://en.wikipedia.org/wiki/List_of_XML_and_HTML_character_entity_references
:url-unicode: https://en.wikipedia.org/wiki/List_of_Unicode_characters

"#
);

// The replacements step turns each textual symbol the partial tabulates into
// its Unicode code point: (C) (R) (TM) become the copyright/registered/
// trademark signs, `--` an em dash flanked by thin spaces, `...` an ellipsis,
// the arrow ligatures their arrows, and a typewriter apostrophe the curly one.
#[test]
fn replacements_symbols() {
    verifies!(
        r#"
The character replacement substitution step processes textual characters such as marks, arrows and dashes and replaces them with the decimal format of their Unicode code point, i.e., their <<char-ref-sidebar,numeric character reference>>.
The replacements step depends on the substitutions completed by the xref:special-characters.adoc[special characters step].

// Table of Textual symbol replacements is inserted below
include::partial$subs-symbol-repl.adoc[]

"#
    );

    let out = convert("(C) (R) (TM) a -- b and ... and -> => <- <= Sam's\n");
    assert!(out.contains(
        "&#169; &#174; &#8482; a&#8201;&#8212;&#8201;b and &#8230;&#8203; \
         and &#8594; &#8658; &#8592; &#8656; Sam&#8217;s"
    ));
}

// A character reference in the source — named (`&sect;`), hexadecimal
// (`&#x00A7;`), or decimal (`&#167;`) — is preserved through to the output, so
// the browser renders the referenced glyph.
#[test]
fn replacements_char_refs() {
    verifies!(
        r#"
This substitution step also recognizes {url-html-ref}[HTML and XML character references^] as well as {url-unicode}[decimal and hexadecimal Unicode code points^] and substitutes them for their corresponding decimal form Unicode code point.

For example, to produce the `&#167;` symbol you could write `\&sect;`, `\&#x00A7;`, or `\&#167;`.
When the document is processed, `replacements` will preserve the section symbol reference so the reference will appear as &#167; when rendered.
In turn, `\&#167;` in the AsciiDoc source will display as &#167; in the rendered document.

"#
    );

    // Each of the three forms is emitted verbatim (not escaped into
    // `&amp;…`), so each renders as the section symbol.
    assert!(convert("&sect;\n").contains("&sect;"));
    assert!(convert("&#x00A7;\n").contains("&#x00A7;"));
    assert!(convert("&#167;\n").contains("&#167;"));
}

non_normative!(
    r#"
An AsciiDoc processor allows you to use any of the named character references (aka named entities) defined in HTML (e.g., \&euro; resolves to &#8364;).
However, using named character references can cause problems when generating non-HTML output such as PDF because the lookup table needed to resolve these names may not be defined.
The recommendation is avoid using named character references, with the exception of the well-known ones defined in XML (i.e., lt, gt, amp, quot, apos).
Instead, use numeric character references (e.g., \&#8364;).

[#char-ref-sidebar]
.Anatomy of a character reference
****
A character reference is a standard sequence of characters that is substituted for a single character by an AsciiDoc processor.
There are two types of character references: named character references and numeric character references.

A named character reference (often called a _character entity reference_) is a short name that refers to a character (i.e., glyph).
To make the reference, the name must be prefixed with an ampersand (`&`) and end with a semicolon (`;`).

For example:

* `\&dagger;` displays as &#8224;
* `\&euro;` displays as &#8364;
* `\&loz;` displays as &#9674;

Numeric character references are the decimal or hexadecimal Universal Character Set/Unicode code points which refer to a character.

* The decimal code point references are prefixed with an ampersand (`&`), followed by a hash (`&#35;`), and end with a semicolon (`;`).
* Hexadecimal code point references are prefixed with an ampersand (`&`), followed by a hash (`&#35;`), followed by a lowercase `x`, and end with a semicolon (`;`).

For example:

* `\&#x2020;` or `\&#8224;` displays as &#8224;
* `\&#x20AC;` or `\&#8364;` displays as &#8364;
* `\&#x25CA;` or `\&#9674;` displays as &#x25CA;

Developers may be more familiar with using *Unicode escape sequences* to perform text substitutions.
For example, to produce an `&#64;` sign using a Unicode escape sequence, you would prefix the hexadecimal Unicode code point with a backslash (`\`) and an uppercase or lowercase `u`, i.e. `u0040`.
However, the AsciiDoc syntax doesn't recognize Unicode escape sequences at this time.
****

TIP: AsciiDoc also provides built-in attributes for representing some common symbols.
These attributes and their corresponding output are listed in xref:attributes:character-replacement-ref.adoc[].

== Default replacements substitution

"#
);

// The applicability table, probed with `(C)`: the replacements step is
// "applied" iff the copyright ligature becomes `&#169;`. (Rows above the
// Headers row.)
#[test]
fn default_replacements_substitution() {
    verifies!(
        r#"
<<table-replace>> lists the specific blocks and inline elements the replacements substitution step applies to automatically.

.Blocks and inline elements subject to the replacements substitution
[#table-replace%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |{n}

|Comments |{n}

|Examples |{y}

"#
    );

    // `(C)` becomes `&#169;` when the replacements step runs.
    let applies = |body: &str| ran("", body, "&#169;");

    // Attribute entry values (no): the stored value keeps `(C)` literal.
    // Isolated with `[subs=attributes]` so the block adds no replacements.
    assert!(!convert(":v: (C)\n[subs=attributes]\n....\n{v}\n....\n").contains("&#169;"));

    // Comments (no): the comment block produces no output.
    assert!(!applies("////\n(C)\n////\n\nsentinel\n"));

    // Examples (yes).
    assert!(applies("====\n(C)\n====\n"));
}

// The `Headers | {n}` cell is a documentation/behavior divergence: the
// asciidoc-lang table says replacements are *not* applied to the header, and
// this crate currently follows that (the author line keeps `(C)` literal),
// whereas Asciidoctor 2.0.26 does apply replacements to the byline (rendering
// `&#169;`). Because the two disagree — and because matching Asciidoctor here
// awaits a public parser substitution API (asciidoc-parser #1077) — this cell
// is tracked as non-normative rather than asserted.
non_normative!(
    r#"
|Headers |{n}

"#
);

// The applicability table, continued: rows from Literal through Sidebars.
#[test]
fn default_replacements_substitution_body_blocks() {
    verifies!(
        r#"
|Literal, listings, and source |{n}

|Macros |{y} +
(except passthrough macros)

|Open |{y}

|Paragraphs |{y}

|Passthrough blocks |{n}

|Quotes and verses |{y}

|Sidebars |{y}

"#
    );

    let applies = |body: &str| ran("", body, "&#169;");

    // Literal, listings, and source (no).
    assert!(!applies("....\n(C)\n....\n"));
    assert!(!applies("----\n(C)\n----\n"));
    assert!(!applies("[source]\n----\n(C)\n----\n"));

    // Macros (yes): replacements run on a macro's text.
    assert!(applies("link:https://example.org[(C)]\n"));

    // Open (yes).
    assert!(applies("--\n(C)\n--\n"));

    // Paragraphs (yes).
    assert!(applies("(C)\n"));

    // Passthrough blocks (no): the raw `(C)` passes straight through.
    assert!(!applies("++++\n(C)\n++++\n"));

    // Quotes and verses (yes).
    assert!(applies("[quote]\n____\n(C)\n____\n"));
    assert!(applies("[verse]\n____\n(C)\n____\n"));

    // Sidebars (yes).
    assert!(applies("****\n(C)\n****\n"));
}

// The table's `Tables` cell is "Varies" (it depends on each cell's format
// specifier), so it is not a single yes/no claim to verify here.
non_normative!(
    r#"
|Tables |Varies

"#
);

// Titles (yes): replacements run on a block title.
#[test]
fn default_replacements_substitution_titles() {
    verifies!(
        r#"
|Titles |{y}
|===

"#
    );

    assert!(convert(".(C)\n====\nbody\n====\n").contains("&#169;"));
}

non_normative!(
    r#"
== replacements substitution value

"#
);

// For blocks, the step's name `replacements` can be assigned to `subs`; for
// inline text, the built-in values `r` or `replacements` add the step through
// the inline pass macro.
#[test]
fn replacements_value() {
    verifies!(
        r#"
The replacements substitution step can be modified on blocks and inline elements.
For blocks, the step's name, `replacements`, can be assigned to the xref:apply-subs-to-blocks.adoc[subs attribute].
For inline elements, the built-in values `r` or `replacements` can be applied to xref:apply-subs-to-text.adoc[inline text] to add the replacements substitution step.

"#
    );

    // Adding `replacements` to a literal block's group replaces the symbols a
    // verbatim block would otherwise leave literal.
    let block = convert("[subs=\"specialchars,replacements\"]\n....\n(C) ...\n....\n");
    assert!(block.contains("&#169; &#8230;&#8203;"));

    // The inline `r` shorthand on the pass macro replaces the enclosed symbols.
    let inline = convert("pass:r[(C) ...]\n");
    assert!(inline.contains("&#169; &#8230;&#8203;"));
}

// Advisory reminder that `replacements` depends on the special characters step
// having run first; it restates the dependency rather than adding new behavior.
non_normative!(
    r#"
WARNING: The replacements step depends on the substitutions completed by the xref:special-characters.adoc[special characters step].
This is important to keep in mind when applying the `replacements` value to blocks and inline elements.
"#
);
