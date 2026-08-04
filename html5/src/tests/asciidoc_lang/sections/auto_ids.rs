//! Coverage of the AsciiDoc language description's *Autogenerate Section IDs*
//! page.
//!
//! The page's verifiable rules are that an auto-generated section ID is derived
//! from the title (lowercased, non-word characters removed, an `_` prefix, and
//! `_` word separators, so `== Wiley & Sons, Inc.` yields `_wiley_sons_inc`),
//! and that unsetting the `sectids` attribute disables ID generation entirely.
//! Both are verified through `convert`. The `page-aliases` header, the
//! step-by-step computed-ID rule list, the NT-Name/DocBook caveat, the
//! forward-xref caution, and the per-section toggling discussion are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_xpath, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/auto-ids.adoc");

// Title and header attribute entries (the `page-aliases` redirect and the
// `url-ntname` link target) carry no HTML rule of their own.
non_normative!(
    r#"
= Autogenerate Section IDs
:page-aliases: ids.adoc
:url-ntname: https://www.w3.org/TR/REC-xml/#NT-Name

"#
);

// Introductory description of ID autogeneration, the "How a section ID is
// computed" heading, the ordered rule list describing how the ID is derived,
// and the caveat that a generated ID need not conform to an XML NT-Name for
// DocBook output: descriptive prose whose end-to-end effect is verified by the
// worked example below rather than step by step.
non_normative!(
    r#"
Sections and discrete headings support automatic ID generation.
Unless you've assigned a custom ID to one of these blocks, or you've unset the `sectids` document attribute, the AsciiDoc processor will automatically generate and assign an ID for the block using the title.
This page explains how the ID is derived and how to control this behavior.

== How a section ID is computed

The AsciiDoc processor builds an ID from the title using the following order of events and rules:

* Inline formatting is applied (in title substitution order).
* All characters are converted to lowercase.
* The value of the xref:id-prefix-and-separator.adoc#prefix[idprefix attribute] (`+_+` by default) is prepended.
* Character references, HTML/XML tags (not their contents), and non-word characters (except for space, hyphen, and period) are removed.
* Spaces, hyphens, and periods are replaced with the value of the xref:id-prefix-and-separator.adoc#separator[idseparator attribute] (`+_+` by default)
* Repeating separator characters are condensed.
* If necessary, a sequence number is appended until the ID is unique within the document.

The generated ID can be expected to be safe to use in an HTML document.
However, it's important to understand that the generated ID does not necessarily conform to an {url-ntname}[NT-Name^], as required by the XML specification.
If you intend to produce DocBook from your AsciiDoc document(s), and your section titles uses word characters that are not permitted in an XML ID, the onus is on you to either provide an explicit ID that is conforming, or encode those invalid ID characters using a character reference (e.g., `\&#x2161;`).

"#
);

// The worked example the page states outright: given `== Wiley & Sons, Inc.`
// the processor generates the ID `_wiley_sons_inc` (title lowercased, the `&`
// and punctuation dropped, spaces collapsed to the `_` separator, and the `_`
// prefix prepended).
#[test]
fn ampersand_title_generates_id() {
    verifies!(
        r#"
With those rules in mind, given the following section title:

----
== Wiley & Sons, Inc.
----

the processor will produce the following ID:

....
_wiley_sons_inc
....

"#
    );

    let html = convert("= D\n\n== Wiley & Sons, Inc.");

    assert!(
        html.contains(r#"<h2 id="_wiley_sons_inc">Wiley &amp; Sons, Inc.</h2>"#),
        "{html}"
    );
}

// Cross-reference to the prefix/separator customization page plus the caution
// about forward-looking xrefs in a title: advisory prose, not a distinct HTML
// rule driven here.
non_normative!(
    r#"
You can toggle ID autogeneration on and off using `sectids` and xref:id-prefix-and-separator.adoc[customize the ID prefix and word separator].

CAUTION: If the section title contains a forward looking xref (i.e., an xref to an element that comes later in document order), you must either assign a custom ID to the block or <<disable,disable ID generation>> around the title.
Otherwise, the AsciiDoc processor may warn that the reference is invalid.
This happens because, in order to generate an ID, the processor must convert the title.
This conversion happens before the processor has visited the target element.
As a result, the processor is not able to lookup the reference and therefore must consider it invalid.

"#
);

// Section heading (with its `[#disable]` anchor) only.
non_normative!(
    r#"
[#disable]
== Disable automatic section ID generation

"#
);

// Unsetting the `sectids` attribute disables autogeneration, so a section title
// under `:!sectids:` renders with no `id` attribute at all.
#[test]
fn unsetting_sectids_disables_id_generation() {
    verifies!(
        r#"
To disable autogeneration of section and discrete heading IDs, unset the `sectids` attribute.

----
:!sectids:
----

"#
    );

    let html = convert("= D\n:!sectids:\n\n== Foo");

    // The section heading is present but carries no auto-generated id.
    assert_xpath(&html, "//h2", 1);
    assert_xpath(&html, "//h2[@id]", 0);
}

// Custom IDs still apply when autogeneration is off, and the attribute can be
// unset anywhere to disable ID generation for only certain sections (shown with
// an illustrative toggling listing), which is why disabling it forfeits
// cross-referencing: descriptive prose and an illustrative listing not driven
// through `convert`.
non_normative!(
    r#"
xref:custom-ids.adoc[Custom IDs] are still used even when automatic section IDs are disabled.

You can unset this attribute anywhere that attribute entries are permitted in the document.
By doing so, you can disable ID generation for only certain sections and discrete headings.

----
== ID generation on

:!sectids:
== ID generation off
:sectids:

== ID generation on again
----

If you disable autogenerated section IDs, and you don't assign a custom ID to a section or discrete headings, you won't be able to create cross references to that element.
"#
);
