//! Coverage of the AsciiDoc language description's *Passthroughs* overview
//! page.
//!
//! The page is a conceptual introduction to the passthrough mechanism: it
//! contrasts the block form (delimited `++++` block or the `pass` paragraph
//! style) with the inline form (the `pass:[]` macro or one-to-three plus
//! shorthands) and explains what each is for. Almost all of it is descriptive
//! prose tracked as non-normative. The one concrete, verifiable claim — that an
//! inline passthrough can output characters that would otherwise be replaced,
//! "such as three sequential periods" — is verified through `convert`.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/pass/pages/index.adoc");

non_normative!(
    r#"
= Passthroughs

A passthrough is a mechanism in AsciiDoc for passing chunks of content directly through to the output.
Most passthroughs give you control over which substitutions are applied to the content.
AsciiDoc provides both block and inline forms of the passthrough.
//Typically, a passthrough is used either to pass raw content like HTML to the output, or to escape content from inline formatting.

The xref:pass-block.adoc[block form] of the passthrough is represented either by the `pass:[++++]` block delimiters or the `pass` style on a paragraph.
The main use of the block form is to pass a chunk of non-AsciiDoc content directly through to the output.
For example, you can use the passthrough block to pass raw HTML to the HTML output.
However, by doing so, you're coupling your AsciiDoc content to an output format, thus making it less portable.
It's best either to leave the use of the passthrough block up to an extension, or enclose it in a preprocessor conditional.

The xref:pass-macro.adoc[inline form] of the passthrough comes in more forms and thus has more uses.
An inline passthrough is represented by the `+pass:[]+` macro or by pairs of one to three pluses.
Only the macro gives you control over the substitutions that are applied.
While an inline passthrough can be used to pass raw content like HTML to the output, far more often it's used as a way to escape content from inline formatting.
"#
);

// The escaping use of an inline passthrough: three sequential periods, which
// the replacements substitution would otherwise turn into a horizontal-ellipsis
// entity, pass through literally when wrapped in a plus pair. The contrast
// confirms the passthrough suppresses the substitution.
#[test]
fn inline_passthrough_escapes_a_would_be_replacement() {
    verifies!(
        r#"
For example, you can use an inline passthrough to output characters that would otherwise be replaced, such as three sequential periods.
"#
    );

    // Without a passthrough, three periods are replaced by the ellipsis entity.
    let replaced = convert("Three periods ... become an ellipsis.\n");
    assert!(replaced.contains("&#8230;"));
    assert!(!replaced.contains("Three periods ..."));

    // Enclosed in a plus pair, they pass through literally.
    let escaped = convert("Three periods +...+ stay literal.\n");
    assert!(escaped.contains("Three periods ... stay literal."));
}
