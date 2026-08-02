//! Coverage of this crate's *Ordered Lists* documentation page.
//!
//! The page shows how `asciidoc-html5` renders ordered lists: the basic and
//! nested forms, the `start` attribute, and a numeration block style. Each
//! shown source-and-output pair is verified by converting the source through
//! `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("docs/modules/ROOT/pages/ordered-lists.adoc");

non_normative!(
    r#"
= Ordered Lists
:navtitle: Ordered Lists
:description: How asciidoc-html5 renders ordered (numbered) lists as HTML.

An ordered list marks each item with a `.`, one dot per nesting level. The
processor supplies the numerals, so you can omit them. `asciidoc-html5` renders
the same `<div class="olist <style>"><ol class="<style>">` markup as
Asciidoctor's `html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Make an ordered list

"#
);

// The basic ordered list and its nested form: the default `arabic` style names
// both classes, and a nested level takes the next scheme (`loweralpha`).
#[test]
fn a_basic_and_nested_ordered_list() {
    verifies!(
        r#"
Start each item's line with `.` followed by a space:

[,asciidoc]
----
. Protons
. Electrons
. Neutrons
----

The default `arabic` style names both the wrapper `<div>` and the `<ol>`:

[,html]
----
<div class="olist arabic">
<ol class="arabic">
<li>
<p>Protons</p>
</li>
<li>
<p>Electrons</p>
</li>
<li>
<p>Neutrons</p>
</li>
</ol>
</div>
----

Nesting works like unordered lists — add another dot — and each level takes a
different number scheme (`arabic`, then `loweralpha`, and so on):

[,asciidoc]
----
. Step 1
.. Step 2a
. Step 3
----

[,html]
----
<div class="olist arabic">
<ol class="arabic">
<li>
<p>Step 1</p>
<div class="olist loweralpha">
<ol class="loweralpha" type="a">
<li>
<p>Step 2a</p>
</li>
</ol>
</div>
</li>
<li>
<p>Step 3</p>
</li>
</ol>
</div>
----

"#
    );

    let basic = convert(". Protons\n. Electrons\n. Neutrons\n");
    assert_css(&basic, "div.olist.arabic > ol.arabic > li", 3);

    let nested = convert(". Step 1\n.. Step 2a\n. Step 3\n");
    assert_css(&nested, ".olist:root.arabic > ol.arabic > li", 2);
    assert_css(
        &nested,
        r#"ol.arabic li div.olist.loweralpha > ol.loweralpha[type="a"]"#,
        1,
    );
}

non_normative!(
    r#"
== Set the starting number

"#
);

// The `start` attribute sets the first numeral.
#[test]
fn the_start_attribute() {
    verifies!(
        r#"
The `start` attribute sets the first numeral, emitted as `start` on the `<ol>`:

[,asciidoc]
----
[start=4]
. Step four
. Step five
----

[,html]
----
<div class="olist arabic">
<ol class="arabic" start="4">
<li>
<p>Step four</p>
</li>
<li>
<p>Step five</p>
</li>
</ol>
</div>
----

The `%reversed` option adds a boolean `reversed` attribute to the `<ol>`.

"#
    );

    let output = convert("[start=4]\n. Step four\n. Step five\n");
    assert_css(&output, r#"ol.arabic[start="4"] > li"#, 2);
    // The `%reversed` option adds a boolean `reversed` attribute.
    assert_css(&convert("[%reversed]\n. a\n. b\n"), "ol[reversed]", 1);
}

non_normative!(
    r#"
== Choose a numeration style

"#
);

// A numeration block style names the `<ol>` class, and the alphabetic/roman
// keywords also set the HTML `type`.
#[test]
fn a_numeration_style() {
    verifies!(
        r#"
A block style picks the numbering: `arabic`, `decimal`, `loweralpha`,
`upperalpha`, `lowerroman`, `upperroman`, or `lowergreek`. The style names the
`<ol>` class, and the alphabetic and roman keywords also set the HTML `type`:

[,asciidoc]
----
[loweralpha]
. one
. two
----

[,html]
----
<div class="olist loweralpha">
<ol class="loweralpha" type="a">
<li>
<p>one</p>
</li>
<li>
<p>two</p>
</li>
</ol>
</div>
----

`start` is always a number, even for an alphabetic style: use `start=3` to begin
a `loweralpha` list at "c".

"#
    );

    let output = convert("[loweralpha]\n. one\n. two\n");
    assert_css(
        &output,
        r#"div.olist.loweralpha > ol.loweralpha[type="a"] > li"#,
        2,
    );
    // `start` is numeric even for an alphabetic style.
    assert_css(
        &convert("[loweralpha,start=3]\n. c\n. d\n"),
        r#"ol.loweralpha[type="a"][start="3"]"#,
        1,
    );
}

non_normative!(
    r#"
== Known limitations

Ordered lists are fully supported: automatic and nested numbering, the `start`
attribute, the `%reversed` option, and every numeration style match
Asciidoctor's output. For unordered lists, see xref:unordered-lists.adoc[]; to
attach blocks to a list item, see xref:list-continuation.adoc[].
"#
);
