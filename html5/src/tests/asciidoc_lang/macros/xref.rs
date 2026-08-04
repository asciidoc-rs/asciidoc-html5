//! Coverage of the AsciiDoc language description's *Cross References* page.
//!
//! An inline cross reference — the `<<id>>` shorthand or the `xref:target[]`
//! macro — links to an anchor. With only an ID, the target's reftext (usually
//! its title) is the link text; with explicit text after a comma, that text is
//! used; a natural cross reference targets a section by its title. The `window`
//! attribute sets the link `target`. All verified through `convert`, matching
//! Asciidoctor 2.0.26.
//!
//! The examples are pulled in with `include::example$xref.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline (with the referenced anchors).

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/xref.adoc");

non_normative!(
    r#"
= Cross References

A link to another location within the current AsciiDoc document or in another AsciiDoc document is called a [.term]*cross reference* (also referred to as an [.term]*xref*).
To create a cross reference, you first need to define the location where the reference will point (i.e., the anchor).
Then, you need to use one of the forms of the inline xref macro to create a reference to that location.
From there, you can customize the text of the reference in various ways.

"#
);

non_normative!(
    r#"
//don't change the id of this section unless you also change the example in the Internal cross references section (below this section)
[#anchors]
== Automatic anchors

It's important to understand that many anchors are already defined for you.
Using default settings, the AsciiDoc processor automatically creates an anchor for every section and discrete heading.
It does so by generating an ID for that section (or discrete heading) and registering that ID in the references catalog.
You can then use that ID as the target of a cross reference.

For example, considering the following section.

[source]
----
= Section Title
----

The AsciiDoc processor automatically assigns the ID `_section_title` to this section, which you can then use as the target of an xref to create a reference to this section.
You can also customize how this ID is generated.
Refer to xref:sections:auto-ids.adoc[] for more information about how an AsciiDoc processor generates these IDs.

If you're referring to a content element other than a section, you'll need to define an anchor on that element explicitly.

"#
);

non_normative!(
    r#"
== Internal cross references

In AsciiDoc, the shorthand xref is used to create a cross reference to an element (e.g., section, block, list item, etc.) that has an ID within the same document.
The shorthand xref is processed by the macros substitution.

If the cross reference specifies both an ID and text, the text is formatted and used as the link text.
If the cross reference only specifies the ID, the reftext of the target element (typically the formatted title) is automatically used as the link text.
If the element does not define reftext, a stylized form of the ID is used instead.
Whether the ID is assigned explicitly on the referenced element or auto-generated does not affect how this mechanism works.

Currently, an AsciiDoc processor can resolve a cross reference to the following elements:

* Section (ID or block anchor)
* Block (ID or block anchor)
* Block macro (ID)
* Inline anchor anywhere in a paragraph
* Inline anchor at the start of a list item or table cell
* Bibliography anchor in a bibliography list

Note that the processor cannot resolve the ID assigned to a span of formatted text.
If the cross reference cannot be resolved, and verbose mode is enabled, the AsciiDoc processor issues a warning about a possible invalid reference.
In this case, the output document will reference the target blindly, so it's possible it will still function.

"#
);

// The `<<id>>` shorthand links to the target anchor, using the target's reftext
// (its section title) as the link text.
#[test]
fn internal_reference_by_id() {
    verifies!(
        r#"
You create a cross reference by enclosing the ID of the target block or section (or the path of another document with an optional anchor) in double angled brackets.

.Cross reference using the ID of the target section
[source#ex-section]
----
include::example$xref.adoc[tag=base]
----

The result of <<ex-section>> is displayed below.

====
include::example$xref.adoc[tag=base]
====

"#
    );

    // The `tag=base` snippet, with the `[#anchors]` section it targets.
    let output = convert(
        "The section <<anchors>> describes how automatic anchors work.\n\n\
         [#anchors]\n== Automatic anchors\n\nBody.",
    );
    assert!(output.contains(r##"<a href="#anchors">Automatic anchors</a>"##));
}

non_normative!(
    r#"
=== Explicit link text

Converters usually use the reftext of the target as the default text of the link.
When the document is parsed, attribute references in the reftext are substituted immediately.
When the reftext is displayed, additional reftext substitutions are applied to the text (specialchars, quotes, and replacements).

"#
);

// Text after the comma in `<<id,text>>` overrides the target's reftext.
#[test]
fn explicit_link_text() {
    verifies!(
        r#"
You can override the reftext of the target by specifying alternative text at the location of the cross reference.
After the ID, add a comma and then enter the custom text you want the cross reference to display.

.Cross reference with custom xreflabel text
[source#ex-custom]
----
include::example$xref.adoc[tag=text]
----

In this case, the target will be assumed to be an ID within the same document even if it contains a dot (`.`).

"#
    );

    // The `tag=text` snippet, with the section it targets.
    let output = convert(
        "Learn how to <<link-macro-attributes,use attributes within the link macro>>.\n\n\
         [#link-macro-attributes]\n== Link macro attributes\n\nBody.",
    );
    assert!(output.contains(
        r##"<a href="#link-macro-attributes">use attributes within the link macro</a>"##
    ));
}

// The inline `xref:target[text]` macro is an alternative to the shorthand.
#[test]
fn xref_macro() {
    verifies!(
        r#"
You can also use the inline xref macro as an alternative to the xref shorthand.

.Inline xref macro
[source]
----
include::example$xref.adoc[tag=xref-macro]
----

However, it's best to reserve the use of the xref macro for creating interdocument cross references.

When using the xref macro, if the target contains a dot (`.`), it will be treated as a reference to another document, not an ID within the same document.
If the intention is to link to an ID within the same document, the target must be proceeded by a hash (`#`).

"#
    );

    // The `tag=xref-macro` snippet, with the section it targets.
    let output = convert(
        "Learn how to xref:link-macro-attributes[use attributes within the link macro].\n\n\
         [#link-macro-attributes]\n== Link macro attributes\n\nBody.",
    );
    assert!(output.contains(
        r##"<a href="#link-macro-attributes">use attributes within the link macro</a>"##
    ));
}

// A natural cross reference targets a section by its title rather than its ID.
#[test]
fn natural_cross_reference() {
    verifies!(
        r#"
=== Natural cross reference

You can also create a reference to a block or section using its title rather than its ID.
This type of reference is referred to as a [.term]*natural cross reference*.
The title must contain at least one space character or contain at least one uppercase letter.
//(If you are using Ruby < 2.4, that uppercase letter is restricted to the basic Latin charset).

.Cross reference using a section's title
[source#ex-title]
----
include::example$xref.adoc[tag=xref-title]
----

"#
    );

    // The `tag=xref-title` snippet: the reference resolves through the section's
    // title to its auto-generated ID.
    let output =
        convert("Refer to <<Internal Cross References>>.\n\n== Internal Cross References\n\nBody.");
    assert!(
        output.contains(r##"<a href="#_internal_cross_references">Internal Cross References</a>"##)
    );
}

non_normative!(
    r#"
As a rule of thumb, the natural cross reference should only be used for rapid development or short-lived content.
As the content matures, you should switch to using IDs for referencing, ideally IDs which are declared explicitly.
By doing so, it ensures your references have maximum stability and are shielded against title revisions.

"#
);

// The `window` attribute on the xref macro sets the link `target`; `_blank`
// also adds `rel="noopener"`.
#[test]
fn target_a_blank_window() {
    verifies!(
        r#"
== Target a blank window

You can use the `window` attribute on the xref macro to control the link target (equivalent to the xref:link-macro-attribute-parsing.adoc#target-a-blank-window[window] attribute of the link macro).
Configuring a link that points to a location outside the current site is common practice to avoid disrupting the reader's flow.
This is a behavior that is specific to HTML output.

Most of the time, you’ll use the window attribute to target a blank window.
To enable this behavior, you set the window attribute to the special value _blank.

[source]
----
xref:page.adoc[window=_blank]
----

In the HTML output, the value of the window attribute is assigned to the target attribute on the <a> tag (e.g., target=_blank).
When the target is _blank, the processor will automatically add the `rel=noopener` attribute as well.

NOTE: The blank window shorthand, `^`, only works with the link macro.
"#
    );

    let output = convert("xref:page.adoc[window=_blank]");
    assert!(output.contains(r#"<a href="page.html" target="_blank" rel="noopener">page.html</a>"#));
}
