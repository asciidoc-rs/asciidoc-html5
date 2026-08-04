//! Coverage of the AsciiDoc language description's *ID Attribute* page.
//!
//! An `id` becomes an HTML `id` attribute — on a block's wrapper `<div>`, on a
//! section heading, or, for an inline anchor, on an empty `<a id="…"></a>`
//! anchor point (or the `<span id="…">` that wraps a shorthand-tagged phrase).
//! Every syntax the page documents (shorthand hash, longhand `id=`, and the
//! `[[…]]` anchor form, on blocks, inline phrases, list items, description-list
//! terms, table cells, images, and section titles) is verified by converting
//! the shown source and checking the rendered `id` / anchor. Claims about
//! *which* IDs are accepted are verified through the rendered output too: an
//! invalid block-anchor name (empty, or containing spaces) is rejected — the
//! renderer emits no anchor and the parser records a warning — while the
//! shorthand form accepts names the double-bracket form rejects (e.g. a leading
//! digit).
//!
//! Only the introduction, the section headings, the naming-recommendation
//! prose, and a couple of tangential cautions are tracked as non-normative; the
//! naming standard itself is verified in `asciidoc-parser`, which validates the
//! raw ID character set.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/id.adoc");

non_normative!(
    r#"
= ID Attribute
:page-aliases: ids.adoc

You can assign an identifier (i.e., unique name) to a block or inline element using the `id` attribute.
The `id` attribute is a xref:positional-and-named-attributes.adoc#named[named attribute].
Its purpose is to identify the element when linking, scripting, or styling.
Thus, the identifier can only be used once in a document.

An ID:

. provides an internal link or cross reference anchor for an element
. can be used for adding additional styling to specific elements (e.g., via a CSS ID selector)

You can assign an ID to blocks using the shorthand hash (`+#+`) syntax, longhand (`id=`) syntax, or the anchor (`[[]]`) syntax.
You can assign an ID to inline elements using the shorthand hash (`+#+`) syntax or by adding an anchor adjacent to the inline element using the anchor (`[[]]`) syntax.
You can assign an ID to a table cell by using an anchor (`[[]]`) at the start of the cell.
Likewise, you can assign an ID to a list item by using an anchor (`[[]]`) at the start of the principal text.

"#
);

mod valid_id_characters {
    use asciidoc_parser::warnings::WarningType;

    use crate::{
        convert, load,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Valid ID characters

AsciiDoc does not restrict the set of characters that can be used for an ID when the ID is defined using the named `id` attribute.
"#
    );

    #[test]
    fn value_non_empty() {
        verifies!(
            r#"
All the language requires in this case is that the value be non-empty.
"#
        );

        // An empty block anchor (`[[]]`) supplies no value, so it is rejected:
        // the parser records a warning and the renderer produces no `id`.
        let doc = load("[[]]\nThis paragraph gets a lot of attention.");

        assert!(
            doc.warnings()
                .any(|w| w.warning == WarningType::EmptyBlockAnchorName),
            "an empty block anchor should be rejected with a warning",
        );

        let output = convert("[[]]\nThis paragraph gets a lot of attention.");

        assert_css(&output, "div.paragraph", 1);
        assert_css(&output, "[id]", 0);
    }

    non_normative!(
        r#"
When the ID is defined using the shorthand hash syntax or the anchor syntax, the acceptable characters is more limited (for example, spaces are not permitted).
Regardless, it's not advisable to exploit the ability to use any characters the AsciiDoc syntax allows.
The reason to be cautious is because the ID is passed through to the output, and not all output formats afford the same latitude.
For example, XML is far more restrictive about which characters are permitted in an ID value.

"#
    );

    #[test]
    fn check_valid_xml_name() {
        verifies!(
            r#"
To ensure portability of your IDs, it's best to conform to a universal standard.
The standard we recommend following is a https://www.w3.org/TR/REC-xml/#NT-Name[Name] value as defined by the XML specification.
At a high level, the first character of a Name must be a letter, colon, or underscore and the optional following characters must be a letter, colon, underscore, hyphen, period, or digit.
You should not use any space characters in an ID.
Starting the ID with a digit is less likely to be problematic, but still best to avoid.
It's best to use lowercase letters whenever possible as this solves portability problem when using case-insensitive platforms.

When the AsciiDoc processor auto-generates IDs for section titles and discrete headings, it adheres to this standard.

Here are examples of valid IDs (according to the recommendations above):

[listing]
----
install
data-structures
error-handling
subject-and-body
unset_an_attribute
----

Here are examples of invalid IDs:

[listing]
----
install the gem
3 blind mice
-about-the-author
----

"#
        );

        // A valid ID (here `data-structures` from the list above) is accepted:
        // the block-anchor form assigns it as the paragraph's `id`.
        let output = convert("[[data-structures]]\nThis paragraph gets a lot of attention.");
        assert_css(&output, r#"div.paragraph[id="data-structures"]"#, 1);

        // An invalid ID containing spaces (`3 blind mice`) is rejected: the
        // parser records a warning and the anchor is left as literal text.
        let doc = load("[[3 blind mice]]\nThis paragraph gets a lot of attention.");
        assert!(
            doc.warnings()
                .any(|w| w.warning == WarningType::InvalidBlockAnchorName),
            "a block anchor containing spaces should be rejected with a warning",
        );

        let output = convert("[[3 blind mice]]\nThis paragraph gets a lot of attention.");
        assert_css(&output, "[id]", 0);
        assert!(
            output.contains("[[3 blind mice]]"),
            "the rejected anchor should pass through as literal text",
        );
    }

    non_normative!(
        r#"
////
BlockId

NOTE: Section pending
////

"#
    );
}

mod block_assignment {
    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Block assignment

// tag::bl[]
You can assign an ID to a block using the shorthand syntax, the longhand syntax, or a block anchor.

"#
    );

    #[test]
    fn shorthand_syntax() {
        verifies!(
            r#"
In the shorthand syntax, you prefix the name with a hash (`#`) in the first position attribute.

[source]
----
[#goals]
* Goal 1
* Goal 2
----

"#
        );

        let output = convert("[#goals]\n* Goal 1\n* Goal 2");

        // The `#` shorthand sets the list block's `id`.
        assert_css(&output, "div.ulist#goals", 1);
    }

    #[test]
    fn longhand_syntax() {
        verifies!(
            r#"
In the longhand syntax, you use a standard named attribute.

[source]
----
[id=goals]
* Goal 1
* Goal 2
----

"#
        );

        let output = convert("[id=goals]\n* Goal 1\n* Goal 2");

        // The longhand `id=` produces the same `id` as the shorthand.
        assert_css(&output, "div.ulist#goals", 1);
    }

    #[test]
    fn block_anchor_syntax() {
        verifies!(
            r#"
In the block anchor syntax, you surround the name with double square brackets:

[source]
----
[[goals]]
* Goal 1
* Goal 2
----

"#
        );

        let output = convert("[[goals]]\n* Goal 1\n* Goal 2");

        // The `[[…]]` block anchor produces the same `id` as the shorthand.
        assert_css(&output, "div.ulist#goals", 1);
    }

    #[test]
    fn block_style_with_shorthand_syntax() {
        verifies!(
            r#"
Let's say you want to create a blockquote from an open block and assign it an ID and xref:role.adoc[role].
You add `quote` (the block style) in front of the `#` (the ID) in the first attribute position, as this example shows:

[source]
----
[quote.movie#roads,Dr. Emmett Brown]
____
Roads? Where we're going, we don't need roads.
____
----

"#
        );

        let output = convert(
            "[quote.movie#roads,Dr. Emmett Brown]\n____\nRoads? Where we're going, we don't need roads.\n____",
        );

        // The style (`quote`), role (`.movie`), and ID (`#roads`) in the first
        // shorthand position become the block context, a class, and the `id`.
        assert_css(&output, "div.quoteblock.movie#roads", 1);
    }

    #[test]
    fn shorthand_id_role_order_independent() {
        verifies!(
            r#"
TIP: The order of ID and role values in the shorthand syntax does not matter.

"#
        );

        // Whether the role precedes the ID (`.movie#roads`) or follows it
        // (`#roads.movie`), the rendered class and `id` are the same.
        let role_then_id = convert("[quote.movie#roads]\n____\nRoads?\n____");
        assert_css(&role_then_id, "div.quoteblock.movie#roads", 1);

        let id_then_role = convert("[quote#roads.movie]\n____\nRoads?\n____");
        assert_css(&id_then_role, "div.quoteblock.movie#roads", 1);
    }

    #[test]
    fn block_id_containing_dot() {
        verifies!(
            r#"
CAUTION: If the ID contains a `.`, you must define it using either a longhand assignment (e.g., `id=classname.propertyname`) or the anchor shorthand (e.g., `+[[classname.propertyname]]+`).
This is necessary since the `.` character in the shorthand syntax is the delimiter for a role, and thus gets misinterpreted as such.
// end::bl[]

"#
        );

        // The longhand and anchor forms keep the dot in the `id`.
        let longhand = convert("[id=classname.propertyname]\nprop");
        assert_css(
            &longhand,
            r#"div.paragraph[id="classname.propertyname"]"#,
            1,
        );

        let anchor = convert("[[classname.propertyname]]\nprop");
        assert_css(&anchor, r#"div.paragraph[id="classname.propertyname"]"#, 1);

        // In the shorthand form the `.` is misinterpreted as the role delimiter:
        // the ID becomes `classname` and `propertyname` becomes a role (class).
        let shorthand = convert("[#classname.propertyname]\nprop");
        assert_css(&shorthand, "div.paragraph.propertyname#classname", 1);
    }
}

mod inline_assignment {
    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Inline assignment

"#
    );

    #[test]
    fn inline_shorthand_syntax() {
        verifies!(
            r#"
// tag::in[]
The id (`#`) shorthand can be used on inline quoted text.

.Quoted text with ID assignment using shorthand syntax
----
[#free_the_world]#free the world#
----

"#
        );

        let output = convert("[#free_the_world]#free the world#");

        // The `#` shorthand on quoted text becomes an `id` on the `<span>`.
        assert_css(&output, "span#free_the_world", 1);
    }

    #[test]
    fn inline_anchor_syntax() {
        verifies!(
            r#"
.General text with preceding ID assignment using inline anchor syntax
----
[[free_the_world]]free the world
----
// end::in[]

"#
        );

        let output = convert("[[free_the_world]]free the world");

        // The `[[…]]` anchor becomes an empty anchor point preceding the text.
        assert_css(&output, "a#free_the_world", 1);
    }
}

mod anchor {
    use crate::{
        convert, load,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
[#anchor]
== Use an ID as an anchor

An anchor (aka ID) can be defined almost anywhere in the document, including on a section title, on a discrete heading, on a paragraph, on an image, on a delimited block, on an inline phrase, and so forth.
The anchor is declared by enclosing a _valid_ XML Name in double square brackets (e.g., `+[[idname]]+`) or using the shorthand ID syntax (e.g., `[#idname]`) at the start of an attribute list.
The shorthand form is the preferred syntax.

"#
    );

    #[test]
    fn double_square_bracket_rules() {
        verifies!(
            r#"
The double square bracket form requires the ID to start with a letter, an underscore, or a colon, ensuring the ID is portable.
According to the https://www.w3.org/TR/REC-xml/#NT-Name[XML Name] rules, a portable ID may not begin with a number, even though a number is allowed elsewhere in the name.
"#
        );

        // A letter, an underscore, or a colon start is accepted by the
        // double-bracket form, producing an anchor point.
        let letter = convert("[[start_with_letter]]#free the world#");
        assert_css(&letter, r#"a[id="start_with_letter"]"#, 1);

        let underscore = convert("[[_start_with_underscore]]#free the world#");
        assert_css(&underscore, r#"a[id="_start_with_underscore"]"#, 1);

        let colon = convert("[[:start_with_colon]]#free the world#");
        assert_css(&colon, r#"a[id=":start_with_colon"]"#, 1);

        // A leading number is rejected by the double-bracket form: no anchor is
        // produced and the `[[…]]` passes through as literal text.
        let number = convert("[[1start_with_number]]#free the world#");
        assert_css(&number, "a", 0);
        assert!(number.contains("[[1start_with_number]]"));
    }

    #[test]
    fn shorthand_form() {
        verifies!(
            r#"
The shorthand form in an attribute list does not impose this restriction.

"#
        );

        // The shorthand form accepts a leading number, which the double-bracket
        // form rejects, and emits it as the `<span>`'s `id`.
        let output = convert("[#1start_with_number]#free the world#");
        assert_css(&output, r#"span[id="1start_with_number"]"#, 1);
    }

    #[test]
    fn block_element_shorthand_syntax() {
        verifies!(
            r#"
=== On block element

To reference a block element, you must assign an ID to that block.
You can define an ID using the shorthand syntax:

.Assign an ID to a paragraph using shorthand syntax
[source]
----
include::example$id.adoc[tag=block-id-shorthand]
----

"#
        );

        // example$id.adoc[tag=block-id-shorthand]:
        //   [#notice]
        //   This paragraph gets a lot of attention.
        let output = convert("[#notice]\nThis paragraph gets a lot of attention.");

        assert_css(&output, "div.paragraph#notice", 1);
        assert!(load("[#notice]\nThis paragraph gets a lot of attention.")
            .catalog()
            .contains_id("notice"));
    }

    #[test]
    fn block_element_anchor_syntax() {
        verifies!(
            r#"
or you can define it using the block anchor syntax:

.Assign an ID to a paragraph using block anchor syntax
[source]
----
include::example$id.adoc[tag=block-id-brackets]
----

"#
        );

        // example$id.adoc[tag=block-id-brackets]:
        //   [[notice]]
        //   This paragraph gets a lot of attention.
        let output = convert("[[notice]]\nThis paragraph gets a lot of attention.");

        assert_css(&output, "div.paragraph#notice", 1);
        assert!(load("[[notice]]\nThis paragraph gets a lot of attention.")
            .catalog()
            .contains_id("notice"));
    }

    #[test]
    fn inline_anchor_brackets() {
        verifies!(
            r#"
=== As an inline anchor

You can also define an anchor anywhere in content that receives normal substitutions (specifically the macros substitution).
You can enclose the ID in double square brackets:

.Define an inline anchor
[source]
----
include::example$id.adoc[tag=anchor-brackets]
----

"#
        );

        // example$id.adoc[tag=anchor-brackets]:
        //   [[bookmark-a]]Inline anchors make arbitrary content referenceable.
        let output = convert("[[bookmark-a]]Inline anchors make arbitrary content referenceable.");

        assert_css(&output, r#"a[id="bookmark-a"]"#, 1);
    }

    #[test]
    fn inline_anchor_shorthand() {
        verifies!(
            r#"
or using the shorthand ID syntax.

.Define an inline anchor using shorthand syntax
[source]
----
include::example$id.adoc[tag=anchor-shorthand]
----

"#
        );

        // example$id.adoc[tag=anchor-shorthand]:
        //   [#bookmark-b]#Inline anchors can be applied to a phrase like this one.#
        let output =
            convert("[#bookmark-b]#Inline anchors can be applied to a phrase like this one.#");

        assert_css(&output, r#"span[id="bookmark-b"]"#, 1);
    }

    #[test]
    fn invalid_anchor_before_list_item() {
        verifies!(
            r#"
=== On a list item

In addition to being able to define anchors on sections and blocks, anchors can be defined inline wherever you can type normal text (anchors are a macros substitution).
The anchors in the text get replaced with invisible anchor points in the output.

For example, you would not put an anchor in front of a list item:

.*Invalid* position for an anchor ID in front of a list item
[source]
----
include::example$id.adoc[tag=anchor-wrong]
----

"#
        );

        // example$id.adoc[tag=anchor-wrong]:
        //   [[anchor-point]]* list item with invalid anchor
        //
        // An anchor in front of the list marker does not make a list item: the
        // line renders as an ordinary paragraph, the inline anchor point is
        // emitted, and the `*` is left as literal text.
        let output = convert("[[anchor-point]]* list item with invalid anchor");

        assert_css(&output, "div.ulist", 0);
        assert_css(&output, r#"div.paragraph a[id="anchor-point"]"#, 1);
        assert!(output.contains("* list item with invalid anchor"));
    }

    #[test]
    fn anchor_on_list_item() {
        verifies!(
            r#"
Instead, you would put it at the start of the text of the list item:

.Define an inline anchor on a list item
[source]
----
include::example$id.adoc[tag=anchor-list-item]
----

"#
        );

        // example$id.adoc[tag=anchor-list-item]:
        //   * First item
        //   * [[step2]]Second item
        //   * Third item
        //
        // The anchor at the start of the principal text becomes the list item's
        // referenceable anchor point.
        let output = convert("* First item\n* [[step2]]Second item\n* Third item");

        assert_css(&output, r#"li a[id="step2"]"#, 1);
        assert!(load("* First item\n* [[step2]]Second item\n* Third item")
            .catalog()
            .contains_id("step2"));
    }

    #[test]
    fn anchor_on_description_list_item() {
        verifies!(
            r#"
For a description list, the anchor must be placed at the start of the term:

.Define an inline anchor on a description list item
[source]
----
include::example$id.adoc[tag=anchor-dlist-item]
----

"#
        );

        // example$id.adoc[tag=anchor-dlist-item]:
        //   [[cpu,CPU]]Central Processing Unit (CPU)::
        //   The brain of the computer.
        //
        //   [[hard-drive]]Hard drive::
        //   Permanent storage for operating system and/or user files.
        //
        // Each anchor at the start of a term becomes an anchor point on the
        // `<dt>`, and its reftext (the explicit `CPU`, else the term text) is
        // registered in the catalog.
        let source = "[[cpu,CPU]]Central Processing Unit (CPU)::\nThe brain of the computer.\n\n[[hard-drive]]Hard drive::\nPermanent storage for operating system and/or user files.";
        let output = convert(source);

        assert_css(&output, r#"dt a[id="cpu"]"#, 1);
        assert_css(&output, r#"dt a[id="hard-drive"]"#, 1);

        let doc = load(source);
        assert_eq!(
            doc.catalog().get_ref("cpu").unwrap().reftext.as_deref(),
            Some("CPU"),
        );
        assert_eq!(
            doc.catalog()
                .get_ref("hard-drive")
                .unwrap()
                .reftext
                .as_deref(),
            Some("Hard drive"),
        );
    }

    #[test]
    fn multiple_anchors_on_list_item() {
        verifies!(
            r#"
You can add multiple anchors to a list item or description list term.
However, only the first anchor is registered for use as an xref within the document.
The remaining anchors are auxiliary and are used for making deep links (i.e., accessible from a URL fragment).

"#
        );

        // Each anchor renders as its own invisible anchor point, and both are
        // registered so either can be the target of a deep link.
        let output = convert("* [[a1]][[a2]]Item");

        assert_css(&output, r#"li a[id="a1"]"#, 1);
        assert_css(&output, r#"li a[id="a2"]"#, 1);

        let doc = load("* [[a1]][[a2]]Item");
        assert!(doc.catalog().contains_id("a1"));
        assert!(doc.catalog().contains_id("a2"));
    }

    #[test]
    fn anchor_on_table_cell() {
        verifies!(
            r#"
=== On a table cell

You can assign an ID to a table cell by placing an inline anchor at the start of the cell.

.Assigning an ID to a table cell using an inline anchor
[source]
----
|===
|[[my_cell]]The table cell I want to jump to.
|===
----

"#
        );

        // The inline anchor at the start of the cell becomes an anchor point in
        // the cell's paragraph and is registered as a referenceable ID.
        let source = "|===\n|[[my_cell]]The table cell I want to jump to.\n|===";
        let output = convert(source);

        assert_css(&output, r#"td a[id="my_cell"]"#, 1);
        assert!(load(source).catalog().contains_id("my_cell"));
    }

    #[test]
    fn inline_image_adjacent_anchor_shorthand() {
        verifies!(
            r#"
=== On an inline image

You cannot currently define an ID on an inline image.
Instead you need to place an inline anchor adjacent to it.

.Placing an inline anchor adjacent to an inline image using shorthand
[source]
----
include::example$id.adoc[tag=inline-anchor-brackets]
----

"#
        );

        // example$id.adoc[tag=inline-anchor-brackets]:
        //   [[tiger-image]]image:tiger.png[Image of a tiger]
        //
        // The adjacent anchor renders as an anchor point immediately before the
        // inline image.
        let output = convert("[[tiger-image]]image:tiger.png[Image of a tiger]");

        assert_css(&output, r#"a[id="tiger-image"]"#, 1);
        assert_css(&output, r#"span.image img[src="tiger.png"]"#, 1);
        assert!(load("[[tiger-image]]image:tiger.png[Image of a tiger]")
            .catalog()
            .contains_id("tiger-image"));
    }

    #[test]
    fn inline_image_adjacent_anchor_macro() {
        verifies!(
            r#"
Instead of the shorthand form, you can use the macro `anchor` to achieve the same goal.

.Placing an inline anchor adjacent to an inline image using a macro
[source]
----
include::example$id.adoc[tag=inline-anchor-macro]
----

"#
        );

        // example$id.adoc[tag=inline-anchor-macro]:
        //   anchor:tiger-image[]image:tiger.png[Image of a tiger]
        //
        // The `anchor` macro produces the same anchor point and registration as
        // the shorthand form.
        let output = convert("anchor:tiger-image[]image:tiger.png[Image of a tiger]");

        assert_css(&output, r#"a[id="tiger-image"]"#, 1);
        assert_css(&output, r#"span.image img[src="tiger.png"]"#, 1);
        assert!(
            load("anchor:tiger-image[]image:tiger.png[Image of a tiger]")
                .catalog()
                .contains_id("tiger-image")
        );
    }
}

mod add_additional_anchors_to_a_section {
    use crate::{
        convert, load,
        tests::{assert_html::assert_css, sdd::*},
    };

    #[test]
    fn section_extra_anchors() {
        verifies!(
            r#"
== Add additional anchors to a section

To add additional anchors to a section (with or without an autogenerated ID), place the anchors in front of the title (without any spaces).

.Add additional anchors to a section using inline anchors
[source]
----
include::example$id.adoc[tag=anchor-header-extra]
----

"#
        );

        // example$id.adoc[tag=anchor-header-extra]:
        //   [#version-4_9]
        //   === [[current]][[latest]]Version 4.9
        //
        // The section keeps its own `id` on the heading, and each anchor placed
        // in front of the title becomes an additional anchor point inside it.
        let output = convert("[#version-4_9]\n=== [[current]][[latest]]Version 4.9");

        assert_css(&output, "h3#version-4_9", 1);
        assert_css(&output, r#"h3 a[id="current"]"#, 1);
        assert_css(&output, r#"h3 a[id="latest"]"#, 1);

        let doc = load("[#version-4_9]\n=== [[current]][[latest]]Version 4.9");
        assert!(doc.catalog().contains_id("version-4_9"));
        assert!(doc.catalog().contains_id("current"));
        assert!(doc.catalog().contains_id("latest"));
    }

    non_normative!(
        r#"
CAUTION: You cannot use inline anchors in a section title to make internal references to that section.
The processor will flag these as possible invalid references.
These additional anchors are only intended for making deep links using an alternate ID.

Remember that inline anchors are discovered wherever the macros substitution is applied (e.g., paragraph text).
If text content doesn't belong somewhere, neither does an inline anchor point.

"#
    );
}

mod customize_automatic_xreftext {
    use crate::{
        convert, load,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Customize automatic xreftext

It's possible to customize the text that will be used in the cross reference link (called `xreflabel`).
If not defined, the AsciiDoc processor does it best to find suitable text (the solution differs from case to case).
In case of an image, the image caption will be used.
In case of a section header, the text of the section's title will be used.

To define the `xreflabel`, add it in the anchor definition right after the ID (separated by a comma).

"#
    );

    #[test]
    fn anchor_xreflabel() {
        verifies!(
            r#"
.An anchor ID with a defined xreflabel. The caption will not be used as link text.
[source]
----
include::example$id.adoc[tag=anchor-xreflabel]
----
"#
        );

        // example$id.adoc[tag=anchor-xreflabel]:
        //   [[tiger-image,Image of a tiger]]
        //   .This image represents a Bengal tiger also called the Indian tiger
        //   image::tiger.png[]
        //
        // The reftext supplied after the ID (the `xreflabel`) is used as the
        // cross-reference link text in preference to the image caption.
        let source = "[[tiger-image,Image of a tiger]]\n.This image represents a Bengal tiger also called the Indian tiger\nimage::tiger.png[]\n\nSee <<tiger-image>>.";
        let output = convert(source);

        assert_css(&output, r##"a[href="#tiger-image"]"##, 1);
        assert!(output.contains(r##"<a href="#tiger-image">Image of a tiger</a>"##));

        assert_eq!(
            load(source)
                .catalog()
                .get_ref("tiger-image")
                .unwrap()
                .reftext
                .as_deref(),
            Some("Image of a tiger"),
        );
    }
}
