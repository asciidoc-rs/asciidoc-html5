//! Coverage of this crate's *Unordered Lists* documentation page.
//!
//! The page shows how `asciidoc-html5` renders unordered lists: the basic form,
//! a titled and nested list, and a custom marker style. Each shown
//! source-and-output pair is verified by converting the source through
//! `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/unordered-lists.adoc");

non_normative!(
    r#"
= Unordered Lists
:navtitle: Unordered Lists
:description: How asciidoc-html5 renders unordered (bulleted) lists as HTML.

An unordered list marks each item with a `*` (asterisk) or `-` (hyphen). Adjacent
items join into one list, a longer or different marker nests, and `asciidoc-html5`
renders the same `<div class="ulist"><ul>` markup as Asciidoctor's `html5`
backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Make an unordered list

"#
);

// The basic unordered list: each `*` item becomes an `<li>` inside `<div
// class="ulist"><ul>`.
#[test]
fn a_basic_unordered_list() {
    verifies!(
        r#"
Start each item's line with `*` followed by a space:

[,asciidoc]
----
* Edgar Allan Poe
* Sheri S. Tepper
* Bill Bryson
----

Each item becomes an `<li>` inside the list wrapper:

[,html]
----
<div class="ulist">
<ul>
<li>
<p>Edgar Allan Poe</p>
</li>
<li>
<p>Sheri S. Tepper</p>
</li>
<li>
<p>Bill Bryson</p>
</li>
</ul>
</div>
----

"#
    );

    let output = convert("* Edgar Allan Poe\n* Sheri S. Tepper\n* Bill Bryson\n");
    assert_css(&output, "div.ulist > ul > li", 3);
    assert_xpath(&output, r#"//li/p[text()="Edgar Allan Poe"]"#, 1);
}

non_normative!(
    r#"
A hyphen (`-`) works too, and is best reserved for single-level lists. An empty
line before and after the list is required; empty lines between items are
optional.

== Title and nesting

"#
);

// A titled, nested list: the `.` line becomes `<div class="title">` and the
// deeper marker nests a fresh `<div class="ulist">` inside its parent `<li>`.
#[test]
fn a_titled_and_nested_list() {
    verifies!(
        r#"
A line beginning with `.` immediately before the list gives it a title. Add
another `*` to the marker to nest an item:

[,asciidoc]
----
.Shopping list
* Fruit
** Apples
** Oranges
* Bread
----

The title becomes a `<div class="title">`, and the nested list is a fresh
`<div class="ulist">` inside its parent `<li>`:

[,html]
----
<div class="ulist">
<div class="title">Shopping list</div>
<ul>
<li>
<p>Fruit</p>
<div class="ulist">
<ul>
<li>
<p>Apples</p>
</li>
<li>
<p>Oranges</p>
</li>
</ul>
</div>
</li>
<li>
<p>Bread</p>
</li>
</ul>
</div>
----

"#
    );

    let output = convert(".Shopping list\n* Fruit\n** Apples\n** Oranges\n* Bread\n");
    assert_css(&output, ".ulist:root > div.title", 1);
    assert_css(&output, ".ulist:root > ul > li", 2);
    assert_css(&output, ".ulist:root > ul > li > div.ulist > ul > li", 2);
    assert_xpath(&output, r#"//li/p[text()="Apples"]"#, 1);
}

non_normative!(
    r#"
== Custom markers

"#
);

// A marker block style names the bullet on both the wrapper `<div>` and the
// `<ul>`.
#[test]
fn a_custom_marker_style() {
    verifies!(
        r#"
A block style names the bullet: `square`, `circle`, `disc`, `none`, `no-bullet`,
or `unstyled`. The style becomes a class on both the wrapper `<div>` and the
`<ul>`, where the default stylesheet turns it into the matching marker:

[,asciidoc]
----
[square]
* Coffee
* Tea
----

[,html]
----
<div class="ulist square">
<ul class="square">
<li>
<p>Coffee</p>
</li>
<li>
<p>Tea</p>
</li>
</ul>
</div>
----

"#
    );

    let output = convert("[square]\n* Coffee\n* Tea\n");
    assert_css(&output, "div.ulist.square > ul.square > li", 2);
}

non_normative!(
    r#"
Once set, a marker style is inherited by nested lists until a nested list sets
its own. The disc/circle/square glyphs of an unstyled list are supplied by the
stylesheet, not named in the HTML.

== Known limitations

Unordered lists are fully supported: single- and multi-level lists, the `*` and
`-` markers, list titles, and every marker block style match Asciidoctor's
output. For ordered lists, see xref:ordered-lists.adoc[]; to attach blocks to a
list item, see xref:list-continuation.adoc[].
"#
);
