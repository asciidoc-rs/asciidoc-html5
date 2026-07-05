//! HTML5 renderer for [AsciiDoc](https://asciidoc.org).
//!
//! This crate converts AsciiDoc source (as parsed by [`asciidoc_parser`]) into
//! an HTML5 document. The goal is output that is compatible with the default
//! `html5` backend of [Asciidoctor], so that documents render identically
//! whether they are processed by Asciidoctor or by this crate.
//!
//! The library deliberately depends only on [`asciidoc_parser`] and the
//! standard library. It carries no CLI or argument-parsing dependencies so that
//! it stays lean enough to embed in larger tools (for example, a future
//! Antora-style static site generator) that need HTML conversion as one step of
//! a bigger pipeline.
//!
//! # Examples
//!
//! ```no_run
//! let html = asciidoc_html5::convert("= Hello\n\nWorld.");
//! println!("{html}");
//! ```
//!
//! [Asciidoctor]: https://asciidoctor.org

use asciidoc_parser::{Document, Parser};

mod html;
mod renderer;

/// Parses `source` as AsciiDoc and renders it to a complete HTML5 document.
///
/// This is the convenience entry point for callers that start from raw
/// AsciiDoc text. It parses the source with a default [`Parser`] and then hands
/// the resulting [`Document`] to [`convert_document`].
///
/// For callers that already hold a parsed [`Document`] (for example, to inspect
/// or transform it first), call [`convert_document`] directly.
pub fn convert(source: &str) -> String {
    let document = Parser::default().parse(source);
    convert_document(&document)
}

/// Renders an already-parsed [`Document`] to a complete HTML5 document.
///
/// The returned string is a standalone HTML5 document: a `<!DOCTYPE html>`
/// declaration followed by `<html>`, a `<head>` carrying the document title and
/// generator metadata, and a `<body>` whose structure mirrors Asciidoctor's
/// default `html5` backend.
///
/// The renderer walks the document in block order, wrapping the HTML fragments
/// the parser has already produced (see the note on inline substitution below)
/// in Asciidoctor's block-level scaffolding. The traversal is described in
/// `src/renderer.rs` and in `ARCHITECTURE.md`.
///
/// # Inline substitution is the parser's job
///
/// `asciidoc-parser` applies inline substitutions (quotes, replacements,
/// macros, cross references, attribute references) *eagerly*, at parse time,
/// through its default HTML [`InlineSubstitutionRenderer`]. Every block's
/// [`rendered_content`] and [`title`] is therefore already an
/// Asciidoctor-compatible inline HTML fragment. This crate does not
/// re-implement inline formatting; it only assembles block structure around
/// those fragments.
///
/// # Baseline coverage
///
/// This is an early baseline. It renders the document skeleton, the header, and
/// paragraphs, sections, the preamble, verbatim (listing/literal) blocks, and
/// thematic breaks. Constructs that are not yet wired up (lists, tables,
/// admonitions, quotes, images, and the delimited example/sidebar/open blocks)
/// emit a visible `<!-- asciidoc-html5: unsupported … -->` comment so the
/// output stays well-formed and the gaps are easy to see. The aim, as coverage
/// grows, is parity with Asciidoctor's `html5` backend.
///
/// [`InlineSubstitutionRenderer`]: asciidoc_parser::parser::InlineSubstitutionRenderer
/// [`rendered_content`]: asciidoc_parser::blocks::IsBlock::rendered_content
/// [`title`]: asciidoc_parser::blocks::IsBlock::title
pub fn convert_document(document: &Document<'_>) -> String {
    renderer::render_document(document)
}
