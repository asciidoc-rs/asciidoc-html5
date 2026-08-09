use std::fs;

use asciidoc_parser::{
    blocks::{Block, FindBlocks, IsBlock},
    parser::SourceLine,
    HasSpan, SafeMode,
};

use crate::{load, load_file_with, tests::sdd::*, Options};

track_file!("ref/asciidoctor/docs/modules/api/pages/sourcemap.adoc");

// Asciidoctor's "Map Source Location of Blocks" page, tracked from the library
// crate. It documents the Ruby sourcemap: an opt-in feature that records a
// `Cursor` (file / dir / lineno / path) on each block's `source_location`.
//
// This project delivers the same capability through `asciidoc-parser`, with two
// structural differences:
//
// * Source locations are always tracked. Every block implements `HasSpan`, so
//   `block.span()` yields a `Span` carrying the block's line, column, and byte
//   offset -- there is no `:sourcemap` option to enable and no preprocessor
//   extension to register. The two "Enable ..." sections are therefore
//   Ruby-specific and non-normative here.
// * The file/line split is recovered differently. `span().line()` is the line
//   in the preprocessed source; `Document::source_map().original_file_and_line`
//   maps that back to a `SourceLine(Option<file>, line)` -- the counterpart to
//   the Cursor's `path`/`lineno`, with `None` standing in for a string input's
//   `nil` file.
//
// One behavioral divergence is called out where it appears below: a block's
// span begins at its first *metadata* line (an anchor/attrlist above the
// block), whereas Asciidoctor's sourcemap skips metadata and reports the first
// content line. That divergence is verified in the docs-page coverage.

// The `doc.adoc` the "Use the sourcemap" example builds: a title, a section,
// and two paragraphs. The first paragraph starts on line 5.
const SAMPLE: &str = "\
= Document Title

== Section

Paragraph.

Another paragraph.
";

// Finds the first paragraph among a node's descendant blocks.
fn first_paragraph<'a, T: FindBlocks<'a>>(node: &'a T) -> &'a Block<'a> {
    node.descendant_blocks()
        .find(|block| block.resolved_context().as_ref() == "paragraph")
        .expect("sample has a paragraph")
}

non_normative!(
    r#"
= Map Source Location of Blocks
:navtitle: Enable the Sourcemap

Since Asciidoctor's primary focus is on converting documents efficiently, it does not attempt to track the source location of blocks when parsing by default.
However, such information can be useful for extracting information from the source document, improving error messages, and for use in extensions.
Therefore, Asciidoctor provides a flag to map the source location of blocks, known as the sourcemap.
This page examples how to enable the sourcemap and how to make use of the information it provides.

"#
);

// Descriptive overview. The claim that "the start of the block does not include
// any block metadata" is where this crate diverges: `asciidoc-parser`'s span
// begins at the first metadata line, not the first content line (verified in
// the docs-page coverage). The concrete "start of each block" behavior is
// verified by the "Use the sourcemap" example below.
non_normative!(
    r#"
== What does the sourcemap provide?

The sourcemap provides line and file information for all blocks in the parsed document.
Specifically, it provides information about the start of each block.
The start of the block does not include any block metadata (block anchor and block attributes) above the block.

TIP: The sourcemap also tracks the start line of every preprocessor conditional directive so its position can be reported if the directive isn't closed.
If the sourcemap isn't enabled, the location at the end of the document is used instead.

The sourcemap only adds source location information to blocks.
It does not track the source location for inline elements, such as formatted text or an inline image, or for attribute entries.

The sourcemap information is available on the `source_location` property of the block.
When the sourcemap is enabled, the value of this property is a `Cursor` object.
The `Cursor` object contains the following properties:

[horizontal]
file:: the absolute filename of the source file where the block starts (if input is a string, the value is `nil`)
dir:: the absolute directory of the source file where the block starts (if input is a string, the value is the base dir)
lineno:: the line number in the source file where the block starts (after any empty or block metadata lines)
path:: the relative path (starting from docdir) of the source file where the block starts (if input is a string, the value is `<stdin>`)

The `lineno` and `file` properties can be accessed as properties with the same name on the block itself.

IMPORTANT: The sourcemap is not perfect.
There are certain edge cases, such as when the block is split across multiple files or the block starts and ends on the last line of an include file, when the sourcemap may report the wrong file or line information.
If you're writing a processor that relies on the sourcemap, it's a good idea to verify that the line at the cursor is the one you expect to find, then adjust accordingly.

"#
);

// The two enabling techniques are Ruby-specific: this crate always tracks
// source locations, so there is no `:sourcemap` option and no preprocessor
// extension to register.
non_normative!(
    r#"
== Enable using :sourcemap option

The sourcemap feature can be controlled from the API using the `:sourcemap` option.
The value of this option is a boolean.
If the value is `false` (default), the sourcemap is not enabled.
If the value is `true`, the sourcemap is enabled.
The `:sourcemap` option is accepted by all xref:index.adoc#entrypoints[entrypoint methods] (e.g., Asciidoctor#load_file).

Here's an example of how to enable the sourcemap using the API:

[,ruby]
----
doc = Asciidoctor.load_file 'doc.adoc', safe: :safe, sourcemap: true
----

== Enable from extension

You can enable the sourcemap using an Asciidoctor preprocessor extension.
This technique is useful if your extension needs access to the source location of blocks, but you don't want to require users to pass an additional option to Asciidoctor.

[,ruby]
----
Asciidoctor::Document.prepend (Module.new do
  attr_writer :sourcemap
end) unless Asciidoctor::Document.method_defined? :sourcemap=

# A preprocessor that enables the sourcemap feature if not already enabled via the API.
Asciidoctor::Extensions.register do
  preprocessor do
    process do |doc, reader|
      doc.sourcemap = true
      nil
    end
  end
end
----

Now that the sourcemap is enabled, your extension can access the source location of the block elements in the parsed document.

"#
);

// The worked example: the first paragraph of the sample document starts on line
// 5, which `span().line()` reports. For a string input the source map has no
// file (the counterpart to the Cursor's `nil` file), so
// `original_file_and_line` yields `SourceLine(None, 5)`.
#[test]
fn use_the_sourcemap_reports_the_start_line() {
    verifies!(
        r#"
== Use the sourcemap

When the sourcemap is enabled, the parser will store source information on the `source_location` property on each block in the parsed document.
Let's look at an example.

Start by creating the following AsciiDoc file named [.path]_doc.adoc_.

.doc.adoc
[,asciidoc]
----
= Document Title

== Section

Paragraph.

Another paragraph.
----

Now, load this file using Asciidoctor with the `:sourcemap` option enabled:

[,ruby]
----
doc = Asciidoctor.load_file 'doc.adoc', safe: :safe, sourcemap: true
----

Let's find the first paragraph in the document and inspect its source location:

[,ruby]
----
first_paragraph = (doc.find_by context: :paragraph)[0]
puts first_paragraph.source_location
----

You'll see output similar to what's shown below:

[.output]
....
doc.adoc: line 5
....

"#
    );

    let doc = load(SAMPLE);
    let paragraph = first_paragraph(&doc);

    // The paragraph starts on line 5, as the example's output shows.
    assert_eq!(paragraph.span().line(), 5);

    // For a string input there is no file (the Cursor's `nil`); the source map
    // translates the line to itself with no file.
    assert_eq!(
        doc.source_map().original_file_and_line(5),
        Some(SourceLine(None, 5)),
    );
}

// The `pp` Cursor representation and the `file`/`lineno` block accessors are
// Ruby-specific. Here the line comes from `span().line()` and the file from the
// source map, shown in the tests above and below.
non_normative!(
    r#"
What you're seeing here is the string value of the cursor.
There's more information to see if you replace `puts` with `pp`:

[.output]
....
#<Asciidoctor::Reader::Cursor
 @dir="/path/to/docdir",
 @file="/path/to/docdir/doc.adoc",
 @lineno=5,
 @path="doc.adoc">
....

Since file and lineno are the most useful properties, they can be accessed directly from the block:

[,ruby]
----
puts first_paragraph.file
puts first_paragraph.lineno
----

"#
);

// Moving the section into an include file: the paragraph's location follows it
// into that file. The source map translates the paragraph's preprocessed line
// back to `partials/section.adoc`, line 3 -- the Cursor's `path` and `lineno`.
#[test]
fn source_location_follows_a_block_into_an_include_file() {
    verifies!(
        r#"
If you move the source of the section to an include file, as shown here:

.doc.adoc
[,asciidoc]
----
= Document Title

\include::partials/section.adoc[]
----

then the source location will follow the paragraph into that file:

[.output]
....
#<Asciidoctor::Reader::Cursor
 @dir="/path/to/docdir/partials",
 @file="/path/to/docdir/partials/section.adoc",
 @lineno=3,
 @path="partials/section.adoc">
....

"#
    );

    // Lay out `doc.adoc` and `partials/section.adoc` in a unique temp directory
    // so the include resolves relative to the primary document.
    let dir = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-sourcemap-{}",
        std::process::id()
    ));
    let partials = dir.join("partials");
    fs::create_dir_all(&partials).expect("create temp dirs");
    fs::write(
        dir.join("doc.adoc"),
        "= Document Title\n\ninclude::partials/section.adoc[]\n",
    )
    .expect("write doc.adoc");
    fs::write(
        partials.join("section.adoc"),
        "== Section\n\nParagraph.\n\nAnother paragraph.\n",
    )
    .expect("write section.adoc");

    // Includes resolve only outside the most restrictive safe modes.
    let options = Options::default().safe_mode(SafeMode::Safe);
    let doc =
        load_file_with(dir.join("doc.adoc"), &options).expect("load_file_with reads the file");

    let paragraph = first_paragraph(&doc);
    let line = paragraph.span().line();

    assert_eq!(
        doc.source_map().original_file_and_line(line),
        Some(SourceLine(Some("partials/section.adoc".to_string()), 3)),
    );

    let _ = fs::remove_dir_all(&dir);
}

// Divergence: Asciidoctor's sourcemap skips block metadata and reports the
// first content line, making `lineno` one greater when an anchor is added. This
// crate's span instead begins at the metadata line, so its `line()` is one
// *smaller*, not greater. The metadata-inclusive behavior is verified in the
// docs-page coverage.
non_normative!(
    r#"
If the block has metadata lines, those lines are skipped when reporting the start location of the block.
For example, let's assume the paragraph is defined as follows:

[,asciidoc]
----
[#p1]
Paragraph.
----

The lineno of the paragraph in the source location is now one greater than before:

[.output]
....
#<Asciidoctor::Reader::Cursor
 @dir="/path/to/docdir/partials",
 @file="/path/to/docdir/partials/section.adoc",
 @lineno=4,
 @path="partials/section.adoc">
....

"#
);

non_normative!(
    r#"
If you're writing a custom converter, the source location is not available for inline elements.
However, you can access the source location of the parent element (e.g., `node.parent.source_location`), which should at least get you close to the location of the element.
"#
);
