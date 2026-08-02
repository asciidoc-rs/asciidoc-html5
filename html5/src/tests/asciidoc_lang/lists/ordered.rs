//! Coverage of the AsciiDoc language description's *Ordered Lists* page.
//!
//! An ordered list marks items with `.` (one dot per nesting level). The
//! processor supplies the numerals, so explicit numbers may be omitted; `<div
//! class="olist <style>"><ol class="<style>">` wraps the items. The `start`
//! attribute sets the first numeral, the `reversed` option reverses the order,
//! and a numeration block style (`arabic`, `decimal`, `loweralpha`,
//! `upperalpha`, `lowerroman`, `upperroman`, `lowergreek`) names both classes
//! and, for the alphabetic/roman keywords, an HTML `type`. This crate renders
//! every one of those forms, so each section's example and claims are verified
//! through `convert`.
//!
//! A few points describe behavior outside the parity oracle (Asciidoctor
//! 2.0.26) — explicit-numeral offsets (an Asciidoctor 2.1.0 feature),
//! sequence-warning messaging, and custom-role numeration via CSS — and are
//! tracked as non-normative with the reason noted inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/ordered.adoc");

non_normative!(
    r#"
= Ordered Lists
:keywords: numbered list

== Basic ordered list

"#
);

// The `.` marker makes an ordered list. Explicit numerals are optional: the
// processor inserts them, so both `1. 2. 3.` and bare `.` produce the same
// `<div class="olist arabic"><ol class="arabic">` of three items.
#[test]
fn basic_ordered_list_numbers_items_automatically() {
    verifies!(
        r#"
Sometimes, we need to number the items in a list.
Instinct might tell you to prefix each item with a number, like in this next list:

----
include::example$ordered.adoc[tag=base-num]
----

The above works, but since the numbering is obvious, the AsciiDoc processor will insert the numbers for you if you omit them:

----
include::example$ordered.adoc[tag=base]
----

====
include::example$ordered.adoc[tag=base]
====

"#
    );

    let bare = convert(". Protons\n. Electrons\n. Neutrons\n");
    assert_css(&bare, "div.olist.arabic > ol.arabic > li", 3);

    let numbered = convert("1. Protons\n2. Electrons\n3. Neutrons\n");
    assert_css(&numbered, "div.olist.arabic > ol.arabic > li", 3);
}

// Explicit-numeral offsets are an Asciidoctor 2.1.0 feature, and the
// sequence-mismatch warning is logger behavior; neither is exercised against
// the parity oracle (Asciidoctor 2.0.26) here. Tracked as non-normative.
non_normative!(
    r#"
If you number the ordered list explicitly, you have to manually keep the list numerals sequential.
Otherwise, you will get a warning.
This differs from other lightweight markup languages.
But there's a reason for it.

Using explicit numbering is one way to adjust the numbering offset of a list (only supported in Asciidoctor 2.1.0 or better).
For instance, you can type:

----
include::example$ordered.adoc[tag=base-num-start]
----

"#
);

// The `start` attribute sets the first numeral: `[start=4]` emits `start="4"`
// on the `<ol>`.
#[test]
fn the_start_attribute_sets_the_first_numeral() {
    verifies!(
        r#"
However, there's a simpler way to accomplish the same result without the manual effort.
You can use the `start` attribute on the list to define the number at which you want the numerals to start.

----
include::example$ordered.adoc[tag=base-start]
----

The start value is always a positive integer value, even when using a different numeration style such as loweralpha.

"#
    );

    let output = convert("[start=4]\n. Step four\n. Step five\n. Step six\n");
    assert_css(&output, r#"ol.arabic[start="4"]"#, 1);
}

non_normative!(
    r#"
.When not to use the start attribute
****
When an ordered list item contains block content--such as an image, source block, or table--you may observe that the number of the next item in the list resets to 1.
In fact, what's happened is that a new list has been started where the number resets due to a missing list continuation.

In these cases, you should not resort to using the `start` attribute to fix the numbering.
Not only does that require manual adjustment as items are added to the list, it doesn't address the underlying semantics problem, which is what is causing it to be broken.
Instead, use a list continuation between each block element you want to attach to the list item to ensure the list item is continuous.
The list continuation glues the blocks together within a given item and keeps them at the same level of indentation.

For details on how to use a list continuation, refer to the xref:continuation.adoc[] page.
****

"#
);

// The `reversed` option adds the boolean `reversed` attribute to the `<ol>`.
#[test]
fn the_reversed_option_reverses_the_list() {
    verifies!(
        r#"
To present list items in reverse order, add the `reversed` option:

----
include::example$ordered.adoc[tag=reversed]
----

====
include::example$ordered.adoc[tag=reversed]
====

"#
    );

    let output = convert("[%reversed]\n.Parts of an atom\n. Protons\n. Electrons\n. Neutrons\n");
    assert_css(&output, "ol.arabic[reversed]", 1);
    assert_css(&output, "div.olist > div.title", 1);
}

// A leading `.`-prefixed line (no space after the dot) titles the list,
// rendered as `<div class="title">`.
#[test]
fn a_leading_dot_line_titles_the_list() {
    verifies!(
        r#"
You can give a list a title by prefixing the line with a dot immediately followed by the text (without leaving any space after the dot).

Here's an example of a list with a title:

----
include::example$ordered.adoc[tag=base-t]
----

====
include::example$ordered.adoc[tag=base-t]
====

"#
    );

    let output = convert(".Parts of an atom\n. Protons\n. Electrons\n. Neutrons\n");
    assert_xpath(
        &output,
        r#"//div[@class="olist arabic"]/div[@class="title"][text()="Parts of an atom"]"#,
        1,
    );
}

non_normative!(
    r#"
== Nested ordered list

"#
);

// Adding dots nests an item, and each level takes a different number scheme:
// the top level is `arabic`, the second `loweralpha`.
#[test]
fn nested_levels_take_different_number_schemes() {
    verifies!(
        r#"
// tag::basic[]
You create a nested item by using one or more dots in front of each the item.

----
include::example$ordered.adoc[tag=nest]
----

AsciiDoc selects a different number scheme for each level of nesting.
Here's how the previous list renders:

.A nested ordered list
====
include::example$ordered.adoc[tag=nest]
====
// end::basic[]

"#
    );

    let output = convert(". Step 1\n. Step 2\n.. Step 2a\n.. Step 2b\n. Step 3\n");
    assert_css(&output, ".olist:root.arabic > ol.arabic", 1);
    assert_css(
        &output,
        "ol.arabic li div.olist.loweralpha > ol.loweralpha",
        1,
    );
}

non_normative!(
    r#"
[TIP]
====
Like with the asterisks in an unordered list, the number of dots in an ordered list doesn't represent the nesting level.
However, it's much more intuitive to follow the convention that the number of dots equals the level of nesting.

*# of dots = level of nesting*

Again, we are shooting for plain text markup that is readable _as is_.
====

"#
);

// Ordered, unordered, and description lists can be mixed in one hybrid list:
// here an unordered list nests inside each item of an ordered list.
#[test]
fn an_unordered_list_nests_inside_an_ordered_list() {
    verifies!(
        r#"
You can mix and match the three list types, ordered, xref:unordered.adoc[unordered], and xref:description.adoc[description], within a single hybrid list.
The AsciiDoc syntax tries hard to infer the relationships between the items that are most intuitive to us humans.

Here's an example of nesting an unordered list inside of an ordered list:

----
include::example$ordered.adoc[tag=mix]
----

====
include::example$ordered.adoc[tag=mix]
====

"#
    );

    let output = convert(". Linux\n* Fedora\n* Ubuntu\n* Slackware\n. BSD\n* FreeBSD\n* NetBSD\n");
    // Two ordered items, each carrying a nested unordered list.
    assert_css(&output, ".olist:root > ol > li", 2);
    assert_css(&output, ".olist:root > ol > li > div.ulist > ul", 2);
}

// Spreading the items out with blank lines and indenting the nested lists is
// only for readability; it produces the same structure as the compact form.
#[test]
fn spreading_items_out_does_not_change_the_structure() {
    verifies!(
        r#"
You can spread the items out and indent the nested lists if that makes it more readable for you:

----
include::example$ordered.adoc[tag=mix-alt]
----

"#
    );

    let output = convert(
        ". Linux\n\n  * Fedora\n  * Ubuntu\n  * Slackware\n\n. BSD\n\n  * FreeBSD\n  * NetBSD\n",
    );
    assert_css(&output, ".olist:root > ol > li", 2);
    assert_css(&output, ".olist:root > ol > li > div.ulist > ul", 2);
}

non_normative!(
    r#"
The description list page demonstrates how to xref:description.adoc#three-hybrid[combine all three list types].

[#styles]
== Number styles

"#
);

// The numeration block styles name the `<ol>` class; the alphabetic and roman
// keywords (`loweralpha`, `upperalpha`, `lowerroman`, `upperroman`) also carry
// the matching HTML `type`, while `arabic`, `decimal`, and `lowergreek` carry
// only the class. `arabic` is the default when no style is given, and
// `decimal`/`lowergreek` are HTML-only styles.
#[test]
fn numeration_styles_name_the_ol_class_and_type() {
    verifies!(
        r#"
For ordered lists, AsciiDoc supports the numeration styles such as lowergreek and decimal-leading-zero.
The full list of numeration styles that can be applied to an ordered list are as follows:

[%autowidth]
|===
|Block style |CSS list-style-type

|arabic
|decimal

|decimal ^[1]^
|decimal-leading-zero

|loweralpha
|lower-alpha

|upperalpha
|upper-alpha

|lowerroman
|lower-roman

|upperroman
|upper-roman

|lowergreek ^[1]^
|lower-greek
|===
^[1]^ These styles are only supported by the HTML converters.

Here are a few examples showing various numeration styles as defined by the block style shown in the header row:

[%autowidth]
|===
|[arabic] ^[2]^ |[decimal] |[loweralpha] |[lowergreek]

a|
. one
. two
. three

a|
[decimal]
. one
. two
. three

a|
[loweralpha]
. one
. two
. three

a|
[lowergreek]
. one
. two
. three
|===
^[2]^ Default numeration if block style is not specified

"#
    );

    // The default (no style) numeration is arabic, with no HTML `type`.
    assert_css(&convert(". one\n. two\n"), "ol.arabic:not([type])", 1);

    // The class names each style; the alphabetic/roman keywords add a `type`.
    for (style, type_letter) in [
        ("arabic", None),
        ("decimal", None),
        ("lowergreek", None),
        ("loweralpha", Some("a")),
        ("upperalpha", Some("A")),
        ("lowerroman", Some("i")),
        ("upperroman", Some("I")),
    ] {
        let output = convert(&format!("[{style}]\n. one\n. two\n. three\n"));
        assert_css(&output, &format!("div.olist.{style} > ol.{style}"), 1);
        match type_letter {
            Some(letter) => assert_css(&output, &format!(r#"ol.{style}[type="{letter}"]"#), 1),

            None => assert_css(&output, &format!("ol.{style}:not([type])"), 1),
        }
    }
}

// Custom numeration via a CSS role/class is a stylesheet technique, not a
// rendering behavior this crate verifies. Tracked as non-normative.
non_normative!(
    r#"
TIP: Custom numeration styles can be implemented using a custom role.
Define a new class selector (e.g., `.custom`) in your stylesheet that sets the `list-style-type` property to the value of your choice.
Then, assign the name of that class as a role on any list to which you want that numeration applied.

When the role shorthand (`.custom`) is used on an ordered list, the numeration style is no longer omitted.

"#
);

// A level's number scheme and its starting number can be set together: the top
// list uses `[lowerroman,start=5]` and a nested list resets to `[loweralpha]`.
#[test]
fn a_level_can_override_its_scheme_and_start() {
    verifies!(
        r#"
You can override the number scheme for any level by setting its style (the first positional entry in a block attribute list).
You can also set the starting number using the `start` attribute:

----
include::example$ordered.adoc[tag=num]
----

====
include::example$ordered.adoc[tag=num]
====

"#
    );

    let output =
        convert("[lowerroman,start=5]\n. Five\n. Six\n[loweralpha]\n.. a\n.. b\n.. c\n. Seven\n");
    assert_css(
        &output,
        r#".olist:root.lowerroman > ol.lowerroman[type="i"][start="5"]"#,
        1,
    );
    assert_css(
        &output,
        r#"div.olist.loweralpha > ol.loweralpha[type="a"]"#,
        1,
    );
}

// The `start` attribute is always a number even for an alphabetic style: to
// begin a `loweralpha` list at "c", set `start=3`.
#[test]
fn start_is_numeric_even_for_an_alphabetic_style() {
    verifies!(
        r#"
IMPORTANT: The `start` attribute must be a number, even when using a different numeration style.
For instance, to start an alphabetic list at letter "c", set the numeration style to loweralpha and the start attribute to 3.

"#
    );

    let output = convert("[loweralpha,start=3]\n. c\n. d\n");
    assert_css(&output, r#"ol.loweralpha[type="a"][start="3"]"#, 1);
}

non_normative!(
    r#"
== Escaping the list marker

If you have paragraph text that begins with a list marker, but you don't intend it to be a list item, you need to escape that marker by using the attribute reference to disrupt the pattern.

Consider the case when the line starts with a P.O. box reference:

"#
);

// A line like `P. O. Box` is parsed as an ordered (upper-alpha) list item;
// replacing the first space with `{empty}` breaks the marker pattern so the
// line stays a paragraph.
#[test]
fn empty_reference_escapes_an_unintended_list_marker() {
    verifies!(
        r#"
----
P. O. Box
----

In order to prevent this paragraph from being parsed as an ordered list, you need to replace the first space with `\{empty}`.

----
P.{empty}O. Box
----

Now the paragraph will remain as a paragraph.

"#
    );

    // `P. O. Box` matches the upper-alpha marker and becomes a one-item list.
    let unescaped = convert("P. O. Box\n");
    assert_css(&unescaped, "div.olist ol > li", 1);

    // With `{empty}`, the marker pattern is broken and the line stays a
    // paragraph.
    let escaped = convert("P.{empty}O. Box\n");
    assert_css(&escaped, "div.olist", 0);
    assert_css(&escaped, "div.paragraph > p", 1);
}

non_normative!(
    r#"
In the future, it will be possible to escape an ordered list marker using a backslash, but that is not currently possible.
"#
);
