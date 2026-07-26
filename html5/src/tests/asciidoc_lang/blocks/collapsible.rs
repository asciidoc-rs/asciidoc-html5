//! Coverage of the AsciiDoc language description's *Collapsible Blocks* page.
//!
//! The page documents the `collapsible` option (and its `open` companion) on
//! the example block: the block is mapped to an HTML disclosure widget
//! (`<details>`/`<summary>`), defaults its summary to "Details" when untitled,
//! opens by default with `%open`, keeps its title unnumbered, and acts as an
//! enclosure for nested blocks. This crate renders every one of those forms, so
//! each section's body — the prose describing the syntax, the example source it
//! shows, and the `include::example$collapsible.adoc[tag=…]` result it displays
//! — is verified through `convert`. Only the page's introduction, its section
//! headings, and a couple of tangential asides are tracked as non-normative.
//!
//! Each `verifies!` block drives the exact source of the example tag the page
//! includes in that section (from
//! `ref/asciidoc-lang/docs/modules/blocks/examples/collapsible.adoc`).

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/collapsible.adoc");

non_normative!(
    r#"
= Collapsible Blocks

You can designate block content to be revealed or hidden on interaction using the collapsible option and its companion, the open option.
When the AsciiDoc source is converted to HTML, this block gets mapped to a disclosure summary element (i.e., a summary/details pair).
If the output format does not support this interaction, it may be rendered as an unstyled block (akin to an open block).

== Collapsible block syntax

"#
);

// The basic collapsible block. The section body describes the syntax — the
// `collapsible` option on an example structural container turns the example
// block into a collapsible block — and shows it: a `[%collapsible]` example
// maps to a `<details>`/`<summary>` disclosure widget. It is closed by default
// (no `open`), and — being untitled — its summary is the default label
// "Details".
#[test]
fn basic_collapsible_block_renders_a_disclosure_widget() {
    verifies!(
        r#"
You make block content collapsible by specifying the `collapsible` option on the example structural container.
This option changes the block from an example block to a collapsible block.

.Collapsible block syntax
[#ex-collapsible]
----
include::example$collapsible.adoc[tag=basic]
----

In the output, the content of this block is hidden until the reader clicks the default title, "`Details`".
The result of <<ex-collapsible>> is displayed below.

include::example$collapsible.adoc[tag=basic]

"#
    );

    let output = convert(
        "[%collapsible]\n====\n\
         This content is only revealed when the user clicks the block title.\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_css(&output, "details[open]", 0);
    assert_css(&output, "details > summary.title", 1);
    assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
    assert_css(&output, "details > summary.title + .content", 1);
}

non_normative!(
    r#"
Like other blocks, the collapsible block recognizes the `id` and `role` attributes.

== Collapsible paragraph syntax

"#
);

// The collapsible paragraph. The section body describes the syntax — the
// example *paragraph* style (`[example%collapsible]`) over a single paragraph —
// and shows that it produces the same disclosure widget, again with the default
// "Details" summary.
#[test]
fn collapsible_paragraph_renders_a_disclosure_widget() {
    verifies!(
        r#"
If the content of the block is only a single paragraph, you can use the example paragraph style instead of the example structural container to make a collapsible paragraph.

.Collapsible paragraph syntax
[#ex-collapsible-paragraph]
----
include::example$collapsible.adoc[tag=paragraph]
----

In the output, the content of this block is hidden until the reader clicks the default title, "`Details`".
The result of <<ex-collapsible-paragraph>> is displayed below.

include::example$collapsible.adoc[tag=paragraph]

"#
    );

    let output = convert(
        "[example%collapsible]\n\
         This content is only revealed when the user clicks the block title.\n",
    );
    assert_css(&output, "details", 1);
    assert_css(&output, "details > summary.title", 1);
    assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
}

non_normative!(
    r#"
== Customize the toggle text

"#
);

// A titled collapsible block. The section body describes the syntax — a title
// on the block sets the toggle text — and notes that, because it is not an
// example block, the title is rendered verbatim with no "Example N." caption
// prefix. The exact summary-text match below confirms both the toggle text and
// the absence of a numbered caption.
#[test]
fn collapsible_title_becomes_the_toggle_text_without_a_caption() {
    verifies!(
        r#"
If you want to customize the text that toggles the display of the collapsible content, specify a title on the block or paragraph.

.Collapsible block with custom title
[#ex-collapsible-with-title]
----
include::example$collapsible.adoc[tag=title]
----

The result of <<ex-collapsible-with-title>> is displayed below.

include::example$collapsible.adoc[tag=title]

Notice that even though this block has a title, it's not numbered and does not have a caption prefix.
That's because it's not an example block and thus does not get a numbered caption prefix like an example block would.

"#
    );

    let output = convert(
        ".Click to reveal the answer\n[%collapsible]\n====\n\
         This is the answer.\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_xpath(
        &output,
        r#"//details/summary[text()="Click to reveal the answer"]"#,
        1,
    );
}

non_normative!(
    r#"
== Default to open

"#
);

// The `%open` option. The section body describes the syntax — adding `open`
// alongside `collapsible` — and shows that it starts the widget expanded: the
// `<details>` element carries the boolean `open` attribute.
#[test]
fn open_option_expands_the_widget_by_default() {
    verifies!(
        r#"
If you want the collapsible block to be open by default, specify the `open` option as well.

.Collapsible block that defaults to open
[#ex-collapsible-open]
----
include::example$collapsible.adoc[tag=open]
----

The result of <<ex-collapsible-open>> is displayed below.

include::example$collapsible.adoc[tag=open]

"#
    );

    let output = convert(
        ".Too much detail? Click here.\n[%collapsible%open]\n====\n\
         This content is revealed by default.\n\n\
         If it's taking up too much space, the reader can hide it.\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_css(&output, "details[open]", 1);
    assert_xpath(
        &output,
        r#"//details/summary[text()="Too much detail? Click here."]"#,
        1,
    );
}

non_normative!(
    r#"
== Use as an enclosure

"#
);

// The collapsible block as an enclosure. The section body describes the syntax
// — nesting other blocks inside a collapsible block to make them collapsible
// together — and shows a literal block enclosed inside the `<details>` content,
// with the block title serving as the toggle text.
#[test]
fn collapsible_block_encloses_nested_blocks() {
    verifies!(
        r#"
Much like the open block, the collapsible block is an enclosure.
If you want to make other types of blocks collapsible, such as an listing block, you can nest the block inside the collapsible block.

.Collapsible block that encloses a literal block
[#ex-collapsible-nested]
----
include::example$collapsible.adoc[tag=nested]
----

The result of <<ex-collapsible-nested>> is displayed below.

include::example$collapsible.adoc[tag=nested]

"#
    );

    let output = convert(
        ".Show stacktrace\n[%collapsible]\n====\n....\n\
         Error: Content repository not found (url: https://git.example.org/repo.git)\n    \
         at transformGitCloneError\n    at git.clone.then.then.catch\n\
         Caused by: HttpError: HTTP Error: 401 HTTP Basic: Access Denied\n    \
         at GitCredentialManagerStore.rejected\n    at fill.then\n....\n====\n",
    );
    assert_css(&output, "details", 1);
    assert_xpath(&output, r#"//details/summary[text()="Show stacktrace"]"#, 1);
    assert_css(&output, "details > .content > .literalblock", 1);
    assert_xpath(
        &output,
        r#"//details//pre[contains(text(), "Error: Content repository not found")]"#,
        1,
    );
}

non_normative!(
    r#"
Since the toggle text acts as the block title, you may decide to not put a title on the nested block, as in this example.
"#
);
