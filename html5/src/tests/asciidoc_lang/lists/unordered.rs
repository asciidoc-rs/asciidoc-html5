//! Coverage of the AsciiDoc language description's *Unordered Lists* page.
//!
//! An unordered list marks items with `*` or `-`; adjacent items join, a
//! different or longer marker nests, and a leading `.` line adds a title. The
//! wrapper is `<div class="ulist"><ul>` with one `<li>` per item. A block style
//! (`square`, `circle`, `disc`, `none`, `no-bullet`, `unstyled`) names the
//! marker as the `<ul>`'s `class`, and once set it is inherited by nested lists
//! (the alternation the renderer's CSS otherwise supplies stops). This crate
//! renders every one of those forms, so each section's example and claims are
//! verified through `convert`.
//!
//! The visual marker glyphs themselves (disc/circle/square) are applied by CSS,
//! not named in the HTML, so lines describing that rendered appearance — and
//! the DocBook mapping this crate does not target — are tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/unordered.adoc");

non_normative!(
    r#"
= Unordered Lists
:keywords: bulleted list

You can make unordered lists in AsciiDoc by starting lines with a designated marker.
An unordered list is a list with items prefixed with symbol, such as a disc (aka bullet).

AsciiDoc builds on the well-established convention of using either an asterisk or hyphen to identify a list item.
Adjacent list items are joined into a single list.
Unordered lists can be nested by varying the marker character or length (asterisk only).
List items may contain attached blocks.
They can also be interleaved with other types of lists.

== Basic unordered list

"#
);

// The asterisk (`*`) marks an unordered list item. Each item becomes a `<li>`
// inside `<div class="ulist"><ul>`. An empty line between items is permitted
// but not required — either way the items stay in one list.
#[test]
fn basic_unordered_list_marks_items_with_an_asterisk() {
    verifies!(
        r#"
In the example below, each list item is marked using an asterisk (`+*+`), the AsciiDoc syntax specifying an unordered list item.

----
include::example$unordered.adoc[tag=base]
----

A list item's first line of text must be offset from the marker (`+*+`) by at least one space.
Empty lines are required before and after a list.
Additionally, empty lines are permitted, but not required, between list items.

.Rendered unordered list
====
include::example$unordered.adoc[tag=base]
====

"#
    );

    let output = convert("* Edgar Allan Poe\n* Sheri S. Tepper\n* Bill Bryson\n");
    assert_css(&output, "div.ulist > ul > li", 3);

    // Empty lines between items are permitted but not required: both forms
    // produce a single list of two items.
    assert_css(&convert("* one\n* two\n"), "div.ulist > ul > li", 2);
    assert_css(&convert("* one\n\n* two\n"), "div.ulist > ul > li", 2);
}

// A line beginning with `.` immediately before the list supplies its title,
// rendered as `<div class="title">` inside the list wrapper.
#[test]
fn a_leading_dot_line_titles_the_list() {
    verifies!(
        r#"
You can add a title to a list by prefixing the title with a period (`.`).

----
include::example$unordered.adoc[tag=base-t]
----

.Rendered unordered list with a title
====
include::example$unordered.adoc[tag=base-t]
====

"#
    );

    let output = convert(
        ".Kizmet's Favorite Authors\n* Edgar Allan Poe\n* Sheri S. Tepper\n* Bill Bryson\n",
    );
    assert_css(&output, "div.ulist > div.title", 1);
    assert_xpath(
        &output,
        r#"//div[@class="ulist"]/div[@class="title"][contains(text(), "Favorite Authors")]"#,
        1,
    );
}

// The hyphen (`-`) is an alternative single-level marker; it produces the same
// `<div class="ulist"><ul>` as the asterisk.
#[test]
fn a_hyphen_also_marks_list_items() {
    verifies!(
        r#"
Was your instinct to use a hyphen (`-`) instead of an asterisk to mark list items?
Guess what?
That works too!

----
include::example$unordered.adoc[tag=base-alt]
----

"#
    );

    let output = convert("- Edgar Allan Poe\n- Sheri S. Tepper\n- Bill Bryson\n");
    assert_css(&output, "div.ulist > ul > li", 3);
}

non_normative!(
    r#"
You should reserve the hyphen for lists that only have a single level because the hyphen marker (`-`) doesn't work for nested lists.
Now that we've mentioned nested lists, let's go to the next section and learn how to create lists with multiple levels.

"#
);

// Forcing adjacent lists apart with a line comment: a `//` between two lists,
// surrounded by empty lines, yields two separate `<div class="ulist">`
// elements instead of one fused list.
#[test]
fn a_line_comment_forces_adjacent_lists_apart() {
    verifies!(
        r#"
[#separating-lists]
.Forcing lists apart
****
If you have adjacent lists, they have the tendency to want to fuse together.
To force lists apart, insert a line comment (`//`) surrounded by empty lines between the two lists.
Here's an example, where the `-` text in the line comment indicates the line serves as an "`end of list`" marker:

----
include::example$unordered.adoc[tag=divide]
----

This technique works for all list types.
See xref:separating.adoc[] for more details.
****

"#
    );

    let output = convert("* Apples\n* Oranges\n\n//-\n\n* Walnuts\n* Almonds\n");
    assert_css(&output, "div.ulist", 2);
    assert_css(&output, "div.ulist > ul > li", 4);
}

non_normative!(
    r#"
== Nested unordered list

"#
);

// Adding another asterisk to the marker nests an item. Each nested list is a
// `<div class="ulist">` inside its parent `<li>`.
#[test]
fn adding_an_asterisk_nests_an_item() {
    verifies!(
        r#"
To nest an item, just add another asterisk (`+*+`) to the marker.
Continue doing this for each subsequent level.

----
include::example$unordered.adoc[tag=nest]
----

.Rendered nested, unordered list
====
include::example$unordered.adoc[tag=nest]
====

"#
    );

    let output = convert(
        ".Possible DefOps manual locations\n\
         * West wood maze\n** Maze heart\n*** Reflection pool\n** Secret exit\n\
         * Untracked file in git repository\n",
    );
    // Two top-level items, one carrying a nested list.
    assert_css(&output, ".ulist:root > ul > li", 2);
    // Second-level nesting (Maze heart, Secret exit) and third-level nesting
    // (Reflection pool) each sit in a fresh `div.ulist` inside their parent
    // `<li>`.
    assert_css(&output, ".ulist:root > ul > li > div.ulist > ul > li", 2);
    assert_css(
        &output,
        ".ulist:root > ul > li > div.ulist > ul > li > div.ulist > ul > li",
        1,
    );
    assert_xpath(&output, r#"//li/p[text()="Maze heart"]"#, 1);
    assert_xpath(&output, r#"//li/p[text()="Reflection pool"]"#, 1);
}

non_normative!(
    r#"
If you prefer, you can indent the marker an arbitrary number of spaces from the left margin.
The indentation is not significant and may aid in visualizing the nesting level.

"#
);

// Unordered lists nest to any depth: the six-level example produces six nested
// `<ul>` elements.
#[test]
fn unordered_lists_nest_to_any_depth() {
    verifies!(
        r#"
You can nest unordered lists to any depth.
Keep in mind, however, that some interfaces will begin flattening lists after a certain depth.
For instance, GitHub starts flattening list after 10 levels of nesting.

----
include::example$unordered.adoc[tag=max]
----

[#ex-deep]
.Unordered lists can be nested to any depth
====
include::example$unordered.adoc[tag=max]
====

"#
    );

    let output = convert(
        "* Level 1 list item\n** Level 2 list item\n*** Level 3 list item\n\
         **** Level 4 list item\n***** Level 5 list item\n****** etc.\n\
         * Level 1 list item\n",
    );
    // Six levels of nesting, and two items at the top level.
    assert_css(&output, "ul ul ul ul ul ul", 1);
    assert_css(&output, ".ulist:root > ul > li", 2);
}

non_normative!(
    r#"
=== Determining list depth

"#
);

// Depth is set by each unique marker encountered, not by marker length: using a
// hyphen for the second level nests it under the asterisk item, and a return to
// the asterisk marker pops back to the top level.
#[test]
fn a_new_level_is_created_for_each_unique_marker() {
    verifies!(
        r#"
While it would seem as though the number of asterisks represents the nesting level, that's not how depth is determined.
A new level is created for each unique marker encountered.
For example, you can create a second level using the hyphen marker instead of two asterisks.

.Using hyphen to mark the second level is not recommended
----
include::example$unordered.adoc[tag=nest-alt]
----

"#
    );

    let output = convert("* Level 1 list item\n- Level 2 list item\n* Level 1 list item\n");
    // Two top-level items; the first carries the hyphen-marked nested list.
    assert_css(&output, ".ulist:root > ul > li", 2);
    assert_css(
        &output,
        ".ulist:root > ul > li:first-child > div.ulist > ul > li",
        1,
    );
    assert_xpath(&output, r#"//li/p[text()="Level 2 list item"]"#, 1);
}

non_normative!(
    r#"
However, it's much more intuitive to follow the convention that the marker length (i.e., number of asterisks) equals the level of nesting.
The hyphen should only be used as the marker for the first level.

*marker length = level of nesting*

After all, we're shooting for plain text markup that is readable _as is_.

[#markers]
== Markers

When rendered, an unordered list item is designated by a leading marker (bullet) (not to be confused with the marker used to define the list).
This marker can be controlled using the list style.
If no marker is specified, a default marker will be selected by the renderer.

=== Default markers

"#
);

// The first three nesting levels default to disc/circle/square. Those glyphs
// are supplied by the renderer's CSS, not named in the output: the HTML is
// plain nested `<ul>` elements with no marker class.
#[test]
fn default_markers_are_not_named_in_the_output() {
    verifies!(
        r#"
By default, AsciiDoc assumes that the first three levels of an unordered list will be styled using the markers disc, circle, and squared when rendered.
Consider the following list:

----
include::example$unordered.adoc[tag=markers]
----

.Default alternating markers for nested lists
====
include::example$unordered.adoc[tag=markers]
====

Observe that the marker for the first level is a disc (filled circle), the second level is a circle (outline), and the third level is a square (filled).
The AsciiDoc processor does not specify these markers explicitly in the model or converted output.
Rather, these defaults are added by the renderer (e.g., CSS), adhering to a convention established by HTML.

"#
    );

    let output = convert("* disc\n** circle\n*** square\n");
    // Three nested lists, none carrying an explicit marker class.
    assert_css(&output, "ul ul ul", 1);
    assert_css(&output, "ul.square, ul.circle, ul.disc", 0);
    assert_css(
        &output,
        "div.ulist.square, div.ulist.circle, div.ulist.disc",
        0,
    );
}

non_normative!(
    r#"
Beyond the third level of nesting, the marker choice is not specified.
Typically, the renderer will continue to use the square marker, as shown in <<ex-deep,an earlier example>>.

=== Custom markers

"#
);

// Each marker block style names the marker as the `class` on both the wrapper
// `<div>` and the `<ul>`: square, circle, disc, none, no-bullet, and unstyled.
#[test]
fn a_block_style_sets_the_marker_class() {
    verifies!(
        r#"
AsciiDoc offers numerous marker styles for lists.
The list marker can be specified using the list's block style.

The unordered list marker can be set using any of the following block styles:

* square
* circle
* disc
* none or no-bullet (indented, but no bullet)
* unstyled (no indentation or bullet) (not supported in DocBook output)

NOTE: These styles are supported by the default Asciidoctor stylesheet.

When present, the style name is assigned to the unordered list element as follows:

For HTML:: the style name is assigned to the `class` attribute on the `<ul>` element.

"#
    );

    for style in ["square", "circle", "disc", "none", "no-bullet", "unstyled"] {
        let output = convert(&format!("[{style}]\n* one\n* two\n"));
        assert_css(&output, &format!("div.ulist.{style} > ul.{style}"), 1);
    }
}

// This crate targets the HTML backend; the DocBook `mark`-attribute mapping is
// out of scope, so this line is tracked as non-normative.
non_normative!(
    r#"
For DocBook:: the style name is assigned to the `mark` attribute on the `<itemizedlist>` element.

"#
);

// A worked square-marker example: the style names `square` on the wrapper and
// `<ul>`.
#[test]
fn square_style_names_the_ul_class() {
    verifies!(
        r#"
Here's an unordered list that has square markers:

----
include::example$unordered.adoc[tag=square]
----

.A list with square markers
====
include::example$unordered.adoc[tag=square]
====

"#
    );

    let output = convert("[square]\n* one\n* two\n* three\n");
    assert_css(&output, "div.ulist.square > ul.square > li", 3);
}

// Once a marker style is set, it is inherited by every nested list: setting
// `circle` on the top-level list names `circle` only on the top `<ul>`, and the
// nested lists carry no class (the renderer's CSS applies the inherited style).
#[test]
fn a_marker_style_is_inherited_by_nested_lists() {
    verifies!(
        r#"
Once the list style is set, that style is used for all nested lists until it is set again.
The assumption is that it's no longer possible to infer the alternation, so it stops.
The inherited style is not specified in the model, but rather applied by the renderer (e.g., CSS).
For example, if we set the list style to circle on the top-level list, it will be used for all levels.

----
include::example$unordered.adoc[tag=marker-lock]
----

.The list style is inherited once set
====
include::example$unordered.adoc[tag=marker-lock]
====

"#
    );

    let output = convert("[circle]\n* circles\n** all\n*** the\n**** way\n***** down\n");
    // Only the top-level list is named; the four nested lists inherit via CSS
    // and carry no class of their own.
    assert_css(&output, "div.ulist.circle > ul.circle", 1);
    assert_css(&output, "ul.circle ul.circle", 0);
    assert_css(&output, "ul.circle ul:not([class])", 4);
}

// The inherited style can be reset at any level by setting a new style on a
// nested list: `square` on the top list then `circle` deeper names each on its
// own `<ul>`, leaving the levels between them unclassed.
#[test]
fn a_marker_style_can_be_reset_at_any_level() {
    verifies!(
        r#"
The inherited style can be set or reset at any level.

----
include::example$unordered.adoc[tag=marker-override]
----

.The list style can be reset
====
include::example$unordered.adoc[tag=marker-override]
====
"#
    );

    let output =
        convert("[square]\n* squares\n** up top\n[circle]\n*** circles\n**** down below\n");
    assert_css(&output, "div.ulist.square > ul.square", 1);
    assert_css(&output, "ul.square div.ulist.circle > ul.circle", 1);
}
