//! Coverage of this crate's *Collapsible Blocks* documentation page.
//!
//! The page shows how `asciidoc-html5` renders a `%collapsible` example block
//! as an HTML `<details>`/`<summary>` disclosure widget, in each documented
//! form: the default (untitled) block, a custom summary title, the `%open`
//! variant, the collapsible paragraph, and the enclosure form nesting another
//! block. Each example's source-and-output pair is verified by converting the
//! shown source through `convert`; the surrounding prose is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/collapsible.adoc");

non_normative!(
    r#"
= Collapsible Blocks
:navtitle: Collapsible Blocks
:description: How asciidoc-html5 renders collapsible example blocks as HTML disclosure widgets.

An example block marked with the `collapsible` option renders as an HTML
disclosure widget -- a `<details>`/`<summary>` pair -- whose content the reader
reveals by clicking the summary. `asciidoc-html5` produces the same markup as
Asciidoctor's `html5` backend, so no stylesheet or script is needed for the
toggle to work.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Make a block collapsible

Add the `%collapsible` option to an example block -- the `====` delimited
container:

"#
);

// The default collapsible block: a `[%collapsible]` example renders as a
// closed disclosure widget whose untitled summary falls back to "Details".
#[test]
fn make_a_block_collapsible() {
    verifies!(
        r#"
[,asciidoc]
----
[%collapsible]
====
Click the summary to reveal this content.
====
----

The renderer emits a disclosure widget. It is closed by default, and because the
block has no title, the summary falls back to the default label `Details`:

[,html]
----
<details>
<summary class="title">Details</summary>
<div class="content">
<div class="paragraph">
<p>Click the summary to reveal this content.</p>
</div>
</div>
</details>
----

"#
    );

    let output = convert(
        "[%collapsible]\n====\n\
         Click the summary to reveal this content.\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_css(&output, "details[open]", 0);
    assert_css(&output, "details > summary.title", 1);
    assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
    assert_css(
        &output,
        "details > summary.title + .content .paragraph p",
        1,
    );
    assert_xpath(
        &output,
        r#"//details//p[text()="Click the summary to reveal this content."]"#,
        1,
    );
}

non_normative!(
    r#"
Because the block is a disclosure widget rather than a numbered example, it does
not consume an example number: a `Example N.` caption elsewhere in the document
is unaffected.

== Customize the summary text

Give the block a title to set the toggle text. Unlike an example block, a
collapsible block is not numbered and carries no `Example N.` caption prefix, so
the title is used verbatim:

"#
);

// A titled collapsible block uses the title verbatim as the `<summary>` text,
// with no "Example N." caption prefix.
#[test]
fn customize_the_summary_text() {
    verifies!(
        r#"
[,asciidoc]
----
.Click to reveal the answer
[%collapsible]
====
This is the answer.
====
----

The title becomes the `<summary>`:

[,html]
----
<summary class="title">Click to reveal the answer</summary>
----

"#
    );

    let output = convert(
        ".Click to reveal the answer\n[%collapsible]\n====\n\
         This is the answer.\n====\n",
    );
    assert_css(&output, "details > summary.title", 1);
    assert_xpath(
        &output,
        r#"//details/summary[text()="Click to reveal the answer"]"#,
        1,
    );
}

non_normative!(
    r#"
== Open by default

Add the `%open` option alongside `%collapsible` to render the widget expanded.
The `<details>` element then carries the boolean `open` attribute:

"#
);

// The `%open` option renders the widget expanded: the `<details>` element
// carries the boolean `open` attribute.
#[test]
fn open_by_default() {
    verifies!(
        r#"
[,asciidoc]
----
[%collapsible%open]
====
This content is revealed by default.
====
----

[,html]
----
<details open>
<summary class="title">Details</summary>
----

"#
    );

    let output = convert(
        "[%collapsible%open]\n====\n\
         This content is revealed by default.\n====\n",
    );
    assert_css(&output, "details[open]", 1);
    assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
}

non_normative!(
    r#"
== Collapsible paragraph

When the content is a single paragraph, the example paragraph style
`[example%collapsible]` produces the same widget without the delimiters:

"#
);

// The example paragraph style produces the same widget from a single paragraph;
// its content is emitted unwrapped inside the widget.
#[test]
fn collapsible_paragraph() {
    verifies!(
        r#"
[,asciidoc]
----
[example%collapsible]
Click the summary to reveal this content.
----

"#
    );

    let output = convert(
        "[example%collapsible]\n\
         Click the summary to reveal this content.\n",
    );
    assert_css(&output, "details", 1);
    assert_css(&output, "details > summary.title", 1);
    assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
}

non_normative!(
    r#"
== Use as an enclosure

Like an open block, a collapsible block is an enclosure: nest other blocks inside
it to make them collapsible together. Here a literal block sits behind a
"`Show stacktrace`" summary:

"#
);

// As an enclosure, the collapsible block renders a nested block (here a literal
// block) inside the widget's content.
#[test]
fn use_as_an_enclosure() {
    verifies!(
        r#"
[,asciidoc]
----
.Show stacktrace
[%collapsible]
====
....
Error: repository not found
....
====
----

The nested literal block is rendered inside the widget's content:

[,html]
----
<details>
<summary class="title">Show stacktrace</summary>
<div class="content">
<div class="literalblock">
<div class="content">
<pre>Error: repository not found</pre>
</div>
</div>
</div>
</details>
----

"#
    );

    let output = convert(
        ".Show stacktrace\n[%collapsible]\n====\n....\n\
         Error: repository not found\n....\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_xpath(&output, r#"//details/summary[text()="Show stacktrace"]"#, 1);
    assert_css(&output, "details > .content > .literalblock", 1);
    assert_xpath(
        &output,
        r#"//details//pre[text()="Error: repository not found"]"#,
        1,
    );
}

non_normative!(
    r#"
== Known limitations

The collapsible block is fully supported: the default and custom-title forms, the
`%open` option, the collapsible paragraph, and the enclosure form all match
Asciidoctor's output. To see exactly which other markup the renderer supports
today, see the xref:ROOT:index.adoc[introduction].
"#
);
