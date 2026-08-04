//! Coverage of the AsciiDoc language description's *Macro Substitutions* page.
//!
//! The macros step processes inline and block macros. This crate processes
//! macros through `convert`, so the behavioral claim, the applicability table
//! (probed block by block), and the block `macros` substitution value are
//! verified. Only the `Tables | Varies` row (which is not a simple yes/no) is
//! tracked as non-normative.

use super::applicability::{ran, ran_in_header};
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

"#
);

// Each row of the applicability table, probed with an inline link: the macros
// step is "applied" iff the link renders as an anchor (`href` appears).
#[test]
fn default_macros_substitution() {
    verifies!(
        r#"
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

"#
    );

    // An inline link renders as `<a href=…>` when the macros step runs.
    let applies = |body: &str| ran("", body, "href");

    // Attribute entry values: only the inline pass macro is processed. A regular
    // macro in an attribute value is left literal, while a pass macro is
    // applied. Isolated with `[subs=attributes]` so the block adds no macros
    // processing of its own.
    assert!(
        !convert(":v: link:https://example.org[t]\n[subs=attributes]\n....\n{v}\n....\n")
            .contains("href")
    );
    assert!(
        convert(":v: pass:[<raw-x>]\n[subs=attributes]\n....\n{v}\n....\n").contains("<raw-x>")
    );

    // Comments (no): the comment block produces no output.
    assert!(!applies(
        "////\nlink:https://example.org[t]\n////\n\nsentinel\n"
    ));

    // Examples (yes).
    assert!(applies("====\nlink:https://example.org[t]\n====\n"));

    // Headers (no): a macro in the author line is not processed.
    assert!(!ran_in_header("", "link:https://example.org[t]", "href"));

    // Literal, listings, and source (no).
    assert!(!applies("....\nlink:https://example.org[t]\n....\n"));
    assert!(!applies("----\nlink:https://example.org[t]\n----\n"));
    assert!(!applies(
        "[source]\n----\nlink:https://example.org[t]\n----\n"
    ));

    // Macros (yes): a block macro is processed by the macros step.
    assert!(convert("image::x.png[Alt]\n").contains("<img src="));

    // Open (yes).
    assert!(applies("--\nlink:https://example.org[t]\n--\n"));

    // Paragraphs (yes).
    assert!(applies("link:https://example.org[t]\n"));

    // Passthrough blocks (no): the raw macro text passes straight through.
    assert!(!applies("++++\nlink:https://example.org[t]\n++++\n"));

    // Quotes and verses (yes).
    assert!(applies(
        "[quote]\n____\nlink:https://example.org[t]\n____\n"
    ));
    assert!(applies(
        "[verse]\n____\nlink:https://example.org[t]\n____\n"
    ));

    // Sidebars (yes).
    assert!(applies("****\nlink:https://example.org[t]\n****\n"));
}

// The table's `Tables` cell is "Varies" (it depends on each cell's format
// specifier), so it is not a single yes/no claim to verify here.
non_normative!(
    r#"
|Tables |Varies

"#
);

// Titles (yes): a macro in a block title is processed.
#[test]
fn default_macros_substitution_titles() {
    verifies!(
        r#"
|Titles |{y}
|===

"#
    );

    assert!(convert(".link:https://example.org[t]\n====\nbody\n====\n").contains("href"));
}

non_normative!(
    r#"
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
