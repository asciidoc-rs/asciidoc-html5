//! Coverage of the AsciiDoc language description's *Post Replacement
//! Substitutions* page.
//!
//! The post replacements step replaces the line break character. This crate
//! renders a trailing `+` line break as `<br>` through `convert`, so the
//! behavioral claim and the `post_replacements`/`p` substitution values are
//! verified. The applicability table is descriptive and tracked as
//! non-normative.

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

non_normative!(
    r#"
== Default post replacements substitution

<<table-post>> lists the specific blocks and inline elements the post replacements substitution step applies to automatically.

.Blocks and inline elements subject to the post replacements substitution
[#table-post%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |{n}

|Comments |{n}

|Examples |{y}

|Headers |{n}

|Literal, listings, and source |{n}

|Macros |{y} +
(except passthrough macros)

|Open |{y}

|Paragraphs |{y}

|Passthrough blocks |{n}

|Quotes and verses |{y}

|Sidebars |{y}

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
