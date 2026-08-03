//! Coverage of the AsciiDoc language description's *Special Character
//! Substitutions* page.
//!
//! The special characters step replaces `<`, `>`, and `&` with their named
//! character references. This crate performs that replacement through
//! `convert`, so the three replacement rules, the applicability table (probed
//! block by block), and the `specialchars`/`c` substitution values are all
//! verified. Only the CLI/API escaping note is descriptive, so it is tracked as
//! non-normative.

use super::applicability::{ran, ran_in_header};
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

"#
);

// Each row of the applicability table, probed with `<x>`: the special
// characters step is "applied" iff the output encodes it as `&lt;x&gt;` rather
// than leaving `<x>` raw.
#[test]
fn default_specialchars_substitution() {
    verifies!(
        r#"
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

"#
    );

    // `<x>` is encoded to `&lt;x&gt;` when the special characters step runs.
    let applies = |body: &str| ran("", body, "&lt;");

    // Attribute entry values (yes): the value is encoded when stored. Isolated
    // with `[subs=attributes]` (which does not itself encode) so only the
    // attribute-entry substitution can produce the entity.
    assert!(convert(":v: <x>\n[subs=attributes]\n....\n{v}\n....\n").contains("&lt;x&gt;"));

    // Comments (no): the comment block produces no output.
    assert!(!applies("////\n<x>\n////\n\nsentinel\n"));

    // Examples (yes).
    assert!(applies("====\n<x>\n====\n"));

    // Headers (yes): special characters in the author line are encoded.
    assert!(ran_in_header("", "<x>", "&lt;"));

    // Literal, listings, and source (yes — the verbatim group encodes them).
    assert!(applies("....\n<x>\n....\n"));
    assert!(applies("----\n<x>\n----\n"));
    assert!(applies("[source]\n----\n<x>\n----\n"));

    // Macros (yes): special characters in a macro's text are encoded.
    assert!(applies("link:https://example.org[<x>]\n"));

    // Open (yes).
    assert!(applies("--\n<x>\n--\n"));

    // Paragraphs (yes).
    assert!(applies("<x>\n"));

    // Passthrough blocks (no): the raw `<x>` passes straight through.
    assert!(!applies("++++\n<x>\n++++\n"));

    // Quotes and verses (yes).
    assert!(applies("[quote]\n____\n<x>\n____\n"));
    assert!(applies("[verse]\n____\n<x>\n____\n"));

    // Sidebars (yes).
    assert!(applies("****\n<x>\n****\n"));

    // Tables (yes): a default table cell encodes special characters.
    assert!(applies("[cols=1]\n|===\n|<x>\n|===\n"));

    // Titles (yes): special characters in a block title are encoded.
    assert!(applies(".<x>\n====\nbody\n====\n"));
}

non_normative!(
    r#"
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
