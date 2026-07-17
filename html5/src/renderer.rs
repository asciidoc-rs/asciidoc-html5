//! The block-structure walker that turns a parsed [`Document`] into HTML5.
//!
//! # How the walk works
//!
//! The parser applies *inline* substitutions eagerly: by the time we hold a
//! [`Document`], every block's content and title is already an
//! Asciidoctor-compatible HTML *fragment* (with `<strong>`, `<a href>`, escaped
//! special characters, and so on). This crate therefore never parses inline
//! markup itself — its whole job is to wrap those fragments in the block-level
//! scaffolding (the `<div class="…">` structure) that Asciidoctor's `html5`
//! backend emits, in document order.
//!
//! [`Renderer`] holds the output buffer and exposes one method per structural
//! concern. [`Renderer::block`] is the dispatch point: it matches on the
//! [`Block`] variant (and, for delimited blocks, on
//! [`IsBlock::resolved_context`]) and delegates. Compound blocks recurse back
//! into [`Renderer::blocks`] over their [`IsBlock::nested_blocks`], so the same
//! machinery handles arbitrary nesting.
//!
//! This is a *baseline*: the constructs wired up below (the document skeleton,
//! header, paragraphs, sections, the preamble, verbatim blocks, and thematic
//! and page breaks) exercise every mechanism the full renderer needs.
//! Everything else falls through [`Renderer::unsupported`], which emits a
//! visible HTML comment rather than guessing — so output stays well-formed and
//! coverage gaps are obvious. Adding a construct means adding one arm and one
//! `render_*` method.

use std::slice::Iter;

use asciidoc_parser::{
    blocks::{
        AdmonitionBlock, Block, Break, BreakType, CompoundDelimitedContext, ContentModel, IsBlock,
        QuoteBlock, QuoteType, SectionBlock, SectionType, SimpleBlockStyle,
    },
    document::{DocinfoLocation, Header, InterpretedValue},
    Document, SafeMode,
};

use crate::html::{class_attribute, escape_attribute, id_attribute};

/// Asciidoctor's compiled default stylesheet, embedded verbatim. This is a copy
/// of `ref/asciidoctor/data/stylesheets/asciidoctor-default.css` (Asciidoctor
/// v2.0.26) — the exact CSS Asciidoctor's `html5` backend inlines into a
/// standalone document via `Stylesheets#primary_stylesheet_data`. It carries
/// its own MIT license header; a drift-guard test keeps this copy identical to
/// the reference one.
pub(crate) const DEFAULT_STYLESHEET: &str = include_str!("../assets/asciidoctor-default.css");

/// The public file name Asciidoctor writes (and links) its default stylesheet
/// under — `Stylesheets::DEFAULT_STYLESHEET_NAME`. The linked reference and the
/// `copycss` destination both use it.
pub(crate) const DEFAULT_STYLESHEET_NAME: &str = "asciidoctor.css";

/// The `family` query string Asciidoctor uses for its Google Fonts `<link>`
/// when the `webfonts` attribute carries no explicit value: Open Sans for
/// headings, Noto Serif for body text, Droid Sans Mono for monospaced text.
const DEFAULT_WEBFONTS: &str = "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700";

/// Reads a document attribute as an explicit string value, if it has one.
/// `Set`/`Unset`/absent all yield `None` (use `is_attribute_set` for booleans).
pub(crate) fn attribute_str(document: &Document<'_>, name: &str) -> Option<String> {
    match document.attribute_value(name) {
        InterpretedValue::Value(value) => Some(value),
        InterpretedValue::Set | InterpretedValue::Unset => None,
    }
}

/// Whether the default stylesheet should be *linked* (to `./asciidoctor.css`)
/// rather than *embedded* inline.
///
/// Following Asciidoctor, the decision keys off `linkcss` and the safe mode:
///
/// - An explicit `linkcss` (set by the document, or seeded and locked by the
///   API under a `Secure` safe mode) links.
/// - An explicit `linkcss!` (unset) embeds, even under `Secure`.
/// - Otherwise, a safe mode of `Secure` or greater links by default and a lower
///   mode embeds. The `_with` entry points seed and lock this at parse time via
///   [`Options`](crate::Options); keying off the safe mode here means
///   [`convert_document`](crate::convert_document) on a document parsed under
///   `Secure` links it too, so the two paths stay consistent.
pub(crate) fn links_stylesheet(document: &Document<'_>) -> bool {
    if document.is_attribute_set("linkcss") {
        return true;
    }

    // Present but not set means an explicit `linkcss!` (unset): embed.
    if document.has_attribute("linkcss") {
        return false;
    }

    // Unmentioned: link under `Secure` (level 20) or greater, else embed. The
    // `safe-mode-level` intrinsic attribute is populated by the parser for every
    // document (its built-in default is `Secure`).
    matches!(attribute_str(document, "safe-mode-level"), Some(level)
        if level.parse::<u32>().is_ok_and(|n| n >= SafeMode::Secure as u32))
}

/// The `stylesheet` value when the document selects a *custom* stylesheet — a
/// non-empty value other than `DEFAULT` that is not an explicit unset
/// (`:stylesheet!:`). The default stylesheet and an unset stylesheet both yield
/// `None`.
pub(crate) fn custom_stylesheet_value(document: &Document<'_>) -> Option<String> {
    match document.attribute_value("stylesheet") {
        InterpretedValue::Unset if document.has_attribute("stylesheet") => None,
        InterpretedValue::Value(value) if !value.is_empty() && value != "DEFAULT" => Some(value),
        _ => None,
    }
}

/// The relative filesystem path of a custom stylesheet that should be *embedded
/// from disk*, or `None` when there is nothing to read: the stylesheet is the
/// default, unset, *linked* (so only a `<link>` is needed), or a URI (which the
/// library never fetches). The returned target joins `stylesdir` and
/// `stylesheet` the way Asciidoctor's `normalize_system_path` would, ready to
/// resolve against the base directory.
///
/// Reading the file is left to [`convert_with`](crate::convert_with), which
/// holds the base directory and safe mode; the renderer itself stays free of
/// filesystem access.
pub(crate) fn embeddable_stylesheet_target(document: &Document<'_>) -> Option<String> {
    let stylesheet = custom_stylesheet_value(document)?;

    // A linked stylesheet needs no file read; a URI cannot be read from disk.
    if links_stylesheet(document) || looks_like_uri(&stylesheet) {
        return None;
    }

    Some(stylesdir_join(document, &stylesheet))
}

/// Joins the `stylesdir` attribute ahead of `stylesheet` to form the
/// filesystem-relative target Asciidoctor's `normalize_system_path` would
/// resolve — the path from which a custom stylesheet is read (to embed it, or
/// to copy it under `copycss`). A trailing separator on `stylesdir` is dropped
/// so the join never doubles the `/`; an empty `stylesdir` leaves the
/// stylesheet untouched.
pub(crate) fn stylesdir_join(document: &Document<'_>, stylesheet: &str) -> String {
    let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();
    if stylesdir.is_empty() {
        stylesheet.to_string()
    } else {
        format!("{}/{stylesheet}", stylesdir.trim_end_matches(['/', '\\']))
    }
}

/// Whether the document has *disabled* its stylesheet with an explicit
/// `:stylesheet!:` (unset). When it has, no stylesheet block is emitted and the
/// `linkcss`/`copycss` attributes are ignored, matching Asciidoctor.
pub(crate) fn stylesheet_disabled(document: &Document<'_>) -> bool {
    matches!(
        document.attribute_value("stylesheet"),
        InterpretedValue::Unset
    ) && document.has_attribute("stylesheet")
}

/// Computes the web path Asciidoctor's `html5` backend uses when linking to a
/// custom stylesheet — a minimal port of its `normalize_web_path(stylesheet,
/// stylesdir)`.
///
/// A URI (`file:///…`, `https://…`, `data:…`, …), an absolute path, or a
/// protocol-relative `//host/…` reference is complete already and is returned
/// unchanged. Otherwise the stylesheet is treated as relative to `stylesdir`:
/// the two are joined, `.` and `..` segments are collapsed, and a relative
/// result is prefixed with `./`, so a bare `custom.css` becomes `./custom.css`
/// and `custom.css` under `stylesdir=css` becomes `./css/custom.css`.
pub(crate) fn normalize_web_path(stylesheet: &str, stylesdir: &str) -> String {
    // A URI is emitted verbatim (Asciidoctor's `preserve_uri_target`).
    if looks_like_uri(stylesheet) {
        return stylesheet.to_string();
    }

    // Posixify (Asciidoctor works in forward-slash web paths) and join with the
    // styles directory, unless the stylesheet is itself an absolute path — which
    // ignores `stylesdir`, matching Asciidoctor's web-root check. A trailing
    // separator on `stylesdir` is dropped so the join never doubles the `/`.
    let sheet = stylesheet.replace('\\', "/");
    let dir = stylesdir.replace('\\', "/");
    let joined = if dir.is_empty() || sheet.starts_with('/') {
        sheet
    } else {
        format!("{}/{}", dir.trim_end_matches('/'), sheet)
    };

    web_normalize(&joined)
}

/// Collapses `.`/`..` segments in a posix `path` and prefixes a plain relative
/// result with `./`, following Asciidoctor's `PathResolver#web_path`.
fn web_normalize(path: &str) -> String {
    let (root, rest) = if let Some(rest) = path.strip_prefix("//") {
        // A leading `//` is a protocol-relative (or UNC) authority; Asciidoctor
        // preserves it rather than collapsing it to a single `/`.
        ("//", rest)
    } else if let Some(rest) = path.strip_prefix('/') {
        ("/", rest)
    } else if let Some(rest) = path.strip_prefix("./") {
        ("./", rest)
    } else {
        ("./", path)
    };

    let mut segments: Vec<&str> = Vec::new();
    for segment in rest.split('/') {
        match segment {
            "" | "." => {}
            ".." => match segments.last() {
                // Pop the previous real segment.
                Some(&last) if last != ".." => {
                    segments.pop();
                }

                // A leading `..` at the web root has nowhere to go; drop it.
                // Below the root, it is kept as a relative step.
                _ if root == "/" => {}
                _ => segments.push(".."),
            },
            other => segments.push(other),
        }
    }

    // The `./` prefix marks a path that stays at or below the current directory.
    // A relative result that already climbs (`../…`) is a complete reference on
    // its own, so it keeps no `./`, matching Asciidoctor.
    let prefix = if root == "./" && segments.first() == Some(&"..") {
        ""
    } else {
        root
    };
    format!("{prefix}{}", segments.join("/"))
}

/// Whether `value` looks like a URI, mirroring Asciidoctor's `UriSniffRx`: a
/// scheme of two or more characters (so a Windows drive letter like `c:` is not
/// mistaken for one) starting with a letter, followed by a colon.
pub(crate) fn looks_like_uri(value: &str) -> bool {
    let Some(scheme_end) = value.find(':') else {
        return false;
    };
    let scheme = &value[..scheme_end];
    scheme.len() >= 2
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-'))
}

/// Renders a parsed [`Document`] to an HTML5 string.
///
/// `standalone` selects the output mode: `true` emits the complete
/// document — the `<!DOCTYPE>`/`<html>`/`<head>`/`<body>` shell around the
/// header, content, and footer — while `false` emits embedded, body-only output
/// (the converted body, with the doctitle `<h1>` only when `showtitle` is set).
///
/// `custom_stylesheet` is the CSS to embed when the document selects a custom
/// stylesheet that is *embedded* rather than linked (see
/// [`Options::stylesheet_content`](crate::Options::stylesheet_content)); it is
/// `None` for callers that cannot supply it, such as the string-only
/// [`convert`](crate::convert) entry point. It is ignored in embedded output,
/// which emits no stylesheet.
pub(crate) fn render_document(
    document: &Document<'_>,
    custom_stylesheet: Option<&str>,
    standalone: bool,
) -> String {
    let mut renderer = Renderer {
        out: String::new(),
        custom_stylesheet,
        standalone,
    };
    renderer.document(document);
    renderer.out
}

/// Accumulates HTML as the document tree is walked.
struct Renderer<'a> {
    out: String,

    /// The CSS to embed for a custom, embedded stylesheet, if the caller
    /// supplied any.
    custom_stylesheet: Option<&'a str>,

    /// Whether to emit the standalone document shell (`true`) or embedded,
    /// body-only output (`false`).
    standalone: bool,
}

impl Renderer<'_> {
    /// Appends a line of markup followed by a newline, matching Asciidoctor's
    /// convention of one element per line with no indentation.
    fn line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    /// Emits the document. In standalone mode this is the complete document —
    /// the `<head>` preamble, the `<div id="header">`, the `<div id="content">`
    /// body, and the footer; in embedded mode it is the body-only output
    /// emitted by [`embedded_document`](Self::embedded_document).
    fn document(&mut self, document: &Document<'_>) {
        // The `inline` doctype converts a fragment, not a document: it emits
        // only the first block's inline content, ignoring the standalone /
        // embedded distinction entirely.
        if attribute_str(document, "doctype").as_deref() == Some("inline") {
            self.inline_document(document);
            return;
        }

        if !self.standalone {
            self.embedded_document(document);
            return;
        }

        // `lang` and the doctype (which drives `<body class>`) come from
        // resolved document attributes, defaulting to Asciidoctor's `en` /
        // `article`. The footer's "Last updated" timestamp still needs a
        // docdatetime the caller supplies, so it stays deferred.
        let doctitle = document.doctitle();
        let lang = attribute_str(document, "lang").unwrap_or_else(|| "en".to_string());
        let doctype = attribute_str(document, "doctype").unwrap_or_else(|| "article".to_string());

        self.line("<!DOCTYPE html>");
        self.line(&format!("<html lang=\"{}\">", escape_attribute(&lang)));
        self.line("<head>");
        self.line("<meta charset=\"UTF-8\">");
        self.line("<meta http-equiv=\"X-UA-Compatible\" content=\"IE=edge\">");
        self.line("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
        self.line(&format!(
            "<meta name=\"generator\" content=\"asciidoc-html5 {}\">",
            env!("CARGO_PKG_VERSION")
        ));

        // The <title> is the plain-text doctitle. The parser's `doctitle()` has
        // had header substitutions applied (special characters escaped), which
        // is what we want inside <title>.
        if let Some(title) = doctitle {
            self.line(&format!("<title>{title}</title>"));
        }

        // Asciidoctor embeds its default stylesheet (and the web-font link it
        // relies on) into the `<head>` of a standalone document, right after
        // the `<title>`. This renderer always produces standalone output, so it
        // does the same unless the document opts out.
        self.stylesheet(document);

        // Head docinfo is appended to the bottom of the `<head>`, below the
        // default stylesheet, matching Asciidoctor.
        self.docinfo(document, DocinfoLocation::Head);

        self.line("</head>");
        self.line(&format!("<body class=\"{}\">", escape_attribute(&doctype)));

        // Header docinfo is inserted immediately before the header `<div>`,
        // whether or not the header itself is suppressed by `noheader` — this
        // is what lets a docinfo header replace the default one.
        self.docinfo(document, DocinfoLocation::Header);

        // The header is suppressed by `noheader`.
        if !document.is_attribute_set("noheader") {
            self.header(document);
        }

        self.line("<div id=\"content\">");
        self.blocks(document.nested_blocks());
        self.line("</div>");

        // The footer is suppressed by `nofooter`. The "Last updated …" text is
        // deferred until a docdatetime attribute is threaded in by the caller.
        if !document.is_attribute_set("nofooter") {
            self.line("<div id=\"footer\">");
            self.line("<div id=\"footer-text\">");
            self.line("</div>");
            self.line("</div>");
        }

        // Footer docinfo is inserted immediately after the footer `<div>`, again
        // whether or not the footer itself is suppressed by `nofooter`.
        self.docinfo(document, DocinfoLocation::Footer);

        self.line("</body>");
        self.line("</html>");
    }

    /// Emits embedded, body-only output: the converted document body with no
    /// shell, stylesheet, or header/footer frame.
    ///
    /// Matching Asciidoctor's embeddable output, the doctitle is emitted as a
    /// bare `<h1>` only when the title is enabled — never wrapped in
    /// `<div id="header">` and never accompanied by the author or revision
    /// details, which an embedded document does not show. The body itself is
    /// not wrapped in `<div id="content">`.
    ///
    /// The title toggle is the resolved `showtitle` attribute, which defaults
    /// off for embedded output. `asciidoc-parser` links `showtitle` and
    /// `notitle` as inverse spellings of the same toggle (its port of
    /// Asciidoctor's linkage), so unsetting `notitle` (`:!notitle:`) enables
    /// the title just as `:showtitle:` does, and when both are given the last
    /// assignment wins — reading `showtitle` alone captures all of it.
    fn embedded_document(&mut self, document: &Document<'_>) {
        if document.is_attribute_set("showtitle") {
            if let Some(title) = document.doctitle() {
                self.line(&format!("<h1>{title}</h1>"));
            }
        }

        self.blocks(document.nested_blocks());
    }

    /// Emits the output for the `inline` doctype: the inline content of the
    /// document's *first* block, on its own line, with no block wrapper and no
    /// document shell.
    ///
    /// This mirrors Asciidoctor's inline doctype, which "converts a single
    /// paragraph, verbatim, or raw block" — the block kinds that expose
    /// rendered inline content ([`IsBlock::rendered_content`]). When the first
    /// block is one of those, its content (already substituted by the parser)
    /// is emitted directly; when it is anything else — a compound block, a
    /// list, a section — there is no inline candidate, and this emits nothing.
    /// (Asciidoctor additionally logs a warning and returns `nil` in that case;
    /// this crate has no logger, so it produces the empty output without the
    /// warning.) Any blocks after the first are ignored, as in Asciidoctor.
    fn inline_document(&mut self, document: &Document<'_>) {
        if let Some(content) = document
            .nested_blocks()
            .next()
            .and_then(|block| block.rendered_content())
        {
            self.line(content);
        }
    }

    /// Emits `<div id="header">` with the `<h1>` doctitle and, when present,
    /// the author and revision details block.
    fn header(&mut self, document: &Document<'_>) {
        let header: &Header<'_> = document.header();

        // A standalone document shows its doctitle as the header `<h1>` by
        // default; the `notitle` attribute suppresses it. (`noheader`, which
        // drops the whole header, is handled by the caller.)
        let title = document
            .doctitle()
            .filter(|_| !document.is_attribute_set("notitle"));
        let author_line = header.author_line();
        let revision_line = header.revision_line();

        if title.is_none() && author_line.is_none() && revision_line.is_none() {
            return;
        }

        self.line("<div id=\"header\">");

        if let Some(title) = title {
            self.line(&format!("<h1>{title}</h1>"));
        }

        let has_details =
            author_line.is_some_and(|a| a.authors().len() > 0) || revision_line.is_some();
        if has_details {
            self.line("<div class=\"details\">");

            if let Some(author_line) = author_line {
                for (index, author) in author_line.authors().enumerate() {
                    let suffix = if index == 0 {
                        String::new()
                    } else {
                        (index + 1).to_string()
                    };
                    // Author name and email arrive unsubstituted from the
                    // parser (unlike the revision fields, which are already
                    // escaped), so we escape them ourselves before placing them
                    // in text and in the `mailto:` href.
                    self.line(&format!(
                        "<span id=\"author{suffix}\" class=\"author\">{}</span><br>",
                        escape_attribute(author.name())
                    ));
                    if let Some(email) = author.email() {
                        let email = escape_attribute(email);
                        self.line(&format!(
                            "<span id=\"email{suffix}\" class=\"email\"><a href=\"mailto:{email}\">{email}</a></span><br>",
                        ));
                    }
                }
            }

            if let Some(revision) = revision_line {
                if let Some(revnumber) = revision.revnumber() {
                    // Asciidoctor prints "version <n>" and appends a comma when
                    // a revision date follows.
                    let comma = if revision.revdate().is_empty() {
                        ""
                    } else {
                        ","
                    };
                    self.line(&format!(
                        "<span id=\"revnumber\">version {revnumber}{comma}</span>"
                    ));
                }
                if !revision.revdate().is_empty() {
                    self.line(&format!(
                        "<span id=\"revdate\">{}</span>",
                        revision.revdate()
                    ));
                }
                if let Some(revremark) = revision.revremark() {
                    self.line(&format!("<br><span id=\"revremark\">{revremark}</span>"));
                }
            }

            self.line("</div>");
        }

        self.line("</div>");
    }

    /// Emits the resolved docinfo content for `location`, if any.
    ///
    /// Docinfo is auxiliary content the caller supplies from *docinfo files*
    /// (via a [`DocinfoFileHandler`]) and AsciiDoc injects verbatim into fixed
    /// positions of the output: the bottom of the `<head>`
    /// ([`Head`](DocinfoLocation::Head)), immediately before the header `<div>`
    /// ([`Header`](DocinfoLocation::Header)), and immediately after the footer
    /// `<div>` ([`Footer`](DocinfoLocation::Footer)). The parser has already
    /// selected the applicable files (per the `docinfo` attribute),
    /// concatenated them, and applied `docinfosubs` substitutions, so this
    /// crate only places the resulting fragment. An empty result emits
    /// nothing.
    ///
    /// [`DocinfoFileHandler`]: asciidoc_parser::parser::DocinfoFileHandler
    fn docinfo(&mut self, document: &Document<'_>, location: DocinfoLocation) {
        let content = document.docinfo(location);
        if !content.is_empty() {
            self.line(content);
        }
    }

    /// Emits the stylesheet portion of the `<head>`, mirroring Asciidoctor's
    /// `html5` backend.
    ///
    /// Which stylesheet applies is keyed off the `stylesheet` attribute:
    ///
    /// - Absent, set with no value, empty, or `DEFAULT` (Asciidoctor's
    ///   `DEFAULT_STYLESHEET_KEYS`): the default stylesheet — the Google Fonts
    ///   `<link>` plus either an inline `<style>` or, under `linkcss`, a
    ///   `<link>` to `./asciidoctor.css`.
    /// - Explicitly unset (`:stylesheet!:`): no stylesheet block at all.
    /// - Any other value: a *custom* stylesheet, handled by
    ///   [`custom_stylesheet`](Self::custom_stylesheet).
    fn stylesheet(&mut self, document: &Document<'_>) {
        // Explicitly unset (`:stylesheet!:`): no stylesheet block at all.
        if stylesheet_disabled(document) {
            return;
        }

        // A custom stylesheet: link to it, or embed CSS the caller supplied /
        // that was read from disk.
        if let Some(value) = custom_stylesheet_value(document) {
            self.custom_stylesheet(document, &value);
            return;
        }

        // Otherwise the default stylesheet applies (absent, `Set`, empty, or
        // `DEFAULT`).

        self.webfonts_link(document);

        if links_stylesheet(document) {
            // Asciidoctor links the default stylesheet under its public name
            // `asciidoctor.css`, normalized to a web path against `stylesdir`
            // (the same join a custom stylesheet's link uses) — so with no
            // `stylesdir` the href is `./asciidoctor.css`, and under
            // `stylesdir=css` it becomes `./css/asciidoctor.css`.
            let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();
            let href = normalize_web_path(DEFAULT_STYLESHEET_NAME, &stylesdir);
            self.line(&format!(
                "<link rel=\"stylesheet\" href=\"{}\">",
                escape_attribute(&href)
            ));
        } else {
            // The template is `<style>\n{data}\n</style>`, where `data` is the
            // stylesheet with a single trailing newline chomped, so no blank
            // line separates the CSS from the closing tag.
            self.line("<style>");
            self.line(
                DEFAULT_STYLESHEET
                    .strip_suffix('\n')
                    .unwrap_or(DEFAULT_STYLESHEET),
            );
            self.line("</style>");
        }
    }

    /// Emits the stylesheet block for a *custom* `stylesheet` value.
    ///
    /// Unlike the default stylesheet, no web-font `<link>` is emitted —
    /// matching Asciidoctor, which loads the web fonts only for its own
    /// default stylesheet. Then:
    ///
    /// - Under `linkcss` (which the `Secure` default turns on), the head links
    ///   to the stylesheet at the web path Asciidoctor would use, computed from
    ///   the `stylesheet` and `stylesdir` attributes by [`normalize_web_path`].
    /// - Otherwise the stylesheet is embedded inline from `custom_stylesheet` —
    ///   the CSS the caller supplied through
    ///   [`Options::stylesheet_content`](crate::Options::stylesheet_content) or
    ///   that [`convert_with`](crate::convert_with) read from disk. When
    ///   neither produced any CSS — as for the string-only
    ///   [`convert`](crate::convert) entry point, which has no base directory
    ///   to read from — the block is omitted.
    fn custom_stylesheet(&mut self, document: &Document<'_>, stylesheet: &str) {
        if links_stylesheet(document) {
            let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();
            let href = normalize_web_path(stylesheet, &stylesdir);
            self.line(&format!(
                "<link rel=\"stylesheet\" href=\"{}\">",
                escape_attribute(&href)
            ));
        } else if let Some(css) = self.custom_stylesheet {
            self.line("<style>");
            self.line(css.strip_suffix('\n').unwrap_or(css));
            self.line("</style>");
        }
    }

    /// Emits the `<link rel="stylesheet">` that loads the web fonts the default
    /// stylesheet prefers, unless the `webfonts` attribute has been explicitly
    /// unset (`:webfonts!:`). A non-empty `webfonts` value replaces the default
    /// font family; an empty value (or a bare `:webfonts:`) keeps the default.
    fn webfonts_link(&mut self, document: &Document<'_>) {
        // Present but unset means the user opted out of web fonts.
        if document.has_attribute("webfonts") && !document.is_attribute_set("webfonts") {
            return;
        }

        let family = match document.attribute_value("webfonts") {
            InterpretedValue::Value(value) if !value.is_empty() => value,
            _ => DEFAULT_WEBFONTS.to_string(),
        };

        // The value reaches us with AsciiDoc's specialchars substitution already
        // applied by the parser, so `&`, `<`, and `>` are escaped — matching
        // Asciidoctor, which then emits the value as-is. That leaves a literal
        // `"` free to break out of the `href` (a header-set `webfonts` value
        // could otherwise inject attributes onto the `<link>`), so we escape the
        // one remaining special character. This is a no-op for the default and
        // any real font query, which contain no `"`, so output stays
        // byte-identical to Asciidoctor for every valid value.
        let family = family.replace('"', "&quot;");
        self.line(&format!(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family={family}\">"
        ));
    }

    /// Walks a sequence of sibling blocks in document order.
    fn blocks<'src>(&mut self, blocks: Iter<'src, Block<'src>>) {
        for block in blocks {
            self.block(block);
        }
    }

    /// The dispatch point: routes one block to the matching renderer.
    fn block<'src>(&mut self, block: &'src Block<'src>) {
        match block {
            Block::Simple(simple) => match simple.style() {
                // A styled paragraph can convert to a different block: `[open]`
                // over a paragraph becomes an open block (an empty content div
                // around the paragraph's text).
                SimpleBlockStyle::Paragraph => match block.declared_style() {
                    Some("open") => self.open_block(block),
                    Some("sidebar") => self.sidebar(block),
                    Some("example") => self.example(block),
                    _ => self.paragraph(block),
                },
                SimpleBlockStyle::Listing => self.verbatim(block, "listingblock"),
                SimpleBlockStyle::Source => self.source(block),
                SimpleBlockStyle::Literal => self.verbatim(block, "literalblock"),
            },
            Block::Section(section) => self.section(block, section),
            Block::Preamble(_) => self.preamble(block),
            Block::Break(brk) => self.break_block(brk),
            Block::RawDelimited(_) => match block.resolved_context().as_ref() {
                "listing" => self.verbatim(block, "listingblock"),
                "literal" => self.verbatim(block, "literalblock"),
                other => self.unsupported(other),
            },
            Block::CompoundDelimited(compound) => match compound.context_kind() {
                CompoundDelimitedContext::Open => self.open_block(block),
                CompoundDelimitedContext::Sidebar => self.sidebar(block),
                CompoundDelimitedContext::Example => self.example(block),
            },
            Block::Quote(quote) => self.quote(block, quote),
            Block::Admonition(admonition) => self.admonition(block, admonition),

            // Deferred to later phases; see ARCHITECTURE.md for the roadmap.
            other => self.unsupported(&other.resolved_context()),
        }
    }

    /// `<div class="paragraph"><p>…</p></div>`, with an optional title and
    /// author roles on the wrapper.
    fn paragraph<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "paragraph");
        self.block_title(block);
        let content = block.rendered_content().unwrap_or_default();
        self.line(&format!("<p>{content}</p>"));
        self.line("</div>");
    }

    /// `<div class="listingblock|literalblock"><div
    /// class="content"><pre>…</pre></div></div>`.
    ///
    /// Verbatim content keeps its literal line breaks, so it is emitted inside
    /// the `<pre>` without added newlines around the text.
    fn verbatim<'src>(&mut self, block: &'src Block<'src>, wrapper_class: &str) {
        self.open_block_wrapper(block, wrapper_class);
        self.block_title(block);
        self.line("<div class=\"content\">");
        let content = block.rendered_content().unwrap_or_default();
        self.line(&format!("<pre>{content}</pre>"));
        self.line("</div>");
        self.line("</div>");
    }

    /// A source block: like a listing block, but the `<pre>` carries the
    /// `highlight` class and wraps the code in a `<code>` element that names
    /// the language (`class="language-…" data-lang="…"`) when one is
    /// declared. This matches Asciidoctor's default output even when no
    /// syntax highlighter is active.
    fn source<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "listingblock");
        self.block_title(block);
        self.line("<div class=\"content\">");

        let content = block.rendered_content().unwrap_or_default();

        // The language is the second positional attribute of `[source, lang]`
        // (the first is the `source` style itself), or an explicit `language=`.
        let language = block
            .attrlist()
            .and_then(|attrlist| attrlist.named_or_positional_attribute("language", 2))
            .map(|attr| attr.value());

        match language {
            Some(language) => {
                let language = escape_attribute(language);
                self.line(&format!(
                    "<pre class=\"highlight\"><code class=\"language-{language}\" \
                     data-lang=\"{language}\">{content}</code></pre>"
                ));
            }
            None => self.line(&format!(
                "<pre class=\"highlight\"><code>{content}</code></pre>"
            )),
        }

        self.line("</div>");
        self.line("</div>");
    }

    /// An open block: `<div class="openblock"><div
    /// class="content">…</div></div>`. Used for the `--` delimited form and for
    /// a paragraph carrying the `[open]` style.
    fn open_block<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "openblock");
        self.block_title(block);
        self.line("<div class=\"content\">");
        self.wrapped_content(block);
        self.line("</div>");
        self.line("</div>");
    }

    /// A sidebar block: `<div class="sidebarblock"><div
    /// class="content">…</div></div>`. Used for the `****` delimited form and
    /// for a paragraph carrying the `[sidebar]` style. Unlike most blocks, the
    /// title sits *inside* the content div (before the content), matching
    /// Asciidoctor.
    fn sidebar<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "sidebarblock");
        self.line("<div class=\"content\">");
        self.block_title(block);
        self.wrapped_content(block);
        self.line("</div>");
        self.line("</div>");
    }

    /// An example block: `<div class="exampleblock">[<div
    /// class="title">…</div>]<div class="content">…</div></div>`. Used for the
    /// `====` delimited form and for a paragraph carrying the `[example]`
    /// style. A titled example is *captioned* — its title div carries the
    /// block's caption prefix (`Example N. `) ahead of the title text; an
    /// untitled example has no title div at all.
    fn example<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "exampleblock");
        self.captioned_title(block);
        self.line("<div class=\"content\">");
        self.wrapped_content(block);
        self.line("</div>");
        self.line("</div>");
    }

    /// A quote block (`<div
    /// class="quoteblock"><blockquote>…</blockquote>…</div>`) or a verse
    /// block (`<div class="verseblock"><pre class="content">…</pre>…</
    /// div>`), distinguished by the block's quote type. A verse preserves
    /// line breaks inside a `<pre>`; a quote wraps prose in a
    /// `<blockquote>`. Both render an optional attribution footer.
    fn quote<'src>(&mut self, block: &'src Block<'src>, quote: &'src QuoteBlock<'src>) {
        match quote.type_() {
            QuoteType::Quote => {
                self.open_block_wrapper(block, "quoteblock");
                self.block_title(block);
                self.line("<blockquote>");
                self.wrapped_content(block);
                self.line("</blockquote>");
            }
            QuoteType::Verse => {
                self.open_block_wrapper(block, "verseblock");
                self.block_title(block);
                let content = block.rendered_content().unwrap_or_default();
                self.line(&format!("<pre class=\"content\">{content}</pre>"));
            }
        }

        self.attribution(quote);
        self.line("</div>");
    }

    /// The `<div class="attribution">` footer of a quote or verse block,
    /// emitted only when an attribution or citation title is present.
    fn attribution(&mut self, quote: &QuoteBlock<'_>) {
        let attribution = quote.attribution();
        let citetitle = quote.citetitle();
        if attribution.is_none() && citetitle.is_none() {
            return;
        }

        self.line("<div class=\"attribution\">");
        if let Some(attribution) = attribution {
            // When a citation title follows on its own `<cite>` line, the
            // attribution line ends with a `<br>`.
            let line_break = if citetitle.is_some() { "<br>" } else { "" };
            self.line(&format!("&#8212; {attribution}{line_break}"));
        }
        if let Some(citetitle) = citetitle {
            self.line(&format!("<cite>{citetitle}</cite>"));
        }
        self.line("</div>");
    }

    /// An admonition block: Asciidoctor's icon-less default renders a two-cell
    /// table, the first cell holding the caption label and the second the
    /// content, wrapped in `<div class="admonitionblock <name>">`.
    fn admonition<'src>(
        &mut self,
        block: &'src Block<'src>,
        admonition: &'src AdmonitionBlock<'src>,
    ) {
        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute(
                &format!("admonitionblock {}", admonition.name()),
                &block.roles()
            )
        ));
        self.line("<table>");
        self.line("<tr>");
        self.line("<td class=\"icon\">");
        self.line(&format!(
            "<div class=\"title\">{}</div>",
            admonition.label()
        ));
        self.line("</td>");
        self.line("<td class=\"content\">");
        self.block_title(block);
        self.wrapped_content(block);
        self.line("</td>");
        self.line("</tr>");
        self.line("</table>");
        self.line("</div>");
    }

    /// Emits the inner content shared by wrapper blocks (open, quote,
    /// admonition): a compound block recurses over its nested blocks, while a
    /// simple block emits its rendered content on its own line, unwrapped (no
    /// `<p>`), matching Asciidoctor.
    fn wrapped_content<'src>(&mut self, block: &'src Block<'src>) {
        if block.content_model() == ContentModel::Compound {
            self.blocks(block.nested_blocks());
        } else {
            let content = block.rendered_content().unwrap_or_default();
            self.line(content);
        }
    }

    /// A section: `<div class="sectN"><hM id>title</hM>…</div>`. Level-1
    /// sections wrap their body in `<div class="sectionbody">`; deeper levels
    /// place children directly after the heading. Discrete headings render as a
    /// bare heading with no wrapper.
    fn section<'src>(&mut self, block: &'src Block<'src>, section: &'src SectionBlock<'src>) {
        let level = section.level();
        let heading_level = (level + 1).min(6);

        // `Block::id()` now surfaces a section's auto-generated id (it delegates
        // to the `SectionBlock` override), so the block-level accessor is enough.
        let id = block.id();
        let title = section.section_title();

        if section.section_type() == SectionType::Discrete {
            // Asciidoctor renders a discrete heading as a bare `<hN>` carrying
            // the `discrete` class plus any roles, e.g. `class="discrete role"`.
            self.line(&format!(
                "<h{heading_level}{}{}>{title}</h{heading_level}>",
                id_attribute(id),
                class_attribute("discrete", &block.roles())
            ));
            return;
        }

        self.line(&format!(
            "<div{}>",
            class_attribute(&format!("sect{level}"), &block.roles())
        ));
        self.line(&format!(
            "<h{heading_level}{}>{title}</h{heading_level}>",
            id_attribute(id)
        ));

        if level == 1 {
            self.line("<div class=\"sectionbody\">");
            self.blocks(section.nested_blocks());
            self.line("</div>");
        } else {
            self.blocks(section.nested_blocks());
        }

        self.line("</div>");
    }

    /// The preamble: content between the doctitle and the first section,
    /// wrapped as `<div id="preamble"><div
    /// class="sectionbody">…</div></div>`.
    fn preamble<'src>(&mut self, block: &'src Block<'src>) {
        self.line("<div id=\"preamble\">");
        self.line("<div class=\"sectionbody\">");
        self.blocks(block.nested_blocks());
        self.line("</div>");
        self.line("</div>");
    }

    /// A break: `<hr>` for a thematic break, or Asciidoctor's page-break
    /// `<div>` for a page break.
    fn break_block(&mut self, brk: &Break<'_>) {
        match brk.type_() {
            BreakType::Thematic => self.line("<hr>"),
            BreakType::Page => self.line("<div style=\"page-break-after: always;\"></div>"),
        }
    }

    /// Opens `<div id=… class="<base> <roles>">` for a leaf block wrapper.
    fn open_block_wrapper<'src>(&mut self, block: &'src Block<'src>, base_class: &str) {
        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute(base_class, &block.roles())
        ));
    }

    /// Emits the block's `<div class="title">…</div>`, if it has a title. The
    /// title text has already had substitutions applied by the parser.
    fn block_title<'src>(&mut self, block: &'src Block<'src>) {
        if let Some(title) = block.title() {
            self.line(&format!("<div class=\"title\">{title}</div>"));
        }
    }

    /// Emits a *captioned* block's `<div class="title">…</div>`, if it has a
    /// title: the caption prefix (a ready-made `"Example 1. "` label from the
    /// parser, including its trailing separator and space) is placed ahead of
    /// the title text. A block with a caption but no title emits nothing, and a
    /// titled block with no caption falls back to the bare title — matching
    /// [`block_title`](Self::block_title). Both the caption and the title have
    /// already had substitutions applied by the parser.
    fn captioned_title<'src>(&mut self, block: &'src Block<'src>) {
        if let Some(title) = block.title() {
            let caption = block.caption().unwrap_or_default();
            self.line(&format!("<div class=\"title\">{caption}{title}</div>"));
        }
    }

    /// Emits a visible placeholder for a construct the baseline does not yet
    /// handle, keeping the output well-formed while making the gap obvious.
    fn unsupported(&mut self, context: &str) {
        self.line(&format!(
            "<!-- asciidoc-html5: unsupported block context '{context}' -->"
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{Options, SafeMode};

    // These renderer tests assert the standalone document shell (the
    // `<!DOCTYPE>`/`<head>`/`<body>` frame, the header, and the footer), so they
    // render in standalone mode explicitly. The string entry points now default
    // to embedded, body-only output, so `convert`/`convert_with` are shadowed
    // here to force `standalone(true)`; the handful of embedded-output checks
    // call `crate::convert_with` directly instead.

    /// Converts `source` to a standalone document under the default safe mode —
    /// the standalone counterpart of [`crate::convert`].
    fn convert(source: &str) -> String {
        crate::convert_with(source, &Options::new().standalone(true))
    }

    /// Converts `source` to a standalone document under `options` — the
    /// standalone counterpart of [`crate::convert_with`].
    fn convert_with(source: &str, options: &Options) -> String {
        crate::convert_with(source, &options.clone().standalone(true))
    }

    /// Converts `source` with the given docinfo files (name → content) written
    /// to a fresh temp directory, under `Safe` safe mode with a primary file of
    /// `mydoc.adoc` in that directory (so both shared and private docinfo files
    /// resolve). `Safe` — not `Server` — is used because these sources enable
    /// docinfo from the *document* (`:docinfo:`), which `Server` and above
    /// forbid; and `Secure` disables docinfo resolution entirely.
    ///
    /// `tag` names the temp directory so concurrent tests do not collide.
    fn with_docinfo(tag: &str, source: &str, files: &[(&str, &str)]) -> String {
        let dir =
            std::env::temp_dir().join(format!("adoc-render-docinfo-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        for (name, content) in files {
            std::fs::write(dir.join(name), content).expect("write scratch file");
        }

        let html = convert_with(
            source,
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file(dir.join("mydoc.adoc")),
        );

        let _ = std::fs::remove_dir_all(&dir);
        html
    }

    /// Converts `source` under a safe mode below `Secure`, so the default
    /// stylesheet is embedded inline (`<style>`) rather than linked. The
    /// default (`Secure`) mode links it; these tests exercise the embed
    /// branch, which is the `adoc` CLI's default behavior.
    fn embed(source: &str) -> String {
        convert_with(source, &Options::new().safe_mode(SafeMode::Unsafe))
    }

    /// Extracts the body of the `<div id="content">…</div>` wrapper so tests
    /// can assert on block structure without repeating the document
    /// skeleton.
    fn content(html: &str) -> String {
        let start = html.find("<div id=\"content\">").expect("content div")
            + "<div id=\"content\">\n".len();
        // Fall back to the end of the string when there is no footer (e.g. a
        // `:nofooter:` document), so this helper never panics.
        let end = html[start..]
            .find("<div id=\"footer\">")
            .map_or(html.len(), |offset| start + offset);
        html[start..end].trim_end().to_string()
    }

    #[test]
    fn document_skeleton() {
        let html = convert("= Title\n\nHi.");
        assert!(html.starts_with("<!DOCTYPE html>\n<html lang=\"en\">\n"));
        assert!(html.contains("<meta charset=\"UTF-8\">"));
        assert!(html.contains("<title>Title</title>"));
        assert!(html.contains("<body class=\"article\">"));
        assert!(html.contains("<div id=\"header\">\n<h1>Title</h1>\n</div>"));
        assert!(html.trim_end().ends_with("</body>\n</html>"));
    }

    #[test]
    fn paragraph_carries_parser_inline_html() {
        // The parser renders inline markup; the block renderer only wraps it.
        let html = convert("A _quiet_ *storm*.");
        assert!(html.contains(
            "<div class=\"paragraph\">\n<p>A <em>quiet</em> <strong>storm</strong>.</p>\n</div>"
        ));
    }

    #[test]
    fn nested_sections_map_to_sect_levels() {
        let html = convert("= Doc\n\n== One\n\nx\n\n=== Two\n\ny");
        let body = content(&html);
        assert!(body.contains(
            "<div class=\"sect1\">\n<h2 id=\"_one\">One</h2>\n<div class=\"sectionbody\">"
        ));
        assert!(body.contains("<div class=\"sect2\">\n<h3 id=\"_two\">Two</h3>"));
    }

    #[test]
    fn preamble_is_wrapped() {
        let html = convert("= Doc\n\nIntro.\n\n== Section\n\nBody.");
        let body = content(&html);
        assert!(body.starts_with("<div id=\"preamble\">\n<div class=\"sectionbody\">"));
    }

    #[test]
    fn verbatim_content_stays_escaped() {
        let html = convert("[listing]\n<html> & co");
        assert!(html.contains(
            "<div class=\"listingblock\">\n<div class=\"content\">\n<pre>&lt;html&gt; &amp; co</pre>"
        ));
    }

    #[test]
    fn thematic_break_renders_hr() {
        let html = convert("before\n\n'''\n\nafter");
        assert!(content(&html).contains("<hr>"));
    }

    #[test]
    fn unsupported_block_leaves_a_marker() {
        let html = convert("* one\n* two");
        assert!(html.contains("<!-- asciidoc-html5: unsupported block context 'list' -->"));
    }

    #[test]
    fn block_title_and_roles_appear_on_wrapper() {
        let html = convert(".A caption\n[.lead]\nParagraph text.");
        assert!(html.contains("<div class=\"paragraph lead\">"));
        assert!(html.contains("<div class=\"title\">A caption</div>"));
    }

    // The following exercise the document-attribute-driven skeleton, reading
    // resolved attributes straight off the `Document` (asciidoc-parser#620).

    #[test]
    fn lang_attribute_drives_html_lang() {
        let html = convert("= Doc\n:lang: de\n\nBody.");
        assert!(html.contains("<html lang=\"de\">"));
    }

    #[test]
    fn doctype_drives_body_class() {
        // `article` is the only doctype this renderer models, so `Options::apply`
        // pins `doctype` to `article` and locks it against the document. A
        // document `:doctype: book` is therefore dropped and the `<body class>`
        // stays `article` (see the pin and its unit tests in `options.rs`).
        let html = convert("= Doc\n:doctype: book\n\nBody.");
        assert!(html.contains("<body class=\"article\">"));
    }

    #[test]
    fn notitle_suppresses_the_header_h1() {
        let html = convert("= Doc\n:notitle:\n\nBody.");
        assert!(!html.contains("<h1>"));

        // The title still populates <head>.
        assert!(html.contains("<title>Doc</title>"));
    }

    #[test]
    fn noheader_suppresses_the_header() {
        let html = convert("= Doc\n:noheader:\n\nBody.");
        assert!(!html.contains("<div id=\"header\">"));
    }

    #[test]
    fn nofooter_suppresses_the_footer() {
        let html = convert("= Doc\n:nofooter:\n\nBody.");
        assert!(!html.contains("<div id=\"footer\">"));
    }

    // Embedded, body-only output shows the doctitle `<h1>` only when the title
    // toggle is enabled, and never emits the header or footer frame. The toggle
    // is the resolved `showtitle` attribute (off by default for embedded
    // output); because `asciidoc-parser` links `showtitle` and `notitle` as
    // inverse spellings of it, unsetting `notitle` enables the title too, and
    // when both are given the last assignment wins. These call
    // `crate::convert_with` directly — the module's `convert`/`convert_with` are
    // shadowed to force standalone output.

    /// Whether embedded output for `source` emits the doctitle `<h1>`,
    /// asserting along the way that neither the header nor the footer frame
    /// appears.
    fn embedded_shows_title(source: &str) -> bool {
        let html = crate::convert_with(source, &Options::new());
        assert!(
            !html.contains("id=\"header\""),
            "embedded has no header: {html}"
        );
        assert!(
            !html.contains("id=\"footer\""),
            "embedded has no footer: {html}"
        );
        html.contains("<h1>Doc</h1>")
    }

    #[test]
    fn embedded_hides_the_title_by_default() {
        assert!(!embedded_shows_title("= Doc\n\nBody."));
    }

    #[test]
    fn embedded_shows_the_title_under_showtitle() {
        assert!(embedded_shows_title("= Doc\n:showtitle:\n\nBody."));
    }

    #[test]
    fn embedded_shows_the_title_when_notitle_is_unset() {
        // `:!notitle:` is the inverse spelling of `:showtitle:`, so it enables
        // the embedded title just the same.
        assert!(embedded_shows_title("= Doc\n:!notitle:\n\nBody."));
    }

    #[test]
    fn embedded_hides_the_title_under_notitle() {
        assert!(!embedded_shows_title("= Doc\n:notitle:\n\nBody."));
    }

    #[test]
    fn embedded_title_toggle_honors_the_last_assignment() {
        // The two attributes track one toggle, so the last assignment wins.
        assert!(embedded_shows_title(
            "= Doc\n:notitle:\n:showtitle:\n\nBody."
        ));
        assert!(!embedded_shows_title(
            "= Doc\n:showtitle:\n:notitle:\n\nBody."
        ));
    }

    #[test]
    fn embedded_title_responds_to_the_notitle_api_toggle() {
        // The linkage also applies to API-supplied attributes: unsetting
        // `notitle` from the API enables the title, setting it hides it.
        let shown = crate::convert_with("= Doc\n\nBody.", &Options::new().unset("notitle"));
        assert!(shown.contains("<h1>Doc</h1>"));
        let hidden = crate::convert_with("= Doc\n\nBody.", &Options::new().set("notitle"));
        assert!(!hidden.contains("<h1>"));
    }

    #[test]
    fn author_name_and_email_are_escaped() {
        // The parser hands these back unsubstituted, so the renderer must escape
        // them itself — otherwise a `"` would break out of the `href`.
        let html = convert("= Doc\nBen & Jerry <a\"b@example.com>\n\nBody.");
        assert!(html.contains("<span id=\"author\" class=\"author\">Ben &amp; Jerry</span>"));
        assert!(html.contains(
            "<span id=\"email\" class=\"email\"><a href=\"mailto:a&quot;b@example.com\">a&quot;b@example.com</a></span>"
        ));
    }

    #[test]
    fn discrete_heading_carries_discrete_class_and_roles() {
        let html = convert("= Doc\n\n[.independent]\n[discrete]\n== Free Heading");
        assert!(content(&html)
            .contains("<h2 id=\"_free_heading\" class=\"discrete independent\">Free Heading</h2>"));
    }

    #[test]
    fn content_helper_tolerates_a_missing_footer() {
        // Exercises the `content()` fallback: a `:nofooter:` document has no
        // footer div for the helper to anchor its end on.
        let body = content(&convert("= Doc\n:nofooter:\n\nBody."));
        assert!(body.contains("<div class=\"paragraph\">\n<p>Body.</p>\n</div>"));
    }

    #[test]
    fn multiple_authors_are_numbered() {
        // The first author has no email; the second does. Only the second
        // carries a numbered suffix.
        let html = convert("= Doc\nJane Doe; John Roe <john@y.com>\n\nBody.");
        assert!(html.contains("<span id=\"author\" class=\"author\">Jane Doe</span>"));
        assert!(html.contains("<span id=\"author2\" class=\"author\">John Roe</span>"));
        assert!(html.contains(
            "<span id=\"email2\" class=\"email\"><a href=\"mailto:john@y.com\">john@y.com</a></span>"
        ));
        assert!(!html.contains("id=\"email\""));
    }

    #[test]
    fn revision_line_renders_number_date_and_remark() {
        let html = convert("= Doc\nJane Doe\nv2.0, 2026-01-01: Initial\n\nBody.");
        assert!(html.contains("<span id=\"revnumber\">version 2.0,</span>"));
        assert!(html.contains("<span id=\"revdate\">2026-01-01</span>"));
        assert!(html.contains("<br><span id=\"revremark\">Initial</span>"));
    }

    #[test]
    fn revision_number_without_date_omits_the_comma_and_date() {
        let html = convert("= Doc\nJane Doe\nv2.0\n\nBody.");
        assert!(html.contains("<span id=\"revnumber\">version 2.0</span>"));
        assert!(!html.contains("id=\"revdate\""));
    }

    #[test]
    fn literal_style_paragraph_renders_a_literalblock() {
        let html = convert("[literal]\n<lit> & co");
        assert!(html.contains(
            "<div class=\"literalblock\">\n<div class=\"content\">\n<pre>&lt;lit&gt; &amp; co</pre>"
        ));
    }

    #[test]
    fn delimited_listing_and_literal_blocks_render() {
        let listing = convert("----\ncode &<\n----");
        assert!(listing.contains(
            "<div class=\"listingblock\">\n<div class=\"content\">\n<pre>code &amp;&lt;</pre>"
        ));
        let literal = convert("....\nlit &<\n....");
        assert!(literal.contains(
            "<div class=\"literalblock\">\n<div class=\"content\">\n<pre>lit &amp;&lt;</pre>"
        ));
    }

    #[test]
    fn delimited_passthrough_is_unsupported_for_now() {
        let html = convert("++++\nraw\n++++");
        assert!(html.contains("<!-- asciidoc-html5: unsupported block context 'pass' -->"));
    }

    // The block shapes below are byte-checked against Asciidoctor 2.0.26's
    // default `html5` output (the parity oracle).

    #[test]
    fn source_block_wraps_code_in_a_highlight_pre() {
        let html = convert("[source]\nuse the source, luke!");
        assert!(html.contains(
            "<div class=\"listingblock\">\n<div class=\"content\">\n\
             <pre class=\"highlight\"><code>use the source, luke!</code></pre>\n\
             </div>\n</div>"
        ));
    }

    #[test]
    fn source_block_names_its_language() {
        let html = convert("[source, perl]\ndie 'zomg perl is tough';");
        assert!(html.contains(
            "<pre class=\"highlight\"><code class=\"language-perl\" data-lang=\"perl\">\
             die 'zomg perl is tough';</code></pre>"
        ));
    }

    #[test]
    fn open_paragraph_and_delimited_open_block_render() {
        let paragraph = convert("[open]\nMake it what you want.");
        assert!(paragraph.contains(
            "<div class=\"openblock\">\n<div class=\"content\">\n\
             Make it what you want.\n</div>\n</div>"
        ));
        let delimited = convert("--\ntext in open block\n--");
        assert!(delimited.contains(
            "<div class=\"openblock\">\n<div class=\"content\">\n\
             <div class=\"paragraph\">\n<p>text in open block</p>\n</div>\n</div>\n</div>"
        ));
    }

    #[test]
    fn quote_paragraph_renders_a_blockquote() {
        let html = convert("[quote]\nFamous quote.");
        assert!(html.contains(
            "<div class=\"quoteblock\">\n<blockquote>\nFamous quote.\n</blockquote>\n</div>"
        ));
    }

    #[test]
    fn quote_renders_its_attribution_footer() {
        let html = convert("[quote,Albert Einstein,Sidebar]\nA clever quote.");
        assert!(html.contains(
            "<blockquote>\nA clever quote.\n</blockquote>\n\
             <div class=\"attribution\">\n&#8212; Albert Einstein<br>\n\
             <cite>Sidebar</cite>\n</div>\n</div>"
        ));
    }

    #[test]
    fn quote_attribution_without_citetitle_has_no_cite() {
        let html = convert("[quote,Gaius]\nVeni, vidi, vici.");
        assert!(html.contains("<div class=\"attribution\">\n&#8212; Gaius\n</div>"));
    }

    #[test]
    fn quote_citetitle_without_attribution_renders_only_the_cite() {
        let html = convert("[quote,,Almanac]\nA stitch in time.");
        assert!(html.contains("<div class=\"attribution\">\n<cite>Almanac</cite>\n</div>"));
    }

    #[test]
    fn verse_paragraph_preserves_content_in_a_pre() {
        let html = convert("[verse]\nFamous verse.");
        assert!(html.contains(
            "<div class=\"verseblock\">\n<pre class=\"content\">Famous verse.</pre>\n</div>"
        ));
    }

    #[test]
    fn admonition_renders_the_icon_less_table() {
        let html = convert("NOTE: This is important, fool!");
        assert!(html.contains(
            "<div class=\"admonitionblock note\">\n<table>\n<tr>\n\
             <td class=\"icon\">\n<div class=\"title\">Note</div>\n</td>\n\
             <td class=\"content\">\nThis is important, fool!\n</td>\n</tr>\n</table>\n</div>"
        ));
    }

    #[test]
    fn admonition_wraps_compound_content() {
        let html = convert("[NOTE]\n====\nThis is a winner.\n====");
        assert!(html.contains(
            "<td class=\"content\">\n<div class=\"paragraph\">\n<p>This is a winner.</p>\n\
             </div>\n</td>"
        ));
    }

    // A sidebar block places its title *inside* the content div (before the
    // content), unlike most blocks; the delimited `****` form nests its
    // children, while the `[sidebar]` styled paragraph drops its text into the
    // content div unwrapped (no `<p>`).
    #[test]
    fn sidebar_delimited_wraps_nested_content() {
        let html = convert("****\nContent here.\n****");
        assert!(html.contains(
            "<div class=\"sidebarblock\">\n<div class=\"content\">\n\
             <div class=\"paragraph\">\n<p>Content here.</p>\n</div>\n</div>\n</div>"
        ));
    }

    #[test]
    fn sidebar_title_sits_inside_the_content_div() {
        let html = convert(".Sidebar Title\n****\nContent here.\n****");
        assert!(html.contains(
            "<div class=\"sidebarblock\">\n<div class=\"content\">\n\
             <div class=\"title\">Sidebar Title</div>\n\
             <div class=\"paragraph\">\n<p>Content here.</p>\n</div>\n</div>\n</div>"
        ));
    }

    #[test]
    fn sidebar_styled_paragraph_emits_unwrapped_content() {
        let html = convert("[sidebar]\nJust some text.");
        assert!(html.contains(
            "<div class=\"sidebarblock\">\n<div class=\"content\">\n\
             Just some text.\n</div>\n</div>"
        ));
    }

    // An example block places a *captioned* title (`Example N. `) before the
    // content div, or no title div at all when untitled; the number increments
    // per titled example in document order.
    #[test]
    fn example_untitled_has_no_title_div() {
        let html = convert("====\nContent here.\n====");
        assert!(html.contains(
            "<div class=\"exampleblock\">\n<div class=\"content\">\n\
             <div class=\"paragraph\">\n<p>Content here.</p>\n</div>\n</div>\n</div>"
        ));
        assert!(!html.contains("<div class=\"title\">"));
    }

    #[test]
    fn example_titled_carries_a_numbered_caption() {
        let html = convert(".An Example\n====\nContent here.\n====");
        assert!(html.contains(
            "<div class=\"exampleblock\">\n\
             <div class=\"title\">Example 1. An Example</div>\n<div class=\"content\">\n\
             <div class=\"paragraph\">\n<p>Content here.</p>\n</div>\n</div>\n</div>"
        ));
    }

    #[test]
    fn titled_examples_are_numbered_in_document_order() {
        let html = convert(".First\n====\none\n====\n\n.Second\n====\ntwo\n====");
        assert!(html.contains("<div class=\"title\">Example 1. First</div>"));
        assert!(html.contains("<div class=\"title\">Example 2. Second</div>"));
    }

    #[test]
    fn example_styled_paragraph_emits_unwrapped_content() {
        let html = convert("[example]\nJust text.");
        assert!(html.contains(
            "<div class=\"exampleblock\">\n<div class=\"content\">\n\
             Just text.\n</div>\n</div>"
        ));

        // A titled styled example is captioned the same as the delimited form.
        let titled = convert(".Titled\n[example]\nJust text.");
        assert!(titled.contains(
            "<div class=\"exampleblock\">\n\
             <div class=\"title\">Example 1. Titled</div>\n<div class=\"content\">\n\
             Just text.\n</div>\n</div>"
        ));
    }

    // The `inline` doctype (selected via `Options::doctype`) converts a
    // fragment: it emits only the first block's inline content, with no block
    // wrapper and no document shell — ignoring the standalone/embedded mode
    // (these use the module's `convert_with`, which forces standalone). The
    // output carries the crate's usual single trailing newline.
    #[test]
    fn inline_doctype_emits_only_the_first_blocks_inline_content() {
        let html = convert_with(
            "http://x[Y] is _z_\n\nignored",
            &Options::new().doctype("inline"),
        );
        assert_eq!(html, "<a href=\"http://x\">Y</a> is <em>z</em>\n");
    }

    #[test]
    fn inline_doctype_takes_a_verbatim_first_block() {
        let html = convert_with("----\ncode &<\n----", &Options::new().doctype("inline"));
        assert_eq!(html, "code &amp;&lt;\n");
    }

    #[test]
    fn inline_doctype_emits_nothing_without_an_inline_candidate() {
        // A list is not a paragraph/verbatim/raw block, so there is no inline
        // candidate; Asciidoctor warns and returns nil, and this crate (having
        // no logger) produces empty output.
        let html = convert_with("* bullet", &Options::new().doctype("inline"));
        assert_eq!(html, "");
    }

    #[test]
    fn page_break_renders_a_page_break_div() {
        let html = convert("before\n\n<<<\n\nafter");
        assert!(content(&html).contains("<div style=\"page-break-after: always;\"></div>"));
    }

    // Under a safe mode below `Secure`, the `<head>` embeds Asciidoctor's
    // default stylesheet and links the web fonts it relies on, in that order,
    // right after the `<title>`. (The default `Secure` mode links the
    // stylesheet instead; see `secure_default_links_the_stylesheet`.)

    #[test]
    fn head_links_web_fonts_then_embeds_the_stylesheet() {
        let html = embed("= Doc\n\nBody.");

        // The web-font link comes first, carrying the default font family.
        assert!(html.contains(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700\">"
        ));

        // Then the stylesheet is embedded inline. The CSS opens with its
        // license banner and ends flush against `</style>` (no blank line).
        assert!(html.contains(
            "<style>\n/*! Asciidoctor default stylesheet | MIT License | https://asciidoctor.org */"
        ));
        assert!(html.contains("{padding:0}}\n</style>"));

        // Ordering: the font link precedes the `<style>`, and both sit inside
        // the head, after the title.
        let title = html.find("<title>").expect("title");
        let fonts = html.find("fonts.googleapis.com").expect("web fonts link");
        let style = html.find("<style>").expect("style");
        let head_end = html.find("</head>").expect("head end");
        assert!(title < fonts && fonts < style && style < head_end);
    }

    #[test]
    fn webfonts_unset_drops_the_font_link_but_keeps_the_stylesheet() {
        let html = embed("= Doc\n:webfonts!:\n\nBody.");
        // No emitted web-font `<link>`. (The embedded CSS mentions Google Fonts
        // in a commented-out `@import`, so match on the `<link>` tag itself.)
        assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
        assert!(html.contains("<style>\n/*! Asciidoctor default stylesheet"));
    }

    // Under the default (`Secure`) safe mode, the head links the stylesheet to
    // `./asciidoctor.css` rather than embedding it, matching Asciidoctor's API.
    #[test]
    fn secure_default_links_the_stylesheet() {
        let html = convert("= Doc\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
        assert!(!html.contains("<style>"));

        // The web-font link is still emitted alongside the linked stylesheet.
        assert!(html.contains("fonts.googleapis.com"));
    }

    #[test]
    fn webfonts_value_overrides_the_font_family() {
        let html = convert("= Doc\n:webfonts: Ubuntu+Mono:400\n\nBody.");
        assert!(html.contains(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Ubuntu+Mono:400\">"
        ));
        // The default-family `<link>` is gone (the CSS comment still names the
        // default fonts, so match on the emitted `<link>` tag).
        assert!(!html.contains(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Open+Sans"
        ));
    }

    #[test]
    fn webfonts_value_double_quote_cannot_break_out_of_the_href() {
        // The parser escapes `&`/`<`/`>` in the value, but not `"`. An
        // unescaped `"` would close the `href` and let a header-set value inject
        // attributes onto the `<link>`; we escape it so the value stays inside.
        let html = convert("= Doc\n:webfonts: x\" onmouseover=\"y\n\nBody.");
        assert!(html.contains(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=x&quot; onmouseover=&quot;y\">"
        ));
        assert!(!html.contains("family=x\" onmouseover"));
    }

    #[test]
    fn linkcss_links_the_stylesheet_instead_of_embedding_it() {
        let html = convert("= Doc\n:linkcss:\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
        assert!(!html.contains("<style>"));
        // The web-font link is still emitted alongside the linked stylesheet.
        assert!(html.contains("fonts.googleapis.com"));
    }

    // The linked default stylesheet honors `stylesdir`, normalized the same way
    // a custom stylesheet's link is — matching Asciidoctor.
    #[test]
    fn linked_default_stylesheet_honors_the_styles_directory() {
        let html = convert("= Doc\n:linkcss:\n:stylesdir: css\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./css/asciidoctor.css\">"));
    }

    #[test]
    fn stylesheet_unset_drops_the_whole_stylesheet_block() {
        let html = convert("= Doc\n:stylesheet!:\n\nBody.");
        assert!(!html.contains("<style>"));
        assert!(!html.contains("fonts.googleapis.com"));
        assert!(!html.contains("asciidoctor.css"));
    }

    #[test]
    fn default_stylesheet_value_still_embeds_the_default() {
        let html = embed("= Doc\n:stylesheet: DEFAULT\n\nBody.");
        assert!(html.contains("<style>\n/*! Asciidoctor default stylesheet"));
    }

    // A custom `stylesheet` under the default (`Secure`) safe mode links to it
    // at its normalized web path, and — unlike the default stylesheet — emits no
    // web-font `<link>`.
    #[test]
    fn custom_stylesheet_links_under_the_secure_default() {
        let html = convert("= Doc\n:stylesheet: my-theme.css\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./my-theme.css\">"));
        assert!(!html.contains("<style>"));
        assert!(!html.contains("./asciidoctor.css"));

        // No web fonts for a custom stylesheet.
        assert!(!html.contains("fonts.googleapis.com"));
    }

    // An explicit `linkcss` links a custom stylesheet even under an embedding
    // safe mode, mirroring the styles directory in the linked path.
    #[test]
    fn custom_stylesheet_link_mirrors_the_styles_directory() {
        let html = convert_with(
            "= Doc\n:stylesheet: custom.css\n:stylesdir: css\n\nBody.",
            &Options::new().safe_mode(SafeMode::Unsafe).set("linkcss"),
        );
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./css/custom.css\">"));
    }

    // A stylesheet given as a URI is linked verbatim.
    #[test]
    fn custom_stylesheet_uri_is_linked_verbatim() {
        let html = convert("= Doc\n:stylesheet: file:///home/user/custom.css\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"file:///home/user/custom.css\">"));
    }

    // Under an embedding safe mode, a custom stylesheet embeds the CSS the caller
    // supplied through `Options::stylesheet_content`.
    #[test]
    fn custom_stylesheet_embeds_supplied_content() {
        let html = convert_with(
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Unsafe)
                .stylesheet_content("body { color: #ff0000; }\n"),
        );
        assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));

        // Still no default stylesheet and no web fonts.
        assert!(!html.contains("/*! Asciidoctor default stylesheet"));
        assert!(!html.contains("fonts.googleapis.com"));
    }

    // When embedding is requested for a custom stylesheet but no content was
    // supplied (the string-only `convert` path cannot read a file), the block is
    // omitted rather than guessed at.
    #[test]
    fn custom_stylesheet_without_content_emits_nothing_when_embedding() {
        let html = convert_with(
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(!html.contains("<style>"));
        assert!(!html.contains("<link rel=\"stylesheet\""));
    }

    // The supplied content is ignored when the stylesheet is linked, not
    // embedded: the head links to the stylesheet path instead.
    #[test]
    fn supplied_content_is_ignored_when_linking() {
        let html = convert_with(
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &Options::new().stylesheet_content("body { color: red; }"),
        );
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./my-theme.css\">"));
        assert!(!html.contains("<style>"));
    }

    // Directly exercise the `normalize_web_path` port against Asciidoctor's
    // documented behavior for the stylesheet link.
    #[test]
    fn normalize_web_path_matches_asciidoctor() {
        use super::normalize_web_path;

        // A bare relative stylesheet gains a `./` prefix.
        assert_eq!(normalize_web_path("custom.css", ""), "./custom.css");

        // An explicit `./` is preserved (not doubled).
        assert_eq!(normalize_web_path("./custom.css", ""), "./custom.css");

        // A relative directory in the stylesheet value is kept.
        assert_eq!(
            normalize_web_path("stylesheets/custom.css", ""),
            "./stylesheets/custom.css"
        );

        // `stylesdir` is mirrored into the linked path.
        assert_eq!(
            normalize_web_path("custom.css", "./stylesheets"),
            "./stylesheets/custom.css"
        );

        // A trailing separator on `stylesdir` does not double up.
        assert_eq!(normalize_web_path("custom.css", "css/"), "./css/custom.css");

        // A `..` segment is collapsed against the styles directory.
        assert_eq!(normalize_web_path("../custom.css", "css"), "./custom.css");

        // A relative path that climbs out is a complete reference: it keeps its
        // leading `..` and gains no `./` prefix.
        assert_eq!(
            normalize_web_path("../shared/theme.css", ""),
            "../shared/theme.css"
        );

        // A `..` at the web root has nowhere to climb, so it is dropped.
        assert_eq!(normalize_web_path("/../secret.css", ""), "/secret.css");

        // A protocol-relative `//host/…` reference keeps its authority `//`
        // rather than collapsing to a single `/` (matches Asciidoctor 2.0.26).
        assert_eq!(
            normalize_web_path("//cdn.example.com/theme.css", ""),
            "//cdn.example.com/theme.css"
        );

        // Asciidoctor's `web_path` treats the segment after `//` as an ordinary
        // path segment, not an RFC-3986 authority: a `..` deeper in the path
        // pops the segment before it and keeps the host, but a `..` right after
        // the authority pops the host itself. We match Asciidoctor 2.0.26, which
        // emits `//cdn.example.com/theme.css` and `//theme.css` respectively.
        assert_eq!(
            normalize_web_path("//cdn.example.com/a/../theme.css", ""),
            "//cdn.example.com/theme.css"
        );
        assert_eq!(
            normalize_web_path("//cdn.example.com/../theme.css", ""),
            "//theme.css"
        );

        // A URI or an absolute path is a complete reference already.
        assert_eq!(
            normalize_web_path("file:///home/user/custom.css", "ignored"),
            "file:///home/user/custom.css"
        );
        assert_eq!(
            normalize_web_path("https://cdn.example/custom.css", ""),
            "https://cdn.example/custom.css"
        );
        assert_eq!(
            normalize_web_path("/abs/custom.css", "css"),
            "/abs/custom.css"
        );
    }

    /// Converts `source` with the given files (name → content) written to a
    /// fresh temp directory, under an embedding safe mode with a primary file
    /// of `mydoc.adoc` in that directory. This exercises the disk-read
    /// embedding path: a custom `stylesheet` is resolved and read from that
    /// directory.
    ///
    /// `tag` names the temp directory so concurrent tests do not collide.
    fn with_files(tag: &str, source: &str, files: &[(&str, &str)]) -> String {
        let dir =
            std::env::temp_dir().join(format!("adoc-render-css-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create scratch subdir");
            }
            std::fs::write(path, content).expect("write scratch file");
        }

        let html = convert_with(
            source,
            &Options::new()
                .safe_mode(SafeMode::Unsafe)
                .input_file(dir.join("mydoc.adoc")),
        );

        let _ = std::fs::remove_dir_all(&dir);
        html
    }

    // Under an embedding safe mode with a base directory, a custom stylesheet is
    // read from disk and embedded — the `adoc` default and the API's file path.
    #[test]
    fn custom_stylesheet_is_read_from_disk_and_embedded() {
        let html = with_files(
            "embed",
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &[("my-theme.css", "body { color: #ff0000; }\n")],
        );
        assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));

        // A custom stylesheet still gets neither the default CSS nor web fonts.
        assert!(!html.contains("/*! Asciidoctor default stylesheet"));
        assert!(!html.contains("fonts.googleapis.com"));
    }

    // `stylesdir` relocates the on-disk lookup, just as it does the linked path.
    #[test]
    fn custom_stylesheet_read_honors_stylesdir() {
        let html = with_files(
            "stylesdir",
            "= Doc\n:stylesheet: theme.css\n:stylesdir: css\n\nBody.",
            &[("css/theme.css", ".from-subdir { color: green; }\n")],
        );
        assert!(html.contains("<style>\n.from-subdir { color: green; }\n</style>"));
    }

    // Unsetting `stylesdir` (`:stylesdir!:`) drops the parser's default styles
    // directory (`.`), so the stylesheet resolves under its bare name against
    // the base directory.
    #[test]
    fn custom_stylesheet_read_with_stylesdir_unset() {
        let html = with_files(
            "no-stylesdir",
            "= Doc\n:stylesheet: theme.css\n:stylesdir!:\n\nBody.",
            &[("theme.css", ".bare { color: blue; }\n")],
        );
        assert!(html.contains("<style>\n.bare { color: blue; }\n</style>"));
    }

    // A caller-supplied `stylesheet_content` wins over the file on disk.
    #[test]
    fn supplied_content_beats_the_file_on_disk() {
        let dir =
            std::env::temp_dir().join(format!("adoc-render-css-{}-supplied", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        std::fs::write(dir.join("my-theme.css"), "body { color: black; }\n").expect("write css");

        let html = convert_with(
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Unsafe)
                .input_file(dir.join("mydoc.adoc"))
                .stylesheet_content("body { color: supplied; }"),
        );
        let _ = std::fs::remove_dir_all(&dir);

        assert!(html.contains("<style>\nbody { color: supplied; }\n</style>"));
        assert!(!html.contains("color: black"));
    }

    // A missing stylesheet file leaves the block out rather than embedding an
    // empty or fabricated one.
    #[test]
    fn a_missing_stylesheet_file_emits_no_style_block() {
        let html = with_files(
            "missing",
            "= Doc\n:stylesheet: absent.css\n\nBody.",
            &[("unrelated.css", "ignored")],
        );
        assert!(!html.contains("<style>"));
        assert!(!html.contains("<link rel=\"stylesheet\""));
    }

    // Without a base directory (plain `convert`, no input file), an embedded
    // custom stylesheet has no source, so its block is omitted.
    #[test]
    fn no_base_directory_means_no_embedded_custom_stylesheet() {
        let html = convert_with(
            "= Doc\n:stylesheet: my-theme.css\n\nBody.",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(!html.contains("<style>"));
    }

    // Docinfo splices caller-supplied content into three fixed positions of the
    // output: the bottom of the `<head>`, before the header `<div>`, and after
    // the footer `<div>`. The parser resolves which files apply (per the
    // `docinfo` attribute) and applies `docinfosubs`; the renderer only places
    // the result.

    #[test]
    fn head_docinfo_is_appended_to_the_bottom_of_the_head() {
        let html = with_docinfo(
            "head",
            "= Doc\n:docinfo: shared\n\nBody.",
            &[("docinfo.html", "<meta name=\"x\" content=\"y\">")],
        );

        // The head docinfo sits below the stylesheet block and just above the
        // closing `</head>`.
        assert!(html.contains("<meta name=\"x\" content=\"y\">\n</head>"));

        let style = html
            .find("<style>")
            .or_else(|| html.find("./asciidoctor.css"));

        let docinfo = html.find("<meta name=\"x\"").expect("head docinfo");
        let head_end = html.find("</head>").expect("head end");
        assert!(style.expect("stylesheet") < docinfo && docinfo < head_end);
    }

    #[test]
    fn header_docinfo_is_inserted_before_the_header_div() {
        let html = with_docinfo(
            "header",
            "= Doc\n:docinfo: shared\n\nBody.",
            &[("docinfo-header.html", "<div class=\"banner\">Hi</div>")],
        );

        assert!(html.contains("<div class=\"banner\">Hi</div>\n<div id=\"header\">"));
    }

    #[test]
    fn footer_docinfo_is_inserted_after_the_footer_div() {
        let html = with_docinfo(
            "footer",
            "= Doc\n:docinfo: shared\n\nBody.",
            &[("docinfo-footer.html", "<p>bye</p>")],
        );

        assert!(html.contains("</div>\n<p>bye</p>\n</body>"));
    }

    #[test]
    fn header_docinfo_survives_noheader_and_footer_docinfo_survives_nofooter() {
        // Docinfo header/footer are emitted whether or not the built-in header
        // and footer are suppressed — this is what lets docinfo replace them.
        let html = with_docinfo(
            "suppressed",
            "= Doc\n:docinfo: shared\n:noheader:\n:nofooter:\n\nBody.",
            &[
                ("docinfo-header.html", "<div class=\"banner\">Hi</div>"),
                ("docinfo-footer.html", "<p>bye</p>"),
            ],
        );

        assert!(!html.contains("<div id=\"header\">"));
        assert!(!html.contains("<div id=\"footer\">"));
        assert!(html.contains("<div class=\"banner\">Hi</div>"));
        assert!(html.contains("<p>bye</p>"));
    }

    #[test]
    fn shared_docinfo_is_placed_before_private_docinfo() {
        // With both scopes enabled, the shared file's content precedes the
        // private file's, matching Asciidoctor's concatenation order.
        let html = with_docinfo(
            "scopes",
            "= Doc\n:docinfo: shared,private\n\nBody.",
            &[
                ("docinfo.html", "<meta name=\"shared\">"),
                ("mydoc-docinfo.html", "<meta name=\"private\">"),
            ],
        );

        let shared = html.find("name=\"shared\"").expect("shared docinfo");
        let private = html.find("name=\"private\"").expect("private docinfo");
        assert!(shared < private);
    }

    #[test]
    fn docinfosubs_resolves_attribute_references_by_default() {
        // With `docinfosubs` at its implied default (`attributes`), attribute
        // references in the docinfo file are resolved.
        let html = with_docinfo(
            "subs",
            "= Doc\n:docinfo: shared\n:project: Widgets\n\nBody.",
            &[("docinfo.html", "<meta name=\"app\" content=\"{project}\">")],
        );

        assert!(html.contains("<meta name=\"app\" content=\"Widgets\">"));
    }

    #[test]
    fn no_base_directory_means_no_docinfo() {
        // With neither a base directory nor a primary file, no docinfo handler
        // is installed, so the `docinfo` attribute has no effect. `Safe` (not
        // `Server`) keeps the document's `:docinfo:` in force, so this isolates
        // the "no handler" path rather than the safe-mode docinfo lock.
        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new().safe_mode(SafeMode::Safe),
        );

        assert!(html.contains("</head>"));

        // Nothing spliced: head still flows stylesheet → `</head>`.
        assert!(!html.contains("<meta name=\"x\""));
    }
}
