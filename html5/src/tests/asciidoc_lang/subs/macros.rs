//! Coverage of the AsciiDoc language description's *Macro Substitutions* page.
//!
//! The macros step processes inline and block macros. This crate processes
//! macros through `convert`, so the behavioral claim and the block `macros`
//! substitution value are verified. The applicability table is descriptive and
//! tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/macros.adoc");

non_normative!(
    r#"
= Macro Substitutions
:navtitle: Macros
:table-caption: Table
:y: Yes
//icon:check[role="green"]
:n: No
//icon:times[role="red"]

"#
);

// The macros step turns a macro such as an inline link into its rendered form
// (here, an anchor element).
#[test]
fn macros_processed() {
    verifies!(
        r#"
The content of inline and block macros, such as cross references, links, and block images, are processed by the macros substitution step.
The macros step replaces a macro's content with the appropriate built-in and user-defined configuration.

"#
    );

    let out = convert("A link:https://x.org[site].\n");
    assert!(out.contains("A <a href=\"https://x.org\">site</a>."));
}

non_normative!(
    r#"
== Default macros substitution

<<table-macros>> lists the specific blocks and inline elements the macros substitution step applies to automatically.

.Blocks and inline elements subject to the macros substitution
[#table-macros%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |Only the xref:pass:pass-macro.adoc#inline-pass[pass macro]

|Comments |{n}

|Examples |{y}

|Headers |{n}

|Literal, listings, and source |{n}

|Macros |{y}

|Open |{y}

|Paragraphs |{y}

|Passthrough blocks |{n}

|Quotes and verses |{y}

|Sidebars |{y}

|Tables |Varies

|Titles |{y}
|===

== macros substitution value

"#
);

// For blocks, the step's name `macros` can be assigned to `subs`, processing a
// macro that a verbatim block would otherwise leave untouched.
#[test]
fn macros_value() {
    verifies!(
        r#"
The macros substitution step can be modified on blocks and inline elements.
For blocks, the step's name, `macros`, can be assigned to the xref:apply-subs-to-blocks.adoc[subs attribute].
For inline elements, the built-in values `m` or `macros` can be applied to xref:apply-subs-to-text.adoc[inline text] to add the macros substitution step.
"#
    );

    // Assigning `macros` to a literal block (which normally applies none) makes
    // the enclosed link macro render as an anchor.
    let block = convert("[subs=\"macros\"]\n....\nSee link:https://x.org[site].\n....\n");
    assert!(block.contains("See <a href=\"https://x.org\">site</a>."));
}
