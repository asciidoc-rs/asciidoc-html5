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
//! To *load* a document — parse it without converting — use [`load`] (or
//! [`load_file`]), inspect or transform the returned [`Document`], then render
//! it with [`convert_document`]:
//!
//! ```
//! let doc = asciidoc_html5::load("= Hello\n\nWorld.");
//! assert_eq!(doc.doctitle(), Some("Hello"));
//! let html = asciidoc_html5::convert_document(&doc);
//! ```
//!
//! [Asciidoctor]: https://asciidoctor.org

use std::{fs, io, path::Path};

use asciidoc_parser::Parser;

mod asset_writer;
mod copycss;
mod docinfo_handler;
mod html;
mod include_handler;
mod options;
mod renderer;

pub use asciidoc_parser::{Document, SafeMode};
pub use asset_writer::{AssetWriter, DirAssetWriter};
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
/// or transform it first), call [`convert_document`] directly; produce that
/// document with [`load`]/[`load_file`], which parse under [`Options`] the way
/// this function does. To supply document attributes from outside the source
/// (Asciidoctor's `-a name=value`), or to embed a custom stylesheet, use
/// [`convert_with`].
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
/// Unlike [`convert_document`], this path also honors a custom stylesheet when
/// the document selects one and it is *embedded* (rather than linked): the CSS
/// comes from [`Options::stylesheet_content`] when the caller supplied it,
/// otherwise it is read from disk relative to the base directory (see
/// [`Options::base_dir`]/[`Options::input_file`]) under the same safe-mode jail
/// as `include::` targets. Without a base directory — the plain [`convert`]
/// case — an embedded custom stylesheet has no source to read, so its block is
/// omitted.
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
    let document = load_with(source, options);
    render(&document, options)
}

/// Parses and renders `source` like [`convert_with`], and additionally emits
/// the companion files the conversion calls for through `writer`.
///
/// Today the only companion file is the stylesheet copied under the `copycss`
/// attribute: when the stylesheet is *linked* and `copycss` is enabled (its
/// default below the `secure` safe mode), the linked stylesheet is written
/// through `writer` at its path relative to the output directory — the default
/// stylesheet as `asciidoctor.css` (under `stylesdir` when set), a custom
/// stylesheet at its `stylesdir` web path, the same location the head links.
/// This is the piece [`convert_with`] cannot do on its own: the library
/// renders text to text and does not own the output directory, so a caller that
/// wants `copycss` to take effect supplies an [`AssetWriter`] (for the
/// filesystem, [`DirAssetWriter`] rooted at the output directory) to receive
/// the write.
///
/// The returned HTML is byte-identical to [`convert_with`]'s; `copycss` is a
/// pure file side effect and never changes the document. A custom stylesheet's
/// bytes are read from disk under the same base directory and safe-mode jail as
/// an embedded stylesheet, so a copy happens only when a base directory anchors
/// the read (see [`Options::base_dir`]/[`Options::input_file`]).
///
/// # Errors
///
/// Returns any [`io::Error`] raised while writing a companion file through
/// `writer`.
pub fn convert_with_writer(
    source: &str,
    options: &Options,
    writer: &mut impl AssetWriter,
) -> io::Result<String> {
    let document = load_with(source, options);
    let html = render(&document, options);
    emit_stylesheet_copy(&document, options, writer)?;
    Ok(html)
}

/// Renders `document` to HTML, resolving the embedded custom stylesheet the way
/// [`convert_with`] does. Shared by the string entry points with and without an
/// [`AssetWriter`].
fn render(document: &Document<'_>, options: &Options) -> String {
    // A custom, embedded stylesheet takes its CSS from the caller when supplied,
    // otherwise from disk. Keeping this a separate binding keeps the borrow of
    // `document` from the read helper separate from the render call.
    let embedded = options
        .custom_stylesheet()
        .map(str::to_owned)
        .or_else(|| read_embedded_stylesheet(document, options));

    renderer::render_document(document, embedded.as_deref())
}

/// Writes the `copycss` stylesheet copy through `writer`, when the document
/// calls for one. A no-op otherwise.
fn emit_stylesheet_copy(
    document: &Document<'_>,
    options: &Options,
    writer: &mut impl AssetWriter,
) -> io::Result<()> {
    if let Some(copy) = copycss::stylesheet_copy(document, options) {
        writer.write_asset(&copy.dest, copy.content.as_bytes())?;
    }
    Ok(())
}

/// Reads a custom stylesheet from disk when the document selects one to
/// *embed*, resolving it against the base directory the way an `include::`
/// target resolves and confining the read to the safe mode's jail. Returns
/// `None` when there is nothing to read — the stylesheet is linked, a URI, the
/// default, or unset — or when no base directory anchors the lookup (the plain
/// [`convert`] case) or the file cannot be read.
fn read_embedded_stylesheet(document: &Document<'_>, options: &Options) -> Option<String> {
    let target = renderer::embeddable_stylesheet_target(document)?;
    let base_dir = options.effective_base_dir()?;
    let safe = options.safe_mode_or_default();

    let path = include_handler::resolve(&base_dir, safe, None, &target);
    include_handler::read_confined(&base_dir, safe, &path)
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

/// Reads the AsciiDoc file at `path`, renders it like [`convert_file_with`],
/// and emits the conversion's companion files through `writer`.
///
/// This is the file-based counterpart to [`convert_with_writer`] and the
/// [`AssetWriter`]-aware counterpart to [`convert_file_with`]: reading `path`
/// records it as the primary document (anchoring its `include::` resolution
/// and, absent an explicit [`base_dir`](Options::base_dir), the base
/// directory), so a custom stylesheet copied under `copycss` is read relative
/// to the file's own directory. See [`convert_with_writer`] for what `copycss`
/// writes.
///
/// # Errors
///
/// Returns the [`io::Error`] from reading `path`, or any error raised while
/// writing a companion file through `writer`.
pub fn convert_file_with_writer<P: AsRef<Path>>(
    path: P,
    options: &Options,
    writer: &mut impl AssetWriter,
) -> io::Result<String> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    convert_with_writer(&source, &options.clone().input_file(path), writer)
}

/// Parses `source` as AsciiDoc into a [`Document`] — the *load* step on its
/// own, without converting.
///
/// This is the parse-only counterpart to [`convert`]: where `convert` parses
/// and renders in one call, `load` stops at the parsed [`Document`], the
/// in-memory model you can inspect (its title, attributes, and block structure)
/// or transform before rendering it with [`convert_document`]. It mirrors
/// Asciidoctor's `load` entry point.
///
/// The returned document is fully owned (`Document<'static>`): the parser
/// copies what it needs from `source`, so the document does not borrow from it
/// and can be returned, stored, or moved freely.
///
/// `load` parses under the default [`Options`] — the same configuration
/// [`convert`] uses — so `convert_document(&load(source))` equals
/// `convert(source)`. To parse under externally-supplied attributes or a chosen
/// safe mode, use [`load_with`].
///
/// # Examples
///
/// ```
/// let doc = asciidoc_html5::load("= Hello\n\nWorld.");
/// assert_eq!(doc.doctitle(), Some("Hello"));
/// ```
pub fn load(source: &str) -> Document<'static> {
    load_with(source, &Options::default())
}

/// Parses `source` as AsciiDoc into a [`Document`], applying the parse-time
/// settings carried by `options` — the *load* step with [`Options`].
///
/// This is the parse-only counterpart to [`convert_with`] and the
/// [`Options`]-aware counterpart to [`load`]. `options` seeds the parser
/// exactly as [`convert_with`] does: the externally-supplied document
/// attributes (with override vs. soft-default precedence), the [safe
/// mode](Options::safe_mode), and — when the caller names a base directory or
/// primary file — `include::` and docinfo resolution confined by the safe
/// mode's jail. These settings live across several lower-level
/// `asciidoc-parser` [`Parser`] builder methods; `load_with` is the single call
/// that applies the whole [`Options`] bundle.
///
/// Options that affect only *rendering* — a custom stylesheet's embedded
/// contents (see [`Options::stylesheet_content`]) — have no effect here, since
/// `load_with` does not render. When those matter, render with [`convert_with`]
/// (passing the same `options`) rather than [`convert_document`], which renders
/// without them.
pub fn load_with(source: &str, options: &Options) -> Document<'static> {
    let mut parser = options.apply(Parser::default());
    parser.parse(source)
}

/// Reads the AsciiDoc file at `path` and parses it into a [`Document`] — the
/// *load* step for a file on disk.
///
/// This is the file-based counterpart to [`load`] and the parse-only
/// counterpart to [`convert_file`]: it reads `path` as UTF-8 and parses the
/// contents, returning the owned [`Document`]. It mirrors Asciidoctor's
/// `load_file` entry point.
///
/// # Errors
///
/// Returns the [`io::Error`] from reading `path` — for example, when the file
/// does not exist or does not contain valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// let doc = asciidoc_html5::load_file("document.adoc")?;
/// println!("{:?}", doc.doctitle());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn load_file<P: AsRef<Path>>(path: P) -> io::Result<Document<'static>> {
    load_file_with(path, &Options::default())
}

/// Reads the AsciiDoc file at `path` and parses it into a [`Document`],
/// applying the parse-time settings carried by `options` — the *load* step for
/// a file, with [`Options`].
///
/// This is the file-based counterpart to [`load_with`] and the parse-only
/// counterpart to [`convert_file_with`]. Like [`convert_file_with`], it records
/// `path` as the primary document (see [`Options::input_file`]), so the file's
/// top-level `include::` directives resolve against its own directory and —
/// absent an explicit [`base_dir`](Options::base_dir) — that directory anchors
/// and (under a jailed safe mode) confines include and docinfo resolution.
///
/// # Errors
///
/// Returns the [`io::Error`] from reading `path` — for example, when the file
/// does not exist or does not contain valid UTF-8.
pub fn load_file_with<P: AsRef<Path>>(path: P, options: &Options) -> io::Result<Document<'static>> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    Ok(load_with(&source, &options.clone().input_file(path)))
}

/// Renders an already-parsed [`Document`] to a complete HTML5 document.
///
/// This is the render-only half of the load-and-convert split: pair it with
/// [`load`]/[`load_file`] (which parse under [`Options`]) to run the load and
/// convert steps separately, capturing or transforming the [`Document`] in
/// between.
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
    renderer::render_document(document, None)
}

#[cfg(test)]
mod writer_tests {
    use std::path::PathBuf;

    use crate::{
        asset_writer::RecordingAssetWriter, convert_file_with_writer, convert_with,
        convert_with_writer, Options, SafeMode,
    };

    // The HTML `convert_with_writer` returns is identical to `convert_with`'s —
    // `copycss` is a side effect that never changes the document — and the
    // default stylesheet is offered to the writer as `asciidoctor.css`.
    #[test]
    fn writer_copies_the_default_stylesheet_without_changing_the_html() {
        let source = "= Doc\n\nBody.";
        let options = Options::new().safe_mode(SafeMode::Safe).set("linkcss");

        let mut writer = RecordingAssetWriter::default();
        let html = convert_with_writer(source, &options, &mut writer).expect("convert");

        assert_eq!(html, convert_with(source, &options));
        assert_eq!(writer.written.len(), 1);
        let (path, content) = &writer.written[0];
        assert_eq!(path, &PathBuf::from("asciidoctor.css"));
        assert!(content.starts_with(b"/*"));
    }

    // With no `linkcss` (the stylesheet is embedded), the writer is never
    // called.
    #[test]
    fn writer_is_untouched_when_the_stylesheet_is_embedded() {
        let mut writer = RecordingAssetWriter::default();
        let options = Options::new().safe_mode(SafeMode::Safe);
        convert_with_writer("= Doc\n\nBody.", &options, &mut writer).expect("convert");
        assert!(writer.written.is_empty());
    }

    // `convert_file_with_writer` anchors the custom stylesheet read at the input
    // file's own directory, so a linked custom stylesheet is copied to its web
    // path with the on-disk contents.
    #[test]
    fn file_writer_copies_a_custom_stylesheet_from_the_input_directory() {
        let dir = std::env::temp_dir().join(format!("adoc-lib-copycss-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("css")).expect("create dirs");
        std::fs::write(dir.join("main.adoc"), "= Doc\n\nBody.").expect("write adoc");
        std::fs::write(dir.join("css/theme.css"), "body { color: teal; }").expect("write css");

        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .attribute("stylesdir", "css")
            .attribute("stylesheet", "theme.css");

        let mut writer = RecordingAssetWriter::default();
        convert_file_with_writer(dir.join("main.adoc"), &options, &mut writer).expect("convert");

        assert_eq!(writer.written.len(), 1);
        let (path, content) = &writer.written[0];
        assert_eq!(path.to_string_lossy().replace('\\', "/"), "css/theme.css");
        assert_eq!(content, b"body { color: teal; }");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // `convert_file_with_writer` honors the `copycss=<path>` read-from override
    // (Asciidoctor's copy/link split): the bytes are read from the `copycss`
    // path, but the copy is written to — and the HTML links — the `stylesheet`
    // web path. This is the API-surface counterpart to the `adoc` copy/link
    // split the CLI crate verifies and the `stylesheet_copy` unit test resolves.
    #[test]
    fn file_writer_honors_the_copycss_read_from_override() {
        let dir =
            std::env::temp_dir().join(format!("adoc-lib-copycss-split-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("vendor")).expect("create dirs");
        std::fs::write(dir.join("main.adoc"), "= Doc\n\nBody.").expect("write adoc");
        std::fs::write(dir.join("vendor/theme.css"), "body { color: maroon; }").expect("write css");

        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .attribute("copycss", "vendor/theme.css")
            .attribute("stylesheet", "published.css");

        let mut writer = RecordingAssetWriter::default();
        let html = convert_file_with_writer(dir.join("main.adoc"), &options, &mut writer)
            .expect("convert");

        // The HTML links the stylesheet at its own web path, not the copycss one.
        assert!(html.contains(r#"<link rel="stylesheet" href="./published.css">"#));

        // The copy is written to that same web path, but its bytes come from the
        // copycss read-from path.
        assert_eq!(writer.written.len(), 1);
        let (path, content) = &writer.written[0];
        assert_eq!(path.to_string_lossy().replace('\\', "/"), "published.css");
        assert_eq!(content, b"body { color: maroon; }");

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod load_tests {
    use asciidoc_parser::{blocks::IsBlock as _, document::InterpretedValue};

    use crate::{
        convert, convert_document, convert_with, load, load_file, load_file_with, load_with,
        Options, SafeMode,
    };

    // `load` returns the parsed document, and rendering it with
    // `convert_document` reproduces `convert` — the load and convert steps split
    // apart and reassembled match the combined path.
    #[test]
    fn load_then_convert_document_matches_convert() {
        let source = "= Hello\n\nWorld.";

        let doc = load(source);
        assert_eq!(doc.doctitle(), Some("Hello"));
        assert!(doc.nested_blocks().next().is_some());

        assert_eq!(convert_document(&doc), convert(source));
    }

    // `load_with` applies the `Options` at parse time: an externally-supplied
    // attribute lands in the document model, and rendering the loaded document
    // matches `convert_with` for the same options.
    #[test]
    fn load_with_applies_options_at_parse_time() {
        let source = "= Doc\n\nBody.";
        let opts = Options::new()
            .safe_mode(SafeMode::Server)
            .attribute("author", "Ada Lovelace");

        let doc = load_with(source, &opts);
        assert_eq!(
            doc.attribute_value("author"),
            InterpretedValue::Value("Ada Lovelace".to_string())
        );

        // The separately loaded-and-rendered path equals the combined one.
        assert_eq!(convert_document(&doc), convert_with(source, &opts));
    }

    // `load_file` reads and parses a file into an owned document, the file-based
    // counterpart to `load`.
    #[test]
    fn load_file_reads_and_parses() {
        let source = "= From File\n\nBody.";
        let path = std::env::temp_dir().join(format!(
            "asciidoc-html5-load-file-{}.adoc",
            std::process::id()
        ));
        std::fs::write(&path, source).expect("write temp input");

        let doc = load_file(&path).expect("load_file reads and parses");
        let _ = std::fs::remove_file(&path);

        assert_eq!(doc.doctitle(), Some("From File"));
    }

    // `load_file_with` records the file as the primary document, so a relative
    // top-level `include::` resolves against the file's own directory — the same
    // anchoring `convert_file_with` performs.
    #[test]
    fn load_file_with_anchors_includes_at_the_file_directory() {
        let dir = std::env::temp_dir().join(format!("adoc-load-file-inc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("main.adoc"), "= Doc\n\ninclude::part.adoc[]\n")
            .expect("write main");
        std::fs::write(dir.join("part.adoc"), "Included body.\n").expect("write part");

        let doc = load_file_with(
            dir.join("main.adoc"),
            &Options::new().safe_mode(SafeMode::Safe),
        )
        .expect("load_file_with reads and parses");

        // The include resolved, so the rendered document carries the included text.
        assert!(convert_document(&doc).contains("Included body."));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
