//! Coverage of the AsciiDoc language description's *Bibliography* page.
//!
//! The page describes AsciiDoc's bibliography support. A `[bibliography]`
//! section renders as a plain section, a `[bibliography]` style applied
//! directly to an unordered list produces `<div class="ulist bibliography">` /
//! `<ul class="bibliography">`, and each `[[[label]]]` anchor converts to
//! `<a id="label"></a>[label]` (or `[xreftext]` for `[[[label,xreftext]]]`).
//! Those behaviors are verified here through `convert`. The implicit
//! section-to-list style propagation, the book-doctype level-0 variant, the
//! `include::` listings, and the escape TIP are tracked non-normatively.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/bibliography.adoc");

// Page title only.
non_normative!(
    r#"
= Bibliography

"#
);

// Descriptive introduction to AsciiDoc's bibliography support.
non_normative!(
    r#"
AsciiDoc has basic support for bibliographies.
AsciiDoc doesn't concern itself with the structure of the bibliography entry itself, which is entirely freeform.
What it does is provide a way to make references to the entries from the same document and output the bibliography with proper semantics for processing by other toolchains (such as DocBook).

"#
);

// Section heading only.
non_normative!(
    r#"
== Bibliography section syntax

"#
);

// Descriptive requirement that a bibliography is its own section carrying the
// `bibliography` style; the section rendering is verified below.
non_normative!(
    r#"
To conform to output formats, a bibliography must be its own section at any level.
The section must be assigned the `bibliography` section style.
"#
);

// The parser does not propagate a bibliography SECTION's style to its child
// unordered lists (the child <ul> lacks the "bibliography" class); this is a
// parser-model gap tracked in asciidoc-parser #1066, so the
// implicit-propagation claim is non-normative here. The explicit list style IS
// verified below.
non_normative!(
    r#"
By adding the `bibliography` style to the section, you implicitly add it to each unordered list in that section.

"#
);

/// A `[bibliography]` level-1 section renders as a plain section (the
/// `bibliography` style has no section-level HTML effect).
#[test]
fn bibliography_section() {
    verifies!(
        r#"
You would define the bibliography as a level 1 section (`==`) when:

* the doctype is `article`
* the doctype is `book` and the book doesn't contain any parts
* the bibliography is for a part

[source]
----
[bibliography]
== Bibliography
----

"#
    );

    let html = convert("[bibliography]\n== Bibliography");

    assert!(html.contains(r#"<h2 id="_bibliography">Bibliography</h2>"#));
}

// Descriptive note about deeper bibliography sections (parser scoping detail
// with no distinct HTML rendering to verify).
non_normative!(
    r#"
You can also define it as a deeper section, in which case the doctype doesn't matter and it's scoped to the parent section.

"#
);

// Book-doctype behavior (parts and the whole-book level-0 special section) is
// out of scope for 1.0 (article is the only structural doctype modeled – see
// issue #188), so this level-0 variant is tracked non-normatively.
non_normative!(
    r#"
If the book has parts, and the bibliography is for the whole book, the section is defined as a level 0 section (`=`).

[source]
----
[bibliography]
= Bibliography
----

"#
);

// Section heading only.
non_normative!(
    r#"
== Bibliography entries syntax

"#
);

/// A `[bibliography]` style applied directly to an unordered list produces
/// `<div class="ulist bibliography">` / `<ul class="bibliography">`.
#[test]
fn bibliography_entries_are_a_list() {
    verifies!(
        r#"
Bibliography entries are declared as items in an unordered list.

"#
    );

    let html = convert(
        "[bibliography]\n* [[[pp]]] Andy Hunt & Dave Thomas. The Pragmatic Programmer.\n* [[[gof,gang]]] Erich Gamma et al. Design Patterns.",
    );

    assert_css(&html, "div.ulist.bibliography", 1);
    assert_css(&html, "ul.bibliography", 1);
}

// The `include::` listing is a build-time include, not literal source that this
// crate runs through `convert`.
non_normative!(
    r#"
.Bibliography with references
[source]
----
include::example$bibliography.adoc[tag=base]
----

"#
);

/// A bibliography entry's `[[[label]]]` anchor converts to
/// `<a id="label"></a>[label]`.
#[test]
fn bibliography_anchor_renders() {
    verifies!(
        r#"
In order to reference a bibliography entry, you need to assign a _non-numeric_ label to the entry.
To assign this label, prefix the entry with the label enclosed in a pair of triple square brackets (e.g., `+[[[label]]]+`).
We call this a bibliography anchor.
Using this label, you can then reference the entry from anywhere above the bibliography in the same document using the normal cross reference syntax (e.g., `+<<label>>+`).

"#
    );

    let html = convert("[bibliography]\n* [[[pp]]] Andy Hunt & Dave Thomas.");

    assert!(html.contains(r#"<a id="pp"></a>[pp] Andy Hunt"#));
}

// The `|===` cell re-embeds the base example through an `include::` directive;
// it is a build-time include, not literal source run through `convert`.
non_normative!(
    r#"
|===
a|
include::example$bibliography.adoc[tag=base]
|===

"#
);

// The escape TIP is descriptive guidance about suppressing anchor matching, not
// a rendering rule verified here.
non_normative!(
    r#"
TIP: To escape a bibliography anchor anywhere in the text, use the syntax `[\[[word]]]`.
This prevents the anchor from being matched as a bibliography anchor or a normal anchor.

"#
);

/// By default an anchor converts to `[<label>]`; supplying an xreftext
/// (`[[[gof,gang]]]`) converts to `[<xreftext>]`, and a numeric xreftext
/// (`[[[label,1]]]`) converts to `[1]`.
#[test]
fn bibliography_anchor_xreftext() {
    verifies!(
        r#"
By default, the bibliography anchor and reference to the bibliography entry is converted to `[<label>]`, where <label> is the ID of the entry.
If you specify xreftext on the bibliography anchor (e.g., `+[[[label,xreftext]]]+`), the bibliography anchor and reference to the bibliography entry converts to `[<xreftext>]` instead.

If you want the bibliography anchor and reference to appear as a number, assign the number of the entry using the xreftext.
For example, `+[[[label,1]]]+` will be converted to `[1]`.

"#
    );

    let default = convert("[bibliography]\n* [[[pp]]] Andy Hunt.");

    assert!(default.contains(r#"<a id="pp"></a>[pp] Andy Hunt."#));

    let xreftext = convert("[bibliography]\n* [[[gof,gang]]] Erich Gamma.");

    assert!(xreftext.contains(r#"<a id="gof"></a>[gang] Erich Gamma."#));

    let numeric = convert("[bibliography]\n* [[[reading,1]]] Andy Hunt.");

    assert!(numeric.contains(r#"<a id="reading"></a>[1] Andy Hunt."#));
}

// The pointer to the asciidoctor-bibtex project is an external "to learn more"
// reference, not a rendering rule verified here.
non_normative!(
    r#"
If you want more advanced features such as automatic numbering and custom citation styles, try the https://github.com/asciidoctor/asciidoctor-bibtex[asciidoctor-bibtex^] project.
"#
);
