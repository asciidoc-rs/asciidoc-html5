use std::fs;

use asciidoc_parser::{
    blocks::{FindBlocks, IsBlock},
    parser::SourceLine,
    HasSpan, SafeMode,
};

use crate::{load, load_file_with, tests::sdd::*, Options};

track_file!("docs/modules/api/pages/sourcemap.adoc");

// This crate's "Source Locations" API page, adapted from Asciidoctor's
// "Map Source Location of Blocks" page. The prose is tracked as non-normative;
// each Rust snippet the page shows is verified by an ordinary test in this
// module that runs the same call. Source locations come from `asciidoc-parser`:
// every block implements `HasSpan`, so `block.span()` yields a `Span` carrying
// the block's line, column, and byte offset, and `Document::source_map`
// translates a preprocessed line back to its original include file and line.

// The document the "Read a block's source location" example loads: a title, a
// section, and two paragraphs. The first paragraph starts on line 5.
const SAMPLE: &str = "\
= Document Title

== Section

Paragraph.

Another paragraph.
";

// Finds the first paragraph among a node's descendant blocks.
fn first_paragraph<'a, T: FindBlocks<'a>>(node: &'a T) -> &'a asciidoc_parser::blocks::Block<'a> {
    node.descendant_blocks()
        .find(|block| block.resolved_context().as_ref() == "paragraph")
        .expect("sample has a paragraph")
}

non_normative!(
    r#"
= Map the Source Location of Blocks
:navtitle: Source Locations
:description: How to read the source line, column, and file of a block from a loaded document, the asciidoc-parser counterpart to Asciidoctor's sourcemap.

Every block in a document loaded by `asciidoc-html5` carries its source location.
Where Asciidoctor tracks block locations only when you enable its sourcemap,
https://crates.io/crates/asciidoc-parser[`asciidoc-parser`] records the location
of every block as it parses, so the information is always available -- there is
no flag to turn on. The location lives on the block's
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/span/struct.Span.html[`Span`],
which you reach through the
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/span/trait.HasSpan.html[`HasSpan`]
trait.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

== What a source location provides

`HasSpan::span` returns a `Span` marking where the block begins. From it you can
read:

`line()`:: the 1-based line number where the block starts.
`col()`:: the 1-based column where the block starts.
`byte_offset()`:: the 0-based byte offset into the parsed source where the block starts.
`data()`:: the source text of the block itself.

Asciidoctor exposes a `Cursor` with `file`, `dir`, `lineno`, and `path`
properties instead. The line number is the property both models share; the column
and byte offset are extra detail `asciidoc-parser` provides, and the file
information is recovered separately through the source map (see below).

[IMPORTANT]
====
Source locations are tracked for blocks. Inline elements -- formatted text, an
inline image -- do not carry their own block location; reach for the parent
block's span instead.
====

== Source locations are always available

`asciidoc-parser` records source locations unconditionally, so unlike Asciidoctor
there is no `:sourcemap` option to pass and no preprocessor extension to register
just to switch the feature on. Every loaded document already carries the
information; you only have to read it.

"#
);

// The "Read a block's source location" worked example: load the sample, find
// the first paragraph, and read its start line and column.
#[test]
fn read_a_blocks_source_location() {
    verifies!(
        r#"
== Read a block's source location

Suppose you load this document:

[,rust]
----
let doc = asciidoc_html5::load(
    "= Document Title\n\
     \n\
     == Section\n\
     \n\
     Paragraph.\n\
     \n\
     Another paragraph.\n",
);
----

Find the first paragraph and read where it starts:

[,rust]
----
use asciidoc_parser::HasSpan;
use asciidoc_parser::blocks::{FindBlocks, IsBlock};

let paragraph = doc
    .descendant_blocks()
    .find(|block| block.resolved_context().as_ref() == "paragraph")
    .unwrap();

assert_eq!(paragraph.span().line(), 5);
assert_eq!(paragraph.span().col(), 1);
----

The paragraph begins on line 5, matching what Asciidoctor's sourcemap reports for
the same document.

"#
    );

    let doc = load(SAMPLE);

    let paragraph = doc
        .descendant_blocks()
        .find(|block| block.resolved_context().as_ref() == "paragraph")
        .unwrap();

    assert_eq!(paragraph.span().line(), 5);
    assert_eq!(paragraph.span().col(), 1);
}

// The "Translate a location through includes" example: with includes resolved
// under `SafeMode::Safe`, the block's preprocessed line translates through the
// source map back to the include file and its own line.
#[test]
fn translate_a_location_through_includes() {
    verifies!(
        r#"
== Translate a location through includes

When a document pulls in other files with `include::`, the line a block reports is
its line in the preprocessed source -- the single stream the parser sees after
includes are spliced in. To recover the original file and line, pass that line to
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/parser/struct.SourceMap.html[`Document::source_map`]
and call `original_file_and_line`. It returns a
https://docs.rs/asciidoc-parser/latest/asciidoc_parser/parser/struct.SourceLine.html[`SourceLine`],
whose first field is the include's path (or `None` for the top-level input) and
whose second field is the 1-based line within that file.

Include resolution runs only outside the most restrictive safe modes, so load the
file under `SafeMode::Safe` (the counterpart to Asciidoctor's `safe: :safe`):

[,rust]
----
use asciidoc_parser::blocks::{FindBlocks, IsBlock};
use asciidoc_parser::parser::SourceLine;
use asciidoc_parser::{HasSpan, SafeMode};

// doc.adoc holds `include::partials/section.adoc[]`, and the section file's
// first paragraph is on its own line 3.
let options = asciidoc_html5::Options::default().safe_mode(SafeMode::Safe);
let doc = asciidoc_html5::load_file_with("doc.adoc", &options)?;

let paragraph = doc
    .descendant_blocks()
    .find(|block| block.resolved_context().as_ref() == "paragraph")
    .unwrap();

let line = paragraph.span().line();
assert_eq!(
    doc.source_map().original_file_and_line(line),
    Some(SourceLine(Some("partials/section.adoc".to_string()), 3)),
);
----

The paragraph's location follows it into the include file, just as Asciidoctor's
sourcemap reports `partials/section.adoc`, line 3.

"#
    );

    // Lay out `doc.adoc` and `partials/section.adoc` in a unique temp directory
    // so the include resolves relative to the primary document.
    let dir = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-sourcemap-{}",
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

// The first limitation: a block's span begins at its first metadata line, not
// at the first content line as Asciidoctor's sourcemap reports. Loading a
// paragraph carrying a `[#p1]` anchor shows the span starting on the metadata
// line.
#[test]
fn span_includes_block_metadata() {
    verifies!(
        r#"
== Known limitations

A block's span begins at the first line of the block, *including* any block
metadata -- an anchor or attribute-list line such as `[#p1]` written above the
block. Asciidoctor's sourcemap skips those metadata lines and reports the first
line of content instead, so for a block that carries metadata this crate's
`line()` points one or more lines earlier than Asciidoctor's `lineno`. If you need
the content line, advance past the metadata lines yourself.

"#
    );

    // The paragraph's content is on line 6, but the `[#p1]` metadata is on line
    // 5; the span starts at the metadata line, one earlier than Asciidoctor.
    let doc = load("= Document Title\n\n== Section\n\n[#p1]\nParagraph.\n");
    let paragraph = first_paragraph(&doc);

    assert_eq!(paragraph.span().line(), 5);
    assert!(paragraph.span().data().starts_with("[#p1]"));
}

non_normative!(
    r#"
Source locations are not available for inline elements. As in Asciidoctor, you can
read the source location of the enclosing block, which at least gets you close to
the element.

That covers reading the source location of blocks in a loaded document.
"#
);
