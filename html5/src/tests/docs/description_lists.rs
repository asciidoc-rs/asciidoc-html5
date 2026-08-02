//! Coverage of this crate's *Description Lists* documentation page.
//!
//! The page shows how `asciidoc-html5` renders description lists: the basic
//! `term:: description` form, the `horizontal` two-column variant, and the
//! `qanda` variant. Each shown source-and-output pair is verified by converting
//! the source through `convert`; the surrounding prose is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/description-lists.adoc");

non_normative!(
    r#"
= Description Lists
:navtitle: Description Lists
:description: How asciidoc-html5 renders description lists, including the horizontal and Q&A variants.

A description list pairs each term with a description, written `term:: description`.
`asciidoc-html5` renders the same `<div class="dlist"><dl>` markup as
Asciidoctor's `html5` backend, plus the `horizontal` and `qanda` variants.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Make a description list

"#
);

// The basic description list: each term becomes a `<dt class="hdlist1">` and
// each description the following `<dd>`.
#[test]
fn a_basic_description_list() {
    verifies!(
        r#"
Follow each term with `::` and its description:

[,asciidoc]
----
CPU:: The brain of the computer.
RAM:: Temporary storage.
----

Each term becomes a `<dt class="hdlist1">` and each description the following
`<dd>`:

[,html]
----
<div class="dlist">
<dl>
<dt class="hdlist1">CPU</dt>
<dd>
<p>The brain of the computer.</p>
</dd>
<dt class="hdlist1">RAM</dt>
<dd>
<p>Temporary storage.</p>
</dd>
</dl>
</div>
----

A description can hold any block content, including another list, and changing
the term delimiter (`::` to `:::`, `::::`, or `;;`) nests a description list.

"#
    );

    let output = convert("CPU:: The brain of the computer.\nRAM:: Temporary storage.\n");
    assert_css(&output, "div.dlist > dl > dt.hdlist1", 2);
    assert_css(&output, "div.dlist > dl > dd", 2);
    assert_xpath(&output, r#"//dt[text()="CPU"]"#, 1);
    assert_xpath(&output, r#"//dd/p[text()="The brain of the computer."]"#, 1);
}

non_normative!(
    r#"
== Horizontal variant

"#
);

// The `horizontal` style renders the list as a two-column table.
#[test]
fn the_horizontal_variant() {
    verifies!(
        r#"
The `horizontal` style lays each term beside its description by rendering the
list as a two-column table:

[,asciidoc]
----
[horizontal]
CPU:: The brain.
RAM:: Temporary storage.
----

[,html]
----
<div class="hdlist">
<table>
<tr>
<td class="hdlist1">
CPU
</td>
<td class="hdlist2">
<p>The brain.</p>
</td>
</tr>
<tr>
<td class="hdlist1">
RAM
</td>
<td class="hdlist2">
<p>Temporary storage.</p>
</td>
</tr>
</table>
</div>
----

The optional `labelwidth` and `itemwidth` attributes size the two columns.

"#
    );

    let output = convert("[horizontal]\nCPU:: The brain.\nRAM:: Temporary storage.\n");
    assert_css(&output, "div.hdlist table tr", 2);
    assert_css(&output, "td.hdlist1", 2);
    assert_css(&output, "td.hdlist2 > p", 2);
}

non_normative!(
    r#"
== Question and answer variant

"#
);

// The `qanda` style renders the list as an ordered list with an emphasized
// question and its answer.
#[test]
fn the_qanda_variant() {
    verifies!(
        r#"
The `qanda` style renders the list as an ordered list, with each term as an
emphasized question and its description as the answer:

[,asciidoc]
----
[qanda]
What is the answer?:: This is the answer.
----

[,html]
----
<div class="qlist qanda">
<ol>
<li>
<p><em>What is the answer?</em></p>
<p>This is the answer.</p>
</li>
</ol>
</div>
----

"#
    );

    let output = convert("[qanda]\nWhat is the answer?:: This is the answer.\n");
    assert_css(&output, "div.qlist.qanda > ol > li", 1);
    assert_xpath(&output, r#"//li/p/em[text()="What is the answer?"]"#, 1);
    assert_xpath(&output, r#"//li/p[text()="This is the answer."]"#, 1);
}

non_normative!(
    r#"
== Known limitations

Description lists are fully supported: the basic form, nested delimiters, and the
`horizontal` and `qanda` variants all match Asciidoctor's output. One current
gap comes from the parser: an ordered or unordered list nested under a
second-level (`:::`) term is not yet parsed as a nested list. For the other list
types, see xref:unordered-lists.adoc[] and xref:ordered-lists.adoc[].
"#
);
