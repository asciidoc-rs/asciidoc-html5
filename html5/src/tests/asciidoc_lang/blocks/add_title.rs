//! Coverage of the AsciiDoc language description's *Add a Title to a Block*
//! page.
//!
//! The page documents the `.Title` block-title syntax across delimited blocks,
//! styled paragraphs, and blocks with attribute lists, plus captioned titles
//! (the `Example N.` caption, its `-caption`/`-number` attributes, and a custom
//! `caption` attribute). This crate renders every one of those, so each example
//! is verified through `convert`. The title, the conceptual introductions, the
//! caveats, the section headings, and the reference table of captioned-title
//! blocks are non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/add-title.adoc");

non_normative!(
    r#"
= Add a Title to a Block

You can assign a title to a block, whether it's styled using its style name or delimiters.

== Block title syntax

"#
);

// The block-title syntax: a line beginning with a dot directly above a block
// becomes the block's title. On a sidebar it renders inside the content div.
#[test]
fn dot_prefixed_line_becomes_a_block_title() {
    verifies!(
        r#"
A block title is defined on its own line directly above the block's attribute list, opening delimiter, or block content--which ever comes first.
As shown in <<ex-basic>>, the line must begin with a dot (`.`) and immediately be followed by the text of the title.
The block title must only occupy a single line and thus cannot be wrapped.

.Block title syntax
[#ex-basic]
----
.This is the title of a sidebar block
****
This is the content of the sidebar block.
****
----

"#
    );

    let output = convert(
        ".This is the title of a sidebar block\n****\nThis is the content of the sidebar block.\n****\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="sidebarblock"]/div[@class="content"]/div[@class="title"][text()="This is the title of a sidebar block"]"#,
        1,
    );
}

non_normative!(
    r#"
CAUTION: The block title line should not be confused with an ordered list item that uses the `.` marker.
A block title line has no space after the `.`, whereas the space after a list marker is required.

The next sections will show how to add titles to delimited blocks and blocks with attribute lists.

== Add a title to a delimited block

"#
);

// A title on a delimited block with no attribute list: the title line sits
// directly above the opening delimiter. Here a literal block is titled
// "Terminal Output".
#[test]
fn delimited_block_title_sits_above_the_delimiter() {
    verifies!(
        r#"
Any delimited block can have a title.
If the block doesn't have an attribute list, enter the title on a new line directly above the opening delimiter.
The delimited literal block in <<ex-title>> is titled _Terminal Output_.

.Add a title to a delimited block
[#ex-title]
----
.Terminal Output <.>
.... <.>
From github.com:asciidoctor/asciidoctor
 * branch        main   -> FETCH_HEAD
Already up to date.
....
----
<.> The block title is entered on a new line.
The title must begin with a dot (`.`).
Don't put a space between the dot and the first character of the title.
<.> If you aren't applying attributes to a block, enter the opening delimiter on a new line directly after the title.

The result of <<ex-title>> is displayed below.

.Terminal Output
....
From github.com:asciidoctor/asciidoctor
 * branch        main   -> FETCH_HEAD
Already up to date.
....

In the next section, you'll see how a title is placed on a block that has an attribute list.

== Add a title to a block with attributes

"#
    );

    let output = convert(
        ".Terminal Output\n....\nFrom github.com:asciidoctor/asciidoctor\n \
         * branch        main   -> FETCH_HEAD\nAlready up to date.\n....\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="literalblock"]/div[@class="title"][text()="Terminal Output"]"#,
        1,
    );
}

// A title on a block that has an attribute list sits above the attribute list.
// The rendered source block uses `[caption=]` so the title shows plain.
#[test]
fn titled_block_with_attribute_list_places_title_above_the_list() {
    verifies!(
        r#"
When you're applying attributes to a block, the title is placed on the line above the attribute list (or lists).
<<ex-title-list>> shows a delimited source code block that's titled _Specify GitLab CI stages_.

.Add a title to a delimited source code block
[source#ex-title-list]
....
.Specify GitLab CI stages <.>
[source,yaml] <.>
----
image: node:16-buster
stages: [ init, verify, deploy ]
----
....
<.> The block title is entered on a new line.
<.> The block's attribute list is entered on a new line directly after the title.

The result of <<ex-title-list>> is displayed below.

[caption=]
.Specify GitLab CI stages
[source,yaml]
----
image: node:16-buster
stages: [ init, verify, deploy ]
----

"#
    );

    let output = convert(
        "[caption=]\n.Specify GitLab CI stages\n[source,yaml]\n----\n\
         image: node:16-buster\nstages: [ init, verify, deploy ]\n----\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="listingblock"]/div[@class="title"][text()="Specify GitLab CI stages"]"#,
        1,
    );
    assert_css(
        &output,
        "div.listingblock pre.highlight > code.language-yaml",
        1,
    );
}

// A title on a non-delimited (styled paragraph) block also sits above the
// attribute list; on a sidebar the title renders inside the content div.
#[test]
fn titled_non_delimited_block_places_title_above_the_style() {
    verifies!(
        r#"
As shown in <<ex-title-style>>, a block's title is placed above the attribute list when a block isn't delimited.

.Add a title to a non-delimited block
[#ex-title-style]
----
.Mint
[sidebar]
Mint has visions of global conquest.
If you don't plant it in a container, it will take over your garden.
----

The result of <<ex-title-style>> is displayed below.

.Mint
[sidebar]
Mint has visions of global conquest.
If you don't plant it in a container, it will take over your garden.

"#
    );

    let output = convert(
        ".Mint\n[sidebar]\nMint has visions of global conquest.\n\
         If you don't plant it in a container, it will take over your garden.\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="sidebarblock"]/div[@class="content"]/div[@class="title"][text()="Mint"]"#,
        1,
    );
}

non_normative!(
    r#"
You may notice that unlike the titles in the previous rendered listing and source block examples, the sidebar's title is centered and displayed inside the sidebar's background.
How the title of a block is displayed depends on the converter and stylesheet you're applying to your AsciiDoc documents.

== Captioned titles

Several block contexts support captioned titles.
A [.term]*captioned title* is a title that's prefixed with a caption label and a number followed by a dot (e.g., `Table 1. Properties`).

The captioned title is only used if the corresponding caption attribute is set.
Otherwise, the original title is displayed.

The following table lists the blocks that support captioned titles and the attributes that the converter uses to generate and control them.

.Blocks that support captioned titles
[cols=1;m;m]
|===
|Block context | Caption attribute | Counter attribute

|appendix
|appendix-caption
|appendix-number

|example
|example-caption
|example-number

|image
|figure-caption
|figure-number

|listing, source
|listing-caption
|listing-number

|table
|table-caption
|table-number
|===

All caption attributes are set by default except for the attribute for listing and source blocks (`listing-caption`).
The number is sequential, computed automatically, and stored in a corresponding counter attribute.

"#
);

// Every block context the table above lists as supporting a captioned title is
// verified. The `example` row is verified immediately below; the other four are
// verified in the module that owns each block type:
//
// - `table` (`table-caption` / `table-number`): its `Table N.` caption is
//   verified by `tables::add_title::add_a_title`.
// - `image` (`figure-caption` / `figure-number`): its `Figure N.` caption is
//   verified by
//   `asciidoctor_rb::attributes_test::nested_document_should_use_counter_from_parent_document`.
// - `listing` and `source` (`listing-caption` / `listing-number`): their
//   `Listing N.` caption is verified by `asciidoctor_rb::blocks_test`'s
//   `should_prepend_caption_specified_by_listing_caption_attribute_and_number_to_title_of_listing_block_with_title`.
// - `appendix` (`appendix-caption` / `appendix-number`): its `Appendix A:`
//   caption is verified by
//   `sections::appendix::appendix_is_lettered_and_captioned`.

// A captioned title: when the `example-caption` attribute is set, an example
// block's title is prefixed with the caption label and the (auto-incremented)
// number — "Example 1. …". Unsetting `example-caption` drops the prefix and the
// title shows plain.
#[test]
fn captioned_title_prefixes_the_caption_label_and_number() {
    verifies!(
        r#"
Let's assume you've added a title to an example block as follows:

[,asciidoc]
----
.Block that supports captioned title
====
Block content
====
----

The block title will be displayed with a caption label and number, as shown here:

:example-caption: Example
ifdef::example-number[:prev-example-number: {example-number}]
:example-number: 0

.Block that supports captioned title
====
Block content
====

:!example-caption:
ifdef::prev-example-number[:example-number: {prev-example-number}]
:!prev-example-number:

If you unset the `example-caption` attribute, the caption will not be prepended to the title.

.Block that supports captioned title
====
Block content
====

"#
    );

    let captioned = convert(
        ":example-caption: Example\n:example-number: 0\n\n\
         .Block that supports captioned title\n====\nBlock content\n====\n",
    );
    assert_xpath(
        &captioned,
        r#"//div[@class="exampleblock"]/div[@class="title"][text()="Example 1. Block that supports captioned title"]"#,
        1,
    );

    let uncaptioned = convert(
        ":!example-caption:\n\n.Block that supports captioned title\n====\nBlock content\n====\n",
    );
    assert_xpath(
        &uncaptioned,
        r#"//div[@class="exampleblock"]/div[@class="title"][text()="Block that supports captioned title"]"#,
        1,
    );
}

non_normative!(
    r#"
The counter attribute (e.g., `example-number`) can be used to influence the start number for the first block with that context or the next number selected in the sequence for subsequent occurrences.
However, this practice should be used judiciously.

"#
);

// A custom caption: the `caption` attribute on the block replaces the entire
// caption prefix, and a counter reference in its value drives a custom sequence
// — here `{counter:my-example-number:A}` yields "Example A: ".
#[test]
fn custom_caption_attribute_replaces_the_caption() {
    verifies!(
        r#"
The caption can be overridden using the `caption` attribute on the block.
The value of the caption attribute replaces the entire caption, including the space that precedes the title.

Here's how to define a custom caption on a block:

[,asciidoc]
----
.Block Title
[caption="Example {counter:my-example-number:A}: "]
====
Block content
====
----

Here's how the block will be displayed with the custom caption:

.Block Title
[caption="Example {counter:my-example-number:A}: "]
====
Block content
====

Notice we've used a counter attribute in the value of the caption attribute to create a custom number sequence.

"#
    );

    let output = convert(
        ".Block Title\n[caption=\"Example {counter:my-example-number:A}: \"]\n\
         ====\nBlock content\n====\n",
    );
    assert_xpath(
        &output,
        r#"//div[@class="exampleblock"]/div[@class="title"][text()="Example A: Block Title"]"#,
        1,
    );
}

// A caveat about xrefs to custom-captioned blocks.
non_normative!(
    r#"
If you refer to a block with a custom caption using an xref, you may not get the result that you expect.
Therefore, it's always best to define custom xref:attributes:id.adoc#customize-automatic-xreftext[xreftext] when you define a custom caption.
"#
);
