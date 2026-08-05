//! Filesystem-backed resolution of inline SVG embedding
//! (`image:target.svg[opts=inline]` / `image::target.svg[opts=inline]`),
//! anchored at a base directory and confined by the [safe mode](SafeMode).
//!
//! When an image macro carries `opts=inline` and its target is an SVG,
//! Asciidoctor embeds the SVG *file's contents* directly as an `<svg>` element
//! rather than linking to it with an `<img>`. This is security-sensitive (it
//! splices file contents into the output), so — like the `interactive`
//! `<object>` referencing — it takes effect only below the [`SafeMode::Secure`]
//! safe mode.
//!
//! `asciidoc-parser` handles *inline* images (`image:…`) itself, reading the
//! SVG through an [`SvgFileHandler`] this module supplies
//! ([`FsSvgFileHandler`]) and doing the content preparation internally. *Block*
//! images (`image::…`) are rendered by this crate's own renderer, which reads
//! the file through the shared [`read_svg_file`] and prepares the contents with
//! [`prepare_inline_svg`] — a hand port of Asciidoctor's `read_svg_contents`.
//!
//! Both paths resolve the target — a web-path `src`, already prefixed with
//! `imagesdir` — against the base directory the same way an `include::` target
//! resolves, confined to the safe mode's jail (see [`crate::include_handler`]).

use std::path::{Path, PathBuf};

use asciidoc_parser::{parser::SvgFileHandler, Parser, SafeMode};

use crate::include_handler::{read_confined, resolve, ReadOutcome};

/// Resolves inline-SVG targets against the filesystem for the parser's
/// *inline*-image handling, anchored at a base directory and honoring the safe
/// mode's jail.
///
/// The parser consults this when an inline `image:…[opts=inline]` macro needs
/// the SVG file's contents; it then performs the preamble stripping and
/// dimension rewriting itself. Block images take the same reading path through
/// [`read_svg_file`] but prepare the contents in the renderer.
#[derive(Debug)]
pub(crate) struct FsSvgFileHandler {
    /// The base directory: the anchor for the resolved target and, when jailed,
    /// the boundary the read may not cross. Expected to be absolute and
    /// canonical, matching the include/docinfo handlers.
    base_dir: PathBuf,

    /// The safe mode in force, which decides whether resolution is jailed.
    safe: SafeMode,
}

impl FsSvgFileHandler {
    /// Creates a handler anchored at `base_dir` and confined according to
    /// `safe`.
    pub(crate) fn new(base_dir: PathBuf, safe: SafeMode) -> Self {
        Self { base_dir, safe }
    }
}

impl SvgFileHandler for FsSvgFileHandler {
    fn resolve_svg(&self, target: &str, _parser: &Parser) -> Option<String> {
        read_svg_file(&self.base_dir, self.safe, target)
    }
}

/// Reads the contents of the SVG at `target` — a web-path `src` already
/// prefixed with `imagesdir`, the same value an `<img>`'s `src` would carry —
/// resolving it against `base_dir` and confining the read to the safe mode's
/// jail.
///
/// The contents are normalized the way Asciidoctor's `read_contents` with
/// `normalize: true` does (see [`normalize_source`]), so the embedded SVG has
/// no spurious trailing newline. Returns `None` when the file is missing,
/// unreadable, non-UTF-8, blocked by the jail, or empty after normalization —
/// each of which makes Asciidoctor fall back to the image's alt text. `target`
/// is resolved with `source: None`, so it anchors at the base directory (the
/// include handler's shared [`resolve`]).
///
/// This is the shared reading path for both the parser's inline-image handler
/// ([`FsSvgFileHandler`]) and the renderer's block-image embedding, so a real
/// on-disk SVG is embedded identically whichever macro form referenced it.
pub(crate) fn read_svg_file(base_dir: &Path, safe: SafeMode, target: &str) -> Option<String> {
    let path = resolve(base_dir, safe, None, target);
    match read_confined(base_dir, safe, &path) {
        ReadOutcome::Read(content) => {
            let normalized = normalize_source(&content);
            (!normalized.is_empty()).then_some(normalized)
        }
        ReadOutcome::NotFound | ReadOutcome::NotReadable | ReadOutcome::NotDecodable => None,
    }
}

/// Normalizes SVG file `content` the way Asciidoctor's `read_contents` with
/// `normalize: true` does: strip a leading UTF-8 BOM, drop trailing whitespace
/// from every line (Ruby's `rstrip`, which also folds `\r\n` to `\n`), and join
/// the lines with `\n` — leaving no trailing newline.
///
/// This keeps the embedded `<svg>` free of the trailing newline a real file
/// carries (which the parser and [`prepare_inline_svg`] do not strip on their
/// own), so the output matches Asciidoctor for both inline and block images.
fn normalize_source(content: &str) -> String {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    content
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Prepares raw SVG file `contents` for inline embedding in a *block* image,
/// porting Asciidoctor's `read_svg_contents`.
///
/// Returns `None` for empty contents (so the caller falls back to the alt
/// text). Otherwise:
///
/// * any preamble before the opening `<svg>` tag (an XML declaration or
///   doctype) is stripped, unless the content already starts with `<svg`; and
/// * when the macro supplied a `width` and/or `height`, the opening `<svg>`
///   tag's own `width`, `height`, and `style` attributes are removed and the
///   requested dimensions appended in their place (a unitless HTML value is
///   assumed to be pixels, so it passes straight through).
pub(crate) fn prepare_inline_svg(
    mut svg: String,
    width: Option<&str>,
    height: Option<&str>,
) -> Option<String> {
    if svg.is_empty() {
        return None;
    }

    // Strip anything preceding the opening `<svg>` tag (e.g. `<?xml … ?>` or a
    // `<!DOCTYPE …>`), but only when the content does not already start with
    // the tag, matching Asciidoctor's `SvgPreambleRx` guard.
    if !svg.starts_with("<svg") {
        if let Some(start) = find_svg_tag_start(&svg) {
            svg = svg[start..].to_string();
        }
    }

    // Rewrite the opening tag's dimensions only when at least one was supplied.
    if let Some(old_tag) = svg_start_tag(&svg).filter(|_| width.is_some() || height.is_some()) {
        let old_tag = old_tag.to_string();

        // Drop the tag's existing width/height/style, then re-close it with the
        // requested dimensions appended (width before height, each when given).
        let mut new_tag = strip_svg_dimension_attrs(&old_tag);
        new_tag.pop(); // The trailing `>`, re-added after the new dimensions.

        if let Some(width) = width {
            new_tag.push_str(&format!(r#" width="{width}""#));
        }

        if let Some(height) = height {
            new_tag.push_str(&format!(r#" height="{height}""#));
        }

        new_tag.push('>');
        svg = format!("{new_tag}{}", &svg[old_tag.len()..]);
    }

    Some(svg)
}

/// Finds the byte offset of the first `<svg` opening tag — a `<svg` immediately
/// followed by whitespace or `>` — or `None` when the content has none.
///
/// This is the target of Asciidoctor's `SvgPreambleRx`
/// (`/\A.*?(?=<svg[\s>])/`): everything before that offset is the preamble to
/// strip.
fn find_svg_tag_start(svg: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(rel) = svg[from..].find("<svg") {
        let index = from + rel;
        match svg[index + 4..].chars().next() {
            Some(next) if next == '>' || next.is_whitespace() => return Some(index),
            _ => from = index + 4,
        }
    }
    None
}

/// Returns the opening `<svg …>` tag at the very start of `svg`, or `None` when
/// the content does not begin with one.
///
/// Ports Asciidoctor's `SvgStartTagRx` (`/\A<svg(?:\s[^>]*)?>/`): the tag is
/// `<svg` followed either directly by `>` or by whitespace, then the run of
/// attribute characters up to the first `>`.
fn svg_start_tag(svg: &str) -> Option<&str> {
    let rest = svg.strip_prefix("<svg")?;
    match rest.chars().next() {
        Some('>') => Some(&svg[.."<svg>".len()]),
        Some(next) if next.is_whitespace() => {
            let close = rest.find('>')?;
            Some(&svg[..4 + close + 1])
        }
        _ => None,
    }
}

/// Removes every `width`, `height`, and `style` attribute (each with its
/// leading whitespace and quoted value) from an SVG opening `tag`.
///
/// Ports Asciidoctor's `DimensionAttributeRx` gsub
/// (`/\s(?:width|height|style)=(["'])CC_ANY*?\1/`): a single leading whitespace
/// character, the attribute name, `=`, and a single- or double-quoted value up
/// to its matching closing quote.
fn strip_svg_dimension_attrs(tag: &str) -> String {
    let mut result = String::with_capacity(tag.len());
    let mut index = 0;

    while index < tag.len() {
        if let Some(end) = match_dimension_attr(tag, index) {
            index = end;
            continue;
        }

        let ch = tag[index..]
            .chars()
            .next()
            .expect("index is a char boundary");
        result.push(ch);
        index += ch.len_utf8();
    }

    result
}

/// If a ` width="…"`, ` height="…"`, or ` style="…"` attribute (single- or
/// double-quoted) begins at byte `start` in `tag`, returns the byte offset just
/// past its closing quote; otherwise `None`.
fn match_dimension_attr(tag: &str, start: usize) -> Option<usize> {
    // Exactly one leading whitespace character, matching the regex's `\s`.
    let whitespace = tag[start..].chars().next()?;
    if !whitespace.is_whitespace() {
        return None;
    }
    let mut pos = start + whitespace.len_utf8();

    // One of the dimension attribute names, immediately followed by `=`.
    let name = ["width=", "height=", "style="]
        .into_iter()
        .find(|name| tag[pos..].starts_with(name))?;
    pos += name.len();

    // A quote, then everything up to its next occurrence (the lazy `CC_ANY*?`).
    let quote = tag[pos..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    pos += quote.len_utf8();

    let close = tag[pos..].find(quote)?;
    Some(pos + close + quote.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::{
        find_svg_tag_start, match_dimension_attr, normalize_source, prepare_inline_svg,
        strip_svg_dimension_attrs, svg_start_tag,
    };

    // `normalize_source` drops a leading BOM, `rstrip`s each line, folds CRLF to
    // LF, and leaves no trailing newline — Asciidoctor's `normalize: true`.
    #[test]
    fn normalize_source_matches_asciidoctor() {
        assert_eq!(
            normalize_source("\u{feff}<?xml?>  \r\n<svg>x</svg>\n"),
            "<?xml?>\n<svg>x</svg>"
        );
        assert_eq!(normalize_source(""), "");
        assert_eq!(normalize_source("\n"), "");
    }

    const SAMPLE: &str = concat!(
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        "\n",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="500" style="fill:red" viewBox="0 0 500 500"><circle cx="250" cy="250" r="200"/></svg>"#,
    );

    // Empty contents yield `None`, so the block image falls back to its alt
    // text (Asciidoctor's `return if svg.empty?`).
    #[test]
    fn empty_contents_are_none() {
        assert_eq!(prepare_inline_svg(String::new(), None, None), None);
    }

    // With no dimensions supplied, only the XML preamble is stripped; the
    // opening tag (width/height/style/viewBox) is preserved verbatim.
    #[test]
    fn no_dimensions_strips_only_the_preamble() {
        let out = prepare_inline_svg(SAMPLE.to_string(), None, None).unwrap();
        assert_eq!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="500" style="fill:red" viewBox="0 0 500 500"><circle cx="250" cy="250" r="200"/></svg>"#
        );
    }

    // A supplied width strips the tag's width/height/style and appends the
    // requested width, preserving the required viewBox — matching Asciidoctor.
    #[test]
    fn width_replaces_the_tag_dimensions() {
        let out = prepare_inline_svg(SAMPLE.to_string(), Some("300"), None).unwrap();
        assert_eq!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 500" width="300"><circle cx="250" cy="250" r="200"/></svg>"#
        );
    }

    // A width and height both append after the stripped dimensions, in order.
    #[test]
    fn width_and_height_both_append() {
        let out = prepare_inline_svg(SAMPLE.to_string(), Some("300"), Some("200")).unwrap();
        assert_eq!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 500 500" width="300" height="200"><circle cx="250" cy="250" r="200"/></svg>"#
        );
    }

    // Content already starting with `<svg` is not preamble-stripped, and a lone
    // height is honored just like a lone width.
    #[test]
    fn height_only_and_no_preamble() {
        let svg = r#"<svg width="10" viewBox="0 0 4 4"><rect/></svg>"#.to_string();
        let out = prepare_inline_svg(svg, None, Some("7")).unwrap();
        assert_eq!(out, r#"<svg viewBox="0 0 4 4" height="7"><rect/></svg>"#);
    }

    #[test]
    fn find_svg_tag_start_skips_the_preamble() {
        assert_eq!(find_svg_tag_start("<?xml?>\n<svg >"), Some(8));
        assert_eq!(find_svg_tag_start("<svg>"), Some(0));
        // A `<svgfoo` that is not a real tag is skipped in favor of a later one.
        assert_eq!(find_svg_tag_start("<svgfoo><svg>"), Some(8));
        assert_eq!(find_svg_tag_start("no tag here"), None);
    }

    #[test]
    fn svg_start_tag_matches_the_opening_tag() {
        assert_eq!(svg_start_tag("<svg>rest"), Some("<svg>"));
        assert_eq!(svg_start_tag(r#"<svg a="1">rest"#), Some(r#"<svg a="1">"#));
        // `<svgfoo>` is not an opening `<svg>` tag (no whitespace or `>` after).
        assert_eq!(svg_start_tag("<svgfoo>"), None);
        assert_eq!(svg_start_tag("nope"), None);
    }

    #[test]
    fn strip_svg_dimension_attrs_removes_dimensions_only() {
        assert_eq!(
            strip_svg_dimension_attrs(r#"<svg width="1" viewBox="0 0 2 2" height='3' style="x">"#),
            r#"<svg viewBox="0 0 2 2">"#
        );
        // Nothing to strip leaves the tag untouched.
        assert_eq!(
            strip_svg_dimension_attrs(r#"<svg viewBox="0 0 2 2">"#),
            r#"<svg viewBox="0 0 2 2">"#
        );
    }

    #[test]
    fn match_dimension_attr_boundaries() {
        let tag = r#" width="1" x="2""#;
        // Matches the leading ` width="1"` (10 bytes: index 0..10).
        assert_eq!(match_dimension_attr(tag, 0), Some(10));
        // A non-dimension attribute does not match.
        assert_eq!(match_dimension_attr(tag, 10), None);
        // Without leading whitespace, no match.
        assert_eq!(match_dimension_attr(r#"width="1""#, 0), None);
    }
}
