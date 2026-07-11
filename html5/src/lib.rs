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
//! To convert an AsciiDoc file on disk, use [`convert_file`]:
//!
//! ```no_run
//! let html = asciidoc_html5::convert_file("document.adoc")?;
//! println!("{html}");
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! [Asciidoctor]: https://asciidoctor.org

use std::{fs, io, path::Path};

use asciidoc_parser::{Document, Parser};

mod docinfo_handler;
mod html;
mod include_handler;
mod options;
mod renderer;

pub use asciidoc_parser::SafeMode;
pub use options::Options;

#[cfg(test)]
mod tests;

/// Parses `source` as AsciiDoc and renders it to a complete HTML5 document.
///
/// This is the convenience entry point for callers that start from raw
/// AsciiDoc text. It parses the source with a default [`Parser`] and then hands
/// the resulting [`Document`] to [`convert_document`].
///
/// For callers that already hold a parsed [`Document`] (for example, to inspect
/// or transform it first), call [`convert_document`] directly. To supply
/// document attributes from outside the source (Asciidoctor's `-a name=value`),
/// use [`convert_with`].
pub fn convert(source: &str) -> String {
    convert_with(source, &Options::default())
}

/// Parses `source` as AsciiDoc and renders it to a complete HTML5 document,
/// seeding the parser with the document attributes carried by `options`.
///
/// This is the attribute-aware counterpart to [`convert`]: the attributes in
/// `options` are the equivalent of Asciidoctor's `-a name=value` CLI option and
/// the `attributes` API option, supplying (and, for overrides, locking) values
/// from outside the document source. See [`Options`] for override vs. soft-set
/// precedence.
///
/// # Examples
///
/// ```
/// use asciidoc_html5::{convert_with, Options};
///
/// let opts = Options::new().set("linkcss");
/// let html = convert_with("= Doc\n\nBody.", &opts);
/// assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
/// ```
pub fn convert_with(source: &str, options: &Options) -> String {
    let mut parser = options.apply(Parser::default());
    let document = parser.parse(source);
    convert_document(&document)
}

/// Reads the AsciiDoc file at `path` and renders it to a complete HTML5
/// document.
///
/// This is the file-based counterpart to [`convert`]: it reads `path` as UTF-8
/// and hands the contents to [`convert`]. It is the simplest way to turn an
/// AsciiDoc file on disk into a full HTML5 document, mirroring Asciidoctor's
/// `convert_file`.
///
/// # Errors
///
/// Returns the [`io::Error`] from reading `path` — for example, when the file
/// does not exist or does not contain valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// let html = asciidoc_html5::convert_file("document.adoc")?;
/// println!("{html}");
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn convert_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    convert_file_with(path, &Options::default())
}

/// Reads the AsciiDoc file at `path` and renders it to a complete HTML5
/// document, seeding the parser with the document attributes carried by
/// `options`.
///
/// This is the attribute-aware counterpart to [`convert_file`], the file-based
/// counterpart to [`convert_with`]. See [`Options`] for the attributes it
/// accepts and their override vs. soft-set precedence.
///
/// The `path` is recorded as the primary document (see
/// [`Options::input_file`]), so its top-level `include::` directives resolve
/// against the file's own directory, and — unless the caller sets one with
/// [`Options::base_dir`] — that directory becomes the base directory that
/// anchors and (under a jailed safe mode) confines include resolution.
///
/// # Errors
///
/// Returns the [`io::Error`] from reading `path` — for example, when the file
/// does not exist or does not contain valid UTF-8.
pub fn convert_file_with<P: AsRef<Path>>(path: P, options: &Options) -> io::Result<String> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    Ok(convert_with(&source, &options.clone().input_file(path)))
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
/// thematic and page breaks. Constructs that are not yet wired up (lists,
/// tables, admonitions, quotes, images, and the delimited example/sidebar/open
/// blocks) emit a visible `<!-- asciidoc-html5: unsupported … -->` comment so
/// the output stays well-formed and the gaps are easy to see. The aim, as
/// coverage grows, is parity with Asciidoctor's `html5` backend.
///
/// [`InlineSubstitutionRenderer`]: asciidoc_parser::parser::InlineSubstitutionRenderer
/// [`rendered_content`]: asciidoc_parser::blocks::IsBlock::rendered_content
/// [`title`]: asciidoc_parser::blocks::IsBlock::title
pub fn convert_document(document: &Document<'_>) -> String {
    renderer::render_document(document)
}
