//! Coverage of the AsciiDoc language description's *Customize the Substitutions
//! Applied to Text* page.
//!
//! The inline pass macro applies a chosen set of substitutions to its content,
//! and a block's `subs` attribute can enable the pass macro inside verbatim
//! content. This crate implements both through `convert`, so the shorthand
//! values and every worked example are verified. The examples are pulled in
//! with `include::pass:example$pass.adoc[…]`; the reproduced lines keep the
//! include directives, while the tests drive the included example text
//! directly.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/apply-subs-to-text.adoc");

non_normative!(
    r#"
= Customize the Substitutions Applied to Text

"#
);

// The inline pass macro accepts a shorthand letter for each substitution type;
// each shorthand is equivalent to its longhand name.
#[test]
fn inline_pass_shorthands() {
    verifies!(
        r#"
The inline pass macro (`++pass:[]++`) accepts the shorthand values in addition to the longhand values for specifying substitution types.

* `c` or `specialchars`
* `q` or `quotes`
* `a` or `attributes`
* `r` or `replacements`
* `m` or `macros`
* `p` or `post_replacements`

"#
    );

    // Each shorthand produces the same output as its longhand form.
    assert_eq!(
        convert("pass:c[<x>]\n"),
        convert("pass:specialchars[<x>]\n")
    );
    assert_eq!(convert("pass:q[*b*]\n"), convert("pass:quotes[*b*]\n"));
    assert_eq!(
        convert(":v: X\n\npass:a[{v}]\n"),
        convert(":v: X\n\npass:attributes[{v}]\n")
    );
    assert_eq!(
        convert("pass:r[(C)]\n"),
        convert("pass:replacements[(C)]\n")
    );
    assert_eq!(
        convert("pass:m[link:https://x.org[s]]\n"),
        convert("pass:macros[link:https://x.org[s]]\n")
    );

    // Spot-check that the shorthands actually perform their step. (The `p`
    // shorthand's effect is checked directly: matching Asciidoctor 2.0.26, the
    // inline pass macro honors only the `p` shorthand for post replacements, not
    // the `post_replacements` longhand.)
    assert!(convert("pass:c[<x>]\n").contains("&lt;x&gt;"));
    assert!(convert("pass:q[*b*]\n").contains("<strong>b</strong>"));
    assert!(convert("pass:p[a +\nb]\n").contains("a<br>\nb"));
}

non_normative!(
    r#"
== Apply substitutions to inline text

"#
);

// The inline pass macro with no substitutions passes raw HTML (`<del>`) through
// to the output.
#[test]
fn inline_pass_del() {
    verifies!(
        r#"
Custom substitutions can also be applied to inline text with the xref:pass:pass-macro.adoc[pass macro].
For instance, let's assume you need to mark a span of text as deleted using the HTML element `<del>` in your AsciiDoc document.
You'd do this with the inline pass macro.

.Inline pass macro syntax
[source#ex-pass]
----
include::pass:example$pass.adoc[tag=in-macro]
----

The result of <<ex-pass>> is rendered below.

====
include::pass:example$pass.adoc[tag=in-macro]
====

"#
    );

    let out = convert("The text pass:[<del>strike this</del>] is marked as deleted.\n");
    assert!(out.contains("The text <del>strike this</del> is marked as deleted."));
}

// Assigning `quotes` to the inline pass macro formats AsciiDoc markup within
// the otherwise-raw content.
#[test]
fn inline_pass_del_quotes() {
    verifies!(
        r#"
However, you also need to bold the text and want to use the AsciiDoc markup for that formatting.
In this case, you'd assign the `quotes` substitution to the inline pass macro.

.Assign quotes to inline pass macro
[source#ex-sub-quotes]
----
include::pass:example$pass.adoc[tag=s-macro]
----

The result of <<ex-sub-quotes>> is rendered below.

====
include::pass:example$pass.adoc[tag=s-macro]
====

"#
    );

    let out = convert(
        "The text pass:q[<del>strike *this*</del>] is marked as deleted, \
         inside of which the word \"`me`\" is bold.\n",
    );
    assert!(out.contains("<del>strike <strong>this</strong></del>"));
    assert!(out.contains("&#8220;me&#8221;"));
}

// Adding `macros` to a listing block's substitutions lets a `pass` macro inside
// the block run, so the marked line is formatted while the rest stays literal.
#[test]
fn inline_pass_in_block() {
    verifies!(
        r#"
You can also assign custom substitutions to inline text that's in a block.
In the listing block below, we want to process the inline formatting on the second line.

.Listing block with inline formatting
[source#ex-listing]
....
include::pass:example$pass.adoc[tag=sub-in]
....
<.> `macros` is assigned to `subs`, which allows the `pass` macro within the block to be processed.
<.> The `pass` macro is assigned the `quotes` value.
Text within the square brackets will be formatted.

The result of <<ex-listing>> is rendered below.

====
include::pass:example$pass.adoc[tag=sub-out]
====
"#
    );

    let out = convert(
        "[subs=+macros]\n----\nI better not contain *bold* or _italic_ text.\n\
         pass:quotes[But I should contain *bold* text.]\n----\n",
    );
    assert!(out.contains("I better not contain *bold* or _italic_ text."));
    assert!(out.contains("But I should contain <strong>bold</strong> text."));
}
