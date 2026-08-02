//! Coverage of the AsciiDoc language description's *Passthrough Blocks* page.
//!
//! The page documents the block form of the passthrough: the `pass` paragraph
//! style and the delimited `++++` block, both of which exclude their content
//! from every substitution unless a `subs` attribute opts some back in. This
//! crate renders each documented form, so the intro rule and every section's
//! example — the `include::example$pass.adoc[tag=…]` source it shows — is
//! verified through `convert`, driving the exact source of the included tag
//! (from `ref/asciidoc-lang/docs/modules/pass/examples/pass.adoc`). Only the
//! section headings, a demonstration aside, and the closing warning are tracked
//! as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/pass/pages/pass-block.adoc");

non_normative!(
    r#"
= Passthrough Blocks

"#
);

// The page's defining rule: both block forms exclude their content from all
// substitutions unless `subs` is set. A `[pass]` paragraph containing markup
// that special-characters, quotes, and macros substitutions would each
// otherwise transform passes straight through untouched.
#[test]
fn block_forms_exclude_all_substitutions() {
    verifies!(
        r#"
The `pass` style and delimited passthrough block exclude the block's content from all substitutions unless the `subs` attribute is set.

"#
    );

    let output = convert("[pass]\n<del>*bold*</del> image:t.png[]\n");

    // The content is emitted verbatim: no special-characters escaping (the
    // `<del>` markup passes through raw), no quotes (`*bold*` stays literal),
    // and no macros (`image:` is not expanded).
    assert!(output.contains("<del>*bold*</del> image:t.png[]"));
    assert_css(&output, "strong", 0);
    assert_css(&output, "img", 0);
}

non_normative!(
    r#"
== Pass style syntax

"#
);

// The `pass` paragraph style: setting `[pass]` on a paragraph passes its
// content through raw, with no enclosing `<p>`/paragraph wrapper.
#[test]
fn pass_style_on_a_paragraph() {
    verifies!(
        r#"
The `pass` style can also be set on a paragraph or an open block.

[source]
----
include::example$pass.adoc[tag=pass-style]
----

"#
    );

    // tag=pass-style
    let output = convert("[pass]\n<del>strike this</del> is marked as deleted.\n");

    assert!(output.contains("<del>strike this</del> is marked as deleted."));
    assert_css(&output, ".paragraph", 0);

    // The `pass` style set on an open block behaves the same way.
    let open = convert("[pass]\n--\n<del>strike this</del> is marked as deleted.\n--\n");
    assert!(open.contains("<del>strike this</del> is marked as deleted."));
    assert_css(&open, ".openblock", 0);
}

non_normative!(
    r#"
== Delimited passthrough block syntax

"#
);

// The delimited `++++` block: its raw content is emitted verbatim, with no
// wrapping element. The video markup here demonstrates passing raw HTML.
#[test]
fn delimited_passthrough_block() {
    verifies!(
        r#"
A passthrough block is delimited by four plus signs (`pass:[++++]`).

[source]
----
include::example$pass.adoc[tag=bl]
----

"#
    );

    // tag=bl
    let output = convert(
        "++++\n\
         <video poster=\"images/movie-reel.png\">\n  \
         <source src=\"videos/writing-zen.webm\" type=\"video/webm\">\n\
         </video>\n++++\n",
    );

    assert_css(&output, "video", 1);
    assert_css(&output, "video > source", 1);
    // The raw markup is not wrapped in any block element.
    assert_css(&output, ".paragraph", 0);
    assert_css(&output, "pre", 0);
}

non_normative!(
    r#"
(Keep in mind that AsciiDoc has a video macro, so this example is merely for demonstration.
However, a passthrough could come in handy if you need to output more sophisticated markup than what the built-in HTML converter produces).

== Control substitutions on a passthrough block

"#
);

// The `subs` attribute opts substitutions back in: `[subs=attributes]` runs
// only the attributes substitution, so an attribute reference is resolved while
// a macro (which is not in the list) is left untouched.
#[test]
fn control_substitutions_with_subs() {
    verifies!(
        r#"
You can use the xref:subs:apply-subs-to-blocks.adoc[subs attribute to specify a comma-separated list of substitutions].
These substitutions will be applied to the content prior to it being reintroduced to the output document.

[source]
----
include::example$pass.adoc[tag=subs-bl]
----

"#
    );

    // tag=subs-bl, with `name` defined so the attributes substitution is
    // observable.
    let output =
        convert(":name: Passed\n\n[subs=attributes]\n++++\n{name}\nimage:tiger.png[]\n++++\n");

    // The attributes substitution resolved `{name}` ...
    assert!(output.contains("Passed"));
    // ... but the macros substitution did not run, so the image macro is raw.
    assert!(output.contains("image:tiger.png[]"));
    assert_css(&output, "img", 0);
}

// The `pass` block does not wrap its content in a paragraph, so pairing it with
// the `normal` substitution group yields fully substituted content with no
// paragraph element around it.
#[test]
fn pass_block_with_normal_subs_emits_no_paragraph() {
    verifies!(
        r#"
The content of the pass block does not get wrapped in a paragraph.
Therefore, you can use the `pass` style in combination with the `normal` substitution category to output content without generating a paragraph.

[source]
----
include::example$pass.adoc[tag=no-para]
----

"#
    );

    // tag=no-para
    let output = convert(
        "[subs=normal]\n++++\nNormal content which is not enclosed in a paragraph.\n++++\n",
    );

    assert!(output.contains("Normal content which is not enclosed in a paragraph."));
    assert_css(&output, ".paragraph", 0);
    assert_css(&output, "p", 0);
}

non_normative!(
    r#"
WARNING: Using passthroughs to pass content (without substitutions) can couple your content to a specific output format, such as HTML.
In these cases, you should use conditional preprocessor directives to route passthrough content for different output formats based on the current backend.
"#
);
