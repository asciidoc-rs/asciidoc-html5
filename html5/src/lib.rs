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
/// Intended behavior, to be filled in as the renderer is built out:
///
/// - Emit the same DOCTYPE, `lang`, and `<meta charset>` preamble Asciidoctor
///   emits, along with the `<title>` drawn from the document header.
/// - Render the document header as Asciidoctor does: a `<div id="header">`
///   containing the `<h1>` title plus author, revision, and (when enabled)
///   table-of-contents markup.
/// - Walk the document's blocks in order, mapping each AsciiDoc block
///   (paragraphs, sections, lists, delimited blocks, tables, admonitions,
///   images, listings, and so on) to the corresponding Asciidoctor HTML shape,
///   including the customary `id`, `class`, and role attributes.
/// - Apply inline substitutions (quotes, replacements, macros, cross
///   references, attribute references) so that inline formatting matches the
///   parser's substitution model.
/// - Close with the `<div id="footer">` and generator note that Asciidoctor
///   appends, then the closing `</body></html>`.
///
/// The aim throughout is byte-for-byte parity with Asciidoctor's `html5`
/// backend for the constructs that are supported.
pub fn convert_document(document: &Document<'_>) -> String {
    // The renderer is not implemented yet. Silence the unused-parameter lint
    // until the real conversion logic lands.
    let _ = document;
    todo!("render the parsed AsciiDoc document to Asciidoctor-compatible HTML5")
}
