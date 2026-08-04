//! Coverage of the AsciiDoc language description's *Escape and Prevent
//! Substitutions* page.
//!
//! Backslashes (single and double) and the `{plus}`/backslash plus escapes
//! prevent substitutions; this crate implements every escape the page's
//! examples demonstrate through `convert`, so they are verified. The
//! conceptual introduction to passthroughs is descriptive and tracked as
//! non-normative. The examples themselves are pulled into the page with
//! `include::example$subs.adoc[…]`; the reproduced lines keep the include
//! directives, while the tests drive the included example text directly.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/prevent.adoc");

non_normative!(
    r#"
= Escape and Prevent Substitutions

The AsciiDoc syntax offers several approaches for preventing substitutions from being applied.

== Escape with backslashes

"#
);

// A single backslash before a punctuation character (or macro, attribute
// reference, or replacement) prevents the substitution and is itself removed
// from the output.
#[test]
fn escape_backslash() {
    verifies!(
        r#"
To prevent a punctuation character from being interpreted as an attribute reference or formatting syntax (e.g., `+_+`, `+^+`) in normal content, prepend the character with a backslash (`\`).

.Prevent unintended substitutions with a backslash in normal content
[source#ex-backslash]
----
include::example$subs.adoc[tag=backslash]
----

The backslash can also prevent character replacements, macros, and attribute replacements.
The results of <<ex-backslash>> are below.

====
include::example$subs.adoc[tag=backslash]
====

Notice that the backslash is removed so it doesn't display in your output.

"#
    );

    // An escaped attribute reference keeps its braces (the backslash is gone).
    assert!(convert("In /items/\\{id}, preserved.\n").contains("In /items/{id}, preserved."));

    // Escaped formatting markup is left literal.
    assert!(convert("\\*Stars* preserved.\n").contains("*Stars* preserved."));

    // An escaped entity reference is emitted as a literal `&sect;` (the `&` is
    // encoded), so it is not resolved.
    assert!(convert("\\&sect; entity.\n").contains("&amp;sect; entity."));

    // An escaped replacement ligature stays literal.
    assert!(convert("\\=> arrow.\n").contains("=&gt; arrow."));

    // Escaped anchor, bibliography-anchor, and flow-index syntax stay literal.
    assert!(convert("\\[[Word]] preserved.\n").contains("[[Word]] preserved."));
    assert!(convert("[\\[[Word]]] preserved.\n").contains("[[[Word]]] preserved."));
    assert!(convert("\\((DD AND CC) OR (DD AND EE)) term.\n")
        .contains("((DD AND CC) OR (DD AND EE)) term."));

    // An escaped URL is not autolinked.
    let url = convert("The URL \\https://example.org isnt a link.\n");
    assert!(url.contains("The URL https://example.org isnt a link."));
    assert!(!url.contains("<a "));
}

// Two adjacent characters (an unconstrained formatting pair) require two
// backslashes to escape; the pair is then left literal and not italicized.
#[test]
fn escape_double_backslash() {
    verifies!(
        r#"
To prevent two adjacent characters (e.g., +__+, pass:[##]), from being interpreted as AsciiDoc syntax you need to precede it with two backslashes (`+\\+`).

.Prevent unintended substitutions with two backslashes in normal content
[source#ex-double-slash]
----
include::example$subs.adoc[tag=double-slash]
----

The results of <<ex-double-slash>> are below.

====
include::example$subs.adoc[tag=double-slash]
====

"#
    );

    let out = convert("The text \\\\__func__ will appear with two underscores.\n");
    assert!(out.contains("The text __func__ will appear with two underscores."));
    assert!(!out.contains("<em>"));
}

non_normative!(
    r#"
== Passthroughs

A passthrough is the primary mechanism by which to escape content in AsciiDoc.
They're far more comprehensive and consistent than using a backslash.
As the name implies, a passthrough passes content directly through to the output document without applying any substitutions.

You can control and prevent substitutions in inline text with the xref:pass:pass-macro.adoc[inline passthrough macros] and for entire blocks of content with the xref:pass:pass-block.adoc[block passthrough].

"#
);

// A literal plus in constrained monospace is produced either by the `{plus}`
// attribute reference or by escaping the pair with a single backslash; both
// yield a monospaced plus.
#[test]
fn plus_escape() {
    verifies!(
        r#"
The inline `{plus}` passthrough takes precedence over all other inline formatting.
Therefore, if you need to output a literal plus when it would otherwise match a passthrough, you have two options.

First, you can escape it using the `\{plus}` attribute reference:

[source]
----
`{plus}` and `{plus}`
----

Alternately, you can escape the pair using a backslash.

[source]
----
`\+` and `+`
----

The backslash is only required before the pair, not before each occurance of the plus.
"#
    );

    // The `{plus}` attribute reference resolves to a monospaced plus (`&#43;`).
    assert!(
        convert("`{plus}` and `{plus}`\n").contains("<code>&#43;</code> and <code>&#43;</code>")
    );

    // Escaping only the first plus of the pair with a backslash yields the same
    // literal monospaced plus.
    assert!(convert("`\\+` and `+`\n").contains("<code>+</code> and <code>+</code>"));
}
