//! Resolution of the `copycss` behavior: deciding which stylesheet (if any)
//! should be copied next to the rendered HTML, and gathering its contents.
//!
//! Under Asciidoctor, `copycss` is a pure *file side effect* that runs after
//! rendering, only when the output is written to a file: it never changes the
//! HTML. When the stylesheet is *linked* (rather than embedded), `copycss`
//! writes the linked stylesheet file into the output directory so the HTML's
//! `<link>` resolves. The default stylesheet is written under the public name
//! `asciidoctor.css`; a custom stylesheet mirrors its `stylesdir` web path.
//!
//! `copycss` is set by default (by the parser's built-in attributes), so a copy
//! happens below the `secure` safe mode unless a document `:!copycss:` (or an
//! API unset) disables it. Under `secure` the stylesheet is linked but never
//! copied, matching Asciidoctor, which copies only when converting below
//! `secure`. When the `stylesheet` attribute is unset outright, `linkcss` and
//! `copycss` are both ignored.
//!
//! This module produces the *plan* — a destination path relative to the output
//! directory and the CSS to write there. Performing the write is the caller's
//! job, through an [`AssetWriter`](crate::AssetWriter); the library never
//! chooses the output directory.

use std::path::PathBuf;

use asciidoc_parser::{document::InterpretedValue, Document, SafeMode};

use crate::{
    include_handler,
    options::Options,
    renderer::{
        attribute_str, coderay_stylesheet_required, custom_stylesheet_value, links_stylesheet,
        looks_like_uri, normalize_web_path, stylesdir_join, stylesheet_disabled,
        CODERAY_STYLESHEET, CODERAY_STYLESHEET_NAME, DEFAULT_STYLESHEET, DEFAULT_STYLESHEET_NAME,
    },
};

/// A stylesheet the converter should copy next to the HTML it produces, under
/// the `copycss` attribute.
pub(crate) struct StylesheetCopy {
    /// Where to write the stylesheet, relative to the output directory — the
    /// same location the HTML's `<link>` references.
    pub(crate) dest: PathBuf,

    /// The CSS to write there: the embedded default stylesheet, or a custom
    /// stylesheet read from disk.
    pub(crate) content: String,
}

/// Computes the [`StylesheetCopy`] the document calls for, or `None` when no
/// stylesheet should be copied.
///
/// A copy happens only when converting below the `secure` safe mode, with the
/// stylesheet *linked*, `copycss` enabled, and the `stylesheet` attribute not
/// disabled — matching Asciidoctor. A custom stylesheet that is a URI (or lives
/// under a URI `stylesdir`), or whose source cannot be read, yields `None`.
pub(crate) fn stylesheet_copy(
    document: &Document<'_>,
    options: &Options,
) -> Option<StylesheetCopy> {
    // Asciidoctor copies the stylesheet only when converting below `secure`;
    // under `secure` the stylesheet is linked but never copied. The parser sets
    // `copycss` by default in every mode, so this safe-mode gate — not the
    // attribute — is what keeps `secure` from copying.
    if options.safe_mode_or_default() >= SafeMode::Secure {
        return None;
    }

    // A disabled stylesheet ignores `linkcss`/`copycss` entirely; otherwise the
    // copy is gated on the stylesheet being linked and `copycss` being enabled.
    if stylesheet_disabled(document) || !links_stylesheet(document) || !copycss_enabled(document) {
        return None;
    }

    let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();

    match custom_stylesheet_value(document) {
        // The default stylesheet: write the embedded `asciidoctor.css` at the
        // same `stylesdir` web path the head links it under (so with no
        // `stylesdir` it lands at the output root as `asciidoctor.css`).
        None => Some(StylesheetCopy {
            dest: relative_web_path(&normalize_web_path(DEFAULT_STYLESHEET_NAME, &stylesdir))?,
            // Match Asciidoctor's `Stylesheets#primary_stylesheet_data`, which
            // `rstrip`s the embedded CSS, so the copied `asciidoctor.css` has no
            // trailing newline — mirroring the `<style>` embed path.
            content: DEFAULT_STYLESHEET.trim_end().to_string(),
        }),

        // A custom stylesheet: mirror its web path and read its bytes.
        Some(stylesheet) => custom_stylesheet_copy(document, options, &stylesheet, &stylesdir),
    }
}

/// Computes the [`StylesheetCopy`] for the active syntax highlighter's own
/// stylesheet, or `None` when none applies.
///
/// Today only CodeRay carries a stylesheet this crate copies. The copy is gated
/// like the primary one — below the `secure` safe mode, with the stylesheet
/// *linked* (`linkcss`) and `copycss` enabled — plus the highlighter-specific
/// condition that the document actually calls for the CodeRay stylesheet
/// ([`coderay_stylesheet_required`]): CodeRay is active, in class CSS mode, and
/// a source block is present. This mirrors Asciidoctor's convert-to-file path,
/// where `copy_syntax_hl_stylesheet = syntax_hl.write_stylesheet? doc` sits
/// under the same `linkcss`/`copycss`/safe-mode gate as the primary copy.
///
/// Unlike [`stylesheet_copy`], this does *not* depend on the `stylesheet`
/// attribute: Asciidoctor copies (and links) the highlighter stylesheet even
/// when the primary stylesheet is disabled, since it is a separate docinfo. A
/// URI `stylesdir` yields no `./`-relative destination, so it is skipped — the
/// same way [`relative_web_path`] filters the primary copy.
pub(crate) fn syntax_highlighter_stylesheet_copy(
    document: &Document<'_>,
    options: &Options,
) -> Option<StylesheetCopy> {
    if options.safe_mode_or_default() >= SafeMode::Secure {
        return None;
    }

    if !links_stylesheet(document) || !copycss_enabled(document) {
        return None;
    }

    if !coderay_stylesheet_required(document) {
        return None;
    }

    let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();
    let dest = relative_web_path(&normalize_web_path(CODERAY_STYLESHEET_NAME, &stylesdir))?;

    Some(StylesheetCopy {
        dest,
        // Asciidoctor writes the CodeRay stylesheet through `read_stylesheet`,
        // which `rstrip`s it, so the file ends without a trailing newline.
        content: CODERAY_STYLESHEET.trim_end().to_string(),
    })
}

/// Whether `copycss` is enabled: set (with or without a value) rather than
/// unset. A `copycss=<path>` value counts as enabled — the path additionally
/// names where the stylesheet is read from (see [`read_override`]). A bare
/// `:copycss!:`, or `copycss` never set, is disabled.
fn copycss_enabled(document: &Document<'_>) -> bool {
    matches!(
        document.attribute_value("copycss"),
        InterpretedValue::Set | InterpretedValue::Value(_)
    )
}

/// The plan for copying a *custom* linked stylesheet: its destination web path
/// (relative to the output directory) paired with the CSS read from its source.
fn custom_stylesheet_copy(
    document: &Document<'_>,
    options: &Options,
    stylesheet: &str,
    stylesdir: &str,
) -> Option<StylesheetCopy> {
    // A URI stylesheet (or styles directory) is a complete remote reference,
    // not a local file to copy — matching Asciidoctor, which skips it.
    if looks_like_uri(stylesheet) || looks_like_uri(stylesdir) {
        return None;
    }

    // The destination is the same normalized web path the head links to, made
    // relative to the output directory. A path that is absolute or climbs out
    // (so `normalize_web_path` gave no `./` prefix) is not a copy target.
    let dest = relative_web_path(&normalize_web_path(stylesheet, stylesdir))?;

    // The source defaults to the stylesheet's own `stylesdir`-joined location,
    // but `copycss=<path>` overrides where the bytes are read from.
    let target = read_override(document).unwrap_or_else(|| stylesdir_join(document, stylesheet));
    let content = read_source(options, &target)?;

    Some(StylesheetCopy { dest, content })
}

/// The read-from path a `copycss=<path>` value names, or `None` for a bare
/// `copycss` (set with no value). This lets a document copy a stylesheet from
/// one location while linking it under another — Asciidoctor's "copy/link
/// split".
fn read_override(document: &Document<'_>) -> Option<String> {
    match document.attribute_value("copycss") {
        InterpretedValue::Value(path) if !path.is_empty() => Some(path),
        _ => None,
    }
}

/// Reads the stylesheet source at `target`, resolving it against the base
/// directory and confining the read to the safe mode's jail — the same
/// machinery that reads an `include::` target or an embedded stylesheet.
/// Returns `None` when there is no base directory to anchor the read or the
/// file cannot be read.
fn read_source(options: &Options, target: &str) -> Option<String> {
    let base_dir = options.effective_base_dir()?;
    let safe = options.safe_mode_or_default();

    let path = include_handler::resolve(&base_dir, safe, None, target);

    // A stylesheet source only needs its content; a missing, unreadable, or
    // non-UTF-8 file is simply unavailable here.
    match include_handler::read_confined(&base_dir, safe, &path) {
        include_handler::ReadOutcome::Read(content) => Some(content),
        include_handler::ReadOutcome::NotFound
        | include_handler::ReadOutcome::NotReadable
        | include_handler::ReadOutcome::NotDecodable => None,
    }
}

/// Turns a normalized web path into a path relative to the output directory, or
/// `None` when it does not stay within it.
///
/// [`normalize_web_path`] prefixes a contained relative path with `./` and
/// leaves absolute paths, URIs, and paths that climb out (`../…`) without one,
/// so stripping the `./` selects exactly the copy-safe targets and rejects the
/// rest.
fn relative_web_path(web_path: &str) -> Option<PathBuf> {
    let relative = web_path.strip_prefix("./")?;
    if relative.is_empty() {
        return None;
    }
    Some(PathBuf::from(relative))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        copycss::{stylesheet_copy, syntax_highlighter_stylesheet_copy},
        options::Options,
        renderer::{CODERAY_STYLESHEET, DEFAULT_STYLESHEET},
        SafeMode,
    };

    /// Parses `source` under `options` and returns the copy plan as
    /// `(dest-with-forward-slashes, content)`, so expectations read the same on
    /// every platform.
    fn plan(source: &str, options: &Options) -> Option<(String, String)> {
        let mut parser = options.apply(asciidoc_parser::Parser::default());
        let document = parser.parse(source);
        stylesheet_copy(&document, options)
            .map(|copy| (copy.dest.to_string_lossy().replace('\\', "/"), copy.content))
    }

    /// Like [`plan`], but for the syntax-highlighter stylesheet copy.
    fn hl_plan(source: &str, options: &Options) -> Option<(String, String)> {
        let mut parser = options.apply(asciidoc_parser::Parser::default());
        let document = parser.parse(source);
        syntax_highlighter_stylesheet_copy(&document, options)
            .map(|copy| (copy.dest.to_string_lossy().replace('\\', "/"), copy.content))
    }

    /// A minimal document with one source block — the trigger for the CodeRay
    /// stylesheet.
    const SOURCE_DOC: &str = "= Doc\n\n[source,ruby]\n----\nputs 1\n----";

    // With `linkcss` and `copycss` set, the default stylesheet is copied as
    // `asciidoctor.css` with the embedded default CSS as its content. Asciidoctor
    // `rstrip`s that CSS, so the copy carries no trailing newline.
    #[test]
    fn default_stylesheet_is_copied_as_asciidoctor_css() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss");
        let (dest, content) = plan("= Doc\n\nBody.", &options).expect("a copy");
        assert_eq!(dest, "asciidoctor.css");
        assert_eq!(content, DEFAULT_STYLESHEET.trim_end());
        assert!(!content.ends_with('\n'));
    }

    // The default stylesheet is copied under `stylesdir`, mirroring the web path
    // the head links it at (`./css/asciidoctor.css` -> `css/asciidoctor.css`).
    #[test]
    fn default_stylesheet_honors_stylesdir() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("stylesdir", "css");
        let (dest, content) = plan("= Doc\n\nBody.", &options).expect("a copy");
        assert_eq!(dest, "css/asciidoctor.css");
        assert_eq!(content, DEFAULT_STYLESHEET.trim_end());
    }

    // Under `secure` the stylesheet is linked but never copied, even though the
    // parser sets `copycss` by default and `linkcss` is on — the safe-mode gate,
    // not the attribute, is what suppresses the copy.
    #[test]
    fn no_copy_under_secure() {
        let options = Options::new().safe_mode(SafeMode::Secure);
        assert!(plan("= Doc\n\nBody.", &options).is_none());
    }

    // Without `linkcss` (the stylesheet is embedded), nothing is copied even
    // when `copycss` is set.
    #[test]
    fn no_copy_when_the_stylesheet_is_embedded() {
        let options = Options::new().safe_mode(SafeMode::Safe).set("copycss");
        assert!(plan("= Doc\n\nBody.", &options).is_none());
    }

    // A document `:!copycss:` disables the copy even under a linking safe mode.
    #[test]
    fn document_can_disable_copycss() {
        let options = Options::new().safe_mode(SafeMode::Safe).set("linkcss");
        assert!(plan("= Doc\n:!copycss:\n\nBody.", &options).is_none());
    }

    // An unset `stylesheet` makes `linkcss`/`copycss` inert: no copy.
    #[test]
    fn no_copy_when_the_stylesheet_is_disabled() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss");
        assert!(plan("= Doc\n:stylesheet!:\n\nBody.", &options).is_none());
    }

    // A custom stylesheet is copied to its `stylesdir` web path, with the CSS
    // read from disk relative to the base directory.
    #[test]
    fn custom_stylesheet_mirrors_its_web_path() {
        let dir = scratch("custom", &[("css/theme.css", "body { color: red; }")]);
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone())
            .set("linkcss")
            .set("copycss")
            .attribute("stylesdir", "css")
            .attribute("stylesheet", "theme.css");
        let (dest, content) = plan("= Doc\n\nBody.", &options).expect("a copy");
        assert_eq!(dest, "css/theme.css");
        assert_eq!(content, "body { color: red; }");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // A custom stylesheet whose source file is missing yields no copy: the read
    // comes back empty (`read_source` -> `None`), so there is nothing to write.
    #[test]
    fn no_copy_when_the_custom_stylesheet_source_is_missing() {
        let dir = scratch("missing-source", &[]);
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone())
            .set("linkcss")
            .set("copycss")
            .attribute("stylesheet", "absent.css");
        assert!(plan("= Doc\n\nBody.", &options).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // A `copycss=<path>` value reads the stylesheet from that path but still
    // writes it to the `stylesheet` destination (the copy/link split).
    #[test]
    fn copycss_path_overrides_the_read_location() {
        let dir = scratch("override", &[("source.css", "body { color: blue; }")]);
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone())
            .set("linkcss")
            .attribute("copycss", "source.css")
            .attribute("stylesheet", "published.css");
        let (dest, content) = plan("= Doc\n\nBody.", &options).expect("a copy");
        assert_eq!(dest, "published.css");
        assert_eq!(content, "body { color: blue; }");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // A custom stylesheet given as a URI is a complete remote reference, so
    // there is nothing to copy.
    #[test]
    fn no_copy_for_a_uri_stylesheet() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("stylesheet", "https://example.org/theme.css");
        assert!(plan("= Doc\n\nBody.", &options).is_none());
    }

    // `relative_web_path` keeps only web paths that stay within the output
    // directory: it strips the `./` a contained path carries and rejects an
    // empty result, an absolute path, a climbing `../` path, and a URI — none of
    // which `normalize_web_path` prefixes with `./`.
    #[test]
    fn relative_web_path_selects_contained_targets() {
        use super::relative_web_path;

        assert_eq!(
            relative_web_path("./css/theme.css"),
            Some(PathBuf::from("css/theme.css"))
        );
        assert_eq!(relative_web_path("./"), None);
        assert_eq!(relative_web_path("../up.css"), None);
        assert_eq!(relative_web_path("/abs.css"), None);
        assert_eq!(relative_web_path("https://example.org/x.css"), None);
    }

    // With coderay active (in its default class CSS mode), `linkcss`, and
    // `copycss`, a source block draws a copy of `coderay-asciidoctor.css` with
    // the embedded CodeRay CSS — trailing newline chomped, as Asciidoctor's
    // `read_stylesheet` `rstrip`s it.
    #[test]
    fn coderay_stylesheet_is_copied_when_a_source_block_is_present() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay");
        let (dest, content) = hl_plan(SOURCE_DOC, &options).expect("a copy");
        assert_eq!(dest, "coderay-asciidoctor.css");
        assert_eq!(content, CODERAY_STYLESHEET.trim_end());
        assert!(!content.ends_with('\n'), "no trailing newline");
    }

    // The coderay stylesheet copy honors `stylesdir`, mirroring the web path the
    // head links it at.
    #[test]
    fn coderay_stylesheet_copy_honors_stylesdir() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay")
            .attribute("stylesdir", "css");
        let (dest, _) = hl_plan(SOURCE_DOC, &options).expect("a copy");
        assert_eq!(dest, "css/coderay-asciidoctor.css");
    }

    // The coderay stylesheet copy is independent of the primary stylesheet: with
    // the primary one disabled (`:stylesheet!:`), the CodeRay stylesheet is still
    // copied, matching Asciidoctor (the highlighter docinfo is separate).
    #[test]
    fn coderay_stylesheet_copy_survives_a_disabled_primary_stylesheet() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay");
        let source = "= Doc\n:stylesheet!:\n\n[source,ruby]\n----\nputs 1\n----";
        assert!(hl_plan(source, &options).is_some());
        // ... while the primary copy is suppressed by the disabled stylesheet.
        assert!(plan(source, &options).is_none());
    }

    // No source block means coderay highlighted nothing, so there is no
    // stylesheet to copy.
    #[test]
    fn no_coderay_copy_without_a_source_block() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay");
        assert!(hl_plan("= Doc\n\nBody.", &options).is_none());
    }

    // `coderay-css=style` inlines the colors, so no stylesheet is copied.
    #[test]
    fn no_coderay_copy_in_style_css_mode() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay")
            .attribute("coderay-css", "style");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
    }

    // An explicit `:coderay-css!:` (unset) falls back to the `class` default —
    // Asciidoctor's `attr('coderay-css', 'class')` returns `class` for an unset
    // attribute — so the stylesheet is still copied.
    #[test]
    fn coderay_copy_survives_an_explicitly_unset_coderay_css() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay");
        let source = "= Doc\n:coderay-css!:\n\n[source,ruby]\n----\nputs 1\n----";
        let (dest, _) = hl_plan(source, &options).expect("a copy");
        assert_eq!(dest, "coderay-asciidoctor.css");
    }

    // A `stylesdir` that resolves outside the output directory (here an absolute
    // path) gives no `./`-relative destination, so the CodeRay stylesheet is not
    // copied — the same containment `relative_web_path` applies to the primary
    // copy.
    #[test]
    fn no_coderay_copy_for_an_escaping_stylesdir() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay")
            .attribute("stylesdir", "/abs");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
    }

    // A URI `stylesdir` links the stylesheet remotely, so there is no local file
    // to copy — the link is a URI (not a `./`-relative path), which
    // `relative_web_path` rejects. This matches Asciidoctor, which skips copying
    // entirely when `stylesdir` is a URI.
    #[test]
    fn no_coderay_copy_for_a_uri_stylesdir() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "coderay")
            .attribute("stylesdir", "https://cdn.example.com/css");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
        // The primary stylesheet copy is skipped for the same reason.
        assert!(plan(SOURCE_DOC, &options).is_none());
    }

    // A different (or no) highlighter carries no stylesheet this crate copies.
    #[test]
    fn no_coderay_copy_for_a_different_highlighter() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .set("copycss")
            .attribute("source-highlighter", "highlightjs");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
    }

    // Under `secure` nothing is copied, even with coderay and a source block.
    #[test]
    fn no_coderay_copy_under_secure() {
        let options = Options::new().attribute("source-highlighter", "coderay");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
    }

    // Without `linkcss` the stylesheet is embedded, not copied.
    #[test]
    fn no_coderay_copy_when_embedding() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("copycss")
            .attribute("source-highlighter", "coderay");
        assert!(hl_plan(SOURCE_DOC, &options).is_none());
    }

    // A `:!copycss:` disables the coderay copy under a linking safe mode.
    #[test]
    fn document_can_disable_the_coderay_copy() {
        let options = Options::new()
            .safe_mode(SafeMode::Safe)
            .set("linkcss")
            .attribute("source-highlighter", "coderay");
        let source = "= Doc\n:!copycss:\n\n[source,ruby]\n----\nputs 1\n----";
        assert!(hl_plan(source, &options).is_none());
    }

    /// Creates a fresh temp directory named after `tag`, populated with `files`
    /// (relative name → content), for a copy test to read a source from.
    fn scratch(tag: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("adoc-copycss-{}-{tag}", std::process::id()));
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create scratch subdir");
            }
            std::fs::write(path, content).expect("write scratch file");
        }
        dir
    }
}
