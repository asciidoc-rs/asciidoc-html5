//! Coverage of the AsciiDoc language description's *Assign an ID* page.
//!
//! The page documents assigning an ID to a block with the `[#id]` shorthand,
//! including the order of the style name, ID, and positional attributes on a
//! blockquote. This crate renders the ID as the block wrapper's `id` attribute,
//! so each example is verified through `convert`. The title, the introductions,
//! the section headings, and the trailing commented-out draft section are
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/assign-id.adoc");

non_normative!(
    r#"
= Assign an ID

You can assign an ID to any block using an attribute list.
Once you've assigned an ID to a block, you can use that ID to link to it using a cross reference.

== Block ID syntax

"#
);

// The `[#id]` shorthand: an ID prefixed with `#` in a block's attribute list
// becomes the `id` attribute on the block's wrapper div.
#[test]
fn hash_shorthand_assigns_the_block_id() {
    verifies!(
        r#"
An ID is assigned to a block by prefixing the ID value with a hash (`#`) and placing it in the block's attribute list.

----
[#the-id-of-this-block]
====
Content of delimited example block
====
----

Let's go through some examples of assigning an ID to a block with several attributes, a title, and delimiters.

"#
    );

    let output =
        convert("[#the-id-of-this-block]\n====\nContent of delimited example block\n====\n");
    assert_css(&output, "div#the-id-of-this-block.exampleblock", 1);
}

non_normative!(
    r#"
== Assign an ID to a block with attributes

"#
);

// On a styled block, the style name comes first in the attribute list and the
// ID attaches directly to the end of the style name (`[quote#roads]`). The
// positional attribution attributes of the `quote` style follow, producing the
// attribution and citation. In every form the ID lands on the `quoteblock`
// wrapper.
#[test]
fn style_name_precedes_id_with_positional_attribution() {
    verifies!(
        r#"
In this section, we'll assign an ID to this blockquote:

[quote#roads,Dr. Emmett Brown,Back to the Future]
Roads? Where we're going, we don't need roads.

When the style attribute is explicitly assigned to a block, the style name is always placed in the first position of the attribute list.
Then, the ID is attached directly to the end of the style name.

The blockquote with an assigned style and ID in <<ex-style-id>> shows this order of attributes.


.Assign a style and ID to a block
[#ex-style-id]
----
[quote#roads]
Roads? Where we're going, we don't need roads.
----

Since <<ex-style-id>> is a blockquote it should have some attribution and citation information.
In <<ex-cite>>, let's attribute this quote to its speaker and original context using the positional attribution attributes that are built into the `quote` style.

.Assign a style, ID, and positional attributes to a block
[#ex-cite]
----
[quote#roads,Dr. Emmett Brown,Back to the Future]
Roads? Where we're going, we don't need roads.
----

Except when the `role` and `options` attributes are assigned values using their shorthand syntax (`.` and `%`, respectively), all other block attributes are typically separated by commas (`,`).

"#
    );

    // Style name, ID, and positional attribution together.
    let full = convert(
        "[quote#roads,Dr. Emmett Brown,Back to the Future]\n\
         Roads? Where we're going, we don't need roads.\n",
    );
    assert_css(&full, "div#roads.quoteblock", 1);
    assert_xpath(
        &full,
        r#"//div[@id="roads"]//div[@class="attribution"]/cite[text()="Back to the Future"]"#,
        1,
    );

    // Style name and ID only: the ID still lands on the wrapper, with no
    // attribution.
    let style_id = convert("[quote#roads]\nRoads? Where we're going, we don't need roads.\n");
    assert_css(&style_id, "div#roads.quoteblock", 1);
    assert_css(&style_id, "div#roads.quoteblock div.attribution", 0);
}

// A commented-out draft section (delimited by `////`) that is not rendered.
non_normative!(
    r#"
////

In addition to a title, a block can be assigned additional metadata including:

* ID (xref:attributes:id.adoc#anchor[anchor])
* Block style (first positional attribute)
* Block attributes

Here's an example of a quote block with metadata:

----
include::example$block.adoc[tag=meta-co]
----
<1> Title: Gettysburg Address
<2> ID: gettysburg
<3> Block name: quote
<4> attribution: Abraham Lincoln (Named block attribute)
<5> citetitle: Address delivered at the dedication of the Cemetery at Gettysburg (Named block attribute)

TIP: A block can have multiple block attribute lines.
The attributes will be aggregated.
If there is a name conflict, the last attribute defined wins.

Some metadata is used as supplementary content, such as the title, whereas other metadata controls how the block is converted, such as the block name.
////
"#
);
