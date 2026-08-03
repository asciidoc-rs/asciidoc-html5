//! Coverage of the AsciiDoc language description's *Post Replacement
//! Substitutions* page.
//!
//! The post replacements step replaces the line break character. This crate
//! renders a trailing `+` line break as `<br>` through `convert`, so the
//! behavioral claim, the observable rows of the applicability table (probed
//! block by block), and the `post_replacements`/`p` substitution values are
//! verified. The single-line rows (attribute entry values, headers) can't carry
//! a line break so aren't observable, `Tables` is "Varies", and `Titles` is a
//! known renderer gap (issue #271); those rows are tracked as non-normative.

use super::applicability::ran;
use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/post-replacements.adoc");

non_normative!(
    r#"
= Post Replacement Substitutions
:navtitle: Post Replacements
:table-caption: Table
:y: Yes
//icon:check[role="green"]
:n: No
//icon:times[role="red"]

"#
);

// A line whose content ends with a space and a plus is a line break; the post
// replacements step renders it as `<br>`.
#[test]
fn post_replacements_line_break() {
    verifies!(
        r#"
The line break character, `{plus}`, is replaced when the `post_replacements` substitution step runs.

"#
    );

    let out = convert("first +\nsecond\n");
    assert!(out.contains("first<br>\nsecond"));
}

// The lead-in and table header are descriptive. The `Attribute entry values |
// {n}` cell isn't observable via `convert`: a stored attribute value is a
// single line and so can't carry the end-of-line `+` a line break requires.
non_normative!(
    r#"
== Default post replacements substitution

<<table-post>> lists the specific blocks and inline elements the post replacements substitution step applies to automatically.

.Blocks and inline elements subject to the post replacements substitution
[#table-post%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |{n}

"#
);

// The applicability table, probed with a trailing-`+` line break `a +\nb`: the
// post replacements step is "applied" iff the break becomes `<br>`.
#[test]
fn default_post_replacements_substitution() {
    verifies!(
        r#"
|Comments |{n}

|Examples |{y}

"#
    );

    let applies = |body: &str| ran("", body, "<br>");

    // Comments (no): the comment block produces no output.
    assert!(!applies("////\na +\nb\n////\n\nsentinel\n"));

    // Examples (yes).
    assert!(applies("====\na +\nb\n====\n"));
}

// The `Headers | {n}` cell isn't observable via `convert`: the author/revision
// metadata is a single line and can't carry the end-of-line `+` a line break
// requires.
non_normative!(
    r#"
|Headers |{n}

"#
);

// The applicability table, continued: rows from Literal through Sidebars.
#[test]
fn default_post_replacements_body_blocks() {
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

    let applies = |body: &str| ran("", body, "<br>");

    // Literal, listings, and source (no).
    assert!(!applies("....\na +\nb\n....\n"));
    assert!(!applies("----\na +\nb\n----\n"));
    assert!(!applies("[source]\n----\na +\nb\n----\n"));

    // Macros (yes): a line break in a macro's text is applied.
    assert!(applies("link:https://example.org[a +\nb]\n"));

    // Open (yes).
    assert!(applies("--\na +\nb\n--\n"));

    // Paragraphs (yes).
    assert!(applies("a +\nb\n"));

    // Passthrough blocks (no): the raw content passes straight through.
    assert!(!applies("++++\na +\nb\n++++\n"));

    // Quotes and verses (yes).
    assert!(applies("[quote]\n____\na +\nb\n____\n"));
    assert!(applies("[verse]\n____\na +\nb\n____\n"));

    // Sidebars (yes).
    assert!(applies("****\na +\nb\n****\n"));
}

// The remaining rows aren't verified here: `Tables` is "Varies" (it depends on
// each cell's format specifier), and `Titles | {y}` is a renderer gap — the
// oracle applies the line break to a block title but this crate does not (see
// asciidoc-html5 issue #271), so asserting it would encode that divergence.
non_normative!(
    r#"
|Tables |Varies

|Titles |{y}
|===

== post_replacements substitution value

"#
);

// For blocks, the step's name `post_replacements` can be assigned to `subs`;
// for inline text, the built-in values `p` or `post_replacements` add the step
// through the inline pass macro.
#[test]
fn post_replacements_value() {
    verifies!(
        r#"
The post replacements substitution step can be modified on blocks and inline elements.
For blocks, the step's name, `post_replacements`, can be assigned to the xref:apply-subs-to-blocks.adoc[subs attribute].
For inline elements, the built-in values `p` or `post_replacements` can be applied to xref:apply-subs-to-text.adoc[inline text] to add the post replacements substitution step.
"#
    );

    // The inline `p` shorthand on the pass macro adds the post replacements step
    // to the enclosed text, so the trailing `+` becomes a line break.
    let inline = convert("pass:p[a +\nb]\n");
    assert!(inline.contains("a<br>\nb"));

    // The block name `post_replacements` added to a verbatim block's default
    // group renders the line break in a literal block.
    let block = convert("[subs=\"verbatim,+post_replacements\"]\n....\na +\nb\n....\n");
    assert!(block.contains("a<br>\nb"));
}
