//! Coverage of the AsciiDoc language description's *Special Character
//! Substitutions* page.
//!
//! The special characters step replaces `<`, `>`, and `&` with their named
//! character references. This crate performs that replacement through
//! `convert`, so the three replacement rules and the `specialchars`/`c`
//! substitution values are verified. The applicability table and the CLI/API
//! escaping note are descriptive, so they are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/special-characters.adoc");

non_normative!(
    r#"
= Special Character Substitutions
:navtitle: Special Characters
:table-caption: Table
:y: Yes
:n: No

"#
);

// The special characters step replaces the three HTML-significant characters
// `<`, `>`, and `&` with their named character references `&lt;`, `&gt;`, and
// `&amp;`.
#[test]
fn special_characters_replaced() {
    verifies!(
        r#"
The special characters substitution step searches for three characters (`<`, `>`, `&`) and replaces them with their xref:replacements.adoc#char-ref-sidebar[named character references].

* The less than symbol, `<`, is replaced with the named character reference `\&lt;`.
* The greater than symbol, `>`, is replaced with the named character reference `\&gt;`.
* An ampersand, `&`, is replaced with the named character reference `\&amp;`.

"#
    );

    let out = convert("A <tag> & more\n");
    assert!(out.contains("A &lt;tag&gt; &amp; more"));
}

non_normative!(
    r#"
== Default special characters substitution

<<table-special>> lists the specific blocks and inline elements the special characters substitution step applies to automatically.

.Blocks and inline elements subject to the special characters substitution
[#table-special%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |{y}

|Comments |{n}

|Examples |{y}

|Headers |{y}

|Literal, listings, and source |{y}

|Macros |{y} +
(except triple plus and inline pass macros)

|Open |{y}

|Paragraphs |{y}

|Passthrough blocks |{n}

|Quotes and verses |{y}

|Sidebars |{y}

|Tables |{y}

|Titles |{y}
|===

== specialchars substitution value

"#
);

// For blocks, the step's name `specialchars` can be assigned to the `subs`
// attribute; for inline text, the built-in values `c` or `specialchars` add the
// special characters step through the inline pass macro.
#[test]
fn specialchars_value() {
    verifies!(
        r#"
The special characters substitution step can be modified on blocks and inline elements.
For blocks, the step's name, `specialchars`, can be assigned to the xref:apply-subs-to-blocks.adoc[subs attribute].
For inline elements, the built-in values `c` or `specialchars` can be applied to xref:apply-subs-to-text.adoc[inline text] to add the special characters substitution step.

"#
    );

    // The block name `specialchars` applied through `subs` encodes the special
    // characters in a literal block.
    let block = convert("[subs=\"specialchars\"]\n....\n<b> & </b>\n....\n");
    assert!(block.contains("&lt;b&gt; &amp; &lt;/b&gt;"));

    // The inline `c` shorthand on the pass macro adds the special characters
    // step to the enclosed text.
    let inline = convert("pass:c[<b> & </b>]\n");
    assert!(inline.contains("&lt;b&gt; &amp; &lt;/b&gt;"));
    assert_css(&inline, "b", 0);
}

// Advisory note about substitution ordering for attributes set via the CLI or
// API; it describes a manual-escaping workflow rather than rendering behavior
// this crate drives through `convert`.
non_normative!(
    r#"
[NOTE]
====
Special character substitution precedes attribute substitution, so you need to manually escape any attributes containing special characters that you set in the CLI or API.
For example, on the command line, type `+-a toc-title="Sections, Tables \&amp; Figures"+` instead of `-a toc-title="Sections, Tables & Figures"`.
====
"#
);
