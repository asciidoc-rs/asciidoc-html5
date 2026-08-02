//! Coverage of this crate's *List Continuations* documentation page.
//!
//! The page shows how `asciidoc-html5` attaches block content to a list item
//! with a list continuation (`+`): a single attached listing block (which keeps
//! an ordered list continuous), and several blocks wrapped in an open block.
//! Each shown source-and-output pair is verified by converting the source
//! through `convert`; the surrounding prose is tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/list-continuation.adoc");

non_normative!(
    r#"
= List Continuations
:navtitle: List Continuations
:description: How asciidoc-html5 attaches block content to a list item with a list continuation.

By default a list item holds only its principal text. To attach a block — a
listing block, a paragraph, an admonition — to the item, use a *list
continuation*: a `+` on a line by itself, adjacent to the block. `asciidoc-html5`
renders the attached block inside the item's `<li>`, matching Asciidoctor's
`html5` backend.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Attach a block

"#
);

// A single continuation attaches a listing block to the first item and keeps
// the ordered list continuous, so the second item still numbers as 2.
#[test]
fn attach_a_block_with_a_continuation() {
    verifies!(
        r#"
Put a `+` on its own line between the item and the block you want to attach. This
also keeps an ordered list continuous, so its numbering does not restart:

[,asciidoc]
------
. Do the first thing.
+
----
run --first
----
. Do the second thing.
------

The listing block is attached to the first item's `<li>`, and the second item
still numbers as 2:

[,html]
----
<div class="olist arabic">
<ol class="arabic">
<li>
<p>Do the first thing.</p>
<div class="listingblock">
<div class="content">
<pre>run --first</pre>
</div>
</div>
</li>
<li>
<p>Do the second thing.</p>
</li>
</ol>
</div>
----

"#
    );

    let output =
        convert(". Do the first thing.\n+\n----\nrun --first\n----\n. Do the second thing.\n");
    // One continuous ordered list of two items.
    assert_css(&output, "div.olist", 1);
    assert_css(&output, ".olist:root > ol > li", 2);
    // The listing block is attached to the first item.
    assert_css(
        &output,
        ".olist:root > ol > li:first-child > div.listingblock",
        1,
    );
    assert_xpath(&output, r#"//li[1]//pre[text()="run --first"]"#, 1);
}

non_normative!(
    r#"
== Attach several blocks with an open block

"#
);

// An open block wrapped with one continuation attaches several blocks: both
// paragraphs live inside the item's open block.
#[test]
fn attach_several_blocks_with_an_open_block() {
    verifies!(
        r#"
To attach more than one block, wrap them in an open block (`--`). A single
continuation attaches the whole open block, and its contents need no further
`+`:

[,asciidoc]
----
* Item with two blocks.
+
--
First paragraph.

Second paragraph.
--
----

[,html]
----
<div class="ulist">
<ul>
<li>
<p>Item with two blocks.</p>
<div class="openblock">
<div class="content">
<div class="paragraph">
<p>First paragraph.</p>
</div>
<div class="paragraph">
<p>Second paragraph.</p>
</div>
</div>
</div>
</li>
</ul>
</div>
----

"#
    );

    let output =
        convert("* Item with two blocks.\n+\n--\nFirst paragraph.\n\nSecond paragraph.\n--\n");
    assert_css(
        &output,
        ".ulist:root > ul > li > div.openblock > div.content > div.paragraph",
        2,
    );
}

non_normative!(
    r#"
An open block is also the clearest way to end a nested list and attach a block to
the parent item: enclose the nested list in the open block, then attach the
following block after it.

== Known limitations

List continuations are fully supported: attaching a single block, chaining
several blocks, wrapping compound content in an open block, dropping the
principal text with `{empty}`, and attaching a block to an ancestor list item
all match Asciidoctor's output, for unordered, ordered, and description lists.
For the list types themselves, see xref:unordered-lists.adoc[],
xref:ordered-lists.adoc[], and xref:description-lists.adoc[].
"#
);
