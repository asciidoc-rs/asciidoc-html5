//! Coverage of the AsciiDoc language description's *Discrete Headings* page.
//!
//! The page documents the `discrete` (and equivalent `float`) style, which
//! demotes a section title to a standalone heading that is not part of the
//! section hierarchy and can be nested in other blocks. This crate renders a
//! discrete heading as an `<h_n>` carrying the `discrete` (or `float`) class,
//! outside any section wrapper, so the example is verified through `convert`.
//! The title, the conceptual introduction, and the historical aside are
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/discrete-headings.adoc");

non_normative!(
    r#"
= Discrete Headings
:page-aliases: sections:discrete-titles.adoc, sections:discrete-headings.adoc

A discrete heading is declared and styled in a manner similar to that of a section title, but:

* it's not part of the section hierarchy,
* it can be nested in other blocks,
* it cannot have any child blocks,
* it's not included in the table of contents.

In other words, it's a unique block element that looks like a section title, but is not an offshoot of a section title.

The `discrete` style effectively demotes the section title to a normal heading.
Discrete headings are the closest match to headings in other markup languages such as Markdown.

"#
);

// A discrete heading: the `[discrete]` style on a section title nested in a
// sidebar renders as a heading (`<h2 class="discrete">`) inside the sidebar's
// content, rather than a section wrapper — it is not part of the section
// hierarchy.
#[test]
fn discrete_style_renders_a_heading_outside_the_section_hierarchy() {
    verifies!(
        r#"
To make a discrete heading, add the `discrete` attribute to any section title.
Here's an example of a discrete heading in use.

[source]
----
**** <.>
Discrete headings are useful for making headings inside of other blocks, like this sidebar.

[discrete] <.>
== Discrete Heading <.>

Discrete headings can be used where sections are not permitted.
****
----
<.> A delimiter line that indicates the start of a sidebar block.
<.> Set the `discrete` attribute above the section title to demote it to a discrete heading.
<.> The discrete heading is designated by one to six equal signs, just like a regular section title.

"#
    );

    let output = convert(
        "****\nDiscrete headings are useful for making headings inside of other blocks, like this sidebar.\n\n\
         [discrete]\n== Discrete Heading\n\n\
         Discrete headings can be used where sections are not permitted.\n****\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="sidebarblock"]//h2[@class="discrete"][text()="Discrete Heading"]"#,
        1,
    );
    assert_css(&output, "div.sect1", 0);
    assert_css(&output, "div.sect2", 0);
}

// The `float` style is an equivalent way to mark a discrete heading; this crate
// renders it as a heading carrying the `float` class.
#[test]
fn float_style_is_equivalent_to_discrete() {
    verifies!(
        r#"
Alternately, you may use the `float` attribute to identify a discrete heading.
"#
    );

    let output = convert("[float]\n== Floating Heading\n\nbody\n");
    assert_xpath(
        &output,
        r#"//h2[@class="float"][text()="Floating Heading"]"#,
        1,
    );
}

// Historical background: the `float` naming and its DocBook "bridgehead"
// equivalent.
non_normative!(
    r#"
In this context, the term "`float`" does not refer to a layout.
Rather, it means not bound to the section hierarchy.
The term comes from an older version of AsciiDoc, in which discrete headings were called Floating Titles.
DocBook refers to a discrete heading as a bridgehead, or free-floating heading.
"#
);
