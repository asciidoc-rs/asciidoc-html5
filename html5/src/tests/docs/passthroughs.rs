//! Coverage of this crate's *Passthroughs* documentation page.
//!
//! The page shows how `asciidoc-html5` renders each passthrough form: the
//! delimited `++++` block and the `pass` paragraph style (both raw), the `subs`
//! attribute opting substitutions back in, the inline `pass:[]` macro with and
//! without substitution values, and the single plus shorthand escaping special
//! characters. Each `[,asciidoc]`/`[,html]` source-and-output pair is verified
//! by converting the shown source through `convert` and asserting the shown
//! output; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("docs/modules/ROOT/pages/passthroughs.adoc");

non_normative!(
    r#"
= Passthroughs
:navtitle: Passthroughs
:description: How asciidoc-html5 renders block and inline passthroughs, which pass content to the output while controlling substitutions.

A passthrough passes a chunk of content directly to the output, giving you
control over which substitutions are applied to it. `asciidoc-html5` supports
both the block form (the `pass` style and the delimited `++++` block) and the
inline form (the `pass:[]` macro and the one-to-three plus shorthands),
producing the same markup as Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Pass a block through

A delimited passthrough block -- four plus signs (`pass:[++++]`) around the
content -- emits its content raw, with no wrapping element and no substitutions
applied:

"#
);

// The delimited `++++` block emits its content verbatim, with no wrapping
// element.
#[test]
fn pass_a_block_through_delimited() {
    verifies!(
        r#"
[,asciidoc]
----
++++
<p class="verse">Roses are red.</p>
++++
----

The markup reaches the output untouched:

[,html]
----
<p class="verse">Roses are red.</p>
----

"#
    );

    let output = convert("++++\n<p class=\"verse\">Roses are red.</p>\n++++\n");
    assert!(output.contains("<p class=\"verse\">Roses are red.</p>"));
}

// The `pass` paragraph style emits its content raw too, with no enclosing
// paragraph wrapper.
#[test]
fn pass_a_block_through_pass_style() {
    verifies!(
        r#"
The `pass` style on a paragraph does the same thing without delimiters:

[,asciidoc]
----
[pass]
<del>strike this</del> is marked as deleted.
----

[,html]
----
<del>strike this</del> is marked as deleted.
----

"#
    );

    let output = convert("[pass]\n<del>strike this</del> is marked as deleted.\n");
    assert!(output.contains("<del>strike this</del> is marked as deleted."));
    assert_css(&output, ".paragraph", 0);
}

non_normative!(
    r#"
Because the content is not wrapped in a paragraph, the `pass` style is a way to
emit content with no enclosing `<div class="paragraph"><p>`.

== Control the substitutions on a block

Set the `subs` attribute to opt specific substitutions back in. They are applied
to the content before it is passed to the output. Here only the `attributes`
substitution runs, so the attribute reference is resolved while the raw markup
is left alone:

"#
);

// The `subs` attribute opts substitutions back in: `[subs=attributes]` resolves
// the `{release}` reference while leaving the raw markup untouched.
#[test]
fn control_the_substitutions_on_a_block() {
    verifies!(
        r#"
[,asciidoc]
----
:release: 2.0

[subs=attributes]
++++
<span>v{release}</span>
++++
----

[,html]
----
<span>v2.0</span>
----

"#
    );

    let output =
        convert(":release: 2.0\n\n[subs=attributes]\n++++\n<span>v{release}</span>\n++++\n");
    assert!(output.contains("<span>v2.0</span>"));
}

non_normative!(
    r#"
WARNING: Passing raw content couples your document to a specific output format,
such as HTML. Prefer to confine such content to an extension or a preprocessor
conditional.

== Pass content inline

"#
);

// The inline `pass:[]` macro with no substitution values passes its target
// through raw.
#[test]
fn pass_content_inline_macro() {
    verifies!(
        r#"
The inline `pass:[]` macro passes its target through. With no substitution
values, none are applied:

[,asciidoc]
----
The text pass:[<del>strike this</del>] is marked as deleted.
----

[,html]
----
<p>The text <del>strike this</del> is marked as deleted.</p>
----

"#
    );

    let output = convert("The text pass:[<del>strike this</del>] is marked as deleted.\n");
    assert!(output.contains("<p>The text <del>strike this</del> is marked as deleted.</p>"));
}

// A named substitution value (`q`) applies the quotes substitution to the
// target while special characters stay off, so `<del>` passes through raw.
#[test]
fn pass_content_inline_macro_with_value() {
    verifies!(
        r#"
Name substitution values in the macro target to apply them selectively. Here
`q` (quotes) formats the enclosed text while the special-characters substitution
stays off, so the `<del>` markup passes through raw:

[,asciidoc]
----
The text pass:q[<del>strike *this*</del>] is marked as deleted.
----

[,html]
----
<p>The text <del>strike <strong>this</strong></del> is marked as deleted.</p>
----

"#
    );

    let output = convert("The text pass:q[<del>strike *this*</del>] is marked as deleted.\n");
    assert!(output
        .contains("<p>The text <del>strike <strong>this</strong></del> is marked as deleted.</p>"));
}

non_normative!(
    r#"
== Escape formatting with the plus shorthands

"#
);

// A single plus pair applies only special characters: `<`/`>` are escaped and
// the three periods stay literal rather than becoming an ellipsis.
#[test]
fn escape_formatting_with_the_plus_shorthands() {
    verifies!(
        r#"
A pair of single pluses (`pass:[+]...pass:[+]`) prevents the enclosed text from
being formatted, applying only the special-characters substitution. This is the
common way to output characters that would otherwise be replaced:

[,asciidoc]
----
The characters +<+ and +>+ are escaped, and +...+ is literal.
----

The angle brackets are escaped as entities, and the three periods stay literal
instead of becoming an ellipsis:

[,html]
----
<p>The characters &lt; and &gt; are escaped, and ... is literal.</p>
----

"#
    );

    let output = convert("The characters +<+ and +>+ are escaped, and +...+ is literal.\n");
    assert!(output.contains("<p>The characters &lt; and &gt; are escaped, and ... is literal.</p>"));
}

non_normative!(
    r#"
A pair of triple pluses (`pass:[+++]...pass:[+++]`) applies no substitutions at
all, not even special characters, so it is the shorthand for passing raw HTML
inline.

== Known limitations

The block and inline passthrough forms shown here all match Asciidoctor's
output: the delimited block, the `pass` paragraph style, the `subs` attribute,
the `pass:[]` macro with and without substitution values, and the single,
double, and triple plus shorthands. To see which other markup the renderer
supports today, see the xref:ROOT:index.adoc[introduction].
"#
);
