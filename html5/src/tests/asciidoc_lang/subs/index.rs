//! Coverage of the AsciiDoc language description's *Substitutions* overview
//! page.
//!
//! This overview names the six substitution types, their order, and the
//! substitution groups. The conceptual introduction, the ordered type list,
//! the group table, and the escaping survey are descriptive, so they are
//! tracked as non-normative. Each group's defining behavior — normal applies
//! every type, header applies only special characters/attributes/pass,
//! verbatim only special characters, pass none (or only special characters for
//! the plus macros), and none nothing — renders through `convert` and is
//! verified.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/index.adoc");

non_normative!(
    r#"
= Substitutions
:page-aliases: substitutions.adoc
:y: Yes
//icon:check[role="green"]
:n: No
//icon:times[role="red"]

////
user manual anchor: subs

This page needs to become a more user friendly overview of what substitutions can do, how to leverage them and how to escape them and/or prevent them with a pass through.

The groups and individual substitution categories (and tables) needs to be reorganized into a more cohesive body (and then squashed into a single reference page maybe?).
The apply, prevent, and control pages need to be organized into a logical flow with clear statements of ... I have this problem: I need to get this attribute reference in this block to be replaced by not have the other <whatever> replaced, what do I use?
All the groups and subs need to be hooked up to clear use cases.
The single plus macro has no explanation (at least not in this module).
The deprecate syntax needs to be moved to the deprecated table and/or migration docs.
////

Substitutions are applied to leaf content of a block.
Substitutions determine how the text is interpreted.
If no substitutions are applied, the text is passed to the converter as entered.
Otherwise, the substitutions transform that text.

Substitutions replace references, formatting marks, characters and character sequences, and macros.
Substitutions are organized into types and those types are bundled into groups.
This page provides an overview of these classifications.
Subsequent pages go into detail about each substitution type.

== Substitution types

Each substitution type replace characters, markup, attribute references, and macros in text with the appropriate output for a given converter.
When a document is processed, up to six substitution types may be carried out depending on the block or inline element's assigned substitution group.
The processor runs the substitutions in the following order:

. xref:special-characters.adoc[]
. xref:quotes.adoc[] (i.e., inline formatting)
. xref:attributes.adoc[]
. xref:replacements.adoc[]
. xref:macros.adoc[]
. xref:post-replacements.adoc[]

For convenience, these types are grouped and ordered into substitution groups.

[#substitution-groups]
== Substitution groups

Each block and inline element has a default substitution group that is applied unless you customize the substitutions for a particular element.
<<table-subs-groups>> shows the substitution types that are executed in each group.

include::partial$subs-group-table.adoc[]

[#normal-group]
=== Normal substitution group

"#
);

// The normal group applies every substitution type: inline formatting, symbol
// replacements, attribute references, and macros are all processed in a
// paragraph.
#[test]
fn normal_group() {
    verifies!(
        r#"
The normal substitution group (`normal`) is applied to the majority of the AsciiDoc block and inline elements except for those specific elements listed under the groups described in the next sections.

"#
    );

    let out = convert(":v: X\n\nnormal *bold* (C) {v} link:https://x.org[s]\n");
    assert!(out.contains("normal <strong>bold</strong> &#169; X <a href=\"https://x.org\">s</a>"));
}

non_normative!(
    r#"
[#header-group]
=== Header substitution group

"#
);

// The header group (which governs attribute entry values) applies only special
// characters, attribute references, and the inline pass macro: `<x>` is
// encoded and `{v}` resolved, but `*b*` is left unformatted.
#[test]
fn header_group() {
    verifies!(
        r#"
The header substitution group (`header`) is applied to metadata lines (author and revision information) in the document header.
It's also applied to the values of attribute entries, regardless of whether those entries are defined in the document header or body.
Only special characters, attribute references, and the inline pass macro are replaced in elements that fall under the header group.

"#
    );

    let out = convert(":v: RESOLVED\n:tt: *b* {v} <x>\n\n{tt}\n");
    assert!(out.contains("*b* RESOLVED &lt;x&gt;"));
}

non_normative!(
    r#"
TIP: You can use the inline pass macro in attribute entries to xref:apply-subs-to-text.adoc[customize the substitution types applied to the attribute's value].

[#verbatim-group]
=== Verbatim substitution group

"#
);

// The verbatim group (literal, listing, source blocks) replaces only special
// characters: `<x>` is encoded, but `*b*` stays literal.
#[test]
fn verbatim_group() {
    verifies!(
        r#"
Literal, listing, and source blocks are processed using the verbatim substitution group (`verbatim`).
Only special characters are replaced in these blocks.

"#
    );

    let out = convert("[listing]\n....\n*b* <x>\n....\n");
    assert!(out.contains("<pre>*b* &lt;x&gt;</pre>"));
}

non_normative!(
    r#"
[#pass-group]
=== Pass substitution group

"#
);

// The pass group applies no substitutions to the passthrough block (its raw
// markup survives untouched), while the inline single/double plus macros apply
// only the special characters step (`<b>` is encoded but `*x*` is not
// formatted).
#[test]
fn pass_group() {
    verifies!(
        r#"
No substitutions are applied to three of the elements in the pass substitution group (`pass`).
These elements include the xref:pass:pass-block.adoc[passthrough block], xref:pass:pass-macro.adoc#inline-pass[inline pass macro], and xref:pass:pass-macro.adoc#triple-plus[triple plus macro].

The xref:pass:pass-macro.adoc#def-plus[inline single plus and double plus macros] also belong to the pass group.
Only the special characters substitution is applied to these elements.

"#
    );

    // The passthrough block emits its content with no substitutions applied.
    let block = convert("++++\n<b>*x*</b>\n++++\n");
    assert!(block.contains("<b>*x*</b>"));

    // The single plus macro applies only special characters, so `<b>` is
    // encoded while `*x*` remains literal.
    let plus = convert("plus +<b>*x*</b>+ done\n");
    assert!(plus.contains("plus &lt;b&gt;*x*&lt;/b&gt; done"));
}

non_normative!(
    r#"
[#none-group]
=== None substitution group

"#
);

// The none group governs comment blocks, which produce no output at all.
#[test]
fn none_group() {
    verifies!(
        r#"
The none substitution group (`none`) is applied to comment blocks.
No substitutions are applied to comments.

"#
    );

    let out = convert("////\n*bold*\n////\n\nafter\n");
    assert!(!out.contains("bold"));
    assert!(!out.contains("<strong>"));
    assert!(out.contains("<p>after</p>"));
}

non_normative!(
    r#"
== Escaping substitutions

The AsciiDoc syntax offers several approaches for preventing substitutions from being applied.
When you want to prevent punctuation and symbols from being interpreted as formatting markup, xref:prevent.adoc[escaping the punctuation with a backslash] may be sufficient.
For more comprehensive substitution prevention and control, you can use xref:pass:pass-macro.adoc[inline passthrough macros] or xref:pass:pass-block.adoc[passthrough blocks].
"#
);
