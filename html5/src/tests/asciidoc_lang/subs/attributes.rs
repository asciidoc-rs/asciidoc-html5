//! Coverage of the AsciiDoc language description's *Attribute References
//! Substitution* page.
//!
//! The attributes step replaces attribute references with the referenced
//! attribute's value. This crate performs that replacement through `convert`,
//! so the behavioral claim, the `attributes`/`a` substitution values, and the
//! backslash escape are verified. The applicability table is descriptive and
//! tracked as non-normative.

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

|Tables |Varies

|Titles |{y}
|===

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
