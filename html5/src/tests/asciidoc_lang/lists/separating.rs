//! Coverage of the AsciiDoc language description's *Separating Lists* page.
//!
//! Adjacent items with the same marker join into one list even across empty
//! lines, and a different marker nests. To force two lists apart, the page
//! documents two techniques: a line comment (`//`) between them, or a block
//! attribute line (even an empty `[]`) above the second. This crate parses all
//! of these, so each claim and its example are verified through `convert`; only
//! the title and the two section headings are tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/separating.adoc");

non_normative!(
    r#"
= Separating Lists

"#
);

// The default affinity rules: items sharing a marker join into a single list
// even when empty lines separate them, while an item with a different marker
// becomes a nested list.
#[test]
fn adjacent_items_join_or_nest_by_marker() {
    verifies!(
        r#"
In AsciiDoc, lists items have natural affinity towards one another.
If adjacent lines start with the same list marker, they are joined together into the same list, even if separated by empty lines.
If the adjacent line starts with a different list marker, even if offset by empty lines, it will be placed into a nested list.

These rules make it easier to keep list items together in a single list.
However, they can present a challenge if what you want is to create separate lists.
Fortunately, there are ways to force a change to this behavior.
The techniques described on this page work for all list types.

"#
    );

    // Same marker, separated by an empty line: one list with two items.
    let joined = convert("* Apples\n\n* Oranges\n");
    assert_css(&joined, "div.ulist", 1);
    assert_css(&joined, "div.ulist > ul > li", 2);

    // Different marker: the second item nests inside the first.
    let nested = convert("* Apples\n\n- Oranges\n");
    assert_css(&nested, "div.ulist", 2);
    assert_css(&nested, "ul > li > div.ulist", 1);
}

non_normative!(
    r#"
== Using a line comment

"#
);

// A line comment (`//`) surrounded by empty lines between two lists forces them
// apart, yielding two separate list elements.
#[test]
fn a_line_comment_forces_two_lists_apart() {
    verifies!(
        r#"
To force lists apart, you can insert a line comment (i.e., `//`) surrounded by empty lines on either side between the two lists.

Here's an example that shows where to place the line comment to separate two adjacent unordered lists.
The `-` following the line comment prefix is a hint to authors to indicate that the comment line serves as an "`end of list`" marker:

----
include::example$unordered.adoc[tag=divide]
----

This technique works for separating any type of list.

"#
    );

    let output = convert("* Apples\n* Oranges\n\n//-\n\n* Walnuts\n* Almonds\n");
    assert_css(&output, "div.ulist", 2);
    assert_css(&output, "div.ulist > ul > li", 4);
}

non_normative!(
    r#"
== Using a block attribute line

"#
);

// A block attribute line (even an empty `[]`) above the second list, offset by
// an empty line, starts a new list — here separating an unordered list from an
// ordered one.
#[test]
fn a_block_attribute_line_starts_a_new_list() {
    verifies!(
        r#"
Another way to start a new list is to place a block attribute line (even an empty one) above the second list, offset by an empty line.

Here's an example that shows where to place the block attribute line to separate unordered and ordered lists that are adjacent.

----
include::example$unordered.adoc[tag=divide-alt]
----

The preceding empty line is important.
If that were not present, the ordered list would still be nested inside the ordered list.
If the second list requires block attributes, you can add them to the block attribute line.

This technique works for separating any type of list.
"#
    );

    let output = convert("* Apples\n* Oranges\n\n[]\n. Wash\n. Slice\n");
    assert_css(&output, "div.ulist", 1);
    assert_css(&output, "div.olist", 1);
    assert_css(&output, "div.ulist > ul > li", 2);
    assert_css(&output, "div.olist > ol > li", 2);
    // The ordered list is a sibling of the unordered list, not nested in it.
    assert_css(&output, "div.ulist div.olist", 0);
}
