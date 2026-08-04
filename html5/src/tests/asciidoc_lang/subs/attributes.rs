//! Coverage of the AsciiDoc language description's *Attribute References
//! Substitution* page.
//!
//! The attributes step replaces attribute references with the referenced
//! attribute's value. This crate performs that replacement through `convert`,
//! so the behavioral claim, the applicability table (probed block by block),
//! the `attributes`/`a` substitution values, and the backslash escape are all
//! verified. Only the `Tables | Varies` row (which is not a simple yes/no) is
//! tracked as non-normative.

use super::applicability::{ran, ran_in_header};
use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/attributes.adoc");

non_normative!(
    r#"
= Attribute References Substitution
:navtitle: Attribute References
:table-caption: Table
:y: Yes
//icon:check[role="green"]
:n: No
//icon:times[role="red"]

"#
);

// An attribute reference such as `{foo}` is replaced with the value of the
// attribute it names.
#[test]
fn attributes_replaced() {
    verifies!(
        r#"
Attribute references are replaced with the values of the attribute they reference when processed by the attributes substitution step.

"#
    );

    let out = convert(":foo: bar\n\nvalue is {foo}\n");
    assert!(out.contains("value is bar"));
}

non_normative!(
    r#"
== Default attributes substitution

"#
);

// Each row of the applicability table, probed with an attribute reference
// `{v}`: the attributes step is "applied" iff the resolved value appears in the
// output instead of the literal `{v}`.
#[test]
fn default_attributes_substitution() {
    verifies!(
        r#"
<<table-attributes>> lists the specific blocks and inline elements the attributes substitution step applies to automatically.

.Blocks and inline elements subject to the attributes substitution
[#table-attributes%autowidth,cols="~,^~"]
|===
|Blocks and elements |Substitution step applied by default

|Attribute entry values |{y}

|Comments |{n}

|Examples |{y}

|Headers |{y}

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

    // `{v}` resolves to `RESOLVED` when the attributes step is applied.
    let s = ":v: RESOLVED\n";
    let applies = |body: &str| ran(s, body, "RESOLVED");

    // Attribute entry values (yes): a nested reference in an attribute's value
    // is resolved when the value is stored. Isolated with `[subs=attributes]`
    // so only the attribute-entry substitution can produce the result.
    assert!(
        convert(":inner: RESOLVED\n:v: {inner}\n[subs=attributes]\n....\n{v}\n....\n")
            .contains("RESOLVED")
    );

    // Comments (no): the comment block produces no output.
    assert!(!applies("////\n{v}\n////\n\nsentinel\n"));

    // Examples (yes).
    assert!(applies("====\n{v}\n====\n"));

    // Headers (yes): the attribute reference in the author line resolves.
    assert!(ran_in_header(s, "{v}", "RESOLVED"));

    // Literal, listings, and source (no).
    assert!(!applies("....\n{v}\n....\n"));
    assert!(!applies("----\n{v}\n----\n"));
    assert!(!applies("[source]\n----\n{v}\n----\n"));

    // Macros (yes): the reference in a macro's attribute text resolves.
    assert!(applies("link:https://example.org[{v}]\n"));

    // Open (yes).
    assert!(applies("--\n{v}\n--\n"));

    // Paragraphs (yes).
    assert!(applies("{v}\n"));

    // Passthrough blocks (no): the raw `{v}` passes straight through.
    assert!(!applies("++++\n{v}\n++++\n"));

    // Quotes and verses (yes).
    assert!(applies("[quote]\n____\n{v}\n____\n"));
    assert!(applies("[verse]\n____\n{v}\n____\n"));

    // Sidebars (yes).
    assert!(applies("****\n{v}\n****\n"));
}

// The table's `Tables` cell is "Varies" (it depends on each cell's format
// specifier), so it is not a single yes/no claim to verify here.
non_normative!(
    r#"
|Tables |Varies

"#
);

// Titles (yes): an attribute reference in a block title is resolved.
#[test]
fn default_attributes_substitution_titles() {
    verifies!(
        r#"
|Titles |{y}
|===

"#
    );

    let out = convert(":v: RESOLVED\n.{v}\n====\nx\n====\n");
    assert!(out.contains("RESOLVED"));
}

non_normative!(
    r#"
== attributes substitution value

"#
);

// For blocks, the step's name `attributes` can be assigned to `subs`; for an
// inline passthrough, the built-in values `a` or `attributes` add the step. A
// single attribute reference is escaped by prefixing it with a backslash.
#[test]
fn attributes_value() {
    verifies!(
        r#"
The attributes substitution step can be modified on blocks and the inline passthrough.
For blocks, the step's name, `attributes`, can be assigned to the xref:apply-subs-to-blocks.adoc[subs attribute].
For an inline passthrough, the built-in values `a` or `attributes` can be applied to xref:apply-subs-to-text.adoc[inline text] to add or remove the attributes substitution step.
Single occurrences of an attribute reference can be escaped by prefixing the expression with a backslash.
"#
    );

    // Adding `attributes` to a listing block's default group (which omits it)
    // resolves the reference in verbatim content.
    let block = convert(":version: 1.0\n[source,xml,subs=\"attributes+\"]\n----\n<version>{version}</version>\n----\n");
    assert!(block.contains("&lt;version&gt;1.0&lt;/version&gt;"));

    // The inline `a` shorthand on the pass macro resolves the reference in the
    // enclosed text.
    let inline = convert(":v: X\n\npass:a[val {v}]\n");
    assert!(inline.contains("val X"));

    // A backslash before a single attribute reference escapes it, leaving the
    // literal braces (and removing the backslash).
    let escaped = convert(":id: 7\n\nIn /items/\\{id}, the reference is preserved.\n");
    assert!(escaped.contains("In /items/{id}, the reference is preserved."));
}
