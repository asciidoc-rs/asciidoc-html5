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
//! concern. [`Renderer::block`] is the dispatch point: it drops comment blocks
//! (see [`renders_nothing`]), then matches on the [`Block`] variant (and, for
//! delimited blocks, on [`IsBlock::resolved_context`]) and delegates. Compound
//! blocks recurse back into [`Renderer::blocks`] over their
//! [`FindBlocks::child_blocks`], so the same machinery handles arbitrary
//! nesting.
//!
//! This is a *baseline*: the constructs wired up below (the document skeleton,
//! header, paragraphs, sections, the preamble, verbatim blocks, and thematic
//! and page breaks) exercise every mechanism the full renderer needs.
//! Everything else falls through [`Renderer::unsupported`], which emits a
//! visible HTML comment rather than guessing — so output stays well-formed and
//! coverage gaps are obvious. Adding a construct means adding one arm and one
//! `render_*` method.

use std::{borrow::Cow, path::PathBuf};

use asciidoc_parser::{
    attributes::Attrlist,
    blocks::{
        AdmonitionBlock, Block, Break, BreakType, ColumnStyle, CompoundDelimitedContext,
        ContentModel, FindBlocks, Frame, Grid, HorizontalAlignment, IsBlock, ListBlock, ListItem,
        ListItemMarker, ListType, MediaBlock, MediaType, QuoteBlock, QuoteType, SectionBlock,
        SectionType, SimpleBlockStyle, Stripes, TableBlock, TableCell, TableCellContent,
        TableColumn, TableRow, TocBlock, VerticalAlignment,
    },
    content::{SubstitutionGroup, SubstitutionStep},
    document::{DocinfoLocation, Footnote, Header, InterpretedValue, TocMode},
    Document, HasSpan, Parser, SafeMode,
};

use crate::{
    html::{class_attribute, escape_attribute, id_attribute},
    svg_file_handler,
};

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

/// The `highlight.js` release Asciidoctor v2.0.26 links from the CDN
/// (`Asciidoctor::HIGHLIGHT_JS_VERSION`).
const HIGHLIGHT_JS_VERSION: &str = "9.18.3";

/// The Font Awesome release Asciidoctor v2.0.26 links from the CDN for
/// font-based icons (`Asciidoctor::FONT_AWESOME_VERSION`).
const FONT_AWESOME_VERSION: &str = "4.7.0";

/// The active *client-side* syntax highlighter, resolved from the
/// `source-highlighter` document attribute.
///
/// Only the two client-side highlighters are modelled. They perform no
/// server-side tokenizing, so honoring them is purely a matter of the CSS
/// classes on the source block's `<pre>`/`<code>` plus a CDN `<link>` in the
/// `<head>` and a `<script>` before `</body>` — no new dependency, matching the
/// library's "depend only on `asciidoc-parser`" constraint. The server-side
/// highlighters (`coderay`, `pygments`, `rouge`), which emit tokenized `<span>`
/// markup, are **not planned** – reproducing their tokenizer output would need
/// an in-process highlighter that constraint forbids, or a subprocess – so a
/// source block that requests one keeps its default unhighlighted shape here.
#[derive(Clone, Copy, PartialEq)]
enum Highlighter {
    /// `:source-highlighter: highlightjs` (also `highlight.js`).
    HighlightJs,

    /// `:source-highlighter: prettify`.
    Prettify,
}

impl Highlighter {
    /// Resolves the active client-side highlighter from the document's
    /// `source-highlighter` attribute, or `None` when unset or set to a
    /// highlighter this crate does not render client-side (a server-side one,
    /// or an unknown name).
    ///
    /// Asciidoctor's safe-mode gating of a *document-set* `source-highlighter`
    /// (locked to unset at `Server` and above, so an untrusted document cannot
    /// enable a highlighter and steer its asset origin) is applied upstream in
    /// [`Options::apply`](crate::Options), so by the time this reads the
    /// resolved attribute a disallowed value has already been dropped.
    fn from_document(document: &Document<'_>) -> Option<Self> {
        match attribute_str(document, "source-highlighter").as_deref() {
            Some("highlightjs" | "highlight.js") => Some(Self::HighlightJs),
            Some("prettify") => Some(Self::Prettify),
            _ => None,
        }
    }
}

/// Whether `block` is one Asciidoctor renders to nothing, so the renderer emits
/// no output for it at all.
///
/// This is how comments are dropped. `asciidoc-parser` keeps them in the parse
/// tree (so other tools can inspect them) and leaves it to the backend to
/// discard them, matching Asciidoctor. Three shapes reach the renderer:
///
/// - the `////` delimited comment block and the `[comment]` open block, which
///   the parser resolves to the `comment` context;
/// - a `[comment]`-styled paragraph, whose declared block style is `comment`
///   (its resolved context is still `paragraph`, so it is matched by style);
/// - a paragraph made up entirely of `//` line comments — the parser retains it
///   with its comment lines stripped to empty content, so it is matched by
///   [`is_line_comment_paragraph`] on the source and dropped, matching
///   Asciidoctor, which emits no block for it.
///
/// The line-comment case keys on the *source* rather than on the rendered
/// content being empty: a paragraph whose content merely *substituted* to
/// nothing — an empty inline passthrough such as `pass:[]` or `$$$$` — is not a
/// comment, so it is kept and rendered as an empty `<p></p>`, matching
/// Asciidoctor.
fn renders_nothing(block: &Block<'_>) -> bool {
    if block.resolved_context().as_ref() == "comment" || block.declared_style() == Some("comment") {
        return true;
    }

    // A document-attribute entry (`:name: value`) that survives into the block
    // stream — as happens inside an AsciiDoc table cell's nested document — sets
    // an attribute and renders nothing, matching Asciidoctor.
    if block.resolved_context().as_ref() == "attribute" {
        return true;
    }

    // A paragraph that is nothing but `//` line comments survives parsing with
    // empty content; Asciidoctor emits no block for it, so drop it. The
    // empty-content guard keeps this from ever dropping a paragraph that
    // actually renders something, while the source check keeps a paragraph whose
    // content merely substituted to nothing (an empty passthrough) — which is
    // not a comment — so it renders as `<p></p>`.
    matches!(block, Block::Simple(simple) if simple.style() == SimpleBlockStyle::Paragraph)
        && block
            .rendered_content()
            .unwrap_or_default()
            .trim()
            .is_empty()
        && is_line_comment_paragraph(block.span().data())
}

/// Whether a `----` listing block is source-styled — so it renders through
/// [`source`](Renderer::source) as `<pre class="highlight"><code …>` rather
/// than a plain listing.
///
/// This mirrors Asciidoctor's listing/source dispatch (`parser.rb`, the
/// `when :listing, :source` arm): a listing becomes a source block when it
/// either declares the `source` style explicitly (`[source,ruby]`) or uses the
/// comma shorthand (`[,ruby]`) — an empty block style with a language in the
/// second positional attribute. Asciidoctor promotes a bare listing to
/// `source` only when its first positional (the block style) is absent and its
/// second positional (the language) is present, so `[ruby]` (style `ruby`) or a
/// plain `----` stays a listing.
///
/// A Markdown-style fenced code block (```` ``` ````) is also always
/// source-styled — see [`opens_with_backtick_fence`].
fn is_source_listing(block: &Block<'_>) -> bool {
    if block.declared_style() == Some("source") {
        return true;
    }

    // A fenced code block is source-styled even without a language, which the
    // parser does not record for a bare fence.
    if opens_with_backtick_fence(block) {
        return true;
    }

    // The `[,ruby]` shorthand: no declared block style, but a language sits in
    // the second positional attribute (Asciidoctor's `attributes[2]`).
    block.declared_style().is_none()
        && block
            .attrlist()
            .and_then(|attrlist| attrlist.nth_attribute(2))
            .is_some()
}

/// Whether a `listing` block originated from a Markdown-style fenced code block
/// (```` ``` ````). Asciidoctor's parser gives every fenced code block the
/// `source` style, even one that declares no language; `asciidoc-parser`
/// records the `source` style only when the fence names a language (````
/// ```ruby ````), leaving a bare fence styleless. The renderer therefore
/// recovers the promotion from the block's raw span, whose first line is the
/// opening delimiter.
///
/// The fence delimiter is exactly three backticks (matching Asciidoctor's
/// `FencedCodeRx`), optionally followed by a language on the same line; a run
/// of four or more backticks is not a fence.
fn opens_with_backtick_fence(block: &Block<'_>) -> bool {
    let first_line = block.span().data().lines().next().unwrap_or_default();
    first_line.starts_with("```") && first_line.as_bytes().get(3) != Some(&b'`')
}

/// Whether a `----` listing is rendered as a source block, given the document's
/// `source-language` attribute.
///
/// A listing is already a source block when it declares the `source` style or
/// uses the `[,lang]` comma shorthand ([`is_source_listing`]). Beyond that,
/// when `source-language` is set, Asciidoctor's parser promotes a *bare*
/// listing — one with no declared block style and no language of its own — to a
/// source block, using `source-language` as the language. A listing that
/// declares any style (e.g. `[listing]`) is never promoted.
fn is_promoted_source_listing(block: &Block<'_>, source_language: Option<&str>) -> bool {
    is_source_listing(block)
        || (source_language.is_some()
            && block.declared_style().is_none()
            && block
                .attrlist()
                .and_then(|attrlist| attrlist.nth_attribute(2))
                .is_none())
}

/// Whether every non-blank line of a paragraph's `source` is an AsciiDoc line
/// comment — a line beginning (at column 0) with `//` not immediately followed
/// by another `/` (which would start a `////` comment-block delimiter),
/// mirroring Asciidoctor's `LineCommentRx` (`^\/\/(?=[^\/]|$)`). A paragraph of
/// only such lines carries no renderable content (the parser strips them), so
/// the renderer drops it; a paragraph with any other line — including one whose
/// content merely substitutes to nothing, like `pass:[]` — is kept. A source
/// with no non-blank line is not treated as a comment paragraph.
fn is_line_comment_paragraph(source: &str) -> bool {
    let mut saw_line = false;
    for line in source.lines() {
        // A comment marker must sit at column 0; only trailing whitespace (a
        // stray `\r` or spaces) is ignored.
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }

        saw_line = true;
        let is_comment = line
            .strip_prefix("//")
            .is_some_and(|rest| !rest.starts_with('/'));
        if !is_comment {
            return false;
        }
    }

    saw_line
}

/// Reads a document attribute as an explicit string value, if it has one.
/// `Set`/`Unset`/absent all yield `None` (use `is_attribute_set` for booleans).
pub(crate) fn attribute_str(document: &Document<'_>, name: &str) -> Option<String> {
    match document.attribute_value(name) {
        InterpretedValue::Value(value) => Some(value),
        InterpretedValue::Set | InterpretedValue::Unset => None,
    }
}

/// The ` style="max-width: <value>;"` fragment Asciidoctor adds to each of the
/// standalone layout's container divs (`#header`, `#content`, `#footnotes`,
/// `#footer`) when the `max-width` attribute is set, or an empty string when it
/// is not. The value is emitted whenever the attribute is *present* — matching
/// Asciidoctor's `node.attr? 'max-width'` gate, which treats a bare
/// `:max-width:` (no value) as present, yielding an empty length.
fn max_width_style(document: &Document<'_>) -> String {
    match document.attribute_value("max-width") {
        InterpretedValue::Value(value) => {
            format!(" style=\"max-width: {};\"", escape_attribute(&value))
        }

        InterpretedValue::Set => " style=\"max-width: ;\"".to_string(),
        InterpretedValue::Unset => String::new(),
    }
}

/// Asciidoctor's default `alt` for an image whose macro carries none: the
/// target's basename (its final path segment, without extension) with each `_`
/// and `-` turned into a space. Mirrors the `basename(target.tr('_-', ' '))`
/// derivation the parser uses for inline images.
fn default_alt(target: &str) -> String {
    let spaced = target.replace(['_', '-'], " ");

    let last_segment = spaced.rsplit(['/', '\\']).next().unwrap_or(&spaced);

    // Drop the extension (the text after the final `.`), matching Rust's
    // `Path::file_stem`: a leading-dot-only name keeps its text.
    match last_segment.rfind('.') {
        Some(dot) if dot > 0 => last_segment[..dot].to_string(),
        _ => last_segment.to_string(),
    }
}

/// The substituted `alt` text for a block image, matching Asciidoctor's
/// `AbstractBlock#alt`. An *explicit* alt (from the macro, or the block
/// attribute line's style) has both the special-character and replacement
/// substitutions applied, so `A tiger's "roar"` becomes
/// `A tiger&#8217;s "roar"`; an auto-generated alt (derived from the target's
/// basename) has only the special-character substitution applied. The `"` in
/// the result is left as-is here — the caller attribute-encodes it when the
/// value lands in an `alt="…"` attribute (Asciidoctor's
/// `encode_attribute_value`), but leaves it raw inside a `<span class="alt">`.
fn subbed_alt(explicit: Option<&str>, target: &str) -> String {
    let parser = Parser::default();

    match explicit {
        Some(text) => {
            let group = SubstitutionGroup::Custom(vec![
                SubstitutionStep::SpecialCharacters,
                SubstitutionStep::CharacterReplacements,
            ]);

            parser.apply_substitutions(text, &group)
        }

        None => {
            let group = SubstitutionGroup::Custom(vec![SubstitutionStep::SpecialCharacters]);

            parser.apply_substitutions(&default_alt(target), &group)
        }
    }
}

/// The ` target="…"`/` rel="…"` fragment Asciidoctor's
/// `append_link_constraint_attrs` adds to an image link: a `window` sets
/// `target` (and, for `_blank` or an explicit `noopener` option, `rel`), while
/// a `nofollow` option folds into `rel`.
fn link_constraint_attrs(window: Option<&str>, nofollow: bool, noopener: bool) -> String {
    let rel = if nofollow { Some("nofollow") } else { None };

    match window {
        Some(window) => {
            let rel_noopener = if window == "_blank" || noopener {
                match rel {
                    Some(rel) => format!(" rel=\"{rel} noopener\""),
                    None => " rel=\"noopener\"".to_string(),
                }
            } else {
                String::new()
            };

            format!(" target=\"{}\"{rel_noopener}", escape_attribute(window))
        }

        None => match rel {
            Some(rel) => format!(" rel=\"{rel}\""),
            None => String::new(),
        },
    }
}

/// The CDN base URL Asciidoctor builds its asset links from —
/// `{scheme}//cdnjs.cloudflare.com/ajax/libs`. The scheme comes from
/// `asset-uri-scheme` (Asciidoctor's `attr 'asset-uri-scheme', 'https'`):
/// absent or explicitly unset falls back to `https`, while a bare
/// `:asset-uri-scheme:` (or an explicit empty value) yields a protocol-relative
/// `//…` URL.
fn cdn_base_url(document: &Document<'_>) -> String {
    let scheme = match document.attribute_value("asset-uri-scheme") {
        InterpretedValue::Value(scheme) if !scheme.is_empty() => format!("{scheme}:"),
        InterpretedValue::Unset => "https:".to_string(),

        // A bare `:asset-uri-scheme:` (`Set`) or an explicit empty value: the
        // scheme is empty, so the URL is protocol-relative.
        _ => String::new(),
    };

    format!("{scheme}//cdnjs.cloudflare.com/ajax/libs")
}

/// The file extension of image-mode admonition icons — the `icontype`
/// attribute, defaulting to `png`.
///
/// Asciidoctor derives this in `Document#initialize`: when the `icons`
/// attribute carries a value that is neither empty, `font`, nor `image`, and no
/// explicit `icontype` is set, that value becomes the `icontype` (and `icons`
/// is normalized to image mode). So `:icons: jpg` renders `tip.jpg`, while an
/// explicit `:icontype:` always wins. `asciidoc-parser` does not perform this
/// normalization, so the renderer replicates it here.
fn icontype(document: &Document<'_>) -> String {
    if !document.has_attribute("icontype") {
        if let Some(value) = attribute_str(document, "icons") {
            if !value.is_empty() && value != "font" && value != "image" {
                return value;
            }
        }
    }

    attribute_str(document, "icontype").unwrap_or_else(|| "png".to_string())
}

/// The base directory `highlightjs` assets load from: the `highlightjsdir`
/// attribute, or `{cdn}/highlight.js/{version}` by default.
fn highlightjs_dir(document: &Document<'_>) -> String {
    attribute_str(document, "highlightjsdir").map_or_else(
        || {
            format!(
                "{}/highlight.js/{HIGHLIGHT_JS_VERSION}",
                cdn_base_url(document)
            )
        },
        |dir| escape_attribute(&dir),
    )
}

/// The base directory `prettify` assets load from: the `prettifydir`
/// attribute, or `{cdn}/prettify/r298` by default.
fn prettify_dir(document: &Document<'_>) -> String {
    attribute_str(document, "prettifydir").map_or_else(
        || format!("{}/prettify/r298", cdn_base_url(document)),
        |dir| escape_attribute(&dir),
    )
}

/// The byte length of the leading run of supplemental inline anchors
/// (`<a id="…"></a>`) at the start of a rendered heading title, or `0` when it
/// does not begin with one. Mirrors Asciidoctor's `LeadingAnchorsRx`
/// (`^(?:<a id="[^"]+"></a>)+`): the id must be non-empty, and anchors are
/// consumed greedily.
fn leading_anchors_len(title: &str) -> usize {
    let mut rest = title;
    let mut consumed = 0;

    while let Some(after_open) = rest.strip_prefix("<a id=\"") {
        // The id runs up to the next `"`; `[^"]+` requires at least one char.
        let Some(quote) = after_open.find('"') else {
            break;
        };

        if quote == 0 {
            break;
        }

        let Some(after_anchor) = after_open[quote + 1..].strip_prefix("></a>") else {
            break;
        };

        consumed = title.len() - after_anchor.len();
        rest = after_anchor;
    }

    consumed
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

    // A URI styles directory joins into a URI href
    // (`https://cdn/css` + `asciidoctor.css` -> `https://cdn/css/asciidoctor.css`)
    // and is returned as-is rather than run through `web_normalize`, which would
    // collapse the `//` in the scheme into `./https:/…`. This mirrors
    // Asciidoctor's `web_path`, which lifts the URI prefix off before
    // normalizing and restores it after. The resulting non-`./` path is also
    // what makes `copycss` skip it — a URI stylesheet has no local file to copy,
    // matching Asciidoctor's URI-`stylesdir` gate. A relative `stylesheet` joins
    // under the URI; an absolute one still ignores `stylesdir` (handled below).
    if !dir.is_empty() && !sheet.starts_with('/') && looks_like_uri(&dir) {
        return format!("{}/{sheet}", dir.trim_end_matches('/'));
    }

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

/// Whether the document's safe mode is below `Secure`, the threshold under
/// which Asciidoctor enables the security-sensitive SVG `interactive`/`inline`
/// image referencing (embedding a live `<object>`, or the file's contents). At
/// `Secure` or above — including the API default — an SVG renders as a plain
/// `<img>`, matching Asciidoctor.
///
/// The parser seeds the `safe-mode-level` intrinsic attribute on every
/// document; a document with none (or an unparseable level) is treated as
/// `Secure`.
fn safe_mode_below_secure(document: &Document<'_>) -> bool {
    attribute_str(document, "safe-mode-level")
        .and_then(|level| level.parse::<u32>().ok())
        .is_some_and(|level| level < SafeMode::Secure as u32)
}

/// Builds the URI reference a media block's `target` resolves to, mirroring
/// Asciidoctor's `AbstractNode#media_uri` (whose asset directory key is
/// `imagesdir`).
///
/// A URI target is preserved verbatim, with only its spaces percent-encoded
/// (Asciidoctor's `encode_spaces_in_uri`). Otherwise the target is resolved as
/// a web path relative to `imagesdir` — a direct port of
/// `PathResolver#web_path`: the target is joined onto a non-empty `imagesdir`
/// (unless it is web-absolute, `/…`), its `.`/`..` segments are collapsed, and
/// any spaces are percent-encoded. Unlike [`normalize_web_path`], a bare
/// relative result is *not* prefixed with `./`, matching `web_path`.
///
/// As with this crate's other web-path helpers, backslashes are posixified
/// unconditionally (Asciidoctor's `posixify` only does so on Windows), keeping
/// the result deterministic across platforms — a divergence invisible to any
/// realistic media target.
fn media_uri(target: &str, imagesdir: &str) -> String {
    // `media_uri` preserves a URI target (Asciidoctor's default
    // `preserve_uri_target`), encoding only its spaces.
    if looks_like_uri(target) {
        return target.replace(' ', "%20");
    }

    // Posixify both (Asciidoctor works in forward-slash web paths), then join
    // the target onto `imagesdir` unless the target is itself web-absolute
    // (`/…`), which ignores the asset directory.
    let target = target.replace('\\', "/");
    let dir = imagesdir.replace('\\', "/");

    // A URI asset directory (e.g. an `imagesdir` set to `https://cdn/images`)
    // joins into a URI reference and is returned as-is rather than run through
    // the segment normalization below, which would collapse the `//` in the
    // scheme into `https:/…`. This mirrors Asciidoctor's `web_path`, which lifts
    // the URI prefix off before normalizing and restores it after. A relative
    // target joins under the URI; a web-absolute one still ignores it (below).
    if !dir.is_empty() && !target.starts_with('/') && looks_like_uri(&dir) {
        return format!("{}/{target}", dir.trim_end_matches('/')).replace(' ', "%20");
    }

    let joined = if dir.is_empty() || target.starts_with('/') {
        target
    } else {
        format!("{}/{}", dir.trim_end_matches('/'), target)
    };

    // Partition off the root the way `web_path` does: `/…` (web root) and `./…`
    // keep their prefix, and every other relative path has *no* root prefix.
    let (root, rest) = if let Some(rest) = joined.strip_prefix('/') {
        ("/", rest)
    } else if let Some(rest) = joined.strip_prefix("./") {
        ("./", rest)
    } else {
        ("", joined.as_str())
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

    // Spaces in the resolved path are percent-encoded, matching `web_path`.
    format!("{root}{}", segments.join("/")).replace(' ', "%20")
}

/// Whether `path`'s final segment carries a file extension, mirroring
/// Asciidoctor's `Helpers.extname?` (a `.` in the basename that is not its
/// leading character). Used by `icon_uri` to decide whether the `icontype`
/// extension must be appended to a per-block `icon` value.
fn has_file_extension(path: &str) -> bool {
    let basename = path.rsplit('/').next().unwrap_or(path);
    matches!(basename.rfind('.'), Some(index) if index > 0)
}

/// The file extension of `path` including the leading dot (e.g. `.png`), or
/// `None` when it has none — a direct port of Asciidoctor's `Helpers.extname`
/// with a `nil` fallback. The extension is the slice from the last `.` to the
/// end, unless that dot lies in a directory component (a `/` follows it), in
/// which case the path has no extension.
fn asset_extname(path: &str) -> Option<&str> {
    let dot = path.rfind('.')?;
    if path[dot..].contains('/') {
        None
    } else {
        Some(&path[dot..])
    }
}

/// The MIME type embedded in a `data:` URI for the image at `target`, mirroring
/// Asciidoctor's `generate_data_uri`: an `.svg` target is `image/svg+xml`; any
/// other extension yields `image/<ext>` (the extension verbatim, without the
/// dot, and *not* lowercased, matching Asciidoctor); a target with no extension
/// is `application/octet-stream`.
///
/// The type is derived *only* from the target's extension, deliberately
/// **not** from a `format` attribute — a faithful port of Asciidoctor's
/// `generate_data_uri`, whose MIME derivation reads `Helpers.extname
/// target_image` alone. So an extensionless target carrying `format=png` still
/// embeds as `application/octet-stream` under `data-uri`, matching Asciidoctor
/// 2.0.26 (verified against the oracle). `format` is honored where Asciidoctor
/// honors it — distinguishing an SVG target for the interactive `<object>`
/// referencing — just not here.
fn data_uri_mimetype(target: &str) -> String {
    match asset_extname(target) {
        Some(".svg") => "image/svg+xml".to_string(),
        Some(ext) => format!("image/{}", &ext[1..]),
        None => "application/octet-stream".to_string(),
    }
}

/// The class-attribute *value* (`"<base> <role>…"`) for a media block wrapper,
/// with each author-supplied role escaped — the inner text of `class="…"`.
fn class_list(base: &str, roles: &[&str]) -> String {
    let mut classes = base.to_string();
    for role in roles {
        classes.push(' ');
        classes.push_str(&escape_attribute(role));
    }
    classes
}

/// The boolean-attribute fragment ` <name>` when the block carries `<option>`,
/// else empty — Asciidoctor's `append_boolean_attribute` for the default
/// (non-XML) HTML5 backend, gated on a block option.
fn boolean_option(attrs: &Attrlist<'_>, option: &str, name: &str) -> String {
    if attrs.has_option(option) {
        format!(" {name}")
    } else {
        String::new()
    }
}

/// The ` controls` boolean attribute a `<video>`/`<audio>` carries unless the
/// `nocontrols` option suppresses it.
fn controls_option(attrs: &Attrlist<'_>) -> String {
    if attrs.has_option("nocontrols") {
        String::new()
    } else {
        " controls".to_string()
    }
}

/// A hosted-embed query fragment (`param`, an already-formed `&amp;key=value`)
/// when the block carries `<option>`, else empty.
fn flag_param(attrs: &Attrlist<'_>, option: &str, param: &str) -> String {
    if attrs.has_option(option) {
        param.to_string()
    } else {
        String::new()
    }
}

/// Joins hosted-embed query `params` into a query string, introducing the
/// first with `?` and the rest with `&amp;` (Asciidoctor's delimiter stack).
fn join_query(params: &[String]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&amp;"))
    }
}

/// The ` allowfullscreen` boolean attribute a hosted embed carries unless the
/// `nofullscreen` option suppresses it.
fn allowfullscreen_attr(attrs: &Attrlist<'_>) -> String {
    if attrs.has_option("nofullscreen") {
        String::new()
    } else {
        " allowfullscreen".to_string()
    }
}

/// The numbering style of an ordered list, matching Asciidoctor's `node.style`
/// for an olist: an explicit declared style from the block (`[loweralpha]`,
/// `[upperroman]`, …) wins, otherwise the style implied by the first item's
/// marker (`.` ⇒ arabic, `..` ⇒ loweralpha, …). Falls back to `arabic`, which a
/// bare `.` marker also yields.
///
/// Like Asciidoctor, a declared style is passed through verbatim even when it
/// is not one of the numbering keywords (`[foo]` ⇒ `class="olist foo"`); the
/// `<ol>` then carries no HTML `type`, since only the numbering keywords map to
/// one.
fn olist_style<'src>(block: &'src Block<'src>, list: &'src ListBlock<'src>) -> &'src str {
    block
        .declared_style()
        .or_else(|| list.marker_style())
        .unwrap_or("arabic")
}

/// Whether an ordered list's `<ol>` should carry an HTML `type` attribute for
/// its alphabetic or roman numbering style.
///
/// Asciidoctor emits `type` only when the numbering style is a *string* in its
/// model: a style set by a declared block attribute (`[loweralpha]`) or one
/// derived from a dot marker (`.`, `..`, …, whose level indexes
/// `ORDERED_LIST_STYLES` and is stored as a string). A style *inferred from an
/// alphabetic marker* (`A.`, `a.`) is stored as a Symbol, which its
/// `ORDERED_LIST_KEYWORDS` lookup (keyed by String) misses, so such a list
/// names the class but carries no `type`. We mirror that: a declared style, or
/// a first item marked with dots, earns the `type`; an alpha-marker-inferred
/// style does not.
fn olist_emits_type(block: &Block<'_>, list: &ListBlock<'_>) -> bool {
    if block.declared_style().is_some() {
        return true;
    }

    list.child_blocks()
        .find_map(as_list_item)
        .is_some_and(|item| matches!(item.list_item_marker(), ListItemMarker::Dots(_)))
}

/// One description-list entry: the terms sharing a single description (their
/// already-rendered `<dt>` text), paired with the list item that carries that
/// description — or `None` for a trailing run of terms with no description.
/// Mirrors Asciidoctor's `[terms, dd]` item pairs.
type DlistEntry<'src> = (Vec<String>, Option<&'src ListItem<'src>>);

/// Regroups a description list's per-term items into Asciidoctor's `[terms,
/// dd]` entries.
///
/// The parser emits one list item per `term::` line, so several terms sharing
/// one description arrive as term-only items (no child blocks) followed by the
/// item holding the description. This accumulates the term-only items and
/// flushes them, together with the next described item, into a single entry; a
/// trailing run of undescribed terms forms a final entry with no description.
fn dlist_entries<'src>(list: &'src ListBlock<'src>) -> Vec<DlistEntry<'src>> {
    let mut entries: Vec<DlistEntry<'src>> = Vec::new();
    let mut pending_terms: Vec<String> = Vec::new();

    // Every child of a description list is a `ListItem` whose marker is a
    // `DefinedTerm`, so the narrowing helpers never actually discard anything
    // here; `filter_map`/`extend` let the loop stay branch-free (the narrowing
    // itself lives in — and is covered through — the helpers).
    for list_item in list.child_blocks().filter_map(as_list_item) {
        pending_terms.extend(dlist_term_text(list_item));

        // A described item (one with child blocks) closes the entry, taking the
        // terms accumulated so far with it.
        if list_item.child_blocks().next().is_some() {
            entries.push((std::mem::take(&mut pending_terms), Some(list_item)));
        }
    }

    // Any terms left over had no description of their own.
    if !pending_terms.is_empty() {
        entries.push((pending_terms, None));
    }

    entries
}

/// Narrows a block to the list item it holds, or `None` for any other block.
/// A [`ListBlock`] only ever holds list items, so list rendering never sees the
/// `None` case; the narrowing lives here (rather than inline) so it can be
/// exercised directly.
fn as_list_item<'src>(block: &'src Block<'src>) -> Option<&'src ListItem<'src>> {
    match block {
        Block::ListItem(list_item) => Some(list_item),
        _ => None,
    }
}

/// The already-rendered term text of a description-list item (its `DefinedTerm`
/// marker's content, with inline substitutions applied), or `None` for a list
/// item carrying any other marker. A description list's items are always
/// `DefinedTerm`, so the `None` case never arises during rendering.
fn dlist_term_text(list_item: &ListItem<'_>) -> Option<String> {
    match list_item.list_item_marker() {
        ListItemMarker::DefinedTerm { term, .. } => Some(term.rendered().to_string()),
        _ => None,
    }
}

/// Whether a list-continuation (`+`) line separates a description item's term
/// from its first child block. Such a block was explicitly attached, so
/// Asciidoctor renders it as a block rather than folding it into the item's
/// principal text. The check reads the item's own source between the term and
/// the first child and looks for a line that is a bare `+`.
fn continuation_before_first_child(item: &ListItem<'_>, first: &Block<'_>) -> bool {
    let item_span = item.span();
    let item_src = item_span.data();

    // The first child lies within the item, so its offset is at or past the
    // item's; `saturating_sub` and the `min` keep the slice in bounds even if a
    // span were ever malformed.
    let start = item_span.byte_offset();
    let end = first.span().byte_offset();
    let between = &item_src[..end.saturating_sub(start).min(item_src.len())];

    // Skip the term line itself; a `+` alone on any later line is a
    // continuation that attached the first block.
    between.split('\n').skip(1).any(|line| line.trim() == "+")
}

/// The `labelwidth`/`itemwidth` value of a `horizontal` description list, with
/// one trailing `%` stripped (Asciidoctor's `chomp '%'`), or `None` when the
/// attribute is absent.
fn dlist_width(list: &ListBlock<'_>, name: &str) -> Option<String> {
    list.attrlist()
        .and_then(|attrlist| attrlist.named_attribute(name))
        .map(|attr| {
            let value = attr.value();
            value.strip_suffix('%').unwrap_or(value).to_string()
        })
}

/// A `<col>` element for a `horizontal` description list's `<colgroup>`,
/// carrying an inline percentage `width` style when one was given, or bare
/// otherwise, matching Asciidoctor's `style="width: N%;"`.
fn dlist_col(width: Option<&str>) -> String {
    match width {
        Some(width) => format!("<col style=\"width: {}%;\">", escape_attribute(width)),
        None => "<col>".to_string(),
    }
}

/// Parses the leading integer of `value`, matching Ruby's `String#to_i`: an
/// optional sign followed by ASCII digits, with any trailing text ignored, and
/// `0` when no digits lead the string. This is how Asciidoctor coerces the
/// `tabsize`, `indent`, and `source-indent` attributes.
fn ruby_to_i(value: &str) -> i64 {
    let value = value.trim_start();
    let (sign, digits) = match value.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, value.strip_prefix('+').unwrap_or(value)),
    };

    let magnitude: i64 = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .fold(0i64, |acc, b| {
            acc.saturating_mul(10).saturating_add((b - b'0') as i64)
        });

    sign * magnitude
}

/// The number of leading whitespace characters on `line` — Asciidoctor's
/// `line.length - line.lstrip.length`. Only the ASCII whitespace Ruby's
/// `String#lstrip` strips is counted; every such byte is one column, so the
/// count doubles as a byte offset into the line.
fn leading_whitespace_len(line: &str) -> usize {
    line.bytes()
        .take_while(|b| matches!(b, b' ' | b'\t' | b'\r' | 0x0b | 0x0c | 0))
        .count()
}

/// Expands the tabs in one verbatim line to spaces on `tab_size`-column tab
/// stops, a direct port of the tab-expansion arm of Asciidoctor's
/// `Parser.adjust_indentation!`. `full_tab_space` is `tab_size` spaces, reused
/// across lines by the caller.
fn expand_tabs(line: &str, tab_size: usize, full_tab_space: &str) -> String {
    if line.is_empty() || !line.contains('\t') {
        return line.to_string();
    }

    // A run of leading tabs expands directly to whole tab widths.
    let mut line = line.to_string();
    if line.starts_with('\t') {
        let leading_tabs = line.bytes().take_while(|&b| b == b'\t').count();
        line = format!(
            "{}{}",
            full_tab_space.repeat(leading_tabs),
            &line[leading_tabs..]
        );
    }

    // If that cleared every tab, the line is done; otherwise embedded tabs
    // remain for the per-stop loop below.
    if !line.contains('\t') {
        return line;
    }

    // Remaining tabs advance to the next tab stop, tracking how many spaces
    // have been added so each stop is measured against the output column.
    let mut spaces_added = 0usize;
    let mut result = String::new();
    for (idx, c) in line.chars().enumerate() {
        if c == '\t' {
            let offset = idx + spaces_added;
            if offset.is_multiple_of(tab_size) {
                spaces_added += tab_size - 1;
                result.push_str(full_tab_space);
            } else {
                let spaces = tab_size - (offset % tab_size);
                if spaces != 1 {
                    spaces_added += spaces - 1;
                }
                result.push_str(&" ".repeat(spaces));
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// The largest reindent margin (`indent`/`source-indent`) this crate will act
/// on. These values come from document-supplied attributes, so an absurd one
/// (e.g. `:source-indent: 999999999999`) would otherwise saturate to `i64::MAX`
/// and drive an unbounded space allocation, aborting the process on untrusted
/// input.
///
/// No real verbatim block needs anywhere near this much of a margin (100 spaces
/// is already well past plausible), so this is a deliberate divergence from
/// Asciidoctor (which bounds neither), guarding availability while leaving
/// every realistic document byte-identical.
const MAX_VERBATIM_INDENT: i64 = 100;

/// The largest `tabsize` this crate will act on — the number of spaces a single
/// tab expands to. Unlike a block margin, this is the width of *one* tab, and
/// no real document uses more than a handful (2/4/8, occasionally 16); a value
/// like `:tabsize: 100` — let alone `i64::MAX` — is nonsensical. Capping it
/// here both avoids the unbounded allocation and keeps tab expansion's
/// amplification (one tab → `tabsize` spaces) to a small, sane factor. As with
/// the margin cap this is a deliberate divergence from Asciidoctor, invisible
/// to any real document.
const MAX_TAB_SIZE: i64 = 16;

/// Restores the original leading indentation of an *implicit* literal
/// paragraph's content `lines` from its raw source span.
///
/// A literal paragraph detected from indentation has its content flattened by
/// the parser: it strips the first line's indent from every line before the
/// renderer sees it (via `rendered_content`). Asciidoctor instead removes only
/// the *common* (minimum) indent, so an over-indented first line keeps the
/// extra. Re-applying each source line's true leading whitespace here lets the
/// common-indent removal in [`adjust_indentation`] reproduce that (#168).
///
/// The block's raw span may carry leading metadata lines (a `.title`, an
/// `[[anchor]]`, or an `[attrlist]`) ahead of the content, so the content lines
/// are the trailing `lines.len()` lines of the span — verbatim substitutions
/// never add or remove lines, and an implicit literal paragraph holds no
/// interior blank lines, so the two align line for line. A raw span with fewer
/// lines than `lines` (which should not happen) leaves them untouched.
fn restore_literal_paragraph_indent(lines: &mut [String], raw_span: &str) {
    // `split_terminator` (not `split`) so a raw span that ends with a newline
    // does not yield a trailing empty element: the content lines are aligned as
    // a *suffix*, so an extra trailing entry would shift every pairing by one.
    let raw_lines: Vec<&str> = raw_span.split_terminator('\n').collect();
    if raw_lines.len() < lines.len() {
        return;
    }

    // Only the trailing lines of the raw span are content; skip any leading
    // metadata lines.
    let offset = raw_lines.len() - lines.len();
    for (line, raw) in lines.iter_mut().zip(&raw_lines[offset..]) {
        let indent = &raw[..leading_whitespace_len(raw)];
        if indent.is_empty() {
            continue;
        }

        // Replace the residual indentation the parser left with the raw line's
        // full leading whitespace.
        let body = &line[leading_whitespace_len(line)..];
        *line = format!("{indent}{body}");
    }
}

/// Whether `line` is a single-line comment the parser drops from a paragraph's
/// rendered content — `//` at column 0 not followed by a third `/` (which would
/// begin a `////` comment block), Asciidoctor's `LineCommentRx`. Such dropped
/// lines must be filtered out of the raw span before it is aligned with the
/// rendered lines, or the pairing shifts by one at every interior comment.
fn is_line_comment(line: &str) -> bool {
    line.starts_with("//") && !line.starts_with("///")
}

/// The common (minimum) leading-whitespace length shared by `lines`, or `None`
/// when any non-empty line is flush left (indent 0) or `lines` is empty — a
/// port of the block-indent scan in [`adjust_indentation`] /
/// `Parser.adjust_indentation!`. A `None` result means no indent is removed.
fn common_leading_indent(lines: &[&str]) -> Option<usize> {
    let mut block_indent: Option<usize> = None;
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let indent = leading_whitespace_len(line);
        if indent == 0 {
            return None;
        }

        block_indent = Some(block_indent.map_or(indent, |b| b.min(indent)));
    }

    block_indent
}

/// Restores the leading indentation of a list item's *principal* paragraph
/// `lines` from its raw source span, reproducing Asciidoctor's dedent.
///
/// `asciidoc-parser` rewrites the leading whitespace of a list item's wrapped
/// principal lines before the renderer sees it via `rendered_content`, and does
/// so inconsistently, so the correct indentation is recovered here from the raw
/// span instead. Asciidoctor keeps the item's inline marker/term text (its
/// first line) verbatim and runs `Parser.adjust_indentation!` over the *folded*
/// continuation lines that follow — removing their common (minimum) indent, or
/// nothing when any of them is flush left. When the item has **no** inline text
/// (a description-list term whose text folds up from a subsequent line), there
/// is no verbatim first line and the whole principal is the folded paragraph.
///
/// `has_inline_text` says which case applies: the caller sets it from whether
/// the principal's first source line coincides with the marker line (a flush-
/// left folded first line is otherwise indistinguishable from inline text).
///
/// The content lines are the trailing `lines.len()` lines of the span, once the
/// interior line comments the parser dropped are filtered out (see
/// [`is_line_comment`]) and any leading `.title`/`[[anchor]]`/`[attrlist]`
/// metadata is skipped; inline substitutions otherwise never add, drop, or
/// reindent lines, so the two align line for line. A raw span that, so aligned,
/// has fewer lines than `lines` (which should not happen) leaves them
/// untouched.
fn restore_list_principal_indent(lines: &mut [String], raw_span: &str, has_inline_text: bool) {
    // `split_terminator` (not `split`) so a raw span ending in a newline does
    // not yield a phantom trailing element: the content lines align as a
    // *suffix*, so an extra entry would shift every pairing by one. Line
    // comments the parser stripped from the rendered content are dropped here
    // too, keeping the pairing aligned.
    let raw_lines: Vec<&str> = raw_span
        .split_terminator('\n')
        .filter(|line| !is_line_comment(line))
        .collect();

    if raw_lines.len() < lines.len() {
        return;
    }

    // Only the trailing lines of the raw span are content; skip any leading
    // metadata lines.
    let offset = raw_lines.len() - lines.len();
    let raw = &raw_lines[offset..];

    // The inline marker/term text (kept verbatim) is the first line; the folded
    // paragraph adjust_indentation! applies to starts after it. With no inline
    // text every line is folded.
    let folded_start = usize::from(has_inline_text);

    // The common indent removed from the folded lines, or `None` when any is
    // flush left — `adjust_indentation!` at indent 0.
    let block_indent = common_leading_indent(raw.get(folded_start..).unwrap_or_default());

    for (i, (line, raw)) in lines.iter_mut().zip(raw).enumerate() {
        let raw_indent = leading_whitespace_len(raw);

        // The verbatim inline line keeps its indent; a folded line drops the
        // common indent from the front of its whitespace (clamped so an interior
        // blank line, which the block indent skips, cannot underflow).
        let start = if i < folded_start {
            0
        } else {
            block_indent.unwrap_or(0).min(raw_indent)
        };

        // Replace the parser's residual leading whitespace with the recovered
        // indentation.
        let body = &line[leading_whitespace_len(line)..];
        *line = format!("{}{body}", &raw[start..raw_indent]);
    }
}

/// The indent-corrected principal (first attached block) text of a list item,
/// the counterpart to Asciidoctor's `item.text`. It is the block's
/// `rendered_content` with a wrapped principal line's hanging indent restored
/// (see [`restore_list_principal_indent`]); indent recovery applies only to a
/// simple paragraph — the shape list principal text always takes — so any
/// other block is returned verbatim.
///
/// `item_line` is the source line of the item's marker (its `HasSpan` line):
/// the principal carries inline marker/term text exactly when its own first
/// line sits on that line, the signal `restore_list_principal_indent` needs to
/// tell inline text from a folded first line.
///
/// Only a *wrapped* (multi-line) principal can have lost indentation — a single
/// line has no continuation to reindent, so the parser's rendered content is
/// already correct. The single-line case (the overwhelming majority of list
/// items) and any non-paragraph block therefore borrow the content untouched,
/// keeping this allocation-free off the hot path; only a multi-line paragraph
/// pays for the split/restore/join.
fn list_principal_content<'src>(block: &'src Block<'src>, item_line: usize) -> Cow<'src, str> {
    let content = block.rendered_content().unwrap_or_default();

    if !content.contains('\n')
        || !matches!(block, Block::Simple(simple) if simple.style() == SimpleBlockStyle::Paragraph)
    {
        return Cow::Borrowed(content);
    }

    let has_inline_text = block.span().line() == item_line;
    let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();
    restore_list_principal_indent(&mut lines, block.span().data(), has_inline_text);
    Cow::Owned(lines.join("\n"))
}

/// Reindents a verbatim block's `lines` in place, a port of Asciidoctor's
/// `Parser.adjust_indentation!`: it expands tabs (when `tab_size` is positive
/// and a tab is present), then — unless `indent_size` is negative — removes the
/// common block indent and re-adds `indent_size` spaces of margin. Empty lines
/// are left untouched throughout.
fn adjust_indentation(lines: &mut [String], indent_size: i64, tab_size: i64) {
    if lines.is_empty() {
        return;
    }

    // Clamp the document-supplied sizes so they cannot drive an unbounded
    // allocation. The margin and a single tab's width are bounded separately
    // (see [`MAX_VERBATIM_INDENT`] and [`MAX_TAB_SIZE`]). `min` preserves a
    // negative `indent_size`, which is the "leave indentation as-is" sentinel.
    let indent_size = indent_size.min(MAX_VERBATIM_INDENT);
    let tab_size = tab_size.min(MAX_TAB_SIZE);

    if tab_size > 0 && lines.iter().any(|line| line.contains('\t')) {
        let full_tab_space = " ".repeat(tab_size as usize);
        for line in lines.iter_mut() {
            *line = expand_tabs(line, tab_size as usize, &full_tab_space);
        }
    }

    // A negative indent preserves the existing indentation.
    if indent_size < 0 {
        return;
    }

    // The block indent is the smallest indent over the non-empty lines; a line
    // flush against the margin (indent 0) means there is nothing to remove.
    let mut block_indent: Option<usize> = None;
    for line in lines.iter() {
        if line.is_empty() {
            continue;
        }

        let line_indent = leading_whitespace_len(line);
        if line_indent == 0 {
            block_indent = None;
            break;
        }

        block_indent = Some(match block_indent {
            Some(current) if current < line_indent => current,
            _ => line_indent,
        });
    }

    let margin = " ".repeat(indent_size.max(0) as usize);
    for line in lines.iter_mut() {
        if line.is_empty() {
            continue;
        }

        let body = match block_indent {
            Some(indent) => &line[indent..],
            None => line.as_str(),
        };
        *line = format!("{margin}{body}");
    }
}

/// Strips leading and trailing blank lines from a verbatim (or raw) block's
/// `lines`, matching the whitespace trimming in Asciidoctor's `Block#content`.
/// A line counts as blank when it holds only whitespace, and — as in
/// Asciidoctor — the trimming applies only once the block has more than one
/// line, so a lone (even blank) line is preserved.
fn strip_surrounding_blank_lines(lines: &mut Vec<String>) {
    if lines.len() < 2 {
        return;
    }

    while lines.first().is_some_and(|line| line.trim_end().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim_end().is_empty()) {
        lines.pop();
    }
}

/// Renders the table-of-contents block Asciidoctor's `html5` backend emits —
/// `<div id="toc" class="…"><div id="toctitle">…</div>{outline}</div>` — for a
/// document whose `toc` attribute enables one, or an empty string when the
/// document has no sections (the outline would be empty, and Asciidoctor emits
/// no TOC). The nested section list comes from
/// [`render_outline`](crate::outline::render_outline); `class` supplies the
/// `<div id="toc">` class (normally the resolved `toc-class`) and the title is
/// the resolved `toc-title`.
///
/// The caller decides *where* to insert this block from the document's
/// [`TocMode`] and which `class` to give it: a standalone document uses the
/// document's `toc-class` (which the side-column placements set to `toc2`),
/// while embeddable output always passes `toc`, since the side-column layout
/// isn't available there (matching Asciidoctor's `convert_embedded`, which
/// hardcodes the class). This function only builds the block.
fn render_toc(document: &Document<'_>, class: &str) -> String {
    let outline = crate::outline::render_outline(document, &crate::OutlineOptions::default());
    if outline.is_empty() {
        return String::new();
    }

    toc_container(class, document.toc_title(), &outline)
}

/// Wraps a pre-rendered `outline` in the header/preamble TOC container —
/// `<div id="toc" class="…"><div id="toctitle">…</div>{outline}</div>`. Shared
/// by [`render_toc`] (top-level document) and the AsciiDoc table cell's leading
/// TOC (its nested document), which builds its outline from a block slice.
///
/// The `class` is escaped defensively (a no-op for the `toc`/`toc2` defaults);
/// the `title` is emitted verbatim, matching Asciidoctor's `#{doc.attr
/// 'toc-title'}`.
fn toc_container(class: &str, title: &str, outline: &str) -> String {
    format!(
        "<div id=\"toc\" class=\"{}\">\n<div id=\"toctitle\">{title}</div>\n{outline}\n</div>",
        escape_attribute(class),
    )
}

/// The filesystem anchor for reading an SVG file to embed inline in a *block*
/// image (`image::…[opts=inline]`): the base directory the target resolves
/// against and the safe mode that confines the read.
///
/// This mirrors the include/docinfo handlers' anchoring (see
/// [`crate::svg_file_handler`]); *inline* images (`image:…`) are handled by the
/// parser through its own registered handler, so this is only for block images.
/// A `None` `svg_source` at [`render_document`] (the plain string
/// [`convert`](crate::convert), which has no base directory) leaves a block
/// `opts=inline` SVG unreadable, so it falls back to its alt text — matching
/// Asciidoctor when it cannot read the file.
#[derive(Clone, Debug)]
pub(crate) struct SvgSource {
    /// The base directory the (imagesdir-prefixed) target resolves against and,
    /// when jailed, the boundary the read may not cross.
    base_dir: PathBuf,

    /// The safe mode in force, which confines the read.
    safe: SafeMode,
}

impl SvgSource {
    /// Creates an SVG read anchor at `base_dir`, confined by `safe`.
    pub(crate) fn new(base_dir: PathBuf, safe: SafeMode) -> Self {
        Self { base_dir, safe }
    }
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
///
/// `svg_source` anchors the render-time filesystem reads: a block image's
/// inline-embedded SVG (`image::…[opts=inline]`) and the `data-uri` embedding
/// of images and image-mode icons. It carries the base directory those targets
/// resolve against and the safe mode confining the read. It is `None` for
/// callers with no base directory (the string-only
/// [`convert`](crate::convert)), which leaves a block `opts=inline` SVG
/// unreadable — so it falls back to its alt text — and a `data-uri` image
/// embedded as an empty data URI. See [`SvgSource`].
pub(crate) fn render_document<'a>(
    document: &'a Document<'a>,
    custom_stylesheet: Option<&'a str>,
    standalone: bool,
    svg_source: Option<SvgSource>,
) -> String {
    // Build the header/preamble TOC block once, up front — it is emitted at the
    // single site the placement selects. Only the placements that render a TOC
    // in the document flow relative to fixed structure
    // (`auto`/`left`/`right`/`top`/`bottom` near the top, `preamble` below the
    // preamble) produce one here; the `macro` placement renders at its `toc::[]`
    // block instead (see [`toc_macro`](Renderer::toc_macro)), and `disabled`
    // produces none.
    let toc_mode = document.toc_mode();

    // Choose the TOC's class once, so the block is built a single time. The
    // header/preamble placements use the document's `toc-class` (which the
    // side-column placements set to `toc2`). The one exception is the leading
    // TOC of *embeddable* output: the side-column layout isn't available there,
    // so it always uses the plain `toc` class, matching Asciidoctor's
    // `convert_embedded` (its `preamble` placement still honors `toc-class`, via
    // the same path as standalone output).
    let toc_class = if !standalone
        && matches!(
            toc_mode,
            TocMode::Auto | TocMode::Left | TocMode::Right | TocMode::Top | TocMode::Bottom
        ) {
        "toc"
    } else {
        document.toc_class()
    };

    let toc_html = match toc_mode {
        TocMode::Auto
        | TocMode::Left
        | TocMode::Right
        | TocMode::Top
        | TocMode::Bottom
        | TocMode::Preamble => render_toc(document, toc_class),

        TocMode::Disabled | TocMode::Macro => String::new(),
    };

    let mut renderer = Renderer {
        out: String::new(),
        document: Some(document),
        custom_stylesheet,
        standalone,
        toc_mode,
        toc_html,
        cell_toc: None,
        icons_set: document.is_attribute_set("icons"),
        icons_font: attribute_str(document, "icons").as_deref() == Some("font"),
        iconsdir: attribute_str(document, "iconsdir")
            .unwrap_or_else(|| "./images/icons".to_string()),
        icontype: icontype(document),
        imagesdir: attribute_str(document, "imagesdir").unwrap_or_default(),
        asset_uri_scheme: attribute_str(document, "asset-uri-scheme")
            .unwrap_or_else(|| "https".to_string()),
        doc_tabsize: ruby_to_i(&attribute_str(document, "tabsize").unwrap_or_default()),
        source_indent: attribute_str(document, "source-indent").map(|value| ruby_to_i(&value)),
        prewrap: document.is_attribute_set("prewrap"),
        source_highlighter: Highlighter::from_document(document),
        source_language: attribute_str(document, "source-language"),
        section_anchors: if !document.is_attribute_set("sectanchors") {
            SectionAnchors::None
        } else if attribute_str(document, "sectanchors").as_deref() == Some("after") {
            SectionAnchors::After
        } else {
            SectionAnchors::Before
        },
        sectlinks: document.is_attribute_set("sectlinks"),
        stem_type: resolve_stem_type(document),
        cellbgcolor: attribute_str(document, "cellbgcolor"),
        svg_below_secure: safe_mode_below_secure(document),
        data_uri: document.is_attribute_set("data-uri"),
        svg_source,
    };
    renderer.document(document);
    renderer.out
}

/// How the `sectanchors` document attribute decorates a non-discrete section
/// heading with its `<a class="anchor">` self-link.
#[derive(Clone, Copy, PartialEq)]
enum SectionAnchors {
    /// `sectanchors` is unset: no anchor is injected.
    None,

    /// `:sectanchors:` — the anchor is prepended before the title text.
    Before,

    /// `:sectanchors: after` — the anchor is appended after the title text.
    After,
}

/// The STEM notation a stem block renders in, selecting its block math
/// delimiter pair (Asciidoctor's `BLOCK_MATH_DELIMITERS`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum StemType {
    /// AsciiMath, delimited by `\$ … \$`.
    AsciiMath,

    /// LaTeX math, delimited by `\[ … \]`.
    LatexMath,
}

impl StemType {
    /// Maps a `stem`-attribute or block-style value to a STEM type, mirroring
    /// Asciidoctor's `STEM_TYPE_ALIASES` (`latexmath`/`latex`/`tex` select
    /// LaTeX; every other value, including an absent one, selects AsciiMath).
    fn from_value(value: &str) -> Self {
        match value {
            "latexmath" | "latex" | "tex" => Self::LatexMath,
            _ => Self::AsciiMath,
        }
    }

    /// The block math delimiter pair (`open`, `close`) this notation wraps its
    /// equation in.
    fn block_delimiters(self) -> (&'static str, &'static str) {
        match self {
            Self::AsciiMath => (r"\$", r"\$"),
            Self::LatexMath => (r"\[", r"\]"),
        }
    }
}

/// Resolves the document's default STEM type from its `stem` attribute, the
/// notation a bare `[stem]` block renders in.
fn resolve_stem_type(document: &Document<'_>) -> StemType {
    StemType::from_value(&attribute_str(document, "stem").unwrap_or_default())
}

/// Rewrites the internal breaks of a multi-line AsciiMath equation, closing and
/// reopening the delimiter pair around each `<br>` so each expression stands
/// alone. Mirrors Asciidoctor's `StemBreakRx` pass (`/
/// *\\\n(?:\\?\n)*|\n\n+/`): a blank-line run or trailing-backslash line
/// continuation becomes `close`, one `<br>` per extra line break, then `open`
/// again.
fn rewrite_asciimath_breaks(equation: &str, open: &str, close: &str) -> String {
    let bytes = equation.as_bytes();
    let mut out = String::with_capacity(equation.len());
    let mut i = 0;
    let mut copied = 0;

    while i < bytes.len() {
        if let Some(end) = match_stem_break(bytes, i) {
            out.push_str(&equation[copied..i]);

            let breaks = bytes[i..end].iter().filter(|&&b| b == b'\n').count();
            out.push_str(close);
            for _ in 1..breaks {
                out.push_str("\n<br>");
            }
            out.push('\n');
            out.push_str(open);

            i = end;
            copied = end;
        } else {
            i += 1;
        }
    }

    out.push_str(&equation[copied..]);
    out
}

/// Matches one `StemBreakRx` occurrence starting at `start`, returning the byte
/// index just past it, or `None` when neither alternative matches there. The
/// first alternative is a line continuation (` *\\\n(?:\\?\n)*`); the second is
/// a run of two or more newlines (`\n\n+`).
fn match_stem_break(bytes: &[u8], start: usize) -> Option<usize> {
    // Alternative 1: optional spaces, a backslash + newline, then any run of
    // (optional backslash) + newline.
    let mut j = start;
    while bytes.get(j) == Some(&b' ') {
        j += 1;
    }

    if bytes.get(j) == Some(&b'\\') && bytes.get(j + 1) == Some(&b'\n') {
        j += 2;
        loop {
            let mut k = j;
            if bytes.get(k) == Some(&b'\\') {
                k += 1;
            }

            if bytes.get(k) == Some(&b'\n') {
                j = k + 1;
            } else {
                break;
            }
        }

        return Some(j);
    }

    // Alternative 2: two or more consecutive newlines.
    if bytes.get(start) == Some(&b'\n') && bytes.get(start + 1) == Some(&b'\n') {
        let mut j = start + 2;
        while bytes.get(j) == Some(&b'\n') {
            j += 1;
        }

        return Some(j);
    }

    None
}

/// The document-level render settings a nested AsciiDoc table cell inherits
/// from its parent document, carried into the cell's sub-renderer.
#[derive(Clone)]
struct CellRenderConfig {
    icons_set: bool,
    icons_font: bool,
    iconsdir: String,
    icontype: String,
    imagesdir: String,
    asset_uri_scheme: String,
    doc_tabsize: i64,
    source_indent: Option<i64>,
    prewrap: bool,
    source_highlighter: Option<Highlighter>,
    source_language: Option<String>,
    section_anchors: SectionAnchors,
    sectlinks: bool,
    stem_type: StemType,
    cellbgcolor: Option<String>,
    /// Inherited from the enclosing document so an AsciiDoc cell's images honor
    /// the same SVG safe-mode gate (see [`Renderer::svg_below_secure`]).
    svg_below_secure: bool,

    /// Inherited so an AsciiDoc cell's images and image-mode icons embed as
    /// `data:` URIs under the same `data-uri` context as the parent document
    /// (see [`Renderer::data_uri`]).
    data_uri: bool,

    /// Inherited from the enclosing document so an AsciiDoc cell's block image
    /// can embed an inline SVG — and its images and icons can embed as `data:`
    /// URIs — from the same anchored, jailed base directory (see
    /// [`Renderer::svg_source`]).
    svg_source: Option<SvgSource>,
}

/// The table-of-contents settings a nested AsciiDoc table cell resolves from
/// its *own* document, independent of the parent document's TOC. Unlike
/// [`CellRenderConfig`], these are not inherited — the cell is a standalone
/// nested document, so its `toc`/`toclevels`/`toc-title`/`toc-class` come from
/// the cell body's own attributes (see
/// [`AsciiDocCell::toc_mode`](asciidoc_parser::blocks::AsciiDocCell::toc_mode)).
struct CellTocConfig {
    /// Where (and whether) the cell renders a table of contents.
    mode: TocMode,

    /// The deepest section level the cell's TOC includes (`toclevels`).
    levels: usize,

    /// The number of section levels that carry a number in the cell's TOC
    /// (`sectnumlevels`).
    sectnumlevels: usize,

    /// The cell's TOC title (`toc-title`).
    title: String,

    /// The CSS class of the cell's TOC container (`toc-class`).
    class: String,
}

/// The context a cell sub-renderer keeps so a `toc::[]` block macro inside the
/// cell can build its outline from the cell's own blocks — the block-slice
/// counterpart of the top-level document a [`Renderer`] otherwise reads its TOC
/// from. Held in [`Renderer::cell_toc`].
struct CellToc<'a> {
    /// The cell's parsed blocks, walked to build the outline.
    blocks: &'a [Block<'a>],

    /// The deepest section level included (`toclevels`), overridable per macro
    /// by a `levels=` attribute.
    toclevels: usize,

    /// The number of section levels that carry a number (`sectnumlevels`).
    sectnumlevels: usize,

    /// The cell's TOC title (`toc-title`), the macro's default title.
    title: String,

    /// The cell's TOC class (`toc-class`), the macro's default container class.
    class: String,
}

/// Accumulates HTML as the document tree is walked.
struct Renderer<'a> {
    out: String,

    /// The document being rendered, or `None` for a nested table-cell
    /// sub-renderer (which is handed a block slice rather than a `Document`).
    /// The `toc::[]` block macro reads it to build its outline and resolve the
    /// TOC defaults; every other construct is self-contained in its block.
    document: Option<&'a Document<'a>>,

    /// The CSS to embed for a custom, embedded stylesheet, if the caller
    /// supplied any.
    custom_stylesheet: Option<&'a str>,

    /// Whether to emit the standalone document shell (`true`) or embedded,
    /// body-only output (`false`).
    standalone: bool,

    /// Where the document's table of contents is placed, or
    /// [`TocMode::Disabled`] when none is generated. Selects which placement
    /// site emits [`toc_html`](Self::toc_html).
    toc_mode: TocMode,

    /// The pre-rendered `<div id="toc">…</div>` block, or empty when the
    /// document generates no TOC (disabled or `macro` placement) or has no
    /// sections. Emitted at the single site selected by
    /// [`toc_mode`](Self::toc_mode).
    toc_html: String,

    /// The table-of-contents context for a nested AsciiDoc table cell, or
    /// `None` for a top-level document (which reads its TOC from
    /// [`document`](Self::document) instead). It carries the cell's parsed
    /// blocks and resolved `toc*` settings so a `toc::[]` block macro inside
    /// the cell can build its outline without a [`Document`].
    cell_toc: Option<CellToc<'a>>,

    /// Whether the document sets `icons` to any value (Asciidoctor's `attr?
    /// 'icons'`). It selects the icon-based rendering of callout lists (a
    /// two-column `<table>` rather than the default `<ol>`).
    icons_set: bool,

    /// Whether the document sets `:icons: font`, which selects Font Awesome
    /// checkbox glyphs for interactive-less checklists (matching Asciidoctor).
    icons_font: bool,

    /// The document `iconsdir` attribute (default `./images/icons`), used to
    /// build the callout-number image URIs of an icon-based callout list.
    iconsdir: String,

    /// The document `icontype` attribute (default `png`), the file extension of
    /// the callout-number images of an icon-based callout list.
    icontype: String,

    /// The document `imagesdir` attribute (empty when unset), prepended to a
    /// media block's target to build its `src`/`poster` URI (Asciidoctor's
    /// `media_uri`, whose asset directory key is `imagesdir`).
    imagesdir: String,

    /// The document `asset-uri-scheme` attribute (default `https`), the URI
    /// scheme prefixed to a hosted video embed (`youtube`/`vimeo`). An empty
    /// value yields a scheme-relative `//…` URL, matching Asciidoctor.
    asset_uri_scheme: String,

    /// The document `tabsize` attribute as an integer (Asciidoctor's
    /// `String#to_i`, so `0` when absent or non-numeric). A verbatim block's
    /// own `tabsize` attribute overrides this; a positive value expands tabs.
    doc_tabsize: i64,

    /// The document `source-indent` attribute as an integer, when present. It
    /// supplies the `indent` for *source* verbatim blocks that carry no
    /// explicit `indent` attribute, matching Asciidoctor.
    source_indent: Option<i64>,

    /// Whether the document `prewrap` attribute is set. Asciidoctor sets it by
    /// default; when it is unset (`:prewrap!:`) every verbatim `<pre>` gains
    /// the `nowrap` class.
    prewrap: bool,

    /// The active client-side syntax highlighter (`highlightjs`/`prettify`),
    /// resolved once from the document's `source-highlighter` attribute. It
    /// selects the CSS classes on each source block's `<pre>`/`<code>` and
    /// drives the CDN `<link>`/`<script>` injected into the document shell.
    source_highlighter: Option<Highlighter>,

    /// The document's `source-language` attribute, when set. It promotes a bare
    /// `----` listing (no declared style, no language of its own) to a source
    /// block and supplies the default language for any source block that does
    /// not declare one of its own. See [`is_promoted_source_listing`].
    source_language: Option<String>,

    /// How the `sectanchors` document attribute decorates non-discrete section
    /// headings with a self-anchor.
    section_anchors: SectionAnchors,

    /// Whether the `sectlinks` document attribute is set, wrapping each
    /// non-discrete section heading's title text in `<a class="link">`.
    sectlinks: bool,

    /// The document's default STEM notation (its `stem` attribute), the
    /// delimiter pair a bare `[stem]` block renders in.
    stem_type: StemType,

    /// The document `cellbgcolor` attribute, when set to a value. Each table
    /// cell (`<td>`/`<th>`) then carries a `style="background-color: …;"`
    /// matching Asciidoctor. This crate honors only the document-attribute form
    /// (`:cellbgcolor: …`), which colors every cell uniformly; Asciidoctor's
    /// per-cell, order-dependent form set through inline attribute entries
    /// (`{set:cellbgcolor:…}`) is not supported, because `asciidoc-parser` does
    /// not implement inline attribute entries.
    cellbgcolor: Option<String>,

    /// Whether the document's safe mode is below `Secure` (see
    /// [`safe_mode_below_secure`]). It gates the SVG `interactive` and `inline`
    /// referencing of a block image: an `opts=interactive` SVG renders as an
    /// `<object>` (and an `opts=inline` SVG as an embedded `<svg>`) only below
    /// `Secure`, and as a plain `<img>` at `Secure` or above (the API default),
    /// matching Asciidoctor. A document-less sub-renderer (an AsciiDoc table
    /// cell) inherits this from its parent.
    svg_below_secure: bool,

    /// Whether the `data-uri` document attribute is set. When it is — and the
    /// safe mode is below `Secure` (see
    /// [`svg_below_secure`](Self::svg_below_secure)) — the renderer embeds
    /// each referenced image (and image-mode icon) into the output as a
    /// base64 `data:` URI instead of linking it, matching Asciidoctor's
    /// `AbstractNode#image_uri`. `Secure` and above disable it. The read is
    /// anchored and confined by [`svg_source`](Self::svg_source).
    data_uri: bool,

    /// The filesystem anchor (base directory and safe mode) for the renderer's
    /// render-time reads: a block image's inline-embedded SVG
    /// (`image::…[opts=inline]`) and the `data-uri` embedding of images and
    /// image-mode icons. `None` when no base directory anchors the read — a
    /// plain string conversion with no base directory or input file — in which
    /// case a block inline SVG falls back to its alt text and a `data-uri`
    /// image embeds as an empty data URI. Inherited by a document-less
    /// sub-renderer (an AsciiDoc table cell) so its block images embed the
    /// same way. See [`SvgSource`].
    svg_source: Option<SvgSource>,
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

        // The `generator` meta names the tool and its version, which changes
        // from build to build; `reproducible` suppresses it (matching
        // Asciidoctor) so reproducible builds emit byte-identical output.
        if !document.is_attribute_set("reproducible") {
            self.line(&format!(
                "<meta name=\"generator\" content=\"asciidoc-html5 {}\">",
                env!("CARGO_PKG_VERSION")
            ));
        }

        // Document metadata, matching Asciidoctor's `<head>` order: `app-name`
        // (as `application-name`), then `description` and `keywords`, the joined
        // `authors`, and finally `copyright`. Every value here except `authors`
        // comes from an attribute *entry*, so the parser has already applied
        // header substitutions (special characters escaped) — they are emitted
        // verbatim, exactly as Asciidoctor does, so an `app-name`/`copyright` of
        // `<script>` reaches the page as `&lt;script&gt;`, not as live markup.
        // Only `authors` (built from the author info line, not an entry) arrives
        // raw and is escaped below.
        if let Some(app_name) = attribute_str(document, "app-name") {
            self.line(&format!(
                "<meta name=\"application-name\" content=\"{app_name}\">"
            ));
        }
        if let Some(description) = attribute_str(document, "description") {
            self.line(&format!(
                "<meta name=\"description\" content=\"{description}\">"
            ));
        }
        if let Some(keywords) = attribute_str(document, "keywords") {
            self.line(&format!("<meta name=\"keywords\" content=\"{keywords}\">"));
        }
        if let Some(authors) = attribute_str(document, "authors") {
            // Unlike the parser-escaped `description`/`keywords`, the `authors`
            // value arrives raw, so escape it for the attribute context. This
            // matches Asciidoctor, which escapes (rather than strips) any angle
            // brackets that appear in the author `<meta>` content.
            self.line(&format!(
                "<meta name=\"author\" content=\"{}\">",
                escape_attribute(&authors)
            ));
        }
        if let Some(copyright) = attribute_str(document, "copyright") {
            self.line(&format!(
                "<meta name=\"copyright\" content=\"{copyright}\">"
            ));
        }

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

        // Under `:icons: font`, the Font Awesome stylesheet `<link>` the icon
        // glyphs depend on sits right after the primary stylesheet, matching
        // Asciidoctor's placement (before the syntax highlighter's `<head>`).
        self.iconfont_head(document);

        // The active syntax highlighter's `<head>` docinfo (its stylesheet
        // `<link>`) sits after the stylesheet and before the head docinfo,
        // matching Asciidoctor's placement.
        self.highlighter_head(document);

        // Head docinfo is appended to the bottom of the `<head>`, below the
        // default stylesheet, matching Asciidoctor.
        self.docinfo(document, DocinfoLocation::Head);

        self.line("</head>");

        // An explicit id and/or role(s) on the document title (the `[#id.role]`
        // shorthand, or the bracketed `[[id]]` anchor form, above the doctitle)
        // move onto the standalone `<body>`: the id becomes `id="…"` and each role
        // is appended to the doctype class, matching Asciidoctor's
        // `<body id="idname" class="article rolename">`.
        let header = document.header();
        let id_attr = header
            .id()
            .map(|id| format!(" id=\"{}\"", escape_attribute(id)))
            .unwrap_or_default();

        let mut classes = doctype;

        // An automatically placed TOC (`auto`/`left`/`right`/`top`/`bottom`)
        // whose `toc-class` is set adds `<toc-class> toc-<toc-position>` to the
        // `<body>` class — the positional placements set `toc-class` to `toc2`,
        // driving the fixed-column styling. This mirrors Asciidoctor's `<body>`
        // classes; a plain `:toc:` (auto, no `toc-class`) leaves the class list
        // untouched.
        if !self.toc_html.is_empty()
            && matches!(
                self.toc_mode,
                TocMode::Auto | TocMode::Left | TocMode::Right | TocMode::Top | TocMode::Bottom
            )
            && document.has_attribute("toc-class")
        {
            let position =
                attribute_str(document, "toc-position").unwrap_or_else(|| "header".into());
            classes.push(' ');
            classes.push_str(document.toc_class());
            classes.push_str(" toc-");
            classes.push_str(&position);
        }

        for role in header.roles() {
            classes.push(' ');
            classes.push_str(role);
        }

        self.line(&format!(
            "<body{} class=\"{}\">",
            id_attr,
            escape_attribute(&classes)
        ));

        // Header docinfo is inserted immediately before the header `<div>`,
        // whether or not the header itself is suppressed by `noheader` — this
        // is what lets a docinfo header replace the default one.
        self.docinfo(document, DocinfoLocation::Header);

        // The `max-width` attribute constrains the body width by adding an
        // inline `style="max-width: …;"` to each of the standalone layout's
        // container divs (`#header`, `#content`, `#footnotes`, `#footer`),
        // matching Asciidoctor's `html5` backend.
        let max_width = max_width_style(document);

        // The header is suppressed by `noheader`.
        if !document.is_attribute_set("noheader") {
            self.header(document, &max_width);
        }

        self.line(&format!("<div id=\"content\"{max_width}>"));
        self.blocks(document.child_blocks());
        self.line("</div>");

        // The footnotes block sits between the content and the footer, matching
        // Asciidoctor's standalone layout.
        self.footnotes(document);

        // The footer is suppressed by `nofooter`. Its text carries two optional
        // lines, matching Asciidoctor's html5 footer: a "{version-label}
        // {revnumber}" line when the document has a revision number, and a
        // "{last-update-label} {docdatetime}" line (the "Last updated …" stamp)
        // unless the document is `reproducible`.
        if !document.is_attribute_set("nofooter") {
            self.line(&format!("<div id=\"footer\"{max_width}>"));
            self.line("<div id=\"footer-text\">");

            // The version line, gated on `revnumber` like Asciidoctor. The
            // `version-label` intrinsic defaults to "Version"; a trailing `<br>`
            // separates it from the "Last updated" line that may follow.
            if let Some(revnumber) = attribute_str(document, "revnumber") {
                let version_label = attribute_str(document, "version-label").unwrap_or_default();
                self.line(&format!("{version_label} {revnumber}<br>"));
            }

            // The "Last updated" line, gated on the `last-update-label`
            // intrinsic (defaulting to "Last updated", unset to drop the line)
            // and suppressed by `reproducible` so reproducible builds omit the
            // volatile timestamp.
            if !document.is_attribute_set("reproducible") {
                if let (Some(label), Some(docdatetime)) = (
                    attribute_str(document, "last-update-label"),
                    attribute_str(document, "docdatetime"),
                ) {
                    self.line(&format!("{label} {docdatetime}"));
                }
            }

            self.line("</div>");
            self.line("</div>");
        }

        // The active syntax highlighter's `<footer>` docinfo (the scripts that
        // load and run the highlighter) is emitted after the footer `<div>`,
        // matching Asciidoctor — where JavaScript loads at the end of the body.
        self.highlighter_footer(document);

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

        // An `auto`/`left`/`right`/`top`/`bottom` TOC leads the embedded body,
        // ahead of the content, matching Asciidoctor's embeddable output. A
        // `preamble` TOC is instead emitted within the preamble itself (see
        // [`preamble`]), so it is excluded here.
        //
        // The side-column placement (`left`/`right`) isn't available in
        // embeddable output, which lacks the standalone frame and CSS that
        // layout needs, so this leading TOC uses the plain `toc` class,
        // regardless of `toc-position` or a custom `toc-class` (matching
        // Asciidoctor's `convert_embedded`, which hardcodes `class="toc"`). The
        // class was already resolved when `toc_html` was built, so it is emitted
        // as-is here.
        if matches!(
            self.toc_mode,
            TocMode::Auto | TocMode::Left | TocMode::Right | TocMode::Top | TocMode::Bottom
        ) && !self.toc_html.is_empty()
        {
            self.line(&self.toc_html.clone());
        }

        self.blocks(document.child_blocks());

        // Embedded output still carries the footnotes block, matching
        // Asciidoctor's `convert_string_to_embedded`.
        self.footnotes(document);
    }

    /// Emits the document-level footnotes block — Asciidoctor's `#footnotes`
    /// section listing every registered footnote's definition, each linking
    /// back to its reference — when the document has footnotes and they are not
    /// suppressed by the `nofootnotes` attribute.
    ///
    /// Each footnote's [`text`](asciidoc_parser::document::Footnote::text) is
    /// already a substituted inline HTML fragment (with cross references
    /// resolved once the document's references are), so it is emitted verbatim,
    /// like all other block content.
    fn footnotes(&mut self, document: &Document<'_>) {
        let footnotes = document.catalog().footnotes();
        if footnotes.is_empty() || document.is_attribute_set("nofootnotes") {
            return;
        }

        // The document-level footnotes block picks up the `max-width` constraint
        // like the other standalone container divs; the cell-local block (which
        // has no document to read the attribute from) never does.
        self.footnotes_block(footnotes, &max_width_style(document));
    }

    /// Emits the `#footnotes` block for the given footnotes — the `<div
    /// id="footnotes"><hr>` frame followed by one `<div class="footnote"
    /// id="_footnotedef_N">` definition per footnote, each linking back to its
    /// reference.
    ///
    /// This is shared between the document-level [`footnotes`](Self::footnotes)
    /// block and the cell-local block an AsciiDoc (`a`) table cell renders from
    /// its own footnote registry. The caller is responsible for the emptiness
    /// and `nofootnotes` checks; this assumes there is at least one footnote to
    /// list. `max_width` is the ` style="max-width: …;"` fragment for the
    /// standalone `#footnotes` div (empty for the cell-local block).
    fn footnotes_block(&mut self, footnotes: &[Footnote], max_width: &str) {
        self.line(&format!("<div id=\"footnotes\"{max_width}>"));
        self.line("<hr>");

        for footnote in footnotes {
            let index = &footnote.index;
            self.line(&format!(
                "<div class=\"footnote\" id=\"_footnotedef_{index}\">"
            ));
            self.line(&format!(
                "<a href=\"#_footnoteref_{index}\">{index}</a>. {text}",
                text = footnote.text
            ));
            self.line("</div>");
        }

        self.line("</div>");
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
            .child_blocks()
            .next()
            .and_then(|block| block.rendered_content())
        {
            self.line(content);
        }
    }

    /// Emits `<div id="header">` with the `<h1>` doctitle and, when present,
    /// the author and revision details block and an automatically placed table
    /// of contents. `max_width` is the ` style="max-width: …;"` fragment
    /// applied to the opening `<div id="header">` when the `max-width`
    /// attribute is set.
    fn header(&mut self, document: &Document<'_>, max_width: &str) {
        let header: &Header<'_> = document.header();

        // A standalone document shows its title as the header `<h1>` by default;
        // the `notitle` attribute suppresses it. (`noheader`, which drops the
        // whole header, is handled by the caller.) This is the section title,
        // matching Asciidoctor's `node.header.title` — not the effective
        // `doctitle`, so a `title` attribute entry (which overrides only the
        // HTML `<title>` element) does not change the `<h1>`.
        let title = header
            .title()
            .filter(|_| !document.is_attribute_set("notitle"));

        // The byline is driven entirely by resolved attributes, matching
        // Asciidoctor's `html5` backend: `authors` (from the author line *or*
        // the `author`/`author_N` attribute entries) and the `revnumber`,
        // `revdate`, and `revremark` document attributes (set by the revision
        // line *or* by attribute entries).
        // An empty revision field is treated as absent, matching Asciidoctor,
        // whose Ruby parser leaves the attribute unset (rather than empty) when
        // a revision line omits it.
        let authors = header.authors();
        let revnumber = attribute_str(document, "revnumber").filter(|s| !s.is_empty());
        let revdate = attribute_str(document, "revdate").filter(|s| !s.is_empty());
        let revremark = attribute_str(document, "revremark").filter(|s| !s.is_empty());
        let has_details =
            !authors.is_empty() || revnumber.is_some() || revdate.is_some() || revremark.is_some();

        // An `auto`/`left`/`right`/`top`/`bottom` TOC is emitted inside the
        // header, after the details, matching Asciidoctor's `html5` backend.
        let header_toc = matches!(
            self.toc_mode,
            TocMode::Auto | TocMode::Left | TocMode::Right | TocMode::Top | TocMode::Bottom
        ) && !self.toc_html.is_empty();

        if title.is_none() && !has_details && !header_toc {
            return;
        }

        self.line(&format!("<div id=\"header\"{max_width}>"));

        if let Some(title) = title {
            self.line(&format!("<h1>{title}</h1>"));
        }

        if has_details {
            self.line("<div class=\"details\">");

            // The author name arrives already header-substituted from the
            // parser (special characters and attribute references applied —
            // asciidoc-parser #1068), so it must not be re-escaped, which would
            // double-encode a name like `Ben & Jerry`. Asciidoctor's byline
            // additionally runs the replacements step on the name (its
            // `sub_replacements` helper), so e.g. `O'Brien` becomes
            // `O&#8217;Brien` and `(C)` becomes `&#169;`; mirror that with the
            // parser's per-string substitution API (asciidoc-parser #1077).
            let replacements =
                SubstitutionGroup::Custom(vec![SubstitutionStep::CharacterReplacements]);
            let byline_parser = Parser::default();

            for (index, author) in authors.iter().enumerate() {
                let suffix = if index == 0 {
                    String::new()
                } else {
                    (index + 1).to_string()
                };

                let name = byline_parser.apply_substitutions(author.name(), &replacements);
                self.line(&format!(
                    "<span id=\"author{suffix}\" class=\"author\">{name}</span><br>",
                ));
                if let Some(email) = author.email() {
                    // The email is raw and lands in a `mailto:` href, so it is
                    // attribute-escaped to keep a `"` from breaking out.
                    let email = escape_attribute(email);
                    self.line(&format!(
                        "<span id=\"email{suffix}\" class=\"email\"><a href=\"mailto:{email}\">{email}</a></span><br>",
                    ));
                }
            }

            if let Some(revnumber) = &revnumber {
                // Asciidoctor prefixes the revision number with the lowercased
                // `version-label` (default "Version", empty when unset) and
                // appends a comma when a revision date follows.
                let version_label = attribute_str(document, "version-label")
                    .unwrap_or_default()
                    .to_lowercase();

                let comma = if revdate.is_some() { "," } else { "" };
                self.line(&format!(
                    "<span id=\"revnumber\">{version_label} {revnumber}{comma}</span>"
                ));
            }
            if let Some(revdate) = &revdate {
                self.line(&format!("<span id=\"revdate\">{revdate}</span>"));
            }
            if let Some(revremark) = &revremark {
                self.line(&format!("<br><span id=\"revremark\">{revremark}</span>"));
            }

            self.line("</div>");
        }

        if header_toc {
            self.line(&self.toc_html.clone());
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

    /// Emits the Font Awesome stylesheet `<link>` that font-based icons
    /// (`:icons: font`) depend on, matching Asciidoctor's `html5` backend (the
    /// block right after the primary stylesheet). Nothing is emitted unless the
    /// resolved `icons` attribute is `font` — the same gate the admonition,
    /// callout, and inline-icon markup use, so the glyphs and the font that
    /// draws them are always emitted together.
    ///
    /// With `iconfont-remote` set (the default), the link points at a CDN: an
    /// `iconfont-cdn` attribute overrides the URL outright, otherwise it is
    /// `{cdn}/font-awesome/{version}/css/font-awesome.min.css`, where `{cdn}`
    /// follows `asset-uri-scheme` ([`cdn_base_url`]) and `{version}` is
    /// [`FONT_AWESOME_VERSION`]. With `iconfont-remote` unset
    /// (`:iconfont-remote!:`), the link is a local `{iconfont-name}.css`
    /// (default `font-awesome`) resolved against `stylesdir` by
    /// [`normalize_web_path`], so no remote resource is referenced.
    ///
    /// Following the resolved `icons` value keeps this consistent with the icon
    /// markup: `icons` is locked against the document at `Secure` (in
    /// `Options::apply`, matching Asciidoctor), so the link, like the markup,
    /// is attribute-driven rather than gated a second time in the renderer
    /// — mirroring how the syntax highlighter's CDN `<link>` follows its
    /// (safe-mode-locked) attribute.
    fn iconfont_head(&mut self, document: &Document<'_>) {
        if !self.icons_font {
            return;
        }

        let href = if document.is_attribute_set("iconfont-remote") {
            attribute_str(document, "iconfont-cdn").unwrap_or_else(|| {
                format!(
                    "{}/font-awesome/{FONT_AWESOME_VERSION}/css/font-awesome.min.css",
                    cdn_base_url(document)
                )
            })
        } else {
            let name = attribute_str(document, "iconfont-name")
                .unwrap_or_else(|| "font-awesome".to_string());
            let stylesdir = attribute_str(document, "stylesdir").unwrap_or_default();

            normalize_web_path(&format!("{name}.css"), &stylesdir)
        };

        self.line(&format!(
            "<link rel=\"stylesheet\" href=\"{}\">",
            escape_attribute(&href)
        ));
    }

    /// Emits the active client-side highlighter's `<head>` docinfo — the
    /// stylesheet `<link>` — matching Asciidoctor's `docinfo :head`.
    ///
    /// `highlightjs` links its theme stylesheet
    /// (`{highlightjsdir}/styles/{highlightjs-theme}.min.css`, default theme
    /// `github`); `prettify` links its theme stylesheet
    /// (`{prettifydir}/{prettify-theme}.min.css`, default theme `prettify`),
    /// or, when `prettify-theme` is itself an `http(s)` URL, that URL
    /// directly.
    fn highlighter_head(&mut self, document: &Document<'_>) {
        let href = match self.source_highlighter {
            // No client-side highlighter, and the server-side ones
            // (`coderay`/`pygments`/`rouge`) are not supported – they emit no
            // stylesheet docinfo here – so there is nothing to link.
            None => return,

            Some(Highlighter::HighlightJs) => {
                let theme = attribute_str(document, "highlightjs-theme")
                    .unwrap_or_else(|| "github".to_string());

                format!(
                    "{}/styles/{}.min.css",
                    highlightjs_dir(document),
                    escape_attribute(&theme)
                )
            }

            Some(Highlighter::Prettify) => {
                let theme = attribute_str(document, "prettify-theme")
                    .unwrap_or_else(|| "prettify".to_string());

                if theme.starts_with("http://") || theme.starts_with("https://") {
                    escape_attribute(&theme)
                } else {
                    format!(
                        "{}/{}.min.css",
                        prettify_dir(document),
                        escape_attribute(&theme)
                    )
                }
            }
        };

        self.line(&format!("<link rel=\"stylesheet\" href=\"{href}\">"));
    }

    /// Emits the active client-side highlighter's `<footer>` docinfo — the
    /// scripts that load and run the highlighter — matching Asciidoctor's
    /// `docinfo :footer`.
    ///
    /// `highlightjs` loads the core script, then one script per
    /// `highlightjs-languages` entry, then an inline initializer;
    /// `prettify` loads its single `run_prettify.min.js`.
    fn highlighter_footer(&mut self, document: &Document<'_>) {
        match self.source_highlighter {
            None => {}

            Some(Highlighter::HighlightJs) => {
                let base = highlightjs_dir(document);
                let mut footer = format!("<script src=\"{base}/highlight.min.js\"></script>\n");

                // Each comma-separated `highlightjs-languages` entry loads an
                // extra language pack (Ruby `lstrip`s the leading whitespace).
                if let Some(languages) = attribute_str(document, "highlightjs-languages") {
                    for language in languages.split(',') {
                        let language = language.trim_start();
                        if language.is_empty() {
                            continue;
                        }

                        footer.push_str(&format!(
                            "<script src=\"{base}/languages/{}.min.js\"></script>\n",
                            escape_attribute(language)
                        ));
                    }
                }

                footer.push_str(
                    "<script>\n\
                     if (!hljs.initHighlighting.called) {\n  \
                     hljs.initHighlighting.called = true\n  \
                     ;[].slice.call(document.querySelectorAll('pre.highlight > code[data-lang]'))\
                     .forEach(function (el) { hljs.highlightBlock(el) })\n\
                     }\n\
                     </script>",
                );

                self.line(&footer);
            }

            Some(Highlighter::Prettify) => {
                self.line(&format!(
                    "<script src=\"{}/run_prettify.min.js\"></script>",
                    prettify_dir(document)
                ));
            }
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
    fn blocks<'src>(&mut self, blocks: impl Iterator<Item = &'src Block<'src>>) {
        for block in blocks {
            self.block(block);
        }
    }

    /// The dispatch point: routes one block to the matching renderer.
    fn block<'src>(&mut self, block: &'src Block<'src>) {
        // Comment blocks render to nothing in Asciidoctor. This crate drops them
        // here, in the renderer, rather than in the parser (which preserves them
        // so other tools can inspect them). See [`renders_nothing`].
        if renders_nothing(block) {
            return;
        }

        match block {
            Block::Simple(simple) => match simple.style() {
                // A styled paragraph can convert to a different block: `[open]`
                // over a paragraph becomes an open block (an empty content div
                // around the paragraph's text).
                SimpleBlockStyle::Paragraph => match block.declared_style() {
                    Some("open") => self.open_block(block),
                    Some("sidebar") => self.sidebar(block),
                    Some("example") => self.example(block),

                    // The `abstract` style over a paragraph renders through
                    // Asciidoctor's quote template: a `<div class="quoteblock
                    // abstract">` wrapping a `<blockquote>`.
                    Some("abstract") => self.abstract_block(block),

                    // The `pass` style over a paragraph emits its content raw,
                    // with no paragraph wrapper — just like a delimited `++++`
                    // block — matching Asciidoctor's `convert_pass`.
                    Some("pass") => self.pass_block(block),
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
                // A source-styled delimited listing renders like a source block
                // (the `<pre class="highlight"><code …>` shape), matching
                // Asciidoctor: `[source,ruby]` or the `[,ruby]` comma shorthand,
                // or a bare `----` promoted by a document `source-language`.
                // A plain `----` listing is a verbatim block.
                "listing" if is_promoted_source_listing(block, self.source_language.as_deref()) => {
                    self.source(block)
                }
                "listing" => self.verbatim(block, "listingblock"),
                "literal" => self.verbatim(block, "literalblock"),
                "pass" => self.pass_block(block),
                "stem" => self.stem(block),
                // The parser assigns a raw-delimited block one of exactly five
                // contexts — `listing`, `literal`, `pass`, `stem`, `comment` —
                // and a `comment` block is dropped by `renders_nothing` before
                // it reaches here, so every value that arrives is matched above.
                // This arm is the required catch-all on the `&str` match and is
                // unreachable in practice (hence uncovered); it keeps the
                // renderer well-formed should the parser ever add a context.
                other => self.unsupported(other),
            },
            Block::CompoundDelimited(compound) => match compound.context_kind() {
                // An open block (`--`) can masquerade as another structural
                // container via its declared style. The parser already remaps
                // the styles that change the block's *type* — `source`, `quote`,
                // `verse`, `listing`, `literal`, and the admonition styles — into
                // their own block kinds, so those never arrive here. The styles
                // that keep the compound-open shape but still change the output —
                // `sidebar`, `example`, and `abstract` (a quote-like abstract,
                // matching Asciidoctor) — are resolved from the declared style.
                CompoundDelimitedContext::Open => match block.declared_style() {
                    Some("sidebar") => self.sidebar(block),
                    Some("example") => self.example(block),
                    Some("abstract") => self.abstract_block(block),
                    _ => self.open_block(block),
                },

                CompoundDelimitedContext::Sidebar => self.sidebar(block),
                CompoundDelimitedContext::Example => self.example(block),
            },
            Block::Quote(quote) => self.quote(block, quote),
            Block::Admonition(admonition) => self.admonition(block, admonition),
            Block::List(list) => match list.type_() {
                ListType::Unordered => self.ulist(block, list),
                ListType::Ordered => self.olist(block, list),
                ListType::Description => self.dlist(block, list),
                ListType::Callout => self.colist(block, list),
            },
            Block::Table(table) => self.table(block, table),
            Block::Toc(toc) => self.toc_macro(block, toc),
            Block::Media(media) if media.type_() == MediaType::Image => self.image(block, media),
            Block::Media(media) if media.type_() == MediaType::Video => self.video(block, media),
            Block::Media(media) if media.type_() == MediaType::Audio => self.audio(block, media),

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
    /// the `<pre>` without added newlines around the text. The content is first
    /// reindented and blank-line-trimmed by
    /// [`verbatim_content`](Self::verbatim_content), and the `<pre>` gains a
    /// `nowrap` class when the block opts out of wrapping.
    ///
    /// The title is captioned: a titled listing block gains its `Listing N. `
    /// caption prefix when `listing-caption` is set (or an explicit
    /// `[caption=]` override is given). A literal block is never
    /// captionable, so [`captioned_title`](Self::captioned_title) falls
    /// back to its bare title.
    fn verbatim<'src>(&mut self, block: &'src Block<'src>, wrapper_class: &str) {
        self.open_block_wrapper(block, wrapper_class);
        self.captioned_title(block);
        self.line("<div class=\"content\">");
        let is_source = block.declared_style() == Some("source");
        let content = self.verbatim_content(block, is_source);
        self.line(&format!("<pre{}>{content}</pre>", self.nowrap_class(block)));
        self.line("</div>");
        self.line("</div>");
    }

    /// A source block: like a listing block, but the `<pre>` carries the
    /// `highlight` class and wraps the code in a `<code>` element that names
    /// the language (`class="language-…" data-lang="…"`) when one is
    /// declared. This matches Asciidoctor's default output even when no
    /// syntax highlighter is active.
    ///
    /// When a client-side highlighter is active
    /// ([`source_highlighter`](Self::source_highlighter)), the `<pre>`/`<code>`
    /// classes take the highlighter-specific shape instead — `highlightjs
    /// highlight` / `prettyprint highlight` on the `<pre>`, and `hljs` (plus
    /// the `linenums`/`start` line-number classes for prettify) — matching
    /// Asciidoctor's `HighlightJsAdapter`/`PrettifyAdapter`.
    ///
    /// A source block resolves to the `listing` context, so its title is
    /// captioned the same way a listing block's is (see
    /// [`verbatim`](Self::verbatim)).
    fn source<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "listingblock");
        self.captioned_title(block);
        self.line("<div class=\"content\">");

        let content = self.verbatim_content(block, true);

        // The language is the second positional attribute of `[source, lang]`
        // (the first is the `source` style itself), or an explicit `language=`.
        // When the block declares no language of its own, it falls back to the
        // document's `source-language` attribute (Asciidoctor's default source
        // language), which is also what promotes a bare listing to a source
        // block.
        let language = block
            .attrlist()
            .and_then(|attrlist| attrlist.named_or_positional_attribute("language", 2))
            .map(|attr| attr.value())
            .or(self.source_language.as_deref())
            // A fenced code block's info string reaches the parser as a single
            // positional attribute — ```` ```ruby,numbered ```` yields the
            // language `ruby,numbered` — because `asciidoc-parser` does not split
            // it into separate positional attributes the way Asciidoctor does.
            // Keep only the language token, the part before the first comma. A
            // bracketed `[source,lang]` never carries a comma in this attribute,
            // so this is a no-op for it.
            .map(|language| language.split(',').next().unwrap_or(language).trim());

        let (pre_class, code_open) = self.source_pre_code(block, language);
        self.line(&format!(
            "<pre class=\"{pre_class}\">{code_open}{content}</code></pre>"
        ));

        self.line("</div>");
        self.line("</div>");
    }

    /// Builds the `<pre>` class list and the opening `<code …>` tag for a
    /// source block, keyed off the active client-side highlighter.
    ///
    /// With no highlighter the default shape is used (`highlight` plus an
    /// optional `nowrap`, and `class="language-…" data-lang="…"` on the
    /// `<code>` when a language is declared). `highlightjs` and `prettify`
    /// prepend their `@pre_class` and reshape the `<code>` attributes to match
    /// their Asciidoctor adapters.
    fn source_pre_code(&self, block: &Block<'_>, language: Option<&str>) -> (String, String) {
        // A `nowrap` block (or a document with `prewrap` disabled) adds the
        // class after `highlight`, matching Asciidoctor's `pre class="highlight
        // nowrap"`.
        let nowrap = if self.is_nowrap(block) { " nowrap" } else { "" };
        let language = language.map(escape_attribute);

        match self.source_highlighter {
            None => {
                let code_open = match &language {
                    Some(language) => {
                        format!("<code class=\"language-{language}\" data-lang=\"{language}\">")
                    }
                    None => "<code>".to_string(),
                };

                (format!("highlight{nowrap}"), code_open)
            }

            Some(Highlighter::HighlightJs) => {
                // `code['class'] = "language-#{lang || 'none'} hljs"`, and
                // `data-lang` is only added when a language is declared.
                let code_open = match &language {
                    Some(language) => format!(
                        "<code class=\"language-{language} hljs\" data-lang=\"{language}\">"
                    ),
                    None => "<code class=\"language-none hljs\">".to_string(),
                };

                (format!("highlightjs highlight{nowrap}"), code_open)
            }

            Some(Highlighter::Prettify) => {
                // Prettify leaves the `<code>` in its default shape (a bare
                // `data-lang`, no class) and, for a `linenums` block, appends
                // ` linenums` (or ` linenums:<start>`) after the `highlight`
                // classes.
                let linenums = if block.has_option("linenums") {
                    match block
                        .attrlist()
                        .and_then(|attrlist| attrlist.named_attribute("start"))
                        .map(|attr| attr.value())
                    {
                        Some(start) => format!(" linenums:{}", escape_attribute(start)),
                        None => " linenums".to_string(),
                    }
                } else {
                    String::new()
                };

                let code_open = match &language {
                    Some(language) => format!("<code data-lang=\"{language}\">"),
                    None => "<code>".to_string(),
                };

                (
                    format!("prettyprint highlight{nowrap}{linenums}"),
                    code_open,
                )
            }
        }
    }

    /// A passthrough block (`++++`): its content is emitted raw and unescaped,
    /// with no wrapping element, matching Asciidoctor's `convert_pass`. Like
    /// other verbatim/raw content, leading and trailing blank lines are
    /// trimmed.
    fn pass_block<'src>(&mut self, block: &'src Block<'src>) {
        let content = block.rendered_content().unwrap_or_default();
        let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();
        strip_surrounding_blank_lines(&mut lines);
        self.line(&lines.join("\n"));
    }

    /// A STEM block (`[stem]`, `[asciimath]`, or `[latexmath]` over a `++++`
    /// delimited block): `<div class="stemblock">` wrapping a `<div
    /// class="content">` whose equation is surrounded by the math delimiter
    /// pair its notation selects. Mirrors Asciidoctor's `convert_stem`,
    /// honoring the block title, `id`, and roles on the wrapper.
    ///
    /// A bare `[stem]` follows the document `stem` attribute default (see
    /// [`stem_type`](Self::stem_type)); an explicit `[asciimath]` /
    /// `[latexmath]` style — or a notation named as the block's second
    /// positional attribute (`[stem, asciimath]`) — overrides it. The
    /// equation's special characters have already been escaped by the parser;
    /// the delimiters are added here unless the author wrote them, and a
    /// multi-line AsciiMath equation has its internal breaks rewritten to
    /// `<br>`-separated expressions.
    fn stem<'src>(&mut self, block: &'src Block<'src>) {
        let stem_type = match block.declared_style() {
            Some("asciimath") => StemType::AsciiMath,
            Some("latexmath") => StemType::LatexMath,

            // A bare `[stem]` may still name its notation as the block's second
            // positional attribute (`[stem, asciimath]`), which Asciidoctor
            // promotes to the block style and which overrides the document
            // `stem` default. An absent or unrecognized second positional falls
            // back to that document default.
            _ => match block
                .attrlist()
                .and_then(|attrlist| attrlist.nth_attribute(2))
                .map(|attr| attr.value())
            {
                Some("latexmath" | "latex" | "tex") => StemType::LatexMath,
                Some("asciimath") => StemType::AsciiMath,
                _ => self.stem_type,
            },
        };

        let (open, close) = stem_type.block_delimiters();

        self.open_block_wrapper(block, "stemblock");
        self.block_title(block);
        self.line("<div class=\"content\">");

        let mut equation = block.rendered_content().unwrap_or_default().to_string();

        if stem_type == StemType::AsciiMath && equation.contains('\n') {
            equation = rewrite_asciimath_breaks(&equation, open, close);
        }

        // Asciidoctor leaves an equation the author already delimited untouched;
        // otherwise it wraps it in the notation's delimiter pair (an empty
        // equation still becomes the bare `open`/`close` pair).
        if !(equation.starts_with(open) && equation.ends_with(close)) {
            equation = format!("{open}{equation}{close}");
        }

        self.line(&equation);
        self.line("</div>");
        self.line("</div>");
    }

    /// The reindented, blank-line-trimmed inner text of a verbatim block's
    /// `<pre>`, applying Asciidoctor's `tabsize`/`indent`/`source-indent`
    /// handling and its leading/trailing blank-line trimming.
    ///
    /// The parser has already applied inline substitutions (so special
    /// characters are escaped), but leaves indentation and surrounding blank
    /// lines untouched; those are normalized here. `is_source` selects whether
    /// the document `source-indent` attribute supplies a default indent, which
    /// Asciidoctor applies only to source blocks.
    fn verbatim_content<'src>(&self, block: &'src Block<'src>, is_source: bool) -> String {
        let content = block.rendered_content().unwrap_or_default();
        let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();

        // An *implicit* literal paragraph — one detected from indentation rather
        // than an explicit `[literal]`/`[source]`/`[listing]` style — is the one
        // verbatim shape Asciidoctor reindents by common (minimum) indent even
        // without an `indent` attribute. The parser instead flattens it,
        // stripping the first line's indent from every line before the renderer
        // sees it (#168); restore each line's true leading whitespace from the
        // raw span so the common-indent removal below can match Asciidoctor. A
        // delimited or explicitly-styled block preserves its indentation as-is,
        // so it is left untouched.
        let is_implicit_literal_paragraph = matches!(block, Block::Simple(simple)
            if simple.style() == SimpleBlockStyle::Literal)
            && block.declared_style().is_none();

        if is_implicit_literal_paragraph {
            restore_literal_paragraph_indent(&mut lines, block.span().data());
        }

        // A block-level `tabsize` overrides the document one; `indent` (falling
        // back to `source-indent` for source blocks) drives reindentation.
        let tab_size = block
            .attrlist()
            .and_then(|attrlist| attrlist.named_attribute("tabsize"))
            .map(|attr| ruby_to_i(attr.value()))
            .unwrap_or(self.doc_tabsize);

        let indent_size = block
            .attrlist()
            .and_then(|attrlist| attrlist.named_attribute("indent"))
            .map(|attr| ruby_to_i(attr.value()))
            .or(if is_source { self.source_indent } else { None });

        // Asciidoctor reindents when an indent is in force, when an implicit
        // literal paragraph has its common indent removed (indent 0, which also
        // expands tabs), or (to expand tabs only) when a positive tabsize is set
        // with the indentation otherwise preserved.
        match indent_size {
            Some(indent) => adjust_indentation(&mut lines, indent, tab_size),
            None if is_implicit_literal_paragraph => adjust_indentation(&mut lines, 0, tab_size),
            None if tab_size > 0 => adjust_indentation(&mut lines, -1, tab_size),
            None => {}
        }

        strip_surrounding_blank_lines(&mut lines);
        lines.join("\n")
    }

    /// Whether a verbatim block's `<pre>` should carry the `nowrap` class: the
    /// block declares the `nowrap` option, or the document has disabled
    /// `prewrap` (`:prewrap!:`).
    fn is_nowrap(&self, block: &Block<'_>) -> bool {
        block.has_option("nowrap") || !self.prewrap
    }

    /// The `class="nowrap"` attribute for a bare verbatim `<pre>`, or an empty
    /// string when wrapping is left enabled.
    fn nowrap_class(&self, block: &Block<'_>) -> &'static str {
        if self.is_nowrap(block) {
            " class=\"nowrap\""
        } else {
            ""
        }
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
        if block.has_option("collapsible") {
            self.collapsible_example(block);
            return;
        }

        self.open_block_wrapper(block, "exampleblock");
        self.captioned_title(block);
        self.line("<div class=\"content\">");
        self.wrapped_content(block);
        self.line("</div>");
        self.line("</div>");
    }

    /// A collapsible example (`[%collapsible]`): a `<details>`/`<summary>`
    /// disclosure widget in place of the standard `exampleblock`. The block's
    /// id and roles carry onto the `<details>` element (`id="…"` and each role
    /// as a class), matching Asciidoctor. The `open` option
    /// (`[%collapsible%open]`) adds the boolean `open` attribute so the widget
    /// starts expanded, and an untitled block falls back to a default `Details`
    /// summary. A collapsible example is never captioned or numbered — the
    /// parser suppresses its caption — so it does not consume an example
    /// number.
    fn collapsible_example<'src>(&mut self, block: &'src Block<'src>) {
        let open = if block.has_option("open") {
            " open"
        } else {
            ""
        };
        self.line(&format!(
            "<details{}{}{open}>",
            id_attribute(block.id()),
            class_attribute("", &block.roles())
        ));
        let summary = block.title().unwrap_or("Details");
        self.line(&format!("<summary class=\"title\">{summary}</summary>"));
        self.line("<div class=\"content\">");
        self.wrapped_content(block);
        self.line("</div>");
        self.line("</details>");
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

    /// An `[abstract]`-styled paragraph or open block: `<div class="quoteblock
    /// abstract">[<div class="title">…</div>]<blockquote>…</blockquote></div>`.
    ///
    /// Asciidoctor routes the `abstract` block style through the same quote
    /// template it uses for `[quote]`, so an abstract shares the quote block's
    /// shape (minus the attribution footer, which an abstract never carries).
    /// A styled paragraph places its inline content directly inside the
    /// `<blockquote>` with no `<p>` wrapper, while a delimited open block
    /// places its child blocks there — the split handled by
    /// [`wrapped_content`](Self::wrapped_content).
    fn abstract_block<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "quoteblock abstract");
        self.block_title(block);
        self.line("<blockquote>");
        self.wrapped_content(block);
        self.line("</blockquote>");
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

    /// An admonition block: Asciidoctor's two-cell table, the first cell
    /// holding the icon (or, by default, the caption label) and the second
    /// the content, wrapped in `<div class="admonitionblock <name>">`.
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
        self.line(&self.admonition_icon(block, admonition));
        self.line("</td>");
        self.line("<td class=\"content\">");
        self.block_title(block);
        self.wrapped_content(block);
        self.line("</td>");
        self.line("</tr>");
        self.line("</table>");
        self.line("</div>");
    }

    /// The markup for an admonition's icon cell, mirroring Asciidoctor's
    /// `convert_admonition`.
    ///
    /// With the `icons` document attribute unset (the default), the caption
    /// label is shown as text (`<div class="title">Note</div>`). With `icons`
    /// set to `font` – and no per-block `icon` attribute – a Font Awesome glyph
    /// is emitted (`<i class="fa icon-note" title="Note">`). Otherwise (image
    /// mode, or a per-block `icon` override) an `<img>` points at the resolved
    /// icon URI. The image target is a port of `AbstractNode#icon_uri`: a
    /// per-block `icon` value is used as-is (with the `icontype` extension
    /// appended only when it has none), while the default target is
    /// `<name>.<icontype>`, each resolved against `iconsdir`.
    fn admonition_icon(&self, block: &Block<'_>, admonition: &AdmonitionBlock<'_>) -> String {
        let label = admonition.label();

        if !self.icons_set {
            return format!("<div class=\"title\">{label}</div>");
        }

        let name = admonition.name();
        let custom_icon = block
            .attrlist()
            .and_then(|attrlist| attrlist.named_attribute("icon"))
            .map(|attr| attr.value().to_string());

        if self.icons_font && custom_icon.is_none() {
            return format!(
                "<i class=\"fa icon-{name}\" title=\"{}\"></i>",
                escape_attribute(label)
            );
        }

        let target = match custom_icon {
            Some(icon) if has_file_extension(&icon) => icon,
            Some(icon) => format!("{icon}.{}", self.icontype),
            None => format!("{name}.{}", self.icontype),
        };
        let src = escape_attribute(&self.image_uri(&target, &self.iconsdir));

        format!("<img src=\"{src}\" alt=\"{}\">", escape_attribute(label))
    }

    /// An unordered list: `<div class="ulist …"><ul …><li>…</li>…</ul></div>`,
    /// matching Asciidoctor's `convert_ulist`.
    ///
    /// The wrapper `<div>` carries the `ulist` class, then (for a checklist) a
    /// `checklist` class, then the list's declared style (`square`, `circle`,
    /// `none`, …) and any roles. The inner `<ul>` carries `class="checklist"`
    /// for a checklist, otherwise the declared style as its class, or no class
    /// at all for a plain bullet list.
    fn ulist<'src>(&mut self, block: &'src Block<'src>, list: &'src ListBlock<'src>) {
        let checklist = list.is_checklist();

        // Use the *resolved* style (Asciidoctor's `node.style`): a top-level list
        // inside a `[bibliography]` section inherits the `bibliography` style even
        // though it carries no declared style of its own, so it renders `<div
        // class="ulist bibliography">` / `<ul class="bibliography">`.
        let style = block.resolved_style();

        // `['ulist', ('checklist')?, style, *roles]` — the checklist class sits
        // right after `ulist`, ahead of the style and roles.
        let mut base = String::from("ulist");
        if checklist {
            base.push_str(" checklist");
        }
        if let Some(style) = style {
            base.push(' ');
            base.push_str(style);
        }

        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute(&base, &block.roles())
        ));
        self.block_title(block);

        // A checklist's `<ul>` always carries `class="checklist"`; otherwise a
        // declared style becomes the `<ul>` class, and a plain list gets none.
        let ul_class = if checklist {
            " class=\"checklist\"".to_string()
        } else if let Some(style) = style {
            format!(" class=\"{}\"", escape_attribute(style))
        } else {
            String::new()
        };
        self.line(&format!("<ul{ul_class}>"));

        let interactive = block.has_option("interactive");
        for item in list.child_blocks() {
            self.list_item(item, checklist, interactive);
        }

        self.line("</ul>");
        self.line("</div>");
    }

    /// An ordered list: `<div class="olist <style>"><ol
    /// class="<style>" …><li>…</li>…</ol></div>`, matching Asciidoctor's
    /// `convert_olist`.
    ///
    /// The numbering style (`arabic`, `loweralpha`, `lowerroman`, `upperalpha`,
    /// `upperroman`) names both the wrapper `<div>` class and the `<ol>` class;
    /// the `<ol>` additionally carries an HTML `type` for the alphabetic and
    /// roman styles, a `start` when `[start=N]` is set, and a bare `reversed`
    /// under the `%reversed` option.
    fn olist<'src>(&mut self, block: &'src Block<'src>, list: &'src ListBlock<'src>) {
        let style = olist_style(block, list);

        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute(&format!("olist {style}"), &block.roles())
        ));
        self.block_title(block);

        // The HTML `type` mirrors Asciidoctor's `ORDERED_LIST_KEYWORDS`: arabic
        // needs none, the others carry the matching numbering letter — but only
        // for a declared or dot-derived style (see `olist_emits_type`).
        let type_attr = if olist_emits_type(block, list) {
            match style {
                "loweralpha" => " type=\"a\"",
                "lowerroman" => " type=\"i\"",
                "upperalpha" => " type=\"A\"",
                "upperroman" => " type=\"I\"",
                _ => "",
            }
        } else {
            ""
        };

        // Only an explicit `[start=N]` attribute sets `start`; Asciidoctor emits
        // its value verbatim (an explicit first marker like `7.` does not).
        let start_attr = list
            .attrlist()
            .and_then(|attrlist| attrlist.named_attribute("start"))
            .map(|attr| format!(" start=\"{}\"", escape_attribute(attr.value())))
            .unwrap_or_default();

        let reversed_attr = if block.has_option("reversed") {
            " reversed"
        } else {
            ""
        };

        self.line(&format!(
            "<ol class=\"{}\"{type_attr}{start_attr}{reversed_attr}>",
            escape_attribute(style)
        ));

        for item in list.child_blocks() {
            self.list_item(item, false, false);
        }

        self.line("</ol>");
        self.line("</div>");
    }

    /// A callout list: `<div class="colist arabic">…</div>`, matching
    /// Asciidoctor's `convert_colist`. Each item annotates a numbered callout
    /// (`<1>`, `<.>`, …) in a preceding verbatim block; the callout numbers
    /// themselves are substituted into that block's content by the parser.
    ///
    /// Without icons the items render as a plain `<ol>` of `<li><p>…</p></li>`
    /// (the same item shape as [`list_item`](Self::list_item)). When the
    /// document sets `icons` the list becomes a two-column `<table>` whose
    /// first cell holds the callout number – a Font Awesome `<i
    /// class="conum">`/`<b>` pair under `:icons: font`, or an `<img>` of
    /// the numbered callout icon otherwise – and whose second cell holds
    /// the item's text and any attached blocks.
    fn colist<'src>(&mut self, block: &'src Block<'src>, list: &'src ListBlock<'src>) {
        // Asciidoctor's classes are `['colist', node.style, node.role]`; a
        // callout list's style is always `arabic`.
        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute("colist arabic", &block.roles())
        ));
        self.block_title(block);

        if self.icons_set {
            self.line("<table>");
            for (index, item) in list.child_blocks().filter_map(as_list_item).enumerate() {
                self.colist_row(item, index + 1);
            }
            self.line("</table>");
        } else {
            self.line("<ol>");
            for item in list.child_blocks() {
                self.list_item(item, false, false);
            }
            self.line("</ol>");
        }

        self.line("</div>");
    }

    /// Emits one `<tr>` of an icon-based callout list: the `num`-th callout
    /// icon in the first cell, then the item's principal text (and any attached
    /// blocks) in the second, matching Asciidoctor's `convert_colist` table
    /// branch.
    fn colist_row<'src>(&mut self, list_item: &'src ListItem<'src>, num: usize) {
        // The Font Awesome pair carries the number in `<b>`; the image form
        // points at `{iconsdir}/callouts/{num}.{icontype}` — a port of
        // Asciidoctor's `icon_uri "callouts/#{num}"`, so under `data-uri` (below
        // `Secure`) the callout icon is embedded as a `data:` URI too (see
        // [`image_uri`](Self::image_uri)). The path is attribute-escaped so a
        // hostile `iconsdir`/`icontype` cannot break out of the quoted `src`
        // (Asciidoctor leaves it raw; we harden it, which only diverges when
        // those attributes contain `& < > "`).
        let num_label = if self.icons_font {
            format!("<i class=\"conum\" data-value=\"{num}\"></i><b>{num}</b>")
        } else {
            let target = format!("callouts/{num}.{}", self.icontype);
            let src = escape_attribute(&self.image_uri(&target, &self.iconsdir));
            format!("<img src=\"{src}\" alt=\"{num}\">")
        };

        self.line("<tr>");
        self.line(&format!("<td>{num_label}</td>"));

        // The first attached block is the item's principal text; the remainder
        // (continuation paragraphs, nested lists) render as ordinary blocks
        // appended inside the same `<td>`, matching Asciidoctor's
        // `#{item.text}#{item.blocks? ? LF + item.content : ''}`.
        let mut blocks = list_item.child_blocks();
        let item_line = list_item.span().line();
        let text = blocks
            .next()
            .map(|block| list_principal_content(block, item_line))
            .unwrap_or_default();

        let content = self.render_blocks_to_string(blocks);
        if content.is_empty() {
            self.line(&format!("<td>{text}</td>"));
        } else {
            self.line(&format!("<td>{text}\n{content}</td>"));
        }

        self.line("</tr>");
    }

    /// Renders `blocks` into a standalone string using the current renderer
    /// state, with no trailing newline – the counterpart to Asciidoctor's
    /// `item.content` (its child blocks' output joined by line feeds). Returns
    /// an empty string when there are no blocks.
    fn render_blocks_to_string<'src>(
        &mut self,
        blocks: impl Iterator<Item = &'src Block<'src>>,
    ) -> String {
        let saved = std::mem::take(&mut self.out);
        for block in blocks {
            self.block(block);
        }
        let rendered = std::mem::replace(&mut self.out, saved);
        rendered.strip_suffix('\n').unwrap_or(&rendered).to_string()
    }

    /// A description list (`term:: definition`), matching Asciidoctor's
    /// `convert_dlist`. The block style selects the layout: `qanda` renders a
    /// numbered question-and-answer `<ol>`, `horizontal` a two-column
    /// `<table>`, and every other value (including none) the default
    /// `<dl>`/`<dt>`/`<dd>` structure.
    ///
    /// The parser models each `term::` line as its own list item, so
    /// consecutive terms that share one description surface as term-only items
    /// (no child blocks) followed by the item that carries the description.
    /// [`dlist_entries`] regroups them into Asciidoctor's `[terms, dd]` pairs
    /// before rendering.
    fn dlist<'src>(&mut self, block: &'src Block<'src>, list: &'src ListBlock<'src>) {
        let style = block.declared_style();
        let entries = dlist_entries(list);

        match style {
            Some("qanda") => self.dlist_qanda(block, &entries),
            Some("horizontal") => self.dlist_horizontal(block, list, &entries),
            _ => self.dlist_labeled(block, style, &entries),
        }
    }

    /// The default `<div class="dlist …"><dl>…</dl></div>` layout: each term
    /// becomes a `<dt>` and each description a following `<dd>`. A plain
    /// description list (no style) tags every `<dt>` with `class="hdlist1"`; a
    /// styled one names the style on the wrapper `<div>` and leaves the `<dt>`
    /// bare, matching Asciidoctor.
    fn dlist_labeled(
        &mut self,
        block: &Block<'_>,
        style: Option<&str>,
        entries: &[DlistEntry<'_>],
    ) {
        // `['dlist', style, *roles]` — the style (when present) sits right after
        // `dlist`, ahead of the roles.
        let mut base = String::from("dlist");
        if let Some(style) = style {
            base.push(' ');
            base.push_str(style);
        }

        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute(&base, &block.roles())
        ));
        self.block_title(block);
        self.line("<dl>");

        // A styled list leaves each `<dt>` bare; a plain one tags it `hdlist1`.
        let dt_open = if style.is_some() {
            "<dt>"
        } else {
            "<dt class=\"hdlist1\">"
        };
        for (terms, description) in entries {
            for term in terms {
                self.line(&format!("{dt_open}{term}</dt>"));
            }

            if let Some(item) = description {
                self.line("<dd>");
                self.dlist_body(item);
                self.line("</dd>");
            }
        }

        self.line("</dl>");
        self.line("</div>");
    }

    /// The `qanda` layout: `<div class="qlist qanda"><ol>…</ol></div>`, each
    /// entry an `<li>` whose terms render as emphasized `<p><em>…</em></p>`
    /// questions ahead of the description, matching Asciidoctor.
    fn dlist_qanda(&mut self, block: &Block<'_>, entries: &[DlistEntry<'_>]) {
        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute("qlist qanda", &block.roles())
        ));
        self.block_title(block);
        self.line("<ol>");

        for (terms, description) in entries {
            self.line("<li>");
            for term in terms {
                self.line(&format!("<p><em>{term}</em></p>"));
            }
            if let Some(item) = description {
                self.dlist_body(item);
            }
            self.line("</li>");
        }

        self.line("</ol>");
        self.line("</div>");
    }

    /// The `horizontal` layout: `<div class="hdlist"><table>…</table></div>`,
    /// each entry a `<tr>` with the terms in an `hdlist1` label cell (separated
    /// by `<br>`) and the description in an `hdlist2` cell, matching
    /// Asciidoctor. The `labelwidth`/`itemwidth` attributes emit a
    /// `<colgroup>`, and the `strong` option bolds the label cell.
    fn dlist_horizontal(
        &mut self,
        block: &Block<'_>,
        list: &ListBlock<'_>,
        entries: &[DlistEntry<'_>],
    ) {
        self.line(&format!(
            "<div{}{}>",
            id_attribute(block.id()),
            class_attribute("hdlist", &block.roles())
        ));
        self.block_title(block);
        self.line("<table>");

        // A `labelwidth`/`itemwidth` pair sizes the two columns; either alone
        // still emits both `<col>`s (the other with no width), matching
        // Asciidoctor.
        let labelwidth = dlist_width(list, "labelwidth");
        let itemwidth = dlist_width(list, "itemwidth");
        if labelwidth.is_some() || itemwidth.is_some() {
            self.line("<colgroup>");
            self.line(&dlist_col(labelwidth.as_deref()));
            self.line(&dlist_col(itemwidth.as_deref()));
            self.line("</colgroup>");
        }

        let strong = if block.has_option("strong") {
            " strong"
        } else {
            ""
        };
        for (terms, description) in entries {
            self.line("<tr>");
            self.line(&format!("<td class=\"hdlist1{strong}\">"));
            for (index, term) in terms.iter().enumerate() {
                // Terms after the first are separated by a line break.
                if index > 0 {
                    self.line("<br>");
                }
                self.line(term);
            }
            self.line("</td>");

            self.line("<td class=\"hdlist2\">");
            if let Some(item) = description {
                self.dlist_body(item);
            }
            self.line("</td>");
            self.line("</tr>");
        }

        self.line("</table>");
        self.line("</div>");
    }

    /// Emits a description entry's body — its principal text as a bare `<p>`
    /// (Asciidoctor's `dd.text`) followed by any attached blocks
    /// (`dd.content`), matching Asciidoctor's `convert_dlist`.
    ///
    /// Asciidoctor *folds* the first block into the item's principal text when
    /// it is a paragraph adjacent to the term — same-line text, or the first
    /// paragraph that follows without a list-continuation (`+`) line between
    /// (`Parser#fold_first` under `content_adjacent`). A first block that is
    /// not a paragraph, or that was explicitly attached by a `+` continuation,
    /// is not folded: the item then has no principal text and every child
    /// renders as an attached block.
    fn dlist_body(&mut self, description: &ListItem<'_>) {
        let blocks: Vec<&Block<'_>> = description.child_blocks().collect();

        // The first block folds into the principal text only when it is a
        // paragraph the term did not attach with a `+` continuation. An entry
        // reaching here always has at least one block (it is why the entry has a
        // description at all), so `first` drives the decision without a separate
        // empty case.
        let foldable = blocks.first().is_some_and(|first| {
            first.resolved_context().as_ref() == "paragraph"
                && !continuation_before_first_child(description, first)
        });

        let attached = if foldable {
            let text = list_principal_content(blocks[0], description.span().line());
            if !text.is_empty() {
                self.line(&format!("<p>{text}</p>"));
            }
            &blocks[1..]
        } else {
            &blocks[..]
        };

        for &block in attached {
            self.block(block);
        }
    }

    /// Emits one `<li>…</li>` for a list item: the principal text as a bare
    /// `<p>`, followed by any attached blocks (continuation paragraphs, nested
    /// lists), matching Asciidoctor's `convert_ulist`/`convert_olist` item
    /// loop.
    ///
    /// An item's own id/roles decorate the `<li>`. When `checklist` is set and
    /// the item carries a checkbox, the principal text is prefixed with the
    /// checkbox marker selected by [`checkbox_marker`](Self::checkbox_marker).
    fn list_item<'src>(&mut self, item: &'src Block<'src>, checklist: bool, interactive: bool) {
        let Block::ListItem(list_item) = item else {
            return;
        };

        // `<li id="…" class="…">`, `<li class="…">`, or a bare `<li>`, following
        // Asciidoctor: the id (if any) comes first, then the item's roles.
        let li_open = if let Some(id) = item.id() {
            format!(
                "<li id=\"{}\"{}>",
                escape_attribute(id),
                class_attribute("", &item.roles())
            )
        } else if !item.roles().is_empty() {
            format!("<li{}>", class_attribute("", &item.roles()))
        } else {
            "<li>".to_string()
        };
        self.line(&li_open);

        // The first attached block is the item's principal text, emitted as a
        // bare `<p>`; the remainder render as ordinary nested blocks. When the
        // principal text is present but renders empty (an `{empty}` reference),
        // the parser drops that node from the child blocks and flags it here:
        // Asciidoctor still emits the empty `<p></p>` ahead of the attached
        // blocks, so the first child block stays an attached block.
        let mut blocks = list_item.child_blocks();
        let principal = if list_item.has_empty_principal_text() {
            Cow::Borrowed("")
        } else {
            let item_line = item.span().line();
            blocks
                .next()
                .map(|block| list_principal_content(block, item_line))
                .unwrap_or_default()
        };

        match (checklist, list_item.checkbox()) {
            (true, Some(checked)) => {
                let marker = self.checkbox_marker(checked, interactive);
                self.line(&format!("<p>{marker}{principal}</p>"));
            }
            _ => self.line(&format!("<p>{principal}</p>")),
        }

        for block in blocks {
            self.block(block);
        }

        self.line("</li>");
    }

    /// The checkbox glyph that prefixes a checklist item's text, mirroring
    /// Asciidoctor's `convert_ulist`: an interactive `<input>` under the
    /// `%interactive` option, a Font Awesome icon when `:icons: font` is set,
    /// and the plain-text ballot-box entities otherwise. The trailing space is
    /// part of the marker.
    fn checkbox_marker(&self, checked: bool, interactive: bool) -> &'static str {
        match (interactive, self.icons_font, checked) {
            (true, _, true) => "<input type=\"checkbox\" data-item-complete=\"1\" checked> ",
            (true, _, false) => "<input type=\"checkbox\" data-item-complete=\"0\"> ",
            (false, true, true) => "<i class=\"fa fa-check-square-o\"></i> ",
            (false, true, false) => "<i class=\"fa fa-square-o\"></i> ",
            (false, false, true) => "&#10003; ",
            (false, false, false) => "&#10063; ",
        }
    }

    /// A table: `<table class="tableblock …">` wrapping an optional
    /// `<caption>`, a `<colgroup>`, and `<thead>`/`<tbody>`/`<tfoot>` sections,
    /// mirroring Asciidoctor's `html5` `convert_table`.
    fn table<'src>(&mut self, block: &'src Block<'src>, table: &'src TableBlock<'src>) {
        // The class list follows Asciidoctor's order exactly: `tableblock`, the
        // frame and grid classes, an optional stripes class, then the
        // width-driven class (`fit-content`/`stretch`) or an inline width style,
        // then an optional float class, then the block's roles.
        let frame = match table.frame() {
            Frame::All => "all",
            Frame::Ends => "ends",
            Frame::Sides => "sides",
            Frame::None => "none",
        };
        let grid = match table.grid() {
            Grid::All => "all",
            Grid::Rows => "rows",
            Grid::Cols => "cols",
            Grid::None => "none",
        };
        let mut classes = format!("tableblock frame-{frame} grid-{grid}");

        if let Some(stripes) = table_stripes_class(table) {
            classes.push(' ');
            classes.push_str(&stripes);
        }

        // `autowidth` sizes the table (and columns) to content; `fit-content`
        // marks that in HTML. Otherwise a table at the full content width gets
        // `stretch`, and any other width becomes an inline `style`.
        let autowidth = table.is_autowidth();
        let has_width = table.width().is_some();
        let mut style_attr = String::new();
        if autowidth && !has_width {
            classes.push_str(" fit-content");
        } else {
            let tablewidth = table.width().unwrap_or(100);
            if tablewidth == 100 {
                classes.push_str(" stretch");
            } else {
                style_attr = format!(" style=\"width: {tablewidth}%;\"");
            }
        }

        if let Some(float) = table
            .attrlist()
            .and_then(|a| a.named_attribute("float"))
            .map(|attr| attr.value())
        {
            classes.push(' ');
            classes.push_str(float);
        }

        for role in block.roles() {
            classes.push(' ');
            classes.push_str(role);
        }

        self.line(&format!(
            "<table{}{}{}>",
            id_attribute(block.id()),
            class_attribute(&classes, &[]),
            style_attr
        ));

        // A titled table is captioned: the ready-made caption prefix
        // (e.g. `"Table 1. "`) sits ahead of the title text.
        if let Some(title) = block.title() {
            let caption = block.caption().unwrap_or_default();
            self.line(&format!(
                "<caption class=\"title\">{caption}{title}</caption>"
            ));
        }

        // Asciidoctor emits the colgroup and rows only when the table has at
        // least one row; a table whose rows were all dropped (e.g. an overrun)
        // is just an empty `<table></table>`.
        let rowcount = table.header_row().is_some() as usize
            + table.body_rows().len()
            + table.footer_row().is_some() as usize;
        if rowcount > 0 {
            self.colgroup(table, autowidth);

            if let Some(header) = table.header_row() {
                self.line("<thead>");
                self.table_row(header, TableSection::Head);
                self.line("</thead>");
            }
            if !table.body_rows().is_empty() {
                self.line("<tbody>");
                for row in table.body_rows() {
                    self.table_row(row, TableSection::Body);
                }
                self.line("</tbody>");
            }
            if let Some(footer) = table.footer_row() {
                self.line("<tfoot>");
                self.table_row(footer, TableSection::Foot);
                self.line("</tfoot>");
            }
        }

        self.line("</table>");
    }

    /// Emits the `<colgroup>` of `<col>` elements. An autowidth table (or an
    /// autowidth column) emits a bare `<col>`; every other column carries its
    /// computed percentage width as an inline style.
    fn colgroup(&mut self, table: &TableBlock<'_>, autowidth: bool) {
        self.line("<colgroup>");
        if autowidth {
            for _ in table.columns() {
                self.line("<col>");
            }
        } else {
            let pcwidths = column_pcwidths(table.columns());
            for (col, pcwidth) in table.columns().iter().zip(pcwidths) {
                if col.is_autowidth() {
                    self.line("<col>");
                } else {
                    self.line(&format!("<col style=\"width: {pcwidth}%;\">"));
                }
            }
        }
        self.line("</colgroup>");
    }

    /// Emits one `<tr>` and its cells, tagging the section so header cells (and
    /// the first row of a header section) render as `<th>`.
    fn table_row<'src>(&mut self, row: &'src TableRow<'src>, section: TableSection) {
        self.line("<tr>");
        for cell in row.cells() {
            self.table_cell(cell, section);
        }
        self.line("</tr>");
    }

    /// Emits one table cell. The tag is `<th>` for a header-section cell or a
    /// cell in a `h`-styled column, `<td>` otherwise; the content is rendered
    /// per the cell's [style](ColumnStyle).
    fn table_cell<'src>(&mut self, cell: &'src TableCell<'src>, section: TableSection) {
        let is_head = section == TableSection::Head;

        // The content variant follows the cell's style: an AsciiDoc (`a`) cell
        // carries block content, every other style carries inline `Simple`
        // content. Matching the variant (rather than the style) keeps the two
        // impossible pairings — an AsciiDoc-content header cell, a Simple-content
        // AsciiDoc cell — off the map entirely.
        let content = match cell.content() {
            // An AsciiDoc cell never appears in the header row (the header is
            // forced to the default style), so this is always a body cell.
            TableCellContent::AsciiDoc(ad) => format!(
                "<div class=\"content\">{}</div>",
                render_cell_document(
                    ad.blocks(),
                    ad.title(),
                    ad.is_inline(),
                    ad.footnotes(),
                    // The cell is a standalone nested document, so its TOC is
                    // resolved from its own `toc*` attributes, not inherited
                    // from the parent (Asciidoctor's `AsciiDocCell` behavior).
                    CellTocConfig {
                        mode: ad.toc_mode(),
                        levels: ad.toc_levels(),
                        // The cell's `sectnumlevels` is read from its own nested
                        // document (default 3), like the outline module's
                        // `attribute_usize`; the cell type is un-nameable here,
                        // so this reads it inline. `sectnumlevels` always
                        // resolves to a `Value` (its default is `3`), so the
                        // non-`Value` arm is a defensive fallback (hence
                        // uncovered). The parser currently surfaces only the
                        // default here, never a cell-set override (see
                        // asciidoc-parser#1092).
                        sectnumlevels: match ad.attribute_value("sectnumlevels") {
                            InterpretedValue::Value(value) => value.parse().unwrap_or(3),
                            _ => 3,
                        },
                        title: ad.toc_title().to_string(),
                        class: ad.toc_class().to_string(),
                    },
                    CellRenderConfig {
                        icons_set: self.icons_set,
                        icons_font: self.icons_font,
                        iconsdir: self.iconsdir.clone(),
                        icontype: self.icontype.clone(),
                        imagesdir: self.imagesdir.clone(),
                        asset_uri_scheme: self.asset_uri_scheme.clone(),
                        doc_tabsize: self.doc_tabsize,
                        source_indent: self.source_indent,
                        prewrap: self.prewrap,
                        source_highlighter: self.source_highlighter,
                        // The cell inherits the enclosing document's
                        // `source-language`, matching Asciidoctor: a value the
                        // parent set wins over a cell-local override or unset
                        // (verified against 2.0.26). Like every other
                        // document-scoped setting here, it is not re-resolved
                        // from the cell's own attribute entries.
                        source_language: self.source_language.clone(),
                        section_anchors: self.section_anchors,
                        sectlinks: self.sectlinks,
                        stem_type: self.stem_type,
                        cellbgcolor: self.cellbgcolor.clone(),
                        svg_below_secure: self.svg_below_secure,
                        data_uri: self.data_uri,
                        svg_source: self.svg_source.clone(),
                    },
                )
            ),
            TableCellContent::Simple(simple) => {
                if is_head {
                    // A header cell is plain inline text, never paragraph-split
                    // or style-wrapped.
                    simple.rendered().to_string()
                } else if cell.style() == ColumnStyle::Literal {
                    format!(
                        "<div class=\"literal\"><pre>{}</pre></div>",
                        simple.rendered()
                    )
                } else {
                    cell_paragraphs(cell, simple.rendered())
                }
            }
        };

        let tag = if is_head || cell.style() == ColumnStyle::Header {
            "th"
        } else {
            "td"
        };
        let h_align = match cell.h_align() {
            HorizontalAlignment::Left => "left",
            HorizontalAlignment::Center => "center",
            HorizontalAlignment::Right => "right",
        };
        let v_align = match cell.v_align() {
            VerticalAlignment::Top => "top",
            VerticalAlignment::Middle => "middle",
            VerticalAlignment::Bottom => "bottom",
        };
        let colspan = if cell.colspan() > 1 {
            format!(" colspan=\"{}\"", cell.colspan())
        } else {
            String::new()
        };
        let rowspan = if cell.rowspan() > 1 {
            format!(" rowspan=\"{}\"", cell.rowspan())
        } else {
            String::new()
        };

        // Asciidoctor stamps `style="background-color: …;"` on every cell when
        // the `cellbgcolor` document attribute is set, reading its value as it
        // renders each cell. This crate honors the document-attribute form
        // (uniform across the table); see the `cellbgcolor` field for why the
        // inline per-cell form is out of scope. The value is document-controlled
        // and lands inside a quoted attribute, so escape it to keep a stray `"`
        // from breaking out and injecting markup (as we do for author fields).
        // This is a deliberate divergence: Asciidoctor interpolates the value
        // raw, but escaping only differs for values that would be malformed
        // anyway, never for a real color.
        let style = match &self.cellbgcolor {
            Some(color) => format!(" style=\"background-color: {};\"", escape_attribute(color)),
            None => String::new(),
        };

        self.line(&format!(
            "<{tag} class=\"tableblock halign-{h_align} valign-{v_align}\"{colspan}{rowspan}{style}>{content}</{tag}>"
        ));
    }

    /// Emits the inner content shared by wrapper blocks (open, quote,
    /// admonition): a compound block recurses over its nested blocks, while a
    /// simple block emits its rendered content on its own line, unwrapped (no
    /// `<p>`), matching Asciidoctor.
    fn wrapped_content<'src>(&mut self, block: &'src Block<'src>) {
        if block.content_model() == ContentModel::Compound {
            self.blocks(block.child_blocks());
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
        let title = self.section_heading_title(block, section);

        if section.section_type() == SectionType::Discrete {
            // Asciidoctor renders a discrete heading as a bare `<hN>` whose
            // leading class is the style keyword that declared it — `float` or
            // `discrete` — followed by any roles, e.g. `class="float role"`.
            // The parser exposes the keyword through `declared_style`; it
            // defaults to `discrete` for the rare case one is not recorded.
            let style = block.declared_style().unwrap_or("discrete");
            self.line(&format!(
                "<h{heading_level}{}{}>{title}</h{heading_level}>",
                id_attribute(id),
                class_attribute(style, &block.roles())
            ));
            return;
        }

        // `sectlinks` / `sectanchors` inject `<a>` elements into a non-discrete
        // heading, keyed off the section's own id.
        let title = self.decorate_section_heading(&title, id);

        if level == 0 {
            // A level-0 section — a book part, or a document title demoted by
            // block metadata above it — renders as a bare `<h1 class="sect0">`:
            // Asciidoctor puts the class and id on the heading itself and emits
            // the section's blocks directly after it, with no wrapping `<div>`
            // and no `sectionbody`.
            self.line(&format!(
                "<h{heading_level}{}{}>{title}</h{heading_level}>",
                id_attribute(id),
                class_attribute("sect0", &block.roles())
            ));
            self.blocks(block.child_blocks());
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
            self.blocks(block.child_blocks());
            self.line("</div>");
        } else {
            self.blocks(block.child_blocks());
        }

        self.line("</div>");
    }

    /// Builds a section heading's displayed text, applying Asciidoctor's
    /// `captioned_title` precedence: a captioned section (an appendix carries
    /// an `"Appendix A: "` / `"A. "` caption prefix) shows its caption
    /// ahead of the title; otherwise a numbered section (`sectnums`) shows
    /// its section number as a `"1.1. "` prefix; otherwise the bare title.
    /// The parser assigns the number and caption, so the renderer only lays
    /// out the prefix. The title itself already has inline substitutions
    /// applied.
    fn section_heading_title<'src>(
        &self,
        block: &'src Block<'src>,
        section: &'src SectionBlock<'src>,
    ) -> String {
        let title = section.section_title();

        if let Some(caption) = block.caption() {
            format!("{caption}{title}")
        } else if let Some(number) = section.section_number() {
            format!("{number}. {title}")
        } else {
            title.to_string()
        }
    }

    /// Applies the `sectlinks` and `sectanchors` decorations to a section
    /// heading's title, matching Asciidoctor's `convert_section`. Both toggles
    /// key off the section's own `id`; a section without an id is left
    /// untouched. `sectlinks` wraps the title text in `<a class="link">`, and
    /// `sectanchors` then adds an `<a class="anchor">` self-link before (or,
    /// with `:sectanchors: after`, after) the whole thing.
    fn decorate_section_heading(&self, title: &str, id: Option<&str>) -> String {
        let Some(id) = id else {
            return title.to_string();
        };

        let href = escape_attribute(id);

        // `sectlinks` wraps the title text in a self-link. A leading run of
        // supplemental inline anchors (`[[fu]]` → `<a id="fu"></a>`) must stay
        // a *sibling before* the link rather than nesting inside it — an `<a>`
        // cannot contain another `<a>` under html5 tree construction — so the
        // link opens only after those anchors.
        let mut title = if self.sectlinks {
            let split = leading_anchors_len(title);
            let (leading, text) = title.split_at(split);

            format!("{leading}<a class=\"link\" href=\"#{href}\">{text}</a>")
        } else {
            title.to_string()
        };

        match self.section_anchors {
            SectionAnchors::Before => {
                title = format!("<a class=\"anchor\" href=\"#{href}\"></a>{title}");
            }
            SectionAnchors::After => {
                title.push_str(&format!("<a class=\"anchor\" href=\"#{href}\"></a>"));
            }
            SectionAnchors::None => {}
        }

        title
    }

    /// The preamble: content between the doctitle and the first section,
    /// wrapped as `<div id="preamble"><div
    /// class="sectionbody">…</div></div>`.
    fn preamble<'src>(&mut self, block: &'src Block<'src>) {
        self.line("<div id=\"preamble\">");
        self.line("<div class=\"sectionbody\">");
        self.blocks(block.child_blocks());
        self.line("</div>");

        // A `preamble`-placed TOC follows the preamble's `sectionbody`, still
        // inside the `<div id="preamble">`, matching Asciidoctor's
        // `convert_preamble`.
        if self.toc_mode == TocMode::Preamble && !self.toc_html.is_empty() {
            self.line(&self.toc_html.clone());
        }

        self.line("</div>");
    }

    /// Emits the `toc::[]` block macro's table of contents (Asciidoctor's
    /// `convert_toc`), the `macro` placement's counterpart to the header /
    /// preamble TOC built by [`render_toc`]. The macro carries its own
    /// per-TOC overrides: `id` (which also renames the title block's id to
    /// `<id>title`), a block `title` (over `toc-title`), `levels` (over
    /// `toclevels`), and a `role` (over `toc-class`).
    ///
    /// When the document does not defer its TOC to a macro — `toc` is off,
    /// `toc-placement` is not `macro`, or there are no sections to list — the
    /// macro emits Asciidoctor's `<!-- toc disabled -->` placeholder instead.
    fn toc_macro<'src>(&mut self, block: &'src Block<'src>, toc: &'src TocBlock<'src>) {
        // The macro's `levels` attribute overrides `toclevels` for this TOC
        // only.
        let macro_levels = toc
            .macro_attrlist()
            .named_attribute("levels")
            .and_then(|attr| attr.value().trim().parse::<usize>().ok());

        // The macro renders a TOC only when this document defers to it. The
        // outline and the default title/class come from the top-level
        // `Document` or, in a nested AsciiDoc cell sub-renderer (which holds no
        // `Document`), from the cell's own TOC context. Anything else falls
        // through to the placeholder below.
        //
        // A `Macro`-mode renderer always has exactly one of `document` /
        // `cell_toc`, so the final `else` is a defensive, unreachable
        // catch-all (hence uncovered) that keeps the expression exhaustive.
        let (outline, default_title, default_class) = if self.toc_mode != TocMode::Macro {
            (String::new(), String::new(), String::new())
        } else if let Some(document) = self.document {
            let mut options = crate::OutlineOptions::new();
            if let Some(levels) = macro_levels {
                options = options.toclevels(levels);
            }

            (
                crate::outline::render_outline(document, &options),
                document.toc_title().to_string(),
                document.toc_class().to_string(),
            )
        } else if let Some(cell_toc) = self.cell_toc.as_ref() {
            let levels = macro_levels.unwrap_or(cell_toc.toclevels);

            (
                crate::outline::render_outline_blocks(
                    cell_toc.blocks,
                    levels,
                    cell_toc.sectnumlevels,
                ),
                cell_toc.title.clone(),
                cell_toc.class.clone(),
            )
        } else {
            (String::new(), String::new(), String::new())
        };

        // No sections yields an empty outline, which is Asciidoctor's
        // `doc.sections?` guard — emit the placeholder comment.
        if outline.is_empty() {
            self.line("<!-- toc disabled -->");
            return;
        }

        // An explicit `id` renames both the container and — suffixed with
        // `title` — the title block; absent, they fall back to `toc` /
        // `toctitle`.
        let (id_attr, title_id_attr) = match block.id() {
            Some(id) => (
                format!(" id=\"{}\"", escape_attribute(id)),
                format!(" id=\"{}title\"", escape_attribute(id)),
            ),
            None => (" id=\"toc\"".to_string(), " id=\"toctitle\"".to_string()),
        };

        // A block title overrides `toc-title`; the block's role(s) override
        // `toc-class`.
        let title = block.title().map(str::to_string).unwrap_or(default_title);

        // The role can arrive two ways: as a `role=` attribute inside the macro
        // (`toc::[role=…]`) or as the block's role shorthand on the line above
        // (`[.role]`). Asciidoctor folds both into `node.role`, with the macro's
        // own `role=` winning; an absent role falls back to `toc-class`.
        let class = toc
            .macro_attrlist()
            .named_attribute("role")
            .map(|attr| attr.value().to_string())
            .or_else(|| {
                let roles = block.roles();
                (!roles.is_empty()).then(|| roles.join(" "))
            })
            .unwrap_or(default_class);

        self.line(&format!(
            "<div{id_attr} class=\"{}\">",
            escape_attribute(&class)
        ));
        self.line(&format!(
            "<div{title_id_attr} class=\"title\">{title}</div>"
        ));
        self.line(&outline);
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

    /// Builds the `src` (or `data`) reference for an image or image-mode icon,
    /// a port of Asciidoctor's `AbstractNode#image_uri`.
    ///
    /// When the document sets `data-uri` and the safe mode is below `Secure`,
    /// the referenced file is embedded as a base64 `data:` URI (see
    /// [`generate_data_uri`](Self::generate_data_uri)) instead of linked. Two
    /// cases are *not* embedded, matching Asciidoctor when `allow-uri-read` is
    /// off (remote reads are a non-goal here — see the crate's `allow-uri-read`
    /// non-goal): a `target` that is itself a URI, and a relative `target`
    /// under a URI `asset_dir` (e.g. an `imagesdir` pointing at a CDN).
    /// Both fall through to the plain [`media_uri`] reference, exactly as
    /// when `data-uri` is unset. `asset_dir` is the asset directory the
    /// target resolves against — `imagesdir` for an image, `iconsdir` for
    /// an icon.
    fn image_uri(&self, target: &str, asset_dir: &str) -> String {
        // `data-uri` embedding applies only below `Secure` (the same gate as the
        // SVG modes), and never to a URI target or under a URI asset directory —
        // those would need a network read this crate does not perform, so they
        // pass through unchanged (Asciidoctor's no-`allow-uri-read` behavior).
        if self.data_uri
            && self.svg_below_secure
            && !looks_like_uri(target)
            && !looks_like_uri(asset_dir)
        {
            return self.generate_data_uri(target, asset_dir);
        }

        media_uri(target, asset_dir)
    }

    /// Reads the local image at `target` (resolved against `asset_dir` within
    /// the base directory) and returns a base64 `data:` URI embedding it — a
    /// port of Asciidoctor's `AbstractNode#generate_data_uri`.
    ///
    /// The MIME type is derived from the target's extension
    /// ([`data_uri_mimetype`]). The file is resolved the way Asciidoctor's
    /// `normalize_system_path` resolves it — `target` against `asset_dir`,
    /// confined to the base directory's jail under `safe`/`server` — and read
    /// as raw bytes, using the base directory and safe mode carried by
    /// [`svg_source`](Self::svg_source). An unreadable target (missing, outside
    /// the jail, or — as when no [`SvgSource`] anchors the lookup —
    /// unresolvable) yields an *empty* data URI (`data:<mime>;base64,`),
    /// matching Asciidoctor's fallback for an image it cannot embed.
    /// (Asciidoctor also logs a warning there; this renderer, which has no
    /// warning channel, matches only the output.)
    fn generate_data_uri(&self, target: &str, asset_dir: &str) -> String {
        let mimetype = data_uri_mimetype(target);

        let bytes = self.svg_source.as_ref().and_then(|svg| {
            let path =
                crate::include_handler::resolve_in_dir(&svg.base_dir, svg.safe, asset_dir, target);
            match crate::include_handler::read_confined_bytes(&svg.base_dir, svg.safe, &path) {
                crate::include_handler::ReadBytesOutcome::Read(bytes) => Some(bytes),
                crate::include_handler::ReadBytesOutcome::NotFound
                | crate::include_handler::ReadBytesOutcome::NotReadable => None,
            }
        });

        match bytes {
            Some(bytes) => format!("data:{mimetype};base64,{}", crate::base64::encode(&bytes)),
            None => format!("data:{mimetype};base64,"),
        }
    }

    /// `<div class="imageblock …"><div class="content"><img
    /// …></div>[title]</div>`, matching Asciidoctor's `convert_image` for a
    /// block image (`image::…[]`).
    ///
    /// The `<img>` carries the resolved `src` (the target joined with
    /// `imagesdir`, or — under `data-uri` below `Secure` — the image embedded
    /// as a base64 `data:` URI; see [`image_uri`](Self::image_uri)), the
    /// `alt` text (an explicit value, else the target's basename with
    /// `_`/`-` turned into spaces), and any `width`/`height`. A `link`
    /// wraps the image in `<a class="image">`. The wrapper's classes are
    /// `imageblock`, then a `float` role, then a `text-<align>` class, then
    /// the block's roles (see [`media_roles`](Self::media_roles)); a titled
    /// image gains a captioned `Figure N.` title *after* its content
    /// (unlike most blocks, whose title precedes their content).
    ///
    /// The remaining advanced image mode builds on this baseline in its own
    /// issue: icons (#50). Below `Secure`, an interactive SVG target
    /// (`opts=interactive`) renders as an `<object>`, and an `opts=inline` SVG
    /// embeds the file's `<svg>` contents directly (see the SVG handling
    /// below); at `Secure` or above both fall back to a plain `<img>`.
    /// Under `data-uri` (below `Secure`) the `<img>`/`<object>`
    /// `src`/`data` is the image embedded as a base64 `data:` URI (see
    /// [`image_uri`](Self::image_uri)). A `title=` given *inside* the macro
    /// is not yet promoted to the figure caption (the parser surfaces it
    /// only as a macro attribute); use the `.Title` block line.
    fn image<'src>(&mut self, block: &'src Block<'src>, media: &'src MediaBlock<'src>) {
        let macro_attrs = media.macro_attrlist();

        // Asciidoctor merges the block attribute line and the macro's own
        // attribute list into one set, with the macro list winning. Named
        // lookups therefore try the macro list first and fall back to the block
        // list; the positional `alt`/`width`/`height` live only in the macro
        // list (the block line has no positional slots), so those fall back to a
        // same-named attribute there.
        let named = |name: &str| -> Option<&str> {
            macro_attrs
                .named_attribute(name)
                .or_else(|| block.attrlist().and_then(|a| a.named_attribute(name)))
                .map(|attr| attr.value())
        };

        let positional = |name: &str, n: usize| -> Option<&str> {
            macro_attrs
                .named_or_positional_attribute(name, n)
                .or_else(|| block.attrlist().and_then(|a| a.named_attribute(name)))
                .map(|attr| attr.value())
        };

        let target = media.resolved_target();
        let src = self.image_uri(target, &self.imagesdir);

        // Asciidoctor's `attributes['alt'] ||= style || default-alt`: an explicit
        // alt from the macro (or block attribute line) wins; failing that, the
        // block attribute line's *style* (its first positional, e.g. `[Tiger]`)
        // stands in as the alt; failing that, the alt is auto-derived from the
        // target's basename. [`subbed_alt`] applies the special-character (and,
        // for an explicit alt, replacement) substitutions; `alt_attr`
        // additionally attribute-encodes the `"` for use inside `alt="…"`, while
        // `alt_span` stays raw for a `<span class="alt">` (matching
        // Asciidoctor's `encode_attribute_value node.alt` vs. bare `node.alt`).
        let explicit_alt = positional("alt", 1).or_else(|| block.declared_style());
        let alt_span = subbed_alt(explicit_alt, target);
        let alt_attr = alt_span.replace('"', "&quot;");

        let mut dimensions = String::new();

        if let Some(width) = positional("width", 2) {
            dimensions.push_str(&format!(" width=\"{}\"", escape_attribute(width)));
        }

        if let Some(height) = positional("height", 3) {
            dimensions.push_str(&format!(" height=\"{}\"", escape_attribute(height)));
        }

        // An SVG target (`format=svg`, or a target *containing* `.svg`) can be
        // referenced two security-sensitive ways, each enabled only below the
        // `Secure` safe mode (at `Secure` or above — the API default — it falls
        // through to a plain `<img>`), matching Asciidoctor's `convert_image`:
        //
        // * `opts=inline` embeds the SVG *file's contents* directly as an `<svg>`
        //   element (read through [`SvgSource`], prepared by
        //   [`svg_file_handler::prepare_inline_svg`]); when the file cannot be read it
        //   falls back to a `<span class="alt">`, as Asciidoctor does.
        // * `opts=interactive` embeds a live `<object>` so its scripting and links stay
        //   active, with a `fallback` image (or the alt text) nested.
        //
        // `inline` takes precedence over `interactive`, matching Asciidoctor's
        // `if node.option? 'inline' … elsif node.option? 'interactive'`.
        //
        // The target test is a deliberate port of Asciidoctor's case-sensitive
        // substring check (`target.include? '.svg'`), quirks included: a
        // `.contains` (not an extension match) treats `chart.svg.png` as an SVG,
        // and an uppercase `.SVG` is *not* matched. Both are verified to match
        // Asciidoctor 2.0.26, so they are kept rather than "corrected".
        let is_svg = named("format") == Some("svg") || target.contains(".svg");

        let has_option = |name: &str| {
            macro_attrs.has_option(name) || block.attrlist().is_some_and(|a| a.has_option(name))
        };

        let inline = has_option("inline");
        let interactive = has_option("interactive");

        let mut img = if is_svg && self.svg_below_secure && inline {
            // Embed the SVG file's contents as an `<svg>`. The file is read from
            // its plain web path (`media_uri`), *not* `src`: under `data-uri`,
            // `src` is a `data:` URI, but `opts=inline` embeds the file contents
            // regardless (Asciidoctor reads the file for inline SVG independent
            // of `data-uri`). When it can't be read (no base directory, a
            // missing/unreadable file, or empty contents), fall back to the alt
            // text — raw, as Asciidoctor emits it.
            self.inline_svg(
                &media_uri(target, &self.imagesdir),
                positional("width", 2),
                positional("height", 3),
            )
            .unwrap_or_else(|| format!("<span class=\"alt\">{alt_span}</span>"))
        } else if is_svg && self.svg_below_secure && interactive {
            // The content nested inside the `<object>`, shown when a user agent
            // can't render it: a `fallback` image if one is supplied, otherwise
            // the alt text (emitted raw, as Asciidoctor does).
            let fallback = match named("fallback") {
                Some(fallback) => format!(
                    "<img src=\"{}\" alt=\"{alt_attr}\"{dimensions}>",
                    escape_attribute(&self.image_uri(fallback, &self.imagesdir)),
                ),

                None => format!("<span class=\"alt\">{alt_span}</span>"),
            };

            format!(
                "<object type=\"image/svg+xml\" data=\"{}\"{dimensions}>{fallback}</object>",
                escape_attribute(&src),
            )
        } else {
            format!(
                "<img src=\"{}\" alt=\"{alt_attr}\"{dimensions}>",
                escape_attribute(&src),
            )
        };

        // A `link` wraps the image in an anchor whose `href` is the link value
        // verbatim (Asciidoctor 2.0.26 does not resolve `link=self` to the
        // image's own `src`). Its `append_link_constraint_attrs` adds
        // `target`/`rel` from `window` and the `nofollow`/`noopener` options.
        if let Some(link) = named("link") {
            let window = named("window");

            let nofollow = has_option("nofollow");
            let noopener = has_option("noopener");

            img = format!(
                "<a class=\"image\" href=\"{}\"{}>{img}</a>",
                escape_attribute(link),
                link_constraint_attrs(window, nofollow, noopener),
            );
        }

        // The wrapper classes follow Asciidoctor's order: `imageblock`, then the
        // `float` role, then a `text-<align>` class, then the block's roles
        // (a macro `role=` overrides the block line's roles — see `media_roles`).
        let mut classes = String::from("imageblock");

        if let Some(float) = named("float") {
            classes.push(' ');
            classes.push_str(&escape_attribute(float));
        }

        if let Some(align) = named("align") {
            classes.push_str(&format!(" text-{}", escape_attribute(align)));
        }

        for role in self.media_roles(block, media) {
            classes.push(' ');
            classes.push_str(&escape_attribute(role));
        }

        self.line(&format!(
            "<div{} class=\"{classes}\">",
            id_attribute(block.id())
        ));
        self.line("<div class=\"content\">");
        self.line(&img);
        self.line("</div>");

        // A block image places its captioned title *after* the content div.
        if let Some(title) = block.title() {
            let caption = block.caption().unwrap_or_default();
            self.line(&format!("<div class=\"title\">{caption}{title}</div>"));
        }

        self.line("</div>");
    }

    /// Reads and prepares the SVG file at `src` for inline embedding in a block
    /// image (`image::…[opts=inline]`), or `None` when the SVG cannot be
    /// embedded — no base directory anchors the read (the plain string
    /// [`convert`](crate::convert)), the file is missing/unreadable, or its
    /// contents are empty — so the caller falls back to the alt text.
    ///
    /// `src` is the resolved web-path `src` (the target joined with
    /// `imagesdir`), the same value passed to the parser's inline-image
    /// handler. It is resolved against the [`SvgSource`] base directory and
    /// confined by the safe mode's jail ([`svg_file_handler::read_svg_file`]),
    /// then prepared — preamble stripped and dimensions rewritten when
    /// `width`/`height` are given — by
    /// [`svg_file_handler::prepare_inline_svg`].
    fn inline_svg(&self, src: &str, width: Option<&str>, height: Option<&str>) -> Option<String> {
        let source = self.svg_source.as_ref()?;
        let raw = svg_file_handler::read_svg_file(&source.base_dir, source.safe, src)?;
        svg_file_handler::prepare_inline_svg(raw, width, height)
    }

    /// The role(s) placed on a media block's wrapper, matching Asciidoctor's
    /// `node.role`. A `role=` attribute *inside* the macro
    /// (`video::x[role=…]`) wins outright over the block's shorthand role(s)
    /// on the line above (`[.role]`), rather than merging with them.
    fn media_roles<'src>(
        &self,
        block: &'src Block<'src>,
        media: &'src MediaBlock<'src>,
    ) -> Vec<&'src str> {
        match media.macro_attrlist().named_attribute("role") {
            Some(role) => role.value().split_whitespace().collect(),
            None => block.roles(),
        }
    }

    /// The URI-scheme prefix for a hosted video embed — `"https:"` for the
    /// default `asset-uri-scheme`, or `""` when it is empty (yielding a
    /// scheme-relative `//…` URL), mirroring Asciidoctor's `convert_video`.
    fn asset_uri_scheme_prefix(&self) -> String {
        if self.asset_uri_scheme.is_empty() {
            String::new()
        } else {
            format!("{}:", self.asset_uri_scheme)
        }
    }

    /// The `#t=<start>,<end>` media fragment appended to a self-hosted
    /// `<video>`/`<audio>` `src`, or an empty string when neither bound is
    /// given. Mirrors Asciidoctor's `#t=#{start || ''}#{end ? ",#{end}" : ''}`,
    /// so a lone `end` yields `#t=,<end>`.
    fn time_anchor(start: Option<&str>, end: Option<&str>) -> String {
        if start.is_none() && end.is_none() {
            return String::new();
        }

        format!(
            "#t={}{}",
            start.unwrap_or_default(),
            end.map(|e| format!(",{e}")).unwrap_or_default()
        )
    }

    /// `<div class="videoblock …">` wrapping either a self-hosted `<video>` or,
    /// when the `poster` attribute names a hosted service, a `youtube`/`vimeo`
    /// `<iframe>` embed — a port of Asciidoctor's `convert_video`.
    fn video<'src>(&mut self, block: &'src Block<'src>, media: &'src MediaBlock<'src>) {
        let attrs = media.macro_attrlist();

        // The `float` and `text-<align>` classes sit between the base
        // `videoblock` class and any role(s).
        let mut classes = vec!["videoblock".to_string()];
        if let Some(float) = attrs.named_attribute("float") {
            classes.push(escape_attribute(float.value()));
        }
        if let Some(align) = attrs.named_attribute("align") {
            classes.push(format!("text-{}", escape_attribute(align.value())));
        }
        for role in self.media_roles(block, media) {
            classes.push(escape_attribute(role));
        }

        self.line(&format!(
            "<div{} class=\"{}\">",
            id_attribute(block.id()),
            classes.join(" ")
        ));
        self.block_title(block);
        self.line("<div class=\"content\">");

        // The media target and the attribute values below (`width`/`height`/
        // `poster`/`preload`, and the embed query parameters) are interpolated
        // *verbatim*, not run through [`escape_attribute`] the way the wrapper's
        // id/roles are. This matches Asciidoctor's `convert_video`/
        // `convert_audio`, which emit these raw — so, as with Asciidoctor, a
        // document built from untrusted input must be sanitized downstream. The
        // choice keeps output byte-identical to the parity oracle (a real target
        // or dimension never carries an HTML delimiter).

        // `width`/`height` are shared by the self-hosted and embed forms
        // (positional 2 and 3, or named).
        let width = attrs.named_or_positional_attribute("width", 2);
        let height = attrs.named_or_positional_attribute("height", 3);
        let width_attr = width
            .map(|w| format!(" width=\"{}\"", w.value()))
            .unwrap_or_default();
        let height_attr = height
            .map(|h| format!(" height=\"{}\"", h.value()))
            .unwrap_or_default();

        // The `poster` attribute (positional 1, or named) selects the form: a
        // `vimeo`/`youtube` value is a hosted embed, anything else is a
        // self-hosted `<video>` whose poster image it supplies.
        let poster = attrs
            .named_or_positional_attribute("poster", 1)
            .map(|p| p.value());

        match poster {
            Some("vimeo") => self.video_vimeo(media, attrs, &width_attr, &height_attr),
            Some("youtube") => self.video_youtube(media, attrs, &width_attr, &height_attr),
            _ => self.video_self_hosted(media, attrs, poster, &width_attr, &height_attr),
        }

        self.line("</div>");
        self.line("</div>");
    }

    /// Emits the self-hosted `<video>` element (the `else` arm of
    /// `convert_video`): the resolved target as `src` with an optional `#t=`
    /// time fragment, the `poster`/`preload` attributes, and the
    /// `autoplay`/`muted`/`controls`/`loop` boolean options.
    fn video_self_hosted<'src>(
        &mut self,
        media: &'src MediaBlock<'src>,
        attrs: &'src Attrlist<'src>,
        poster: Option<&str>,
        width_attr: &str,
        height_attr: &str,
    ) {
        let poster_attr = match poster {
            Some(p) if !p.is_empty() => {
                format!(" poster=\"{}\"", media_uri(p, &self.imagesdir))
            }

            _ => String::new(),
        };

        let preload_attr = match attrs.named_attribute("preload").map(|p| p.value()) {
            Some(p) if !p.is_empty() => format!(" preload=\"{p}\""),
            _ => String::new(),
        };

        let start = attrs.named_attribute("start").map(|a| a.value());
        let end = attrs.named_attribute("end").map(|a| a.value());
        let time_anchor = Self::time_anchor(start, end);

        self.line(&format!(
            "<video src=\"{}{time_anchor}\"{width_attr}{height_attr}{poster_attr}{}{}{}{}{preload_attr}>",
            media_uri(media.resolved_target(), &self.imagesdir),
            boolean_option(attrs, "autoplay", "autoplay"),
            boolean_option(attrs, "muted", "muted"),
            controls_option(attrs),
            boolean_option(attrs, "loop", "loop"),
        ));
        self.line("Your browser does not support the video tag.");
        self.line("</video>");
    }

    /// Emits the Vimeo `<iframe>` embed (the `poster=vimeo` arm of
    /// `convert_video`).
    fn video_vimeo<'src>(
        &mut self,
        media: &'src MediaBlock<'src>,
        attrs: &'src Attrlist<'src>,
        width_attr: &str,
        height_attr: &str,
    ) {
        // The target splits into `<video-id>/<hash>`; a `hash=` attribute
        // supplies the hash when the target carries none.
        let (target, hash) = match media.resolved_target().split_once('/') {
            Some((t, h)) => (t, Some(h)),
            None => (media.resolved_target(), None),
        };
        let hash = hash.or_else(|| attrs.named_attribute("hash").map(|a| a.value()));

        // The query parameters, in Asciidoctor's order; the first present one
        // is introduced by `?` and the rest by `&amp;`.
        let mut params: Vec<String> = Vec::new();
        if let Some(hash) = hash {
            params.push(format!("h={hash}"));
        }
        if attrs.has_option("autoplay") {
            params.push("autoplay=1".to_string());
        }
        if attrs.has_option("loop") {
            params.push("loop=1".to_string());
        }
        if attrs.has_option("muted") {
            params.push("muted=1".to_string());
        }
        let query = join_query(&params);

        // The start time is a `#at=` fragment appended after the query.
        let start_anchor = attrs
            .named_attribute("start")
            .map(|s| format!("#at={}", s.value()))
            .unwrap_or_default();

        self.line(&format!(
            "<iframe{width_attr}{height_attr} src=\"{}//player.vimeo.com/video/{target}{query}{start_anchor}\" frameborder=\"0\"{}></iframe>",
            self.asset_uri_scheme_prefix(),
            allowfullscreen_attr(attrs),
        ));
    }

    /// Emits the YouTube `<iframe>` embed (the `poster=youtube` arm of
    /// `convert_video`).
    fn video_youtube<'src>(
        &mut self,
        media: &'src MediaBlock<'src>,
        attrs: &'src Attrlist<'src>,
        width_attr: &str,
        height_attr: &str,
    ) {
        let rel = if attrs.has_option("related") { 1 } else { 0 };

        // Every parameter after the leading `?rel=` is introduced by `&amp;`.
        let named_param = |name: &str, key: &str| {
            attrs
                .named_attribute(name)
                .map(|a| format!("&amp;{key}={}", a.value()))
                .unwrap_or_default()
        };
        let start_param = named_param("start", "start");
        let end_param = named_param("end", "end");
        let autoplay_param = flag_param(attrs, "autoplay", "&amp;autoplay=1");
        let has_loop = attrs.has_option("loop");
        let loop_param = flag_param(attrs, "loop", "&amp;loop=1");
        let mute_param = flag_param(attrs, "muted", "&amp;mute=1");
        let controls_param = flag_param(attrs, "nocontrols", "&amp;controls=0");

        // Fullscreen can be controlled by a query parameter and an attribute.
        let (fs_param, fs_attribute) = if attrs.has_option("nofullscreen") {
            ("&amp;fs=0", "")
        } else {
            ("", " allowfullscreen")
        };
        let modest_param = flag_param(attrs, "modest", "&amp;modestbranding=1");
        let theme_param = named_param("theme", "theme");
        let hl_param = named_param("lang", "hl");

        // Parse `<video-id>/<list-id>`; failing a list, parse a dynamic
        // `<id1>,<id2>,…` playlist. Named `list=`/`playlist=` attributes fill
        // in when the target carries neither.
        let (mut target, list) = match media.resolved_target().split_once('/') {
            Some((t, l)) => (t, Some(l)),
            None => (media.resolved_target(), None),
        };
        let list = list.or_else(|| attrs.named_attribute("list").map(|a| a.value()));

        let list_param = if let Some(list) = list {
            format!("&amp;list={list}")
        } else {
            let (id, playlist) = match target.split_once(',') {
                Some((id, playlist)) => (id, Some(playlist)),
                None => (target, None),
            };
            target = id;
            let playlist =
                playlist.or_else(|| attrs.named_attribute("playlist").map(|a| a.value()));
            if let Some(playlist) = playlist {
                format!("&amp;playlist={target},{playlist}")
            } else if has_loop {
                // A loop needs an explicit playlist; fall back to the video id.
                format!("&amp;playlist={target}")
            } else {
                String::new()
            }
        };

        self.line(&format!(
            "<iframe{width_attr}{height_attr} src=\"{}//www.youtube.com/embed/{target}?rel={rel}{start_param}{end_param}{autoplay_param}{loop_param}{mute_param}{controls_param}{list_param}{fs_param}{modest_param}{theme_param}{hl_param}\" frameborder=\"0\"{fs_attribute}></iframe>",
            self.asset_uri_scheme_prefix(),
        ));
    }

    /// `<div class="audioblock …">` wrapping a self-hosted `<audio>` element —
    /// a port of Asciidoctor's `convert_audio`. Audio has no poster, width,
    /// height, or hosted-service embeds.
    fn audio<'src>(&mut self, block: &'src Block<'src>, media: &'src MediaBlock<'src>) {
        let attrs = media.macro_attrlist();

        self.line(&format!(
            "<div{} class=\"{}\">",
            id_attribute(block.id()),
            class_list("audioblock", &self.media_roles(block, media)),
        ));
        self.block_title(block);
        self.line("<div class=\"content\">");

        let start = attrs.named_attribute("start").map(|a| a.value());
        let end = attrs.named_attribute("end").map(|a| a.value());
        let time_anchor = Self::time_anchor(start, end);

        // The target is interpolated verbatim, matching Asciidoctor (see the
        // note in [`video`](Self::video)).
        self.line(&format!(
            "<audio src=\"{}{time_anchor}\"{}{}{}>",
            media_uri(media.resolved_target(), &self.imagesdir),
            boolean_option(attrs, "autoplay", "autoplay"),
            controls_option(attrs),
            boolean_option(attrs, "loop", "loop"),
        ));
        self.line("Your browser does not support the audio tag.");
        self.line("</audio>");

        self.line("</div>");
        self.line("</div>");
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

/// Which horizontal band of a table a row belongs to. The head band renders its
/// cells as `<th>` and never paragraph-wraps or style-wraps them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TableSection {
    Head,
    Body,
    Foot,
}

/// The `stripes-<value>` class for a table, or `None` when no striping applies.
///
/// Asciidoctor emits the class whenever the resolved `stripes` value is truthy
/// — an explicit `stripes` attribute on the table (even `stripes=none`) or a
/// `table-stripes` document default. The parser folds the document default into
/// [`TableBlock::stripes`], so a non-`None` value means striping is on; an
/// explicit attribute is honored on top of that so `[stripes=none]` still emits
/// `stripes-none`.
fn table_stripes_class(table: &TableBlock<'_>) -> Option<String> {
    let has_attr = table
        .attrlist()
        .and_then(|a| a.named_attribute("stripes"))
        .is_some();
    let stripes = table.stripes();
    if !has_attr && stripes == Stripes::None {
        return None;
    }
    let value = match stripes {
        Stripes::None => "none",
        Stripes::Even => "even",
        Stripes::Odd => "odd",
        Stripes::All => "all",
        Stripes::Hover => "hover",
    };
    Some(format!("stripes-{value}"))
}

/// The formatted percentage-width string for every column, mirroring
/// Asciidoctor's `Table#assign_column_widths` (round to 4 decimal places,
/// donating any balance to the final column).
///
/// A `~` autowidth column contributes no proportional width; the remaining
/// 100% is split evenly across the autowidth columns. The strings are used only
/// for non-autowidth columns' `<col style>`, but every column is computed so
/// the balance donation lands on the correct final column.
///
/// Asciidoctor also has a "no base" path that divides the width equally when
/// the column-width total is zero; `asciidoc-parser` clamps every column width
/// to at least 1 (a `0` specifier keeps the default width of 1), so that total
/// is never zero and the path is unreachable here.
fn column_pcwidths(columns: &[TableColumn]) -> Vec<String> {
    let n = columns.len();

    // Autowidth (`~`) columns carry no proportional width; `width_base` is the
    // sum of the remaining columns' widths (matching Asciidoctor, which excludes
    // the `-1` autowidth widths from the base).
    let autowidth_count = columns.iter().filter(|c| c.is_autowidth()).count();
    let width_base: f64 = columns
        .iter()
        .filter(|c| !c.is_autowidth())
        .map(|c| c.width() as f64)
        .sum();

    // The autowidth columns absorb whatever the fixed columns leave of the 100%,
    // split evenly; the base then becomes the full 100. With any fixed column
    // `width_base` is already positive, and an all-autowidth table lands here,
    // so `base` is never zero.
    let mut base = width_base;
    let mut autowidth_value = 0.0;
    if autowidth_count > 0 && width_base <= 100.0 {
        autowidth_value = truncate4((100.0 - width_base) / autowidth_count as f64);
        base = 100.0;
    }

    let mut pcwidths = vec![0.0_f64; n];
    let mut total = 0.0_f64;
    let mut last = 0.0_f64;
    for (i, col) in columns.iter().enumerate() {
        let width = if col.is_autowidth() {
            autowidth_value
        } else {
            col.width() as f64
        };
        let pc = truncate4(width * 100.0 / base);
        pcwidths[i] = pc;
        total += pc;
        last = pc;
    }

    // Any rounding balance is donated to the final column (half-up rounding).
    if n > 0 && (total - 100.0).abs() > f64::EPSILON {
        pcwidths[n - 1] = round4(100.0 - total + last);
    }

    pcwidths.iter().map(|pc| format_pcwidth(*pc)).collect()
}

/// Truncates `value` toward zero to four decimal places (Ruby's
/// `Float#truncate 4`).
fn truncate4(value: f64) -> f64 {
    (value * 10000.0).trunc() / 10000.0
}

/// Rounds `value` half away from zero to four decimal places (Ruby's
/// `Float#round 4`).
fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

/// Formats a percentage width the way Ruby prints it: a whole number drops its
/// decimals (`50`), otherwise up to four decimals with trailing zeros trimmed
/// (`33.3333`, `17.647`).
fn format_pcwidth(value: f64) -> String {
    let scaled = (value * 10000.0).round() as i64;
    let whole = scaled / 10000;
    let frac = (scaled % 10000).abs();
    if frac == 0 {
        return whole.to_string();
    }
    let frac = format!("{frac:04}");
    let frac = frac.trim_end_matches('0');
    format!("{whole}.{frac}")
}

/// Renders the paragraph content of a non-header, non-literal, non-AsciiDoc
/// cell, mirroring Asciidoctor's `Table::Cell#content`. `content` is the cell's
/// already-substituted inline text (its `Simple` rendered value).
///
/// The cell text is split into paragraphs on blank lines — but only when the
/// *raw* cell text contains a blank line, so a line reduced to blank by a
/// substitution (e.g. `{blank}`) does not split the paragraph. Each paragraph
/// is wrapped in `<p class="tableblock">`, and a styled column additionally
/// wraps the text in the style's inline element (`<em>`/`<strong>`/`<code>`).
/// An empty cell renders no `<p>` at all.
fn cell_paragraphs(cell: &TableCell<'_>, content: &str) -> String {
    // The style's inline wrapper (`e`/`s`/`m`); the default and header styles
    // add none.
    let (open, close) = match cell.style() {
        ColumnStyle::Emphasis => ("<em>", "</em>"),
        ColumnStyle::Strong => ("<strong>", "</strong>"),
        ColumnStyle::Monospace => ("<code>", "</code>"),
        _ => ("", ""),
    };
    let wrap = |para: &str| {
        if open.is_empty() {
            para.to_string()
        } else {
            format!("{open}{para}{close}")
        }
    };

    // The split decision keys off the raw (untrimmed, but stripped) cell source,
    // matching Asciidoctor's `@text.include? DOUBLE_LF`.
    let raw = cell.span().data().trim();
    let paragraphs: Vec<String> = if raw.contains("\n\n") {
        split_blank_lines(content).map(wrap).collect()
    } else if content.is_empty() {
        vec![]
    } else {
        vec![wrap(content)]
    };

    if paragraphs.is_empty() {
        String::new()
    } else {
        format!(
            "<p class=\"tableblock\">{}</p>",
            paragraphs.join("</p>\n<p class=\"tableblock\">")
        )
    }
}

/// Splits `text` on runs of two or more newlines (Asciidoctor's
/// `BlankLineRx = /\n{2,}/`), dropping any trailing empty segment the way
/// Ruby's `String#split` does.
fn split_blank_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut parts: Vec<&str> = vec![];
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b'\n' {
                j += 1;
            }
            if j - i >= 2 {
                parts.push(&text[start..i]);
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    parts.push(&text[start..]);

    // Ruby's default split trims trailing empty strings.
    while parts.last() == Some(&"") {
        parts.pop();
    }
    parts.into_iter()
}

/// Renders the nested document of an AsciiDoc (`a`) table cell to a body-only
/// HTML fragment (no trailing newline), the way Asciidoctor's
/// `cell.content` calls `@inner_document.convert`.
///
/// An `inline`-doctype cell renders just the inline content of its first block.
/// Otherwise the cell's blocks are rendered as embedded output, preceded by the
/// nested-document `<h1>` when the cell's title is shown, and (for an
/// `auto`-placed TOC) by the cell's leading table of contents.
fn render_cell_document<'s>(
    blocks: &'s [Block<'s>],
    title: Option<&str>,
    inline: bool,
    footnotes: &[Footnote],
    toc: CellTocConfig,
    config: CellRenderConfig,
) -> String {
    if inline {
        // The inline doctype renders the first block's inline content; skip any
        // leading attribute-entry blocks (which carry no rendered content) so the
        // real first paragraph is what shows.
        return blocks
            .iter()
            .find_map(|block| block.rendered_content())
            .unwrap_or_default()
            .to_string();
    }

    // The cell is a standalone nested document with its own TOC. An
    // `auto`/`left`/`right`/`top`/`bottom` placement is built here and emitted
    // ahead of the content (like embedded output); a `macro` placement is built
    // at its `toc::[]` block from the cell blocks carried in `cell_toc`. As in
    // embedded output, a leading TOC uses the plain `toc` class (the side-column
    // layout the other classes drive isn't available), while `toc-class` still
    // rides through `cell_toc` for the macro form.
    let leading_toc = matches!(
        toc.mode,
        TocMode::Auto | TocMode::Left | TocMode::Right | TocMode::Top | TocMode::Bottom
    );

    let toc_html = if leading_toc {
        let outline = crate::outline::render_outline_blocks(blocks, toc.levels, toc.sectnumlevels);
        if outline.is_empty() {
            String::new()
        } else {
            toc_container("toc", &toc.title, &outline)
        }
    } else {
        String::new()
    };

    // The cell is a nested document that inherits the parent's verbatim-layout
    // attributes (`tabsize`, `source-indent`, `prewrap`) and section-heading
    // toggles (`sectanchors`, `sectlinks`), so the sub-renderer carries them
    // forward.
    let mut renderer = Renderer {
        out: String::new(),
        // The cell sub-renderer is handed a block slice, not a `Document`; the
        // `toc::[]` macro path reads the cell's TOC from `cell_toc` instead.
        document: None,
        custom_stylesheet: None,
        standalone: false,
        toc_mode: toc.mode,
        toc_html,
        cell_toc: Some(CellToc {
            blocks,
            toclevels: toc.levels,
            sectnumlevels: toc.sectnumlevels,
            title: toc.title,
            class: toc.class,
        }),
        icons_set: config.icons_set,
        icons_font: config.icons_font,
        iconsdir: config.iconsdir.clone(),
        icontype: config.icontype.clone(),
        imagesdir: config.imagesdir.clone(),
        asset_uri_scheme: config.asset_uri_scheme.clone(),
        doc_tabsize: config.doc_tabsize,
        source_indent: config.source_indent,
        prewrap: config.prewrap,
        source_highlighter: config.source_highlighter,
        source_language: config.source_language,
        section_anchors: config.section_anchors,
        sectlinks: config.sectlinks,
        stem_type: config.stem_type,
        cellbgcolor: config.cellbgcolor,
        svg_below_secure: config.svg_below_secure,
        data_uri: config.data_uri,
        svg_source: config.svg_source,
    };
    if let Some(title) = title {
        renderer.line(&format!("<h1>{title}</h1>"));
    }

    // An `auto`-placed TOC leads the cell body, after the optional title and
    // ahead of the content, mirroring embedded output (see
    // [`embedded_document`](Renderer::embedded_document)).
    if !renderer.toc_html.is_empty() {
        renderer.line(&renderer.toc_html.clone());
    }

    renderer.blocks(blocks.iter());

    // An AsciiDoc cell keeps its own footnote registry, isolated from the
    // enclosing document; Asciidoctor renders those footnotes as a cell-local
    // `#footnotes` block inside the cell content, after the cell's blocks. The
    // `footnote-number` counter is document-wide, so the indices (and thus the
    // `_footnotedef_N` ids) stay globally unique.
    if !footnotes.is_empty() {
        renderer.footnotes_block(footnotes, "");
    }

    // `convert` joins its lines with no trailing newline; drop the one the
    // line-oriented renderer left behind.
    match renderer.out.strip_suffix('\n') {
        Some(trimmed) => trimmed.to_string(),
        None => renderer.out,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Options, ReferenceTime, SafeMode};

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
    fn footnotes_block_lists_registered_footnotes() {
        // A registered footnote renders both its inline `<sup>` reference and,
        // at the document level, a `#footnotes` definition block — in both the
        // embedded and standalone paths.
        let embedded = convert("text.footnote:[a note] more.footnote:[another]");
        assert!(embedded.contains(
            "<div id=\"footnotes\">\n<hr>\n\
             <div class=\"footnote\" id=\"_footnotedef_1\">\n\
             <a href=\"#_footnoteref_1\">1</a>. a note\n</div>\n\
             <div class=\"footnote\" id=\"_footnotedef_2\">\n\
             <a href=\"#_footnoteref_2\">2</a>. another\n</div>\n</div>"
        ));

        // Standalone output carries the same block, between the content and the
        // footer.
        let standalone = convert_with(
            "text.footnote:[a note]",
            &crate::Options::new().standalone(true),
        );
        assert!(standalone.contains("</div>\n<div id=\"footnotes\">\n<hr>\n"));

        // A document with no footnotes emits no `#footnotes` block.
        assert!(!convert("plain text").contains("id=\"footnotes\""));
    }

    #[test]
    fn asciidoc_cell_renders_its_own_footnotes_block() {
        // An AsciiDoc (`a`) cell is a nested document with its own footnote
        // registry: Asciidoctor renders the cell's footnotes as a `#footnotes`
        // block inside the cell's `<div class="content">`, isolated from the
        // enclosing document (asciidoc-html5#231).
        let cell_only =
            convert("|===\na|AsciiDoc footnote:[A lightweight markup language.]\n|===\n");

        // The cell's footnote definition renders inside the cell content — the
        // block closes with the cell wrapper (`</div></div></td>`), not at the
        // document level ...
        assert!(cell_only.contains(
            "<div id=\"footnotes\">\n<hr>\n\
             <div class=\"footnote\" id=\"_footnotedef_1\">\n\
             <a href=\"#_footnoteref_1\">1</a>. A lightweight markup language.\n\
             </div>\n</div></div></td>"
        ));

        // ... and, the sole footnote being the cell's, the document adds no
        // footnotes block of its own: exactly one `#footnotes` block exists.
        assert_eq!(cell_only.matches("id=\"footnotes\"").count(), 1);

        // A document footnote *and* a cell footnote keep separate registries:
        // each block lists only its own definition, while the document-wide
        // `footnote-number` counter still numbers them uniquely (document = 1,
        // cell = 2).
        let both =
            convert("Doc footnote:[Doc note.]\n\n|===\na|Cell footnote:[Cell note.]\n|===\n");
        assert_eq!(both.matches("id=\"footnotes\"").count(), 2);

        // The cell-local block carries only the cell's definition, inside the
        // cell.
        assert!(both.contains(
            "<div id=\"footnotes\">\n<hr>\n\
             <div class=\"footnote\" id=\"_footnotedef_2\">\n\
             <a href=\"#_footnoteref_2\">2</a>. Cell note.\n\
             </div>\n</div></div></td>"
        ));

        // The document-level block, a sibling after the content, carries only
        // the document's own definition.
        assert!(both.contains(
            "</div>\n<div id=\"footnotes\">\n<hr>\n\
             <div class=\"footnote\" id=\"_footnotedef_1\">\n\
             <a href=\"#_footnoteref_1\">1</a>. Doc note.\n\
             </div>\n</div>"
        ));
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
    fn leading_anchors_len_matches_leading_anchors_rx() {
        use super::leading_anchors_len;

        // No leading anchor: bare text, or a title opening with some other
        // element, consumes nothing.
        assert_eq!(leading_anchors_len("Foo"), 0);
        assert_eq!(leading_anchors_len("<em>Foo</em>"), 0);

        // A single bare anchor is consumed up to and including its `</a>`; the
        // trailing title text is left behind.
        assert_eq!(leading_anchors_len(r#"<a id="fu"></a>Foo"#), 15);

        // Anchors are consumed greedily: a run of them all counts.
        assert_eq!(
            leading_anchors_len(r#"<a id="fu"></a><a id="bar"></a>Foo"#),
            31
        );

        // The three non-match branches mirror the regex `<a id="[^"]+"></a>`:
        // an unterminated id (no closing quote), an empty id (`[^"]+` needs one
        // char), and an anchor not closed by `></a>` (e.g. a real link that
        // carries an `id`) each stop the scan with nothing consumed.
        assert_eq!(leading_anchors_len(r#"<a id="fu"#), 0);
        assert_eq!(leading_anchors_len(r#"<a id=""></a>Foo"#), 0);
        assert_eq!(leading_anchors_len(r##"<a id="fu" href="#x">Foo</a>"##), 0);
    }

    #[test]
    fn preamble_is_wrapped() {
        let html = convert("= Doc\n\nIntro.\n\n== Section\n\nBody.");
        let body = content(&html);
        assert!(body.starts_with("<div id=\"preamble\">\n<div class=\"sectionbody\">"));
    }

    // The TOC block Asciidoctor's `html5` backend emits, shared by every
    // placement — a `<div id="toc" class="toc">` with a `<div id="toctitle">`
    // and the nested section list.
    const SAMPLE_TOC: &str = "\
<div id=\"toc\" class=\"toc\">
<div id=\"toctitle\">Table of Contents</div>
<ul class=\"sectlevel1\">
<li><a href=\"#_section_one\">Section One</a></li>
<li><a href=\"#_section_two\">Section Two</a></li>
</ul>
</div>";

    #[test]
    fn auto_toc_renders_in_the_header() {
        // A plain `:toc:` (auto placement) puts the TOC inside `<div
        // id="header">`, after the `<h1>`, and leaves the `<body>` class alone.
        let html = convert("= Doc\n:toc:\n\nIntro.\n\n== Section One\n\nx\n\n== Section Two\n\ny");
        assert!(html.contains("<body class=\"article\">"));
        assert!(html.contains(&format!(
            "<div id=\"header\">\n<h1>Doc</h1>\n{SAMPLE_TOC}\n</div>"
        )));
    }

    #[test]
    fn preamble_toc_renders_below_the_preamble() {
        // `:toc: preamble` places the TOC after the preamble's `sectionbody`,
        // still inside `<div id="preamble">`, and not in the header.
        let html = convert(
            "= Doc\n:toc: preamble\n\nIntro.\n\n== Section One\n\nx\n\n== Section Two\n\ny",
        );

        // The header shows only the title — no TOC.
        assert!(html.contains("<div id=\"header\">\n<h1>Doc</h1>\n</div>"));

        // The TOC sits between the preamble's `sectionbody` close and the
        // preamble's own close.
        assert!(html.contains(&format!(
            "<p>Intro.</p>\n</div>\n</div>\n{SAMPLE_TOC}\n</div>\n<div class=\"sect1\">"
        )));
    }

    #[test]
    fn side_column_toc_adds_body_classes() {
        // `left`/`right` place the TOC in the header like `auto`, but also add
        // `toc2 toc-<side>` to the `<body>` class (the side-column styling).
        let left = convert("= Doc\n:toc: left\n\nIntro.\n\n== Section One\n\nx");
        assert!(left.contains("<body class=\"article toc2 toc-left\">"));
        assert!(left.contains("<div id=\"toc\" class=\"toc2\">"));

        let right = convert("= Doc\n:toc: right\n\nIntro.\n\n== Section One\n\nx");
        assert!(right.contains("<body class=\"article toc2 toc-right\">"));
    }

    #[test]
    fn embedded_auto_toc_leads_the_body() {
        // In embedded output an `auto` TOC leads the body, ahead of the content.
        let html = crate::convert_with(
            "= Doc\n:toc:\n\nIntro.\n\n== Section One\n\nx\n\n== Section Two\n\ny",
            &Options::new(),
        );
        assert!(html.starts_with(SAMPLE_TOC));
    }

    #[test]
    fn macro_placement_renders_at_the_toc_block() {
        // `macro` placement renders the TOC where the `toc::[]` block appears —
        // never in the header — with the `convert_toc` shape (the title block
        // carries `class="title"`). The `<body>` class is untouched.
        let html = convert(
            "= Doc\n:toc: macro\n\nIntro.\n\ntoc::[]\n\n== Section One\n\nx\n\n== Section Two\n\ny",
        );
        assert!(html.contains("<body class=\"article\">"));
        assert!(html.contains("<div id=\"header\">\n<h1>Doc</h1>\n</div>"));
        assert!(html.contains(
            "<div id=\"toc\" class=\"toc\">\n\
             <div id=\"toctitle\" class=\"title\">Table of Contents</div>\n\
             <ul class=\"sectlevel1\">\n\
             <li><a href=\"#_section_one\">Section One</a></li>\n\
             <li><a href=\"#_section_two\">Section Two</a></li>\n\
             </ul>\n</div>"
        ));
    }

    #[test]
    fn macro_placement_honors_per_macro_overrides() {
        // `id`/title/`levels`/`role` on the macro override the document
        // defaults; the `id` also renames the title block's id to `<id>title`.
        let html = convert(
            "= Doc\n:toc: macro\n\n[#mytoc.fancy]\n.My Contents\ntoc::[levels=1]\n\n== Section One\n\n=== Nested\n\nx",
        );
        assert!(html.contains(
            "<div id=\"mytoc\" class=\"fancy\">\n<div id=\"mytoctitle\" class=\"title\">My Contents</div>"
        ));
        // `levels=1` drops the nested subsection.
        assert!(!html.contains("Nested</a>"));
    }

    #[test]
    fn toc_macro_without_macro_placement_is_disabled() {
        // A `toc::[]` block whose document does not defer to it (here the
        // placement is `auto`) renders Asciidoctor's placeholder comment, while
        // the automatic TOC still appears in the header.
        let html = convert("= Doc\n:toc:\n\ntoc::[]\n\n== Section One\n\nx");
        assert!(html.contains("<!-- toc disabled -->"));
        assert!(html.contains("<div id=\"header\">\n<h1>Doc</h1>\n<div id=\"toc\" class=\"toc\">"));
    }

    #[test]
    fn toc_needs_sections() {
        // With no sections the outline is empty, so Asciidoctor (and this crate)
        // emit no TOC even when `:toc:` is set.
        let html = convert("= Doc\n:toc:\n\nJust a paragraph, no sections.");
        assert!(!html.contains("<div id=\"toc\""));
    }

    #[test]
    fn toclevels_limits_the_toc_depth() {
        // `:toclevels: 1` drops the nested subsection from the TOC while the
        // top-level entries remain.
        let html = convert("= Doc\n:toc:\n:toclevels: 1\n\n== Section One\n\n=== Nested\n\nx");
        assert!(html.contains("<li><a href=\"#_section_one\">Section One</a></li>"));
        assert!(!html.contains("Nested</a>"));
    }

    #[test]
    fn toc_title_is_configurable() {
        let html = convert("= Doc\n:toc:\n:toc-title: On this page\n\n== Section One\n\nx");
        assert!(html.contains("<div id=\"toctitle\">On this page</div>"));
    }

    #[test]
    fn asciidoc_cell_auto_toc_leads_the_cell_body() {
        // A nested AsciiDoc (`a`) cell resolves its TOC from its own `:toc:`,
        // independent of the parent. An `auto` placement leads the cell body
        // (before its sections) with the plain `toc` class, like embedded
        // output, matching Asciidoctor 2.0.26's nested-document conversion.
        let html = crate::convert_with(
            "|===\na|\n:toc:\n\n== Cell Section\n\nbody\n|===\n",
            &Options::new(),
        );
        assert!(html.contains(
            "<div class=\"content\"><div id=\"toc\" class=\"toc\">\n\
             <div id=\"toctitle\">Table of Contents</div>\n\
             <ul class=\"sectlevel1\">\n\
             <li><a href=\"#_cell_section\">Cell Section</a></li>\n\
             </ul>\n</div>"
        ));
    }

    #[test]
    fn asciidoc_cell_macro_toc_renders_at_the_toc_block() {
        // A cell's `:toc: macro` defers to a `toc::[]` block inside the cell,
        // which renders with the `convert_toc` shape (title block carries
        // `class="title"`) and honors the block's own id.
        let html = crate::convert_with(
            "|===\na|\n:toc: macro\n\n[#cell-toc]\ntoc::[]\n\n== Cell Section\n\nbody\n|===\n",
            &Options::new(),
        );
        assert!(html.contains(
            "<div id=\"cell-toc\" class=\"toc\">\n\
             <div id=\"cell-toctitle\" class=\"title\">Table of Contents</div>\n\
             <ul class=\"sectlevel1\">\n\
             <li><a href=\"#_cell_section\">Cell Section</a></li>\n\
             </ul>\n</div>"
        ));
    }

    #[test]
    fn asciidoc_cell_auto_toc_needs_sections() {
        // A cell whose `:toc:` outline would be empty (no sections) emits no
        // TOC, like a top-level document (see `toc_needs_sections`).
        let html = crate::convert_with("|===\na|\n:toc:\n\njust text\n|===\n", &Options::new());
        assert!(!html.contains("<div id=\"toc\""));
    }

    #[test]
    fn asciidoc_cell_toc_numbers_sections() {
        // With `:sectnums:`, the cell's TOC entries carry their section numbers,
        // resolved from the cell's own nested document, matching Asciidoctor
        // 2.0.26. (The `sectnumlevels` cap is not honored inside a cell — the
        // parser surfaces only the default; see asciidoc-parser#1092.)
        let html = crate::convert_with(
            "|===\na|\n:toc:\n:sectnums:\n\n== Alpha\n\n== Bravo\n\nx\n|===\n",
            &Options::new(),
        );
        assert!(html.contains("<li><a href=\"#_alpha\">1. Alpha</a></li>"));
        assert!(html.contains("<li><a href=\"#_bravo\">2. Bravo</a></li>"));
    }

    #[test]
    fn asciidoc_cell_macro_toc_honors_levels_override() {
        // A `levels=` attribute on the cell's `toc::[]` macro caps the depth for
        // that TOC only: `levels=1` drops the nested level-2 entry.
        let html = crate::convert_with(
            "|===\na|\n:toc: macro\n\ntoc::[levels=1]\n\n== Alpha\n\n=== Beta\n\nx\n|===\n",
            &Options::new(),
        );
        assert!(html.contains("<li><a href=\"#_alpha\">Alpha</a></li>"));
        assert!(!html.contains("Beta</a>"));
    }

    #[test]
    fn asciidoc_cell_macro_toc_without_sections_is_disabled() {
        // A cell's `toc::[]` macro with no sections to list emits Asciidoctor's
        // placeholder comment, like the top-level document
        // (`toc_macro_without_macro_placement_is_disabled`).
        let html = crate::convert_with(
            "|===\na|\n:toc: macro\n\ntoc::[]\n\njust text\n|===\n",
            &Options::new(),
        );
        assert!(html.contains("<!-- toc disabled -->"));
        assert!(!html.contains("<div id=\"toc\""));
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

    // Note: there is no longer a `unsupported_block_leaves_a_marker` test —
    // every block construct the parser produces now renders (block images
    // landed here; video/audio and STEM landed alongside), so no standard
    // input reaches [`Renderer::unsupported`], which remains only as a
    // defensive fallback for future parser additions.

    #[test]
    fn image_block_renders_imageblock() {
        // The basic block image: `src`, positional `alt`/`width`/`height`, and
        // the `imageblock` wrapper, matching Asciidoctor's `convert_image`.
        let html = convert("image::screenshot.png[block image,800,450]");
        assert!(content(&html).contains(
            "<div class=\"imageblock\">\n\
             <div class=\"content\">\n\
             <img src=\"screenshot.png\" alt=\"block image\" width=\"800\" height=\"450\">\n\
             </div>\n\
             </div>"
        ));
    }

    #[test]
    fn image_block_default_alt_from_target() {
        // With no explicit alt, Asciidoctor derives it from the target basename
        // with `_`/`-` turned into spaces.
        let html = convert("image::my_cool-pic.png[]");
        assert!(html.contains("<img src=\"my_cool-pic.png\" alt=\"my cool pic\">"));
    }

    #[test]
    fn image_block_alt_gets_specialchars_and_replacements() {
        // An explicit alt runs both the special-character and replacement
        // substitutions (Asciidoctor's `AbstractBlock#alt`), so an apostrophe
        // becomes `&#8217;` and `<` becomes `&lt;`, then the `"` is
        // attribute-encoded for the `alt="…"` slot.
        let html = convert(r#"image::t.png[A tiger's "roar" is < a bear's "growl"]"#);
        assert!(
            html.contains(
                r#"alt="A tiger&#8217;s &quot;roar&quot; is &lt; a bear&#8217;s &quot;growl&quot;""#
            ),
            "{html}"
        );
    }

    #[test]
    fn image_block_style_stands_in_as_alt() {
        // Asciidoctor's `attributes['alt'] ||= style`: the block attribute line's
        // style (its first positional) becomes the alt when the macro carries
        // none — and as an explicit alt it, too, gets the replacement subs.
        let html = convert("[Tiger]\nimage::images/tiger.png[]\n");
        assert!(
            html.contains("<img src=\"images/tiger.png\" alt=\"Tiger\">"),
            "{html}"
        );
    }

    #[test]
    fn image_block_title_is_a_captioned_figure_after_content() {
        // A titled image gains a `Figure N.` caption in a title div placed after
        // the content div.
        let html = convert(".A caption\nimage::b.png[Alt Text]");
        assert!(content(&html).contains(
            "<img src=\"b.png\" alt=\"Alt Text\">\n\
             </div>\n\
             <div class=\"title\">Figure 1. A caption</div>"
        ));
    }

    #[test]
    fn image_block_honors_id_float_align_and_roles() {
        let html = convert("[#myid.role1.role2]\nimage::c.png[align=center,float=left]");
        assert!(
            html.contains("<div id=\"myid\" class=\"imageblock left text-center role1 role2\">")
        );
    }

    #[test]
    fn image_block_link_wraps_the_image() {
        let html = convert("image::e.png[link=https://example.com]");
        assert!(html.contains(
            "<a class=\"image\" href=\"https://example.com\">\
             <img src=\"e.png\" alt=\"e\"></a>"
        ));
    }

    #[test]
    fn image_block_svg_interactive_renders_object() {
        // Below `Secure`, an SVG block image with `opts=interactive` renders as
        // an `<object>` (its alt text nested as the fallback), matching
        // Asciidoctor's `convert_image`.
        let html = convert_with(
            "image::diagram.svg[Diagram,300,opts=interactive]",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(content(&html).contains(
            "<object type=\"image/svg+xml\" data=\"diagram.svg\" width=\"300\">\
             <span class=\"alt\">Diagram</span></object>"
        ));
    }

    #[test]
    fn image_block_svg_interactive_uses_fallback_image() {
        // A `fallback` image is nested inside the `<object>` instead of the alt
        // text, resolved under `imagesdir` like any other relative target.
        let html = convert_with(
            ":imagesdir: img\n\nimage::diagram.svg[Diagram,opts=interactive,fallback=fallback.png]",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(html.contains(
            "<object type=\"image/svg+xml\" data=\"img/diagram.svg\">\
             <img src=\"img/fallback.png\" alt=\"Diagram\"></object>"
        ));
    }

    #[test]
    fn image_block_svg_interactive_is_plain_img_at_secure() {
        // The interactive referencing is security-sensitive: at `Secure` (the
        // default safe mode) the same SVG renders as a plain `<img>`, matching
        // Asciidoctor.
        let html = convert("image::diagram.svg[Diagram,opts=interactive]");
        assert!(content(&html).contains("<img src=\"diagram.svg\" alt=\"Diagram\">"));
        assert!(!html.contains("<object"));
    }

    #[test]
    fn image_block_svg_detection_matches_asciidoctor_substring_quirks() {
        // The SVG target test is a case-sensitive substring match ported from
        // Asciidoctor's `target.include? '.svg'`, quirks and all. These are
        // verified against Asciidoctor 2.0.26; they are intentional parity, not
        // bugs to "fix" (fixing them would diverge from the oracle).
        let opts = Options::new().safe_mode(SafeMode::Unsafe);

        // A non-SVG target that merely *contains* `.svg` is still treated as an
        // SVG `<object>` (Asciidoctor does the same).
        let double_ext = convert_with("image::chart.svg.png[C,opts=interactive]", &opts);
        assert!(double_ext.contains("<object type=\"image/svg+xml\" data=\"chart.svg.png\">"));

        // An uppercase `.SVG` extension is *not* matched, so it renders a plain
        // `<img>` (the check is case-sensitive, matching Asciidoctor).
        let upper = convert_with("image::diagram.SVG[D,opts=interactive]", &opts);
        assert!(content(&upper).contains("<img src=\"diagram.SVG\" alt=\"D\">"));
        assert!(!upper.contains("<object"));
    }

    #[test]
    fn image_block_svg_inline_without_a_base_dir_falls_back_to_alt() {
        // The `inline` (embedded `<svg>`) referencing reads the SVG file's
        // contents; the plain string entry point has no base directory to anchor
        // that read (`convert_with` sets no `input_file`), so — like Asciidoctor
        // when it cannot read the file — an `opts=inline` block image below
        // `Secure` falls back to a `<span class="alt">` rather than an `<img>`.
        // (Reading a real on-disk SVG is exercised via `convert_file_with` in the
        // SVG Images page coverage.)
        let html = convert_with(
            "image::diagram.svg[Diagram,opts=inline]",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(content(&html).contains("<span class=\"alt\">Diagram</span>"));
        assert!(!html.contains("<object"));
        assert!(!html.contains("<svg"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn image_svg_interactive_in_asciidoc_table_cell() {
        // An interactive SVG inside an AsciiDoc (`a|`) table cell renders as an
        // `<object>`: the cell's document-less sub-renderer inherits the enclosing
        // document's below-`Secure` safe mode through `CellRenderConfig`.
        let html = convert_with(
            "|===\na|image::diagram.svg[Diagram,opts=interactive]\n|===",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(html.contains(
            "<object type=\"image/svg+xml\" data=\"diagram.svg\">\
             <span class=\"alt\">Diagram</span></object>"
        ));
    }

    /// Writes a `sample.svg` (XML preamble + a `<svg>` tag carrying
    /// width/height/style/viewBox, and a trailing newline like a real file) and
    /// a `main.adoc` holding `body` into a fresh temp directory, converts the
    /// file embedded under `safe`, and returns the HTML. `tag` names the temp
    /// directory so concurrent tests do not collide.
    fn with_svg(tag: &str, safe: SafeMode, body: &str) -> String {
        let dir =
            std::env::temp_dir().join(format!("adoc-render-svg-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        std::fs::write(
            dir.join("sample.svg"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"500\" height=\"500\" \
             style=\"fill:red\" viewBox=\"0 0 500 500\">\
             <circle cx=\"250\" cy=\"250\" r=\"200\"/></svg>\n",
        )
        .expect("write sample.svg");

        let main = dir.join("main.adoc");
        std::fs::write(&main, format!("= Doc\n\n{body}\n")).expect("write main.adoc");

        let html = crate::convert_file_with(&main, &Options::new().safe_mode(safe).embedded(true))
            .expect("convert the file");

        let _ = std::fs::remove_dir_all(&dir);
        html
    }

    #[test]
    fn image_block_svg_inline_embeds_the_file_contents() {
        // Below `Secure`, an `opts=inline` block image embeds the SVG file's
        // contents as an `<svg>`: the XML preamble is stripped, and the macro's
        // width (300) replaces the tag's own width/height/style while the
        // required `viewBox` is preserved — byte-for-byte matching Asciidoctor.
        let html = with_svg(
            "inline-embed",
            SafeMode::Unsafe,
            "image::sample.svg[Diagram,300,opts=inline]",
        );
        assert!(
            html.contains(
                "<div class=\"imageblock\">\n<div class=\"content\">\n\
                 <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 500 500\" width=\"300\">\
                 <circle cx=\"250\" cy=\"250\" r=\"200\"/></svg>\n\
                 </div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn image_block_svg_inline_in_asciidoc_table_cell() {
        // An `opts=inline` block image inside an AsciiDoc (`a|`) cell embeds the
        // SVG too: the cell's document-less sub-renderer inherits the base
        // directory and safe mode through `CellRenderConfig`.
        let html = with_svg(
            "inline-cell",
            SafeMode::Unsafe,
            "|===\na|image::sample.svg[Diagram,300,opts=inline]\n|===",
        );
        assert!(
            html.contains(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 500 500\" width=\"300\">\
                 <circle cx=\"250\" cy=\"250\" r=\"200\"/></svg>"
            ),
            "{html}"
        );
    }

    #[test]
    fn image_block_svg_inline_is_plain_img_at_secure() {
        // The inline embedding is security-sensitive: at `Secure` the same
        // `opts=inline` block image renders a plain `<img>` (its width preserved
        // as an attribute), never reading the file — matching Asciidoctor.
        let html = with_svg(
            "inline-secure",
            SafeMode::Secure,
            "image::sample.svg[Diagram,300,opts=inline]",
        );
        assert!(
            html.contains("<img src=\"sample.svg\" alt=\"Diagram\" width=\"300\">"),
            "{html}"
        );
        assert!(!html.contains("<svg"), "{html}");
    }

    #[test]
    fn image_block_svg_inline_missing_file_falls_back_to_alt() {
        // When the `opts=inline` target cannot be read (here it does not exist),
        // the block image falls back to a `<span class="alt">`, as Asciidoctor
        // does — not a broken `<img>` or an empty `<svg>`.
        let html = with_svg(
            "inline-missing",
            SafeMode::Unsafe,
            "image::absent.svg[Gone,opts=inline]",
        );
        assert!(html.contains("<span class=\"alt\">Gone</span>"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
        assert!(!html.contains("<img"), "{html}");
    }

    #[test]
    fn image_block_prefixes_imagesdir() {
        let html = convert(":imagesdir: assets/img\n\nimage::a.png[]");
        assert!(html.contains("<img src=\"assets/img/a.png\" alt=\"a\">"));
    }

    #[test]
    fn image_block_uri_target_passes_through() {
        let html = convert("image::http://example.com/a b.png[Remote]");
        assert!(html.contains("<img src=\"http://example.com/a%20b.png\" alt=\"Remote\">"));
    }

    #[test]
    fn image_block_rooted_target_is_not_prefixed() {
        // A web-root target (leading `/`) is never joined with `imagesdir`, and
        // its `default_alt` is the basename with the extension dropped.
        let html = convert(":imagesdir: assets\n\nimage::/rooted.png[]");
        assert!(html.contains("<img src=\"/rooted.png\" alt=\"rooted\">"));
    }

    #[test]
    fn image_block_default_alt_for_extensionless_target() {
        // A target with no extension keeps its final path segment verbatim as
        // the derived `alt`.
        let html = convert("image::gallery/photo[]");
        assert!(html.contains("<img src=\"gallery/photo\" alt=\"photo\">"));
    }

    // The `data-uri` document attribute embeds a referenced image as a base64
    // `data:` URI instead of linking it, below the `Secure` safe mode. These
    // exercise the renderer's `image_uri`/`generate_data_uri` against real
    // files, and were checked byte-for-byte against Asciidoctor 2.0.26.
    mod data_uri {
        use std::{
            fs,
            path::PathBuf,
            sync::atomic::{AtomicU64, Ordering},
        };

        use crate::{convert_with, Options, SafeMode};

        /// A one-pixel transparent GIF — the smallest real image to embed. Its
        /// strict-base64 encoding is [`GIF_B64`].
        const GIF: &[u8] = &[
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
            0x3b,
        ];
        const GIF_B64: &str = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==";

        /// Creates a fresh, canonicalized temp directory (unique per call) and
        /// writes each `(relative path, bytes)` into it, returning the
        /// directory.
        fn scratch(files: &[(&str, &[u8])]) -> PathBuf {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir()
                .join(format!("ahtml5-datauri-{}-{unique}", std::process::id()));
            fs::create_dir_all(&dir).expect("create scratch dir");
            for (name, bytes) in files {
                let path = dir.join(name);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).expect("create parent dir");
                }
                fs::write(path, bytes).expect("write scratch file");
            }
            dir.canonicalize().expect("canonicalize scratch dir")
        }

        /// Converts `source` with `data-uri` set, anchored at a base directory
        /// holding `files`, under `safe` (below `Secure` to enable embedding).
        fn convert_data_uri(source: &str, safe: SafeMode, files: &[(&str, &[u8])]) -> String {
            let dir = scratch(files);
            let html = convert_with(
                source,
                &Options::new()
                    .safe_mode(safe)
                    .base_dir(dir.clone())
                    .set("data-uri"),
            );
            let _ = fs::remove_dir_all(&dir);
            html
        }

        // A local image is read from disk and embedded as a base64 `data:` URI,
        // its MIME type taken from the extension.
        #[test]
        fn embeds_a_local_image_below_secure() {
            let html = convert_data_uri(
                "image::dot.gif[A dot,1,1]",
                SafeMode::Unsafe,
                &[("dot.gif", GIF)],
            );
            assert!(
                html.contains(&format!(
                    "<img src=\"data:image/gif;base64,{GIF_B64}\" alt=\"A dot\" width=\"1\" height=\"1\">"
                )),
                "{html}"
            );
        }

        // `Secure` (and above) disables `data-uri`: the image is linked, not
        // embedded, even with the attribute set.
        #[test]
        fn is_disabled_at_secure() {
            let html = convert_data_uri(
                "image::dot.gif[A dot]",
                SafeMode::Secure,
                &[("dot.gif", GIF)],
            );
            assert!(
                html.contains("<img src=\"dot.gif\" alt=\"A dot\">"),
                "{html}"
            );
            assert!(!html.contains("data:image/gif"), "{html}");
        }

        // Without the `data-uri` attribute, the image is linked as usual.
        #[test]
        fn is_disabled_without_the_attribute() {
            let dir = scratch(&[("dot.gif", GIF)]);
            let html = convert_with(
                "image::dot.gif[A dot]",
                &Options::new()
                    .safe_mode(SafeMode::Unsafe)
                    .base_dir(dir.clone()),
            );
            let _ = fs::remove_dir_all(&dir);
            assert!(
                html.contains("<img src=\"dot.gif\" alt=\"A dot\">"),
                "{html}"
            );
        }

        // The MIME type follows the target's extension: `.svg` is the special
        // `image/svg+xml`; any other extension is `image/<ext>` verbatim.
        #[test]
        fn mimetype_follows_the_extension() {
            let svg = convert_data_uri("image::pic.svg[X]", SafeMode::Unsafe, &[("pic.svg", GIF)]);
            assert!(
                svg.contains(&format!("data:image/svg+xml;base64,{GIF_B64}")),
                "{svg}"
            );

            let png = convert_data_uri("image::pic.png[X]", SafeMode::Unsafe, &[("pic.png", GIF)]);
            assert!(
                png.contains(&format!("data:image/png;base64,{GIF_B64}")),
                "{png}"
            );
        }

        // A target with no usable extension gets the generic
        // `application/octet-stream` MIME type — both when the target simply has
        // no dot, and when its only dot lies in a *directory* component (so the
        // file name itself has no extension), matching Asciidoctor's
        // `Helpers.extname` (a dot with a `/` after it is not an extension).
        #[test]
        fn an_extensionless_target_is_application_octet_stream() {
            // No dot anywhere in the target.
            let bare = convert_data_uri("image::photo[X]", SafeMode::Unsafe, &[("photo", GIF)]);
            assert!(
                bare.contains(&format!("data:application/octet-stream;base64,{GIF_B64}")),
                "{bare}"
            );

            // The only dot is in a directory component, so the file name has no
            // extension (exercises `asset_extname`'s dot-in-directory branch).
            let dotdir = convert_data_uri(
                "image::my.dir/pic[X]",
                SafeMode::Unsafe,
                &[("my.dir/pic", GIF)],
            );
            assert!(
                dotdir.contains(&format!("data:application/octet-stream;base64,{GIF_B64}")),
                "{dotdir}"
            );
        }

        // A `format` attribute does *not* steer the `data-uri` MIME type:
        // Asciidoctor's `generate_data_uri` derives it from the target's
        // extension alone, so an extensionless target with `format=png` still
        // embeds as `application/octet-stream`, not `image/png`. Verified
        // against Asciidoctor 2.0.26 — matching the oracle outranks the more
        // "intuitive" `image/png` a naive reading would expect. (`format` is
        // still honored for SVG detection, exercised elsewhere.)
        #[test]
        fn format_attribute_does_not_steer_the_data_uri_mimetype() {
            let html = convert_data_uri(
                "image::photo[X,format=png]",
                SafeMode::Unsafe,
                &[("photo", GIF)],
            );
            assert!(
                html.contains(&format!("data:application/octet-stream;base64,{GIF_B64}")),
                "{html}"
            );
            assert!(!html.contains("image/png"), "{html}");
        }

        // An unreadable (here, missing) image embeds as an *empty* data URI,
        // carrying only its derived MIME type — Asciidoctor's fallback for an
        // image it cannot embed.
        #[test]
        fn a_missing_image_embeds_as_an_empty_data_uri() {
            let html = convert_data_uri("image::nope.png[X]", SafeMode::Unsafe, &[]);
            assert!(
                html.contains("<img src=\"data:image/png;base64,\" alt=\"X\">"),
                "{html}"
            );
        }

        // A URI target is never embedded (a network read this crate does not
        // perform): it passes through unchanged, exactly as when `data-uri` is
        // off.
        #[test]
        fn a_uri_target_passes_through() {
            let html =
                convert_data_uri("image::https://example.com/x.png[X]", SafeMode::Unsafe, &[]);
            assert!(
                html.contains("<img src=\"https://example.com/x.png\" alt=\"X\">"),
                "{html}"
            );
        }

        // The image target resolves against `imagesdir` before it is read, so an
        // image in a subdirectory is found and embedded.
        #[test]
        fn resolves_the_target_against_imagesdir() {
            let html = convert_data_uri(
                ":imagesdir: assets\n\nimage::a.png[X]",
                SafeMode::Unsafe,
                &[("assets/a.png", GIF)],
            );
            assert!(
                html.contains(&format!("data:image/png;base64,{GIF_B64}")),
                "{html}"
            );
        }

        // Under a jailed safe mode (`safe`/`server`), a target that climbs out of
        // the base directory cannot be read, so it embeds as an empty data URI —
        // the read is confined to the jail. (`Unsafe` would follow it.)
        #[test]
        fn a_jailed_read_refuses_to_escape_the_base_directory() {
            // The image sits one level *above* the base directory. `resolve` is
            // relative to `base_dir`, so put the base a level down and reference
            // `../secret.png`.
            let root = scratch(&[("secret.png", GIF), ("base/doc.txt", b"x")]);
            let base = root.join("base");

            let html = convert_with(
                "image::../secret.png[X]",
                &Options::new()
                    .safe_mode(SafeMode::Safe)
                    .base_dir(base)
                    .set("data-uri"),
            );
            let _ = fs::remove_dir_all(&root);

            assert!(
                html.contains("<img src=\"data:image/png;base64,\" alt=\"X\">"),
                "{html}"
            );
            assert!(!html.contains(GIF_B64), "{html}");
        }

        // Image-mode icons embed too: an admonition's icon and a callout list's
        // icon both flow through `image_uri` (Asciidoctor's `icon_uri`), so
        // `data-uri` embeds them from `iconsdir`.
        #[test]
        fn embeds_image_mode_icons() {
            let html = convert_data_uri(
                ":icons:\n\nNOTE: Heed me.\n\n----\ncode <1>\n----\n<1> A callout.",
                SafeMode::Unsafe,
                &[
                    ("images/icons/note.png", GIF),
                    ("images/icons/callouts/1.png", GIF),
                ],
            );
            // The admonition icon.
            assert!(
                html.contains(&format!(
                    "<img src=\"data:image/png;base64,{GIF_B64}\" alt=\"Note\">"
                )),
                "{html}"
            );
            // The callout-list icon.
            assert!(
                html.contains(&format!(
                    "<img src=\"data:image/png;base64,{GIF_B64}\" alt=\"1\">"
                )),
                "{html}"
            );
        }

        // `data-uri` and `opts=inline` coexist: an inline SVG still embeds the
        // file's `<svg>` contents directly (Asciidoctor reads the file for
        // `opts=inline` regardless of `data-uri`). The read must use the plain
        // web path, not the `data:` URI `src` would otherwise carry — a
        // regression guard for that interaction.
        #[test]
        fn does_not_disturb_an_inline_svg() {
            let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"4\"/></svg>";
            let html = convert_data_uri(
                "image::c.svg[Circle,opts=inline]",
                SafeMode::Unsafe,
                &[("c.svg", svg)],
            );

            // The SVG is inlined (not embedded as a `data:` URI, not the alt
            // fallback).
            assert!(
                html.contains("<svg xmlns=\"http://www.w3.org/2000/svg\"><circle r=\"4\"/></svg>"),
                "{html}"
            );
            assert!(!html.contains("data:image/svg"), "{html}");
            assert!(!html.contains("<span class=\"alt\">"), "{html}");
        }
    }

    #[test]
    fn image_block_role_attribute_overrides_block_roles() {
        // A macro `role=` supplies the wrapper roles (splitting on whitespace),
        // overriding any `[.role]` on the block line — matching Asciidoctor's
        // last-write-wins merge of the `role` attribute.
        let html = convert("[.ignored]\nimage::r.png[role=\"r1 r2\"]");
        assert!(html.contains("<div class=\"imageblock r1 r2\">"));
    }

    #[test]
    fn image_block_link_to_blank_window_adds_target_and_noopener() {
        let html = convert("image::a.png[link=https://x.com,window=_blank]");
        assert!(html.contains(
            "<a class=\"image\" href=\"https://x.com\" target=\"_blank\" rel=\"noopener\">"
        ));
    }

    #[test]
    fn image_block_link_nofollow_option_adds_rel() {
        let html = convert("image::b.png[link=https://x.com,opts=nofollow]");
        assert!(html.contains("<a class=\"image\" href=\"https://x.com\" rel=\"nofollow\">"));
    }

    #[test]
    fn image_block_link_named_window_adds_only_target() {
        // A named window that is not `_blank` and carries no `noopener` option
        // sets `target` alone — no `rel`.
        let html = convert("image::x.png[link=https://x.com,window=help]");
        assert!(html.contains("<a class=\"image\" href=\"https://x.com\" target=\"help\">"));
    }

    #[test]
    fn image_block_link_named_window_with_noopener() {
        let html = convert("image::c.png[link=https://x.com,window=name,opts=noopener]");
        assert!(html.contains(
            "<a class=\"image\" href=\"https://x.com\" target=\"name\" rel=\"noopener\">"
        ));
    }

    #[test]
    fn image_block_link_blank_window_with_nofollow_folds_both_into_rel() {
        let html = convert("image::d.png[link=https://x.com,window=_blank,opts=nofollow]");
        assert!(html.contains(
            "<a class=\"image\" href=\"https://x.com\" target=\"_blank\" rel=\"nofollow noopener\">"
        ));
    }

    #[test]
    fn media_uri_matches_web_path() {
        use super::media_uri;

        // A bare relative target keeps no `./` prefix (unlike a stylesheet web
        // path); an `imagesdir` is joined ahead of it.
        assert_eq!(media_uri("movie.mp4", ""), "movie.mp4");
        assert_eq!(media_uri("movie.mp4", "media"), "media/movie.mp4");
        assert_eq!(media_uri("movie.mp4", "media/"), "media/movie.mp4");
        assert_eq!(media_uri("movie.mp4", "./media"), "./media/movie.mp4");

        // A web-absolute target ignores `imagesdir`; `.`/`..` segments collapse,
        // and a leading `..` at the web root is dropped — but a leading `..` on
        // a bare relative path is kept as a relative step.
        assert_eq!(media_uri("/abs.mp4", "media"), "/abs.mp4");
        assert_eq!(media_uri("sub/../c.mp4", "media"), "media/c.mp4");
        assert_eq!(media_uri("/x/../../y.mp4", ""), "/y.mp4");
        assert_eq!(media_uri("../up.mp4", ""), "../up.mp4");

        // Backslashes are posixified unconditionally, matching this crate's
        // other web-path helpers (Asciidoctor only does so on Windows); a URI
        // target is preserved with its spaces percent-encoded and ignores
        // `imagesdir`.
        assert_eq!(media_uri("a\\b.mp4", "media"), "media/a/b.mp4");
        assert_eq!(
            media_uri("https://ex.com/a b.mp4", "media"),
            "https://ex.com/a%20b.mp4"
        );
    }

    #[test]
    fn video_self_hosted_renders_video_element() {
        // The `width` positional maps to the `width` attribute; `controls` is
        // present unless `nocontrols` is set. Matches Asciidoctor's
        // `convert_video`.
        let html = crate::convert("video::movie.mp4[width=640]");
        assert_eq!(
            html.trim_end(),
            "<div class=\"videoblock\">\n\
             <div class=\"content\">\n\
             <video src=\"movie.mp4\" width=\"640\" controls>\n\
             Your browser does not support the video tag.\n\
             </video>\n\
             </div>\n\
             </div>"
        );
    }

    #[test]
    fn audio_renders_audio_element() {
        let html = crate::convert("audio::podcast.mp3[]");
        assert_eq!(
            html.trim_end(),
            "<div class=\"audioblock\">\n\
             <div class=\"content\">\n\
             <audio src=\"podcast.mp3\" controls>\n\
             Your browser does not support the audio tag.\n\
             </audio>\n\
             </div>\n\
             </div>"
        );
    }

    #[test]
    fn video_title_start_end_and_options() {
        // A title becomes the `<div class="title">`; `start`/`end` form the
        // `#t=` media fragment; `autoplay`/`loop` are boolean attributes and
        // `nocontrols` suppresses `controls`.
        let html = crate::convert(
            ".My video\nvideo::vid.webm[start=10,end=20,opts=\"autoplay,loop,nocontrols\"]",
        );
        assert_eq!(
            html.trim_end(),
            "<div class=\"videoblock\">\n\
             <div class=\"title\">My video</div>\n\
             <div class=\"content\">\n\
             <video src=\"vid.webm#t=10,20\" autoplay loop>\n\
             Your browser does not support the video tag.\n\
             </video>\n\
             </div>\n\
             </div>"
        );
    }

    #[test]
    fn audio_start_only_and_autoplay() {
        // A lone `start` yields `#t=<start>`; a lone `end` would yield
        // `#t=,<end>`. Audio has no `muted` option.
        let html = convert("audio::a.ogg[opts=autoplay,start=5]");
        assert!(content(&html).contains("<audio src=\"a.ogg#t=5\" autoplay controls>"));
    }

    #[test]
    fn video_youtube_embed() {
        // A `youtube` poster value selects the YouTube `<iframe>` embed; the
        // `?rel=0` parameter is always present and `allowfullscreen` is on by
        // default.
        let html = crate::convert("video::12345[youtube]");
        assert_eq!(
            html.trim_end(),
            "<div class=\"videoblock\">\n\
             <div class=\"content\">\n\
             <iframe src=\"https://www.youtube.com/embed/12345?rel=0\" frameborder=\"0\" allowfullscreen></iframe>\n\
             </div>\n\
             </div>"
        );
    }

    #[test]
    fn video_youtube_dynamic_playlist() {
        // A comma-separated target is a dynamic playlist: the embed targets the
        // first id and adds a `playlist=` of the whole list.
        let html = convert("video::v1,v2,v3[youtube]");
        assert!(content(&html).contains(
            "src=\"https://www.youtube.com/embed/v1?rel=0&amp;playlist=v1,v2,v3\" frameborder=\"0\" allowfullscreen"
        ));
    }

    #[test]
    fn video_vimeo_embed() {
        // A `vimeo` poster value selects the Vimeo `<iframe>` embed; `width`
        // is honored on the iframe.
        let html = crate::convert("video::67890[vimeo,width=300]");
        assert_eq!(
            html.trim_end(),
            "<div class=\"videoblock\">\n\
             <div class=\"content\">\n\
             <iframe width=\"300\" src=\"https://player.vimeo.com/video/67890\" frameborder=\"0\" allowfullscreen></iframe>\n\
             </div>\n\
             </div>"
        );
    }

    #[test]
    fn media_id_and_role_on_wrapper() {
        // The `[#id.role]` shorthand above the macro puts the id on the wrapper
        // and appends the role to the class list.
        let html = convert("[#myid.myrole]\nvideo::x.mp4[]");
        assert!(content(&html).starts_with("<div id=\"myid\" class=\"videoblock myrole\">"));
    }

    #[test]
    fn video_macro_role_overrides_block_role() {
        // A `role=` inside the macro wins outright over the block's shorthand
        // role, matching Asciidoctor's `node.role`.
        let html = convert("[#aa.bb]\naudio::s.mp3[role=cc]");
        assert!(content(&html).starts_with("<div id=\"aa\" class=\"audioblock cc\">"));
    }

    #[test]
    fn video_float_and_align_classes() {
        // `float` and `text-<align>` classes sit between `videoblock` and the
        // role(s).
        let html = convert("video::z.mp4[float=left,align=center,role=baz]");
        assert!(content(&html).starts_with("<div class=\"videoblock left text-center baz\">"));
    }

    #[test]
    fn video_poster_and_preload_self_hosted() {
        // A non-service `poster` value is a poster image resolved through
        // `imagesdir`; `preload` is emitted verbatim. `muted` is a boolean
        // attribute for video (but not audio).
        let html = convert(
            ":imagesdir: media\n\nvideo::v.mp4[height=200,poster=thumb.jpg,preload=auto,opts=\"muted,nocontrols\"]",
        );
        assert!(content(&html).contains(
            "<video src=\"media/v.mp4\" height=\"200\" poster=\"media/thumb.jpg\" muted preload=\"auto\">"
        ));
    }

    #[test]
    fn media_uri_prefixes_imagesdir_and_preserves_uri_target() {
        // A bare target is resolved against `imagesdir` with no `./` prefix; a
        // URI target is preserved (spaces percent-encoded) and ignores
        // `imagesdir`.
        let html = convert(
            ":imagesdir: assets\n\nvideo::sub/clip.webm[]\n\nvideo::https://ex.com/a b.mp4[]",
        );
        let body = content(&html);
        assert!(body.contains("<video src=\"assets/sub/clip.webm\" controls>"));
        assert!(body.contains("<video src=\"https://ex.com/a%20b.mp4\" controls>"));
    }

    #[test]
    fn video_vimeo_hash_query_params_and_nofullscreen() {
        // A `<video-id>/<hash>` target supplies the `h=` parameter; the query
        // params are introduced by `?` then `&amp;`; `nofullscreen` drops the
        // `allowfullscreen` attribute.
        let html = convert("video::vid/myhash[vimeo,opts=\"autoplay,loop,muted,nofullscreen\"]");
        assert!(content(&html).contains(
            "<iframe src=\"https://player.vimeo.com/video/vid?h=myhash&amp;autoplay=1&amp;loop=1&amp;muted=1\" frameborder=\"0\"></iframe>"
        ));
    }

    #[test]
    fn video_youtube_list_options_and_nofullscreen() {
        // A `<video-id>/<list-id>` target sets `list=`; the query flags and the
        // `fs=0`/no-`allowfullscreen` pair for `nofullscreen` are all covered.
        let html = convert(
            "video::abc/PL123[youtube,start=30,end=90,opts=\"autoplay,loop,modest,nocontrols,nofullscreen\"]",
        );
        assert!(content(&html).contains(
            "<iframe src=\"https://www.youtube.com/embed/abc?rel=0&amp;start=30&amp;end=90&amp;autoplay=1&amp;loop=1&amp;controls=0&amp;list=PL123&amp;fs=0&amp;modestbranding=1\" frameborder=\"0\"></iframe>"
        ));
    }

    #[test]
    fn video_youtube_loop_falls_back_to_video_id_playlist() {
        // With `loop` but no explicit list/playlist, the playlist falls back to
        // the video id so the loop works.
        let html = convert("video::vid[youtube,opts=loop]");
        assert!(content(&html).contains(
            "src=\"https://www.youtube.com/embed/vid?rel=0&amp;loop=1&amp;playlist=vid\""
        ));
    }

    #[test]
    fn video_embed_empty_asset_uri_scheme_is_scheme_relative() {
        // An empty `asset-uri-scheme` yields a scheme-relative `//…` embed URL.
        let html = convert_with(
            "video::v[youtube]",
            &Options::new().attribute("asset-uri-scheme", ""),
        );
        assert!(content(&html).contains("src=\"//www.youtube.com/embed/v?rel=0\""));
    }

    #[test]
    fn unordered_list_renders_ulist() {
        let html = convert("* one\n* two");
        assert!(html.contains(
            "<div class=\"ulist\">\n<ul>\n<li>\n<p>one</p>\n</li>\n<li>\n<p>two</p>\n</li>\n</ul>\n</div>"
        ));
    }

    #[test]
    fn ordered_list_renders_olist_with_style() {
        let html = convert(". one\n. two");
        assert!(html.contains(
            "<div class=\"olist arabic\">\n<ol class=\"arabic\">\n<li>\n<p>one</p>\n</li>"
        ));
    }

    #[test]
    fn ordered_list_honors_an_explicit_numbering_style() {
        // An explicit `[loweralpha]` overrides the marker-derived style (the
        // marker here is a plain `.`, which alone would be arabic), driving both
        // the wrapper/`<ol>` class and the HTML `type`.
        let html = convert("[loweralpha]\n. one\n. two");
        assert!(
            html.contains("<div class=\"olist loweralpha\">\n<ol class=\"loweralpha\" type=\"a\">")
        );
    }

    #[test]
    fn ordered_list_passes_a_non_numbering_style_through() {
        // A declared style that is not a numbering keyword is still passed
        // through as the wrapper/`<ol>` class, matching Asciidoctor; the `<ol>`
        // then carries no HTML `type`, since only the numbering keywords map to
        // one.
        let html = convert("[foo]\n. one\n. two");
        assert!(
            html.contains("<div class=\"olist foo\">\n<ol class=\"foo\">\n<li>"),
            "{html}"
        );
    }

    #[test]
    fn callout_list_renders_ol() {
        // A callout list annotating a verbatim block renders as `<div
        // class="colist arabic"><ol>` of `<li><p>…</p></li>`, and the parser
        // substitutes the callout numbers into the `<pre>` as `<b
        // class="conum">(n)</b>`.
        let html = convert("----\ncode <1>\nmore <2>\n----\n\n<1> first\n<2> second");
        assert!(html.contains("code <b class=\"conum\">(1)</b>"));
        assert!(html.contains(
            "<div class=\"colist arabic\">\n<ol>\n<li>\n<p>first</p>\n</li>\n<li>\n<p>second</p>\n</li>\n</ol>\n</div>"
        ));
    }

    #[test]
    fn callout_list_with_icons_renders_image_table() {
        // With `icons` set, the list becomes a two-column `<table>` whose first
        // cell is the numbered callout image.
        let html = crate::convert_with(
            "----\ncode <1>\n----\n\n<1> first",
            &Options::new().attribute("icons", ""),
        );
        assert!(html.contains(
            "<div class=\"colist arabic\">\n<table>\n<tr>\n\
             <td><img src=\"./images/icons/callouts/1.png\" alt=\"1\"></td>\n\
             <td>first</td>\n</tr>\n</table>\n</div>"
        ));
    }

    #[test]
    fn callout_icon_src_escapes_hostile_iconsdir() {
        // A double quote in `iconsdir` must not break out of the callout list's
        // `src` attribute: the path is attribute-escaped (`"` -> `&quot;`), so no
        // raw event-handler markup reaches the `<td>` image. (The image the
        // parser substitutes into the preceding `<pre>` is rendered by
        // `asciidoc-parser`, not here.)
        let html = crate::convert_with(
            "----\ncode <1>\n----\n\n<1> first",
            &Options::new()
                .attribute("icons", "")
                .attribute("iconsdir", "x\"onerror=alert(1)"),
        );
        assert!(html
            .contains("<td><img src=\"x&quot;onerror=alert(1)/callouts/1.png\" alt=\"1\"></td>"));
    }

    #[test]
    fn callout_list_with_icons_font_renders_conum_table() {
        // Under `:icons: font`, the first cell holds the Font Awesome
        // `<i class="conum">`/`<b>` pair.
        let html = crate::convert_with(
            "----\ncode <1>\n----\n\n<1> first",
            &Options::new().attribute("icons", "font"),
        );
        assert!(html.contains(
            "<div class=\"colist arabic\">\n<table>\n<tr>\n\
             <td><i class=\"conum\" data-value=\"1\"></i><b>1</b></td>\n\
             <td>first</td>\n</tr>\n</table>\n</div>"
        ));
    }

    #[test]
    fn callout_list_with_icons_appends_attached_blocks_in_the_cell() {
        // An icon-based callout item that carries attached blocks (a
        // continuation paragraph here) renders them inside the second `<td>`,
        // after the principal text and with no trailing newline before
        // `</td>`, matching Asciidoctor's `#{item.text}#{LF + item.content}`.
        let html = crate::convert_with(
            "----\ncode <1>\n----\n\n<1> first line\n+\nsecond paragraph\n",
            &Options::new().attribute("icons", "font"),
        );
        assert!(html.contains(
            "<td><i class=\"conum\" data-value=\"1\"></i><b>1</b></td>\n\
             <td>first line\n<div class=\"paragraph\">\n<p>second paragraph</p>\n</div></td>"
        ));
    }

    #[test]
    fn checklist_renders_default_ballot_markers() {
        // An unordered list with any checkbox item becomes a checklist: the
        // wrapper and `<ul>` gain the `checklist` class, and each checkbox item's
        // text is prefixed with the ballot-box entity — `&#10063;` unchecked,
        // `&#10003;` checked. A plain item in the same list keeps a bare `<p>`.
        let html = convert("* [ ] todo\n* [x] done\n* plain");
        assert!(html.contains("<div class=\"ulist checklist\">"));
        assert!(html.contains("<ul class=\"checklist\">"));
        assert!(html.contains("<p>&#10063; todo</p>"));
        assert!(html.contains("<p>&#10003; done</p>"));
        assert!(html.contains("<p>plain</p>"));
    }

    #[test]
    fn checklist_interactive_renders_input_checkboxes() {
        // The `%interactive` option swaps the entity markers for real
        // `<input type="checkbox">` controls, `checked` for a checked item.
        let html = convert("[%interactive]\n* [ ] todo\n* [x] done");
        assert!(html.contains("<p><input type=\"checkbox\" data-item-complete=\"0\"> todo</p>"));
        assert!(
            html.contains("<p><input type=\"checkbox\" data-item-complete=\"1\" checked> done</p>")
        );
    }

    #[test]
    fn checklist_with_icons_font_renders_font_awesome_markers() {
        // Under `:icons: font`, a non-interactive checklist uses Font Awesome
        // glyphs instead of the ballot-box entities.
        let html = crate::convert_with(
            "* [ ] todo\n* [x] done",
            &Options::new().attribute("icons", "font"),
        );
        assert!(html.contains("<p><i class=\"fa fa-square-o\"></i> todo</p>"));
        assert!(html.contains("<p><i class=\"fa fa-check-square-o\"></i> done</p>"));
    }

    #[test]
    fn list_item_id_and_role_decorate_the_li() {
        // When the parser attaches an id and/or roles to a list item (here via a
        // block anchor / shorthand before the item), the renderer places them on
        // the `<li>`, matching Asciidoctor's `convert_ulist` item loop: id first,
        // then the item's roles as its class.
        let id_only = convert("* one\n[[second]]\n* two");
        assert!(id_only.contains("<li id=\"second\">\n<p>two</p>"));

        let role_only = convert("* one\n[.special]\n* two");
        assert!(role_only.contains("<li class=\"special\">\n<p>two</p>"));

        let id_and_role = convert("* one\n[#second.special]\n* two");
        assert!(id_and_role.contains("<li id=\"second\" class=\"special\">\n<p>two</p>"));
    }

    #[test]
    fn description_list_renders_dlist() {
        // A plain description list is `<div class="dlist"><dl>`, each term a
        // `<dt class="hdlist1">` and each description a `<dd>` holding the
        // principal text as a bare `<p>`, matching Asciidoctor's `convert_dlist`.
        let html = convert("CPU:: The brain\nRAM:: The memory");
        assert!(html.contains(
            "<div class=\"dlist\">\n<dl>\n\
             <dt class=\"hdlist1\">CPU</dt>\n<dd>\n<p>The brain</p>\n</dd>\n\
             <dt class=\"hdlist1\">RAM</dt>\n<dd>\n<p>The memory</p>\n</dd>\n</dl>\n</div>"
        ));
    }

    #[test]
    fn description_list_folds_subsequent_line_and_appends_continuation_block() {
        // A term with no inline text folds the immediately-following paragraph
        // into its principal text (a bare `<p>`), while a paragraph attached by
        // a `+` continuation stays a block (`<div class="paragraph">`).
        let html = convert("term::\ndef\n+\nattached");
        assert!(html.contains(
            "<dt class=\"hdlist1\">term</dt>\n<dd>\n<p>def</p>\n\
             <div class=\"paragraph\">\n<p>attached</p>\n</div>\n</dd>"
        ));
    }

    #[test]
    fn description_list_groups_multiple_terms_with_one_description() {
        // Consecutive terms with no description of their own share the next
        // term's `<dd>`: each becomes its own `<dt>` ahead of the single `<dd>`.
        let html = convert("term1::\nterm2:: shared");
        assert!(html.contains(
            "<dt class=\"hdlist1\">term1</dt>\n\
             <dt class=\"hdlist1\">term2</dt>\n<dd>\n<p>shared</p>\n</dd>"
        ));
    }

    #[test]
    fn qanda_description_list_renders_ordered_questions() {
        // The `qanda` style renders `<div class="qlist qanda"><ol>`, each entry
        // an `<li>` with emphasized `<p><em>…</em></p>` questions.
        let html = convert("[qanda]\nWhat?:: This.");
        assert!(html.contains(
            "<div class=\"qlist qanda\">\n<ol>\n<li>\n<p><em>What?</em></p>\n<p>This.</p>\n</li>\n</ol>\n</div>"
        ));
    }

    #[test]
    fn horizontal_description_list_renders_table() {
        // The `horizontal` style renders `<div class="hdlist"><table>`, the
        // terms in an `hdlist1` label cell and the description in an `hdlist2`
        // cell.
        let html = convert("[horizontal]\nCPU:: brain");
        assert!(html.contains(
            "<div class=\"hdlist\">\n<table>\n<tr>\n\
             <td class=\"hdlist1\">\nCPU\n</td>\n\
             <td class=\"hdlist2\">\n<p>brain</p>\n</td>\n</tr>\n</table>\n</div>"
        ));
    }

    #[test]
    fn horizontal_description_list_sets_column_widths() {
        // `labelwidth`/`itemwidth` emit a `<colgroup>` whose `<col>`s carry the
        // widths as inline styles (Asciidoctor's `style="width: N%;"`), and the
        // `strong` option bolds the label cell.
        let html = convert("[horizontal%strong,labelwidth=20%,itemwidth=80%]\nCPU:: brain");
        assert!(html.contains(
            "<colgroup>\n<col style=\"width: 20%;\">\n<col style=\"width: 80%;\">\n</colgroup>"
        ));
        assert!(html.contains("<td class=\"hdlist1 strong\">"));
    }

    #[test]
    fn horizontal_description_list_with_one_column_width_leaves_the_other_bare() {
        // With only `labelwidth` set, Asciidoctor still emits both `<col>`s: the
        // label column carries its width, the item column is a bare `<col>`.
        let html = convert("[horizontal,labelwidth=30%]\nCPU:: brain");
        assert!(html.contains("<colgroup>\n<col style=\"width: 30%;\">\n<col>\n</colgroup>"));
    }

    #[test]
    fn dlist_narrowing_helpers_handle_both_arms() {
        // `dlist_entries` narrows through these helpers on the assumption that a
        // description list only holds `DefinedTerm` list items. That always
        // holds during rendering, so exercise both arms directly here: a
        // non-list block and a non-description marker take the `None` paths a
        // real document never reaches.
        use asciidoc_parser::{blocks::FindBlocks, Parser};

        use super::{as_list_item, dlist_term_text};

        // A bullet list: the `<ul>` block is not a list item, its child is, and
        // that child's marker is not a description term. `child_blocks()` on the
        // `&Block` descends into the list's items without a narrowing branch.
        let mut parser = Parser::default();
        let ulist = parser.parse("* bullet\n");
        let list_block = ulist.child_blocks().next().unwrap();
        assert!(as_list_item(list_block).is_none());

        let bullet_item = as_list_item(list_block.child_blocks().next().unwrap())
            .expect("a list's child is always a list item");
        assert!(dlist_term_text(bullet_item).is_none());

        // A description list: the item's `DefinedTerm` marker yields its term.
        let mut parser = Parser::default();
        let dlist = parser.parse("term:: def\n");
        let dlist_block = dlist.child_blocks().next().unwrap();
        let term_item = as_list_item(dlist_block.child_blocks().next().unwrap()).unwrap();
        assert_eq!(dlist_term_text(term_item).as_deref(), Some("term"));
    }

    // Comments render to nothing, matching Asciidoctor. The parser preserves
    // them; the renderer drops them (see `renders_nothing`). These use the real
    // embedded `crate::convert` (the module's `convert` is shadowed to
    // standalone) so the assertions see only the body.

    #[test]
    fn block_comment_is_dropped() {
        let html =
            crate::convert("first paragraph\n\n////\nblock comment\n////\n\nsecond paragraph");
        assert!(!html.contains("block comment"));
        assert!(!html.contains("unsupported"));
        assert_eq!(html.matches("class=\"paragraph\"").count(), 2);
    }

    #[test]
    fn isolated_line_comment_creates_no_empty_paragraph() {
        // An isolated `//` line survives parsing as an empty paragraph; it must
        // not render as `<p></p>`, so only the two real paragraphs remain.
        let html = crate::convert("first paragraph\n\n// line comment\n\nsecond paragraph");
        assert!(!html.contains("line comment"));
        assert!(!html.contains("<p></p>"));
        assert_eq!(html.matches("<p>").count(), 2);
    }

    #[test]
    fn empty_inline_passthrough_paragraph_renders_an_empty_p() {
        // A paragraph whose entire content is an empty inline passthrough
        // substitutes to nothing, but — unlike a line comment — it is real
        // content, so Asciidoctor keeps it as `<p></p>`. The two spellings
        // (`pass:[]` and `$$$$`) both render the empty paragraph.
        for input in ["pass:[]", "$$$$"] {
            let html = crate::convert(input);
            assert_eq!(
                html.trim_end(),
                "<div class=\"paragraph\">\n<p></p>\n</div>"
            );
        }
    }

    #[test]
    fn adjacent_line_comment_is_stripped_within_a_paragraph() {
        // A `//` line between two content lines is stripped, joining them into a
        // single paragraph rather than dropping the whole block.
        let html = crate::convert("first line\n// line comment\nsecond line");
        assert!(!html.contains("line comment"));
        assert!(html.contains("<p>first line\nsecond line</p>"));
    }

    #[test]
    fn comment_styled_paragraph_is_dropped() {
        let html = crate::convert("Before.\n\n[comment]\nhidden text\nmore\n\nAfter.");
        assert!(!html.contains("hidden text"));
        assert_eq!(html.matches("class=\"paragraph\"").count(), 2);
    }

    #[test]
    fn comment_styled_open_block_is_dropped() {
        let html = crate::convert("Before.\n\n[comment]\n--\nhidden\n--\n\nAfter.");
        assert!(!html.contains("hidden"));
        assert!(!html.contains("unsupported"));
        assert_eq!(html.matches("class=\"paragraph\"").count(), 2);
    }

    #[test]
    fn triple_slash_is_not_a_line_comment() {
        // Only a `//` prefix begins a line comment; `///` is ordinary text.
        let html = crate::convert("/// not a line comment");
        assert!(html.contains("/// not a line comment"));
    }

    #[test]
    fn is_line_comment_paragraph_classifies_sources() {
        use super::is_line_comment_paragraph;

        // A single line comment, and several stacked comment lines, are comment
        // paragraphs; the multi-line case exercises the per-line scan.
        assert!(is_line_comment_paragraph("// a comment"));
        assert!(is_line_comment_paragraph("//c1\n//c2\n//c3"));

        // A bare `//` (empty comment) still counts.
        assert!(is_line_comment_paragraph("//"));

        // A whitespace-only line among the comments is skipped, not treated as
        // non-comment content — the branch a paragraph's own source never
        // reaches (a blank line ends the paragraph), so it is pinned here.
        assert!(is_line_comment_paragraph("//c1\n   \n//c2"));
        assert!(is_line_comment_paragraph("//c1\n\t\n//c2"));

        // Any non-comment line disqualifies the whole paragraph: real text, an
        // empty inline passthrough (the #200 case), or a `///` that is ordinary
        // text rather than a comment marker.
        assert!(!is_line_comment_paragraph("pass:[]"));
        assert!(!is_line_comment_paragraph("$$$$"));
        assert!(!is_line_comment_paragraph("/// not a comment"));
        assert!(!is_line_comment_paragraph("//c1\ntext"));
        assert!(!is_line_comment_paragraph("text\n//c1"));

        // A source with no non-blank line is not a comment paragraph.
        assert!(!is_line_comment_paragraph(""));
        assert!(!is_line_comment_paragraph("   \n\t"));
    }

    #[test]
    fn block_comment_at_end_of_document_creates_no_paragraph() {
        // Trailing newlines after a closing comment block must not produce a
        // spurious empty paragraph.
        let html = crate::convert("paragraph\n\n////\nblock comment\n////\n\n\n");
        assert!(!html.contains("block comment"));
        assert_eq!(html.matches("class=\"paragraph\"").count(), 1);
    }

    #[test]
    fn block_title_and_roles_appear_on_wrapper() {
        let html = convert(".A caption\n[.lead]\nParagraph text.");
        assert!(html.contains("<div class=\"paragraph lead\">"));
        assert!(html.contains("<div class=\"title\">A caption</div>"));
    }

    // The following exercise the document-attribute-driven skeleton, reading
    // resolved attributes straight off the `Document`.

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

    #[test]
    fn footer_stamps_last_updated_docdatetime() {
        // A standalone document's footer stamps "{last-update-label}
        // {docdatetime}". A pinned reference time makes the stamp deterministic:
        // 2019-01-02 03:04:05 at a +06:00 offset.
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new().reference_time(ReferenceTime::from_local(
                2019,
                1,
                2,
                3,
                4,
                5,
                6 * 3600,
            )),
        );

        assert!(
            html.contains("Last updated 2019-01-02 03:04:05 +0600"),
            "footer should stamp the docdatetime: {html}"
        );
    }

    #[test]
    fn input_mtime_drives_the_footer_stamp_over_the_reference_time() {
        // The footer's docdatetime belongs to the `doc*` family, which follows a
        // pinned input mtime even as the reference time ("now", the `local*`
        // family) differs – mirroring Asciidoctor's `input_mtime`.
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new()
                .reference_time(ReferenceTime::from_unix_timestamp(0))
                .input_mtime(ReferenceTime::from_local(2019, 1, 2, 3, 4, 5, 0)),
        );

        assert!(
            html.contains("Last updated 2019-01-02 03:04:05 UTC"),
            "footer docdatetime should follow the input mtime: {html}"
        );
        assert!(
            !html.contains("1970"),
            "the reference time should not leak in: {html}"
        );
    }

    #[test]
    fn reproducible_suppresses_the_footer_stamp_and_generator_meta() {
        // `reproducible` drops the two build-volatile pieces – the "Last
        // updated" stamp and the `generator` meta – so the output is stable
        // across builds, matching Asciidoctor.
        let html = convert_with(
            "= Doc\n:reproducible:\n\nBody.",
            &Options::new().reference_time(ReferenceTime::from_unix_timestamp(0)),
        );

        assert!(
            !html.contains("Last updated"),
            "the footer stamp should be suppressed: {html}"
        );
        assert!(
            !html.contains("name=\"generator\""),
            "the generator meta should be suppressed: {html}"
        );

        // Only the footer's text is dropped; the footer frame itself remains.
        assert!(html.contains("<div id=\"footer\">"));
    }

    #[test]
    fn footer_shows_version_label_and_revnumber() {
        // A revision line's number drives the footer's "{version-label}
        // {revnumber}" line; `version-label` defaults to "Version".
        let html = convert_with(
            "= Doc\nAuthor Name\nv2.0, 2020-01-01\n\nBody.",
            &Options::new().reference_time(ReferenceTime::from_unix_timestamp(0)),
        );

        assert!(
            html.contains("Version 2.0<br>"),
            "footer should show the version line: {html}"
        );
    }

    #[test]
    fn unsetting_last_update_label_drops_the_footer_stamp() {
        // Unsetting `last-update-label` removes the "Last updated" stamp while
        // leaving the footer frame (and its now-empty text div) in place,
        // matching Asciidoctor's `(attr? 'last-update-label')` gate.
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new()
                .unset("last-update-label")
                .reference_time(ReferenceTime::from_unix_timestamp(0)),
        );

        assert!(
            !html.contains("Last updated"),
            "the footer stamp should be gone: {html}"
        );
        assert!(html.contains("<div id=\"footer-text\">"));
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
        // The name arrives already header-substituted (`&` encoded to `&amp;`),
        // so it is placed as-is. The email is raw and must be escaped by the
        // renderer — otherwise a `"` would break out of the `mailto:` href.
        let html = convert("= Doc\nBen & Jerry <a\"b@example.com>\n\nBody.");
        assert!(html.contains("<span id=\"author\" class=\"author\">Ben &amp; Jerry</span>"));
        assert!(html.contains(
            "<span id=\"email\" class=\"email\"><a href=\"mailto:a&quot;b@example.com\">a&quot;b@example.com</a></span>"
        ));
    }

    #[test]
    fn byline_author_name_gets_replacements() {
        // Asciidoctor runs the replacements step on the byline author name, so a
        // typewriter apostrophe becomes typographic and `(C)` the copyright
        // sign — matching `asciidoctor`'s output exactly.
        let html = convert("= Doc\nJoan O'Brien (C)\n\nBody.");
        assert!(
            html.contains("<span id=\"author\" class=\"author\">Joan O&#8217;Brien &#169;</span>")
        );
    }

    #[test]
    fn author_meta_joins_and_escapes_the_authors() {
        // The head `<meta name="author">` carries the comma-joined author names.
        let html = convert("= Doc\nKismet Lee; Pax Draeke\n\nBody.");
        assert!(html.contains("<meta name=\"author\" content=\"Kismet Lee, Pax Draeke\">"));

        // The joined value is raw, so the renderer escapes it for the attribute
        // context — Asciidoctor escapes (rather than strips) any angle brackets.
        let html = convert("= Doc\n:author: Foo <bar> Baz\n\nBody.");
        assert!(html.contains("<meta name=\"author\" content=\"Foo &lt;bar&gt; Baz\">"));
    }

    #[test]
    fn app_name_and_copyright_metas_escape_markup() {
        // `app-name` and `copyright` come from attribute entries, so the parser
        // has already special-char-escaped their values and the renderer emits
        // them verbatim. A value containing markup therefore reaches the page
        // escaped — not as live markup — matching Asciidoctor. (Escaping here
        // would double-escape and diverge from the oracle.)
        let html =
            convert("= Doc\n:app-name: <script>x</script>\n:copyright: A & <b>B</b>\n\nBody.");
        assert!(
            html.contains(
                "<meta name=\"application-name\" content=\"&lt;script&gt;x&lt;/script&gt;\">"
            ),
            "{html}"
        );
        assert!(
            html.contains("<meta name=\"copyright\" content=\"A &amp; &lt;b&gt;B&lt;/b&gt;\">"),
            "{html}"
        );
    }

    #[test]
    fn discrete_heading_carries_discrete_class_and_roles() {
        let html = convert("= Doc\n\n[.independent]\n[discrete]\n== Free Heading");
        assert!(content(&html)
            .contains("<h2 id=\"_free_heading\" class=\"discrete independent\">Free Heading</h2>"));
    }

    #[test]
    fn floating_title_leading_class_is_the_style_keyword() {
        // A `[float]` heading keeps `float` as its leading class (not
        // `discrete`), matching Asciidoctor; roles still follow.
        let html = convert("= Doc\n\n[.independent]\n[float]\n=== Free Heading");
        assert!(
            content(&html)
                .contains("<h3 id=\"_free_heading\" class=\"float independent\">Free Heading</h3>"),
            "{}",
            content(&html)
        );
    }

    #[test]
    fn numbered_sections_prefix_the_heading_with_the_section_number() {
        // With `:sectnums:` each heading carries its dotted section number and a
        // trailing separator ahead of the title, matching Asciidoctor.
        let html = convert("= Doc\n:sectnums:\n\n== One\n\ntext\n\n=== One One\n\n== Two");
        let body = content(&html);
        assert!(body.contains("<h2 id=\"_one\">1. One</h2>"), "{body}");
        assert!(
            body.contains("<h3 id=\"_one_one\">1.1. One One</h3>"),
            "{body}"
        );
        assert!(body.contains("<h2 id=\"_two\">2. Two</h2>"), "{body}");
    }

    #[test]
    fn unnumbered_sections_show_the_bare_title() {
        // Without `:sectnums:` the parser assigns no section number, so the
        // heading is just the title.
        let html = convert("= Doc\n\n== One\n\n== Two");
        let body = content(&html);
        assert!(body.contains("<h2 id=\"_one\">One</h2>"), "{body}");
        assert!(body.contains("<h2 id=\"_two\">Two</h2>"), "{body}");
    }

    #[test]
    fn appendix_heading_carries_its_caption_prefix() {
        // An appendix is always lettered and captioned, even without
        // `:sectnums:`; the caption prefix precedes the title.
        let html = convert("= Doc\n\n[appendix]\n== Grammar\n\ntext");
        assert!(
            content(&html).contains("<h2 id=\"_grammar\">Appendix A: Grammar</h2>"),
            "{}",
            content(&html)
        );
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
    fn revision_line_with_only_a_date_omits_the_version() {
        // A revision line that carries no version number – just a date, which
        // does not match the `v<digit>` standalone-revision shape – yields no
        // `revnumber`, so only the `revdate` span is emitted. Matches
        // Asciidoctor 2.0.26.
        let html = convert("= Doc\nJane Doe\n2026-01-01\n\nBody.");
        assert!(
            html.contains("<span id=\"revdate\">2026-01-01</span>"),
            "{html}"
        );
        assert!(!html.contains("id=\"revnumber\""), "{html}");
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
    fn delimited_passthrough_emits_raw_content() {
        // A `++++` block emits its content unescaped, with no wrapping element,
        // matching Asciidoctor's `convert_pass`.
        let html = crate::convert("++++\n<b>raw</b>\n++++");
        assert!(html.contains("<b>raw</b>"));
        assert!(!html.contains("&lt;b&gt;"));
        assert!(!html.contains("unsupported"));
    }

    #[test]
    fn pass_style_paragraph_emits_raw_content() {
        // A `[pass]` style over a paragraph emits its content unescaped, with no
        // paragraph wrapper, just like a delimited `++++` block — matching
        // Asciidoctor's `convert_pass`.
        let html = crate::convert("[pass]\n<b>raw</b>\n");
        assert!(html.contains("<b>raw</b>"));
        assert!(!html.contains("&lt;b&gt;"));
        assert!(!html.contains("<div class=\"paragraph\">"));
    }

    #[test]
    fn stem_block_wraps_the_equation_in_math_delimiters() {
        // A `[stem]` block resolves to the document's `stem` notation, which
        // defaults to AsciiMath (`\$ … \$`), matching Asciidoctor's
        // `convert_stem`.
        let html = convert("[stem]\n++++\nx = y^2\n++++\n");
        assert!(
            html.contains(
                "<div class=\"stemblock\">\n<div class=\"content\">\n\\$x = y^2\\$\n</div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn latexmath_block_uses_bracket_delimiters() {
        // An `[latexmath]` block wraps its equation in `\[ … \]`, and the
        // parser has already escaped its special characters.
        let html = convert("[latexmath]\n++++\nC = \\alpha < 1\n++++\n");
        assert!(
            html.contains(
                "<div class=\"stemblock\">\n<div class=\"content\">\n\
                 \\[C = \\alpha &lt; 1\\]\n</div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn bare_stem_block_follows_the_document_stem_attribute() {
        // With `:stem: latexmath`, a bare `[stem]` block renders in LaTeX
        // notation (`\[ … \]`) rather than the AsciiMath default.
        let html = convert(":stem: latexmath\n\n[stem]\n++++\nC = 1\n++++\n");
        assert!(
            html.contains(
                "<div class=\"stemblock\">\n<div class=\"content\">\n\\[C = 1\\]\n</div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn stem_block_honors_id_role_and_title() {
        // The wrapper carries the block `id` and roles; a titled stem block
        // gains a plain `<div class="title">` (stem blocks are not captioned).
        let html = convert("[stem#eq1.myrole]\n.My equation\n++++\nx\n++++\n");
        assert!(
            html.contains(
                "<div id=\"eq1\" class=\"stemblock myrole\">\n\
                 <div class=\"title\">My equation</div>\n\
                 <div class=\"content\">\n\\$x\\$\n</div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn asciimath_block_breaks_multiple_equations_with_br() {
        // AsciiMath keeps blank-line-separated equations distinct, closing and
        // reopening the delimiter around a `<br>` (Asciidoctor's `StemBreakRx`).
        let html = convert("[asciimath]\n++++\ns = ut\n\nv = u + at\n++++\n");
        assert!(
            html.contains("<div class=\"content\">\n\\$s = ut\\$\n<br>\n\\$v = u + at\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn asciimath_block_breaks_on_a_line_continuation() {
        // A trailing-backslash line continuation is also a break: the space and
        // backslash are dropped and the delimiter closes and reopens with no
        // `<br>` (one line break yields zero `<br>`s), matching `StemBreakRx`.
        let html = convert("[asciimath]\n++++\na \\\nb\n++++\n");
        assert!(
            html.contains("<div class=\"content\">\n\\$a\\$\n\\$b\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn asciimath_block_break_adds_one_br_per_extra_blank_line() {
        // Two blank lines (three newlines) between equations emit two `<br>`
        // elements — one per line break beyond the first.
        let html = convert("[asciimath]\n++++\na\n\n\nb\n++++\n");
        assert!(
            html.contains("<div class=\"content\">\n\\$a\\$\n<br>\n<br>\n\\$b\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn asciimath_break_absorbs_a_multi_line_continuation_run() {
        // A line continuation may span several lines (`\` then a bare `\`
        // line); the whole run is one break, with one `<br>` per line break
        // beyond the first — exercising `StemBreakRx`'s `(?:\\?\n)*` tail.
        let html = convert("[asciimath]\n++++\na \\\n\\\nb\n++++\n");
        assert!(
            html.contains("<div class=\"content\">\n\\$a\\$\n<br>\n\\$b\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn empty_stem_block_renders_the_bare_delimiter_pair() {
        // An empty stem block still emits the delimiter pair (`\$\$`), matching
        // Asciidoctor — the wrap applies even when there is no equation text.
        let html = convert("[stem]\n++++\n++++\n");
        assert!(
            html.contains("<div class=\"stemblock\">\n<div class=\"content\">\n\\$\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn stem_block_does_not_double_wrap_an_already_delimited_equation() {
        // An equation the author already delimited is left untouched.
        let html = convert("[asciimath]\n++++\n\\$x\\$\n++++\n");
        assert!(
            html.contains("<div class=\"content\">\n\\$x\\$\n</div>"),
            "{html}"
        );
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
    fn delimited_source_block_wraps_code_in_a_highlight_pre() {
        // A `[source,<lang>]` *delimited* (`----`) listing must render the same
        // `<pre class="highlight"><code …>` wrapper as the paragraph form, not a
        // bare `<pre>` listing block (see #159). The delimited form resolves to
        // the `listing` context with `declared_style() == Some("source")`, so it
        // is routed to `source()` rather than the plain verbatim path.
        let html = convert("[source,ruby]\n----\ndef x\nend\n----\n");
        assert!(
            html.contains(
                "<div class=\"listingblock\">\n<div class=\"content\">\n\
                 <pre class=\"highlight\"><code class=\"language-ruby\" data-lang=\"ruby\">\
                 def x\nend</code></pre>\n\
                 </div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn comma_shorthand_listing_renders_as_a_source_block() {
        // The `[,<lang>]` comma shorthand is equivalent to `[source,<lang>]`, so
        // a `----` listing declared that way must render the same
        // `<pre class="highlight"><code …>` wrapper, not a bare `<pre>` listing
        // block (see #207). Here the declared style is absent and the language
        // sits in the second positional attribute, so the block is promoted to
        // `source()` just as Asciidoctor promotes it.
        let html = convert("[,ruby]\n----\nputs 1\n----\n");
        assert!(
            html.contains(
                "<div class=\"listingblock\">\n<div class=\"content\">\n\
                 <pre class=\"highlight\"><code class=\"language-ruby\" data-lang=\"ruby\">\
                 puts 1</code></pre>\n\
                 </div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn plain_style_listing_stays_a_verbatim_listing() {
        // A first-positional style that is *not* a language (`[ruby]`) leaves the
        // block a plain listing with a bare `<pre>` — Asciidoctor only promotes a
        // listing to `source` when the block style is absent and a language sits
        // in the second position, so a single positional attribute is a style,
        // not a comma-shorthand language.
        let html = convert("[ruby]\n----\nputs 1\n----\n");
        assert!(
            html.contains(
                "<div class=\"listingblock\">\n<div class=\"content\">\n\
                 <pre>puts 1</pre>\n\
                 </div>\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn source_block_honors_nowrap() {
        // A source block adds `nowrap` after `highlight` when wrapping is off
        // (here via `:prewrap!:`), matching Asciidoctor's `highlight nowrap`.
        let html = convert(":prewrap!:\n\n[source,ruby]\n    def x");
        assert!(
            html.contains("<pre class=\"highlight nowrap\"><code class=\"language-ruby\""),
            "{html}"
        );
    }

    // The client-side syntax highlighters (`highlightjs`, `prettify`) reshape
    // the source block's `<pre>`/`<code>` and add a CDN `<link>`/`<script>` to
    // the document shell. Each fragment below is byte-checked against
    // Asciidoctor 2.0.26 with the matching `-a source-highlighter=…` (the parity
    // oracle). See #215.
    //
    // These set `source-highlighter` through the API (Asciidoctor's `-a
    // source-highlighter=…`), the trusted opt-in honored even under the default
    // `Secure` safe mode. A *document-set* `:source-highlighter:` is locked out
    // at `Server` and above as a security measure (see the safe-mode tests
    // below and `Options::apply`), so the byte-parity tests use the API form.

    /// Converts `source` to a standalone document with the client-side
    /// `highlighter` enabled through the API, so the highlighter takes effect
    /// even under the default `Secure` safe mode.
    fn convert_hl(highlighter: &str, source: &str) -> String {
        convert_with(
            source,
            &Options::new().attribute("source-highlighter", highlighter),
        )
    }

    #[test]
    fn highlightjs_shapes_the_source_pre_and_code() {
        // The `<pre>` gains the `highlightjs` class before `highlight`, and the
        // `<code>` names the language with the trailing `hljs` class Asciidoctor
        // adds for the browser-side highlighter.
        let html = convert_hl("highlightjs", "[source,ruby]\n----\nputs 1\n----");
        assert!(
            html.contains(
                "<pre class=\"highlightjs highlight\">\
                 <code class=\"language-ruby hljs\" data-lang=\"ruby\">puts 1</code></pre>"
            ),
            "{html}"
        );
    }

    #[test]
    fn highlightjs_uses_language_none_without_a_language() {
        // With no declared language the `<code>` is `language-none hljs` and
        // carries no `data-lang`, matching `HighlightJsAdapter#format`.
        let html = convert_hl("highlightjs", "[source]\n----\nplain\n----");
        assert!(
            html.contains(
                "<pre class=\"highlightjs highlight\">\
                 <code class=\"language-none hljs\">plain</code></pre>"
            ),
            "{html}"
        );
    }

    #[test]
    fn highlightjs_emits_head_link_and_footer_scripts() {
        // The head links the theme stylesheet (default `github`) and the footer
        // loads highlight.js and runs the initializer, both from the CDN.
        let html = convert_hl("highlightjs", "[source,ruby]\n----\nputs 1\n----");
        assert!(
            html.contains(
                "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/\
                 highlight.js/9.18.3/styles/github.min.css\">"
            ),
            "{html}"
        );
        assert!(
            html.contains(
                "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/\
                 highlight.js/9.18.3/highlight.min.js\"></script>\n\
                 <script>\n\
                 if (!hljs.initHighlighting.called) {\n  \
                 hljs.initHighlighting.called = true\n  \
                 ;[].slice.call(document.querySelectorAll('pre.highlight > code[data-lang]'))\
                 .forEach(function (el) { hljs.highlightBlock(el) })\n\
                 }\n\
                 </script>"
            ),
            "{html}"
        );
    }

    #[test]
    fn highlightjs_honors_theme_dir_and_languages() {
        // `highlightjsdir` overrides the CDN base, `highlightjs-theme` the
        // stylesheet, and each `highlightjs-languages` entry loads a language
        // pack (with leading whitespace stripped). These secondary attributes
        // are not safe-mode-locked, so the document may set them.
        let html = convert_hl(
            "highlightjs",
            ":highlightjsdir: /assets/hjs\n\
             :highlightjs-theme: monokai\n\
             :highlightjs-languages: ruby, python\n\n\
             [source,ruby]\n----\nputs 1\n----",
        );
        assert!(
            html.contains("<link rel=\"stylesheet\" href=\"/assets/hjs/styles/monokai.min.css\">"),
            "{html}"
        );
        assert!(
            html.contains("<script src=\"/assets/hjs/highlight.min.js\"></script>\n"),
            "{html}"
        );
        assert!(
            html.contains("<script src=\"/assets/hjs/languages/ruby.min.js\"></script>\n"),
            "{html}"
        );
        assert!(
            html.contains("<script src=\"/assets/hjs/languages/python.min.js\"></script>\n"),
            "{html}"
        );
    }

    #[test]
    fn prettify_shapes_the_source_pre_and_code() {
        // Prettify prepends `prettyprint` and leaves the `<code>` in its default
        // shape — a bare `data-lang`, no class.
        let html = convert_hl("prettify", "[source,ruby]\n----\nputs 1\n----");
        assert!(
            html.contains(
                "<pre class=\"prettyprint highlight\">\
                 <code data-lang=\"ruby\">puts 1</code></pre>"
            ),
            "{html}"
        );
    }

    #[test]
    fn prettify_appends_the_linenums_classes() {
        // A `linenums` source block appends ` linenums` (or ` linenums:<start>`
        // when a `start` is given) after the `highlight` classes.
        let plain = convert_hl("prettify", "[source%linenums,ruby]\n----\nputs 1\n----");
        assert!(
            plain.contains("<pre class=\"prettyprint highlight linenums\">"),
            "{plain}"
        );

        let started = convert_hl(
            "prettify",
            "[source%linenums,ruby,start=5]\n----\nputs 1\n----",
        );
        assert!(
            started.contains("<pre class=\"prettyprint highlight linenums:5\">"),
            "{started}"
        );
    }

    #[test]
    fn prettify_emits_head_link_and_footer_script() {
        let html = convert_hl("prettify", "[source,ruby]\n----\nputs 1\n----");
        assert!(
            html.contains(
                "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/\
                 prettify/r298/prettify.min.css\">"
            ),
            "{html}"
        );
        assert!(
            html.contains(
                "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/\
                 prettify/r298/run_prettify.min.js\"></script>"
            ),
            "{html}"
        );
    }

    #[test]
    fn serverside_highlighter_keeps_the_default_shape() {
        // The coderay server-side highlighter is not supported — server-side
        // syntax highlighting is not planned — so `source-highlighter=coderay`
        // behaves like any unknown highlighter: the source block keeps its
        // default unhighlighted shape, no CDN assets are added, and no CodeRay
        // stylesheet is linked or embedded (a divergence from Asciidoctor, which
        // links `coderay-asciidoctor.css` for the spans it emits and this crate
        // does not).
        let html = convert_hl("coderay", "[source,ruby]\n----\nputs 1\n----");
        assert!(
            html.contains(
                "<pre class=\"highlight\">\
                 <code class=\"language-ruby\" data-lang=\"ruby\">puts 1</code></pre>"
            ),
            "{html}"
        );
        assert!(!html.contains("cdnjs.cloudflare.com"), "{html}");
        assert!(!html.contains("coderay-asciidoctor.css"), "{html}");
        assert!(!html.contains("pre.CodeRay"), "{html}");
    }

    #[test]
    fn highlighter_reaches_a_source_block_in_a_table_cell() {
        // The highlighter is a document-level property, so a source block nested
        // in an AsciiDoc table cell is highlighted too.
        let html = convert_hl(
            "highlightjs",
            "|===\na|\n[source,ruby]\n----\nputs 1\n----\n|===",
        );
        assert!(
            html.contains(
                "<pre class=\"highlightjs highlight\">\
                 <code class=\"language-ruby hljs\" data-lang=\"ruby\">puts 1</code></pre>"
            ),
            "{html}"
        );
    }

    #[test]
    fn asset_uri_scheme_controls_the_cdn_scheme() {
        // `asset-uri-scheme` sets the scheme of the CDN base URL: an explicit
        // empty value (Asciidoctor's `-a asset-uri-scheme=`) yields a
        // protocol-relative `//…`, and any other value is used as-is. (A *bare*
        // `:asset-uri-scheme:` resolves to the `https` default in the parser, so
        // the protocol-relative form is reached through an empty value.)
        let relative = convert_with(
            "[source,ruby]\n----\nputs 1\n----",
            &Options::new()
                .attribute("source-highlighter", "highlightjs")
                .attribute("asset-uri-scheme", ""),
        );
        assert!(
            relative.contains(
                "<link rel=\"stylesheet\" href=\"//cdnjs.cloudflare.com/ajax/libs/\
                 highlight.js/9.18.3/styles/github.min.css\">"
            ),
            "{relative}"
        );

        let http = convert_hl(
            "highlightjs",
            ":asset-uri-scheme: http\n\n[source,ruby]\n----\nputs 1\n----",
        );
        assert!(
            http.contains(
                "<link rel=\"stylesheet\" href=\"http://cdnjs.cloudflare.com/ajax/libs/\
                 highlight.js/9.18.3/styles/github.min.css\">"
            ),
            "{http}"
        );
    }

    #[test]
    fn prettify_honors_dir_and_theme() {
        // `prettifydir` overrides the CDN base for both the stylesheet and the
        // script, and `prettify-theme` selects the stylesheet.
        let html = convert_hl(
            "prettify",
            ":prettifydir: /assets/pf\n\
             :prettify-theme: sunburst\n\n\
             [source,ruby]\n----\nputs 1\n----",
        );
        assert!(
            html.contains("<link rel=\"stylesheet\" href=\"/assets/pf/sunburst.min.css\">"),
            "{html}"
        );
        assert!(
            html.contains("<script src=\"/assets/pf/run_prettify.min.js\"></script>"),
            "{html}"
        );
    }

    #[test]
    fn prettify_theme_may_be_a_full_url() {
        // An `http(s)` `prettify-theme` is linked directly rather than resolved
        // against `prettifydir`.
        let html = convert_hl(
            "prettify",
            ":prettify-theme: https://cdn.example.com/pp.css\n\n\
             [source,ruby]\n----\nputs 1\n----",
        );
        assert!(
            html.contains("<link rel=\"stylesheet\" href=\"https://cdn.example.com/pp.css\">"),
            "{html}"
        );
    }

    #[test]
    fn highlightjs_skips_empty_language_entries() {
        // A trailing comma in `highlightjs-languages` leaves an empty entry,
        // which is dropped rather than emitting a `languages/.min.js` script —
        // matching Asciidoctor, whose `split` drops the trailing empty.
        let html = convert_hl(
            "highlightjs",
            ":highlightjs-languages: ruby,\n\n[source,ruby]\n----\nputs 1\n----",
        );
        assert!(html.contains("/languages/ruby.min.js"), "{html}");
        assert!(!html.contains("/languages/.min.js"), "{html}");
    }

    #[test]
    fn prettify_uses_a_bare_code_without_a_language() {
        // With no declared language prettify leaves the `<code>` bare (no class,
        // no `data-lang`).
        let html = convert_hl("prettify", "[source]\n----\nplain\n----");
        assert!(
            html.contains("<pre class=\"prettyprint highlight\"><code>plain</code></pre>"),
            "{html}"
        );
    }

    // Safe-mode gating of `source-highlighter` (Greptile P1). A
    // highlighter emits `<link>`/`<script>` tags whose origin a document
    // attribute can steer, so an untrusted document must not be able to enable
    // one under a server-side safe mode. This mirrors Asciidoctor's
    // `attr_overrides['source-highlighter'] ||= nil` at `Server` and above.

    #[test]
    fn document_set_source_highlighter_is_ignored_at_server_and_above() {
        // A document that both enables a highlighter and points its asset dir at
        // an attacker origin must render neither the highlighter nor the script
        // when converted under `Server` or `Secure` – it falls back to the plain
        // unhighlighted shape.
        for mode in [SafeMode::Server, SafeMode::Secure] {
            let html = convert_with(
                ":source-highlighter: highlightjs\n\
                 :highlightjsdir: https://evil.example.com\n\n\
                 [source,ruby]\n----\nputs 1\n----",
                &Options::new().safe_mode(mode),
            );
            assert!(!html.contains("evil.example.com"), "{mode:?}: {html}");
            assert!(!html.contains("highlightjs highlight"), "{mode:?}: {html}");
            assert!(!html.contains("cdnjs.cloudflare.com"), "{mode:?}: {html}");
            assert!(
                html.contains(
                    "<pre class=\"highlight\">\
                     <code class=\"language-ruby\" data-lang=\"ruby\">puts 1</code></pre>"
                ),
                "{mode:?}: {html}"
            );
        }
    }

    #[test]
    fn document_set_source_highlighter_is_honored_below_server() {
        // Below `Server` (here `Unsafe`) a document may enable a highlighter,
        // matching Asciidoctor.
        let html = convert_with(
            ":source-highlighter: highlightjs\n\n[source,ruby]\n----\nputs 1\n----",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(html.contains("highlightjs highlight"), "{html}");
    }

    #[test]
    fn api_set_source_highlighter_is_honored_under_secure() {
        // A trusted API opt-in (`-a source-highlighter=…`) works even under the
        // default `Secure` safe mode, matching Asciidoctor.
        let html = convert_with(
            "[source,ruby]\n----\nputs 1\n----",
            &Options::new()
                .safe_mode(SafeMode::Secure)
                .attribute("source-highlighter", "highlightjs"),
        );
        assert!(html.contains("highlightjs highlight"), "{html}");
    }

    // Tab expansion (`tabsize`) beyond the leading-tab fast path: an embedded
    // tab advances to the next tab stop measured against the output column.

    #[test]
    fn verbatim_expands_an_embedded_tab() {
        // A tab after two characters advances to column 4 (two spaces).
        let html = convert(":tabsize: 4\n\n----\nab\tcd\n----");
        assert!(html.contains("<pre>ab  cd</pre>"), "{html}");
    }

    #[test]
    fn verbatim_tab_landing_on_a_stop_expands_a_full_width() {
        // A tab exactly on a stop expands to a whole `tabsize` run.
        let html = convert(":tabsize: 4\n\n----\nabcd\te\n----");
        assert!(html.contains("<pre>abcd    e</pre>"), "{html}");
    }

    #[test]
    fn verbatim_tab_one_short_of_a_stop_expands_to_a_single_space() {
        // A tab one column short of a stop expands to exactly one space.
        let html = convert(":tabsize: 4\n\n----\nabc\td\n----");
        assert!(html.contains("<pre>abc d</pre>"), "{html}");
    }

    #[test]
    fn verbatim_honors_a_block_level_tabsize() {
        // A block-level `tabsize` attribute overrides the document one.
        let html = convert("[tabsize=4]\n----\n\tx\n----");
        assert!(html.contains("<pre>    x</pre>"), "{html}");
    }

    #[test]
    fn verbatim_expands_tabs_without_an_indent_attribute() {
        // A positive `tabsize` expands tabs even with no `indent` set; the
        // indentation is otherwise preserved (`indent_size` of -1).
        let html = convert(":tabsize: 4\n\n----\n\tx\n----");
        assert!(html.contains("<pre>    x</pre>"), "{html}");
    }

    #[test]
    fn indent_with_a_flush_line_adds_only_the_margin() {
        // When a non-empty line is already flush against the margin there is no
        // common indent to strip, so `indent=N` just prepends the margin.
        let html = convert("[indent=\"2\"]\n----\nflush\n  x\n----");
        assert!(html.contains("<pre>  flush\n    x</pre>"), "{html}");
    }

    #[test]
    fn adjust_indentation_tolerates_empty_input() {
        // A defensive no-op guard matching Asciidoctor; the render path splits
        // rendered content and so never passes an empty line set.
        let mut lines: Vec<String> = Vec::new();
        super::adjust_indentation(&mut lines, 0, 4);
        assert!(lines.is_empty());
    }

    #[test]
    fn restore_literal_paragraph_indent_is_a_no_op_when_the_raw_span_is_short() {
        // Defensive guard: a raw span with fewer lines than the rendered content
        // (which should not happen — verbatim substitutions never drop lines)
        // leaves the lines untouched rather than underflowing the line offset.
        let mut lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        super::restore_literal_paragraph_indent(&mut lines, "only one line");
        assert_eq!(lines, ["a", "b", "c"]);
    }

    #[test]
    fn restore_literal_paragraph_indent_ignores_a_trailing_newline_in_the_raw_span() {
        // A raw span ending in a newline must not shift the suffix alignment: the
        // first rendered line still pairs with the first raw line, so the leading
        // indent lands on `literal` rather than being lost to a phantom line.
        let mut lines = vec!["literal".to_string(), "next".to_string()];
        super::restore_literal_paragraph_indent(&mut lines, "  literal\nnext\n");
        assert_eq!(lines, ["  literal", "next"]);
    }

    #[test]
    fn verbatim_restores_indent_for_a_titled_literal_paragraph() {
        // An implicit literal paragraph's raw span carries its `.title` ahead of
        // the content, so indent restoration must align to the trailing content
        // lines; the common indent (2) is then removed, matching Asciidoctor.
        let html = convert(".Cap\n    x\n  y\n");
        assert!(html.contains("<pre>  x\ny</pre>"), "{html}");
    }

    #[test]
    fn restore_list_principal_indent_keeps_inline_text_and_dedents_wrapped_lines() {
        // With inline text the first line is kept verbatim and the common indent
        // (2) of the two continuation lines is removed — Asciidoctor's dedent.
        let mut lines = vec!["Foo".to_string(), "five".to_string(), "two".to_string()];
        super::restore_list_principal_indent(&mut lines, "Foo\n     five\n  two", true);
        assert_eq!(lines, ["Foo", "   five", "two"]);
    }

    #[test]
    fn restore_list_principal_indent_keeps_indent_when_a_wrapped_line_is_flush_left() {
        // A flush-left continuation line (`second`) zeroes the common indent, so
        // the indented `// ...` line keeps its two spaces (the hanging-indent
        // shape this fix restores).
        let mut lines = vec![
            "list item 1".to_string(),
            "// not line comment".to_string(),
            "second wrapped line".to_string(),
        ];

        super::restore_list_principal_indent(
            &mut lines,
            "list item 1\n  // not line comment\nsecond wrapped line",
            true,
        );

        assert_eq!(
            lines,
            [
                "list item 1",
                "  // not line comment",
                "second wrapped line"
            ]
        );
    }

    #[test]
    fn restore_list_principal_indent_dedents_the_first_line_without_inline_text() {
        // A description term whose text folds up from an indented line has no
        // inline first line, so every line is folded and the common indent (2)
        // comes off the first line too.
        let mut lines = vec!["def1".to_string()];
        super::restore_list_principal_indent(&mut lines, "  def1", false);
        assert_eq!(lines, ["def1"]);
    }

    #[test]
    fn restore_list_principal_indent_drops_interior_comment_lines_before_aligning() {
        // The parser strips a column-0 line comment from the rendered content; it
        // must be filtered from the raw span too, or the surviving lines pair with
        // the wrong raw line and the dedent misfires.
        let mut lines = vec!["def2".to_string(), "def2 continued".to_string()];

        super::restore_list_principal_indent(
            &mut lines,
            "  def2\n// comment\n  def2 continued",
            false,
        );

        assert_eq!(lines, ["def2", "def2 continued"]);
    }

    #[test]
    fn restore_list_principal_indent_is_a_no_op_when_the_raw_span_is_short() {
        // Defensive guard: a raw span with fewer lines than the rendered content
        // (which should not happen) leaves the lines untouched rather than
        // underflowing the suffix offset.
        let mut lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        super::restore_list_principal_indent(&mut lines, "only one line", true);
        assert_eq!(lines, ["a", "b", "c"]);
    }

    #[test]
    fn common_leading_indent_skips_empty_lines_and_finds_the_minimum() {
        // An empty line is skipped (Asciidoctor's `next if line.empty?`), so it
        // does not count as a flush-left line that would force `None`; the common
        // indent is the minimum of the two non-empty lines.
        assert_eq!(super::common_leading_indent(&["    a", "", "  b"]), Some(2));

        // A genuinely flush-left non-empty line does force `None` (no dedent).
        assert_eq!(super::common_leading_indent(&["  a", "b"]), None);

        // No lines (all folded content skipped) removes nothing.
        assert_eq!(super::common_leading_indent(&[]), None);
    }

    #[test]
    fn is_line_comment_matches_only_a_column_zero_double_slash() {
        assert!(super::is_line_comment("// comment"));
        assert!(super::is_line_comment("//"));
        // A third slash begins a `////` comment block, not a line comment; an
        // indented `//` is body text, not a comment.
        assert!(!super::is_line_comment("/// not a line comment"));
        assert!(!super::is_line_comment("  // indented, kept as text"));
    }

    /// Counts the leading spaces of the sole `<pre>`'s content.
    fn pre_leading_spaces(html: &str) -> usize {
        let start = html.find("<pre>").expect("a <pre>") + "<pre>".len();
        let end = html[start..].find("</pre>").expect("a closing </pre>") + start;
        html[start..end].chars().take_while(|c| *c == ' ').count()
    }

    #[test]
    fn verbatim_clamps_a_pathological_indent() {
        // A document-supplied `indent` far beyond any real use is clamped so it
        // cannot drive an unbounded space allocation; rendering still completes
        // rather than aborting the process. (Would OOM before the clamp.)
        let html = convert("[indent=\"999999999999\"]\n----\n  x\n----");
        assert_eq!(
            pre_leading_spaces(&html),
            super::MAX_VERBATIM_INDENT as usize
        );
    }

    #[test]
    fn verbatim_clamps_a_pathological_tabsize() {
        // Likewise for a huge `tabsize`: a single leading tab expands to at most
        // the tab-size clamp, not gigabytes of spaces.
        let html = convert(":tabsize: 999999999999\n\n----\n\tx\n----");
        assert_eq!(pre_leading_spaces(&html), super::MAX_TAB_SIZE as usize);
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
    fn abstract_paragraph_renders_a_quoteblock_abstract() {
        // An `[abstract]` paragraph shares the quote template: its inline
        // content sits directly in the `<blockquote>` with no `<p>` wrapper.
        let html = convert("[abstract]\nA concise overview.");
        assert!(html.contains(
            "<div class=\"quoteblock abstract\">\n<blockquote>\n\
             A concise overview.\n</blockquote>\n</div>"
        ));
    }

    #[test]
    fn abstract_paragraph_keeps_its_title() {
        // A titled abstract paragraph emits its `<div class="title">` ahead of
        // the blockquote.
        let html = convert("[abstract]\n.Abstract\nA concise overview.");
        assert!(html.contains(
            "<div class=\"quoteblock abstract\">\n<div class=\"title\">Abstract</div>\n\
             <blockquote>\nA concise overview.\n</blockquote>\n</div>"
        ));
    }

    #[test]
    fn abstract_open_block_wraps_child_blocks() {
        // An `[abstract]` open block places its child blocks inside the
        // `<blockquote>`, unlike a plain `--` open block.
        let html = convert("[abstract]\n--\nFirst paragraph.\n\nSecond paragraph.\n--");
        assert!(html.contains(
            "<div class=\"quoteblock abstract\">\n<blockquote>\n\
             <div class=\"paragraph\">\n<p>First paragraph.</p>\n</div>\n\
             <div class=\"paragraph\">\n<p>Second paragraph.</p>\n</div>\n\
             </blockquote>\n</div>"
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

    #[test]
    fn admonition_custom_icon_with_extension_is_used_as_is() {
        // In image-icon mode, a per-block `icon` value that already has a file
        // extension is used verbatim under `iconsdir` (Asciidoctor's `icon_uri`
        // appends the `icontype` only when the value has no extension). A
        // document-set `:icons:` only takes effect below `Secure` (#50), so this
        // converts under `Server`.
        let html = convert_with(
            ":icons:\n\n[NOTE,icon=tip.png]\nSave often.",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert!(html.contains(
            "<td class=\"icon\">\n\
             <img src=\"./images/icons/tip.png\" alt=\"Note\">\n</td>"
        ));
    }

    #[test]
    fn admonition_custom_icon_without_extension_gets_icontype() {
        // A per-block `icon` value with no extension gains the document's
        // `icontype` extension. A document-set `:icons:` only takes effect below
        // `Secure` (#50), so this converts under `Server`.
        let html = convert_with(
            ":icons:\n:icontype: svg\n\n[NOTE,icon=hint]\nSave often.",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert!(html.contains(
            "<td class=\"icon\">\n\
             <img src=\"./images/icons/hint.svg\" alt=\"Note\">\n</td>"
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

        // A URI `stylesdir` joins with a relative stylesheet into a URI href,
        // preserving the scheme's `//` rather than collapsing it to `./https:/…`
        // — matching Asciidoctor 2.0.26, which lifts the URI prefix off before
        // normalizing and restores it afterward.
        assert_eq!(
            normalize_web_path("asciidoctor.css", "https://cdn.example.com/css"),
            "https://cdn.example.com/css/asciidoctor.css"
        );

        // A trailing slash on the URI `stylesdir` is not doubled.
        assert_eq!(
            normalize_web_path("custom.css", "https://cdn.example.com/css/"),
            "https://cdn.example.com/css/custom.css"
        );

        // An absolute stylesheet ignores `stylesdir`, even a URI one.
        assert_eq!(
            normalize_web_path("/abs/custom.css", "https://cdn.example.com/css"),
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

    // Table rendering is verified end-to-end by the `tables_test.rb` port
    // (`tests::asciidoctor_rb::tables_test`); these unit tests cover the handful
    // of attribute-value branches that suite does not exercise, each checked
    // against Asciidoctor 2.0.26's `html5` output.

    #[test]
    fn table_frame_and_grid_values_map_to_classes() {
        // Every non-default `frame`/`grid` value produces its own class.
        for (spec, class) in [
            ("frame=sides", "frame-sides grid-all"),
            ("frame=none", "frame-none grid-all"),
            ("grid=rows", "frame-all grid-rows"),
            ("grid=cols", "frame-all grid-cols"),
            ("grid=none", "frame-all grid-none"),
        ] {
            let html = crate::convert(&format!("[{spec}]\n|===\n|a |b\n|===\n"));
            assert!(
                html.contains(&format!("<table class=\"tableblock {class} stretch\">")),
                "{spec}: {html}"
            );
        }
    }

    #[test]
    fn table_stripes_value_adds_a_class() {
        // A `stripes` value — including an explicit `none` — adds `stripes-<v>`.
        for value in ["even", "all", "hover", "none"] {
            let html = crate::convert(&format!("[stripes={value}]\n|===\n|a |b\n|===\n"));
            assert!(
                html.contains(&format!(
                    "<table class=\"tableblock frame-all grid-all stripes-{value} stretch\">"
                )),
                "{value}: {html}"
            );
        }
    }

    #[test]
    fn table_role_becomes_a_trailing_class() {
        let html = crate::convert("[.myrole]\n|===\n|a |b\n|===\n");
        assert!(
            html.contains("<table class=\"tableblock frame-all grid-all stretch myrole\">"),
            "{html}"
        );
    }

    #[test]
    fn cellbgcolor_document_attribute_styles_every_cell() {
        // The `cellbgcolor` document attribute stamps
        // `style="background-color: …;"` on every cell – header (`<th>`) and
        // body (`<td>`) alike – matching Asciidoctor 2.0.26.
        let html = crate::convert(
            ":cellbgcolor: red\n\n[cols=\"1,1\", options=\"header\"]\n|===\n|H1 |H2\n|a |b\n|===\n",
        );
        assert!(
            html.contains(
                "<th class=\"tableblock halign-left valign-top\" style=\"background-color: red;\">H1</th>"
            ),
            "{html}"
        );
        assert!(
            html.contains(
                "<td class=\"tableblock halign-left valign-top\" style=\"background-color: red;\"><p class=\"tableblock\">a</p></td>"
            ),
            "{html}"
        );
    }

    #[test]
    fn cellbgcolor_styles_an_asciidoc_cell() {
        // The style attribute lands on the outer cell tag regardless of the
        // cell's content model, including an AsciiDoc (`a`) cell.
        let html = crate::convert(":cellbgcolor: yellow\n\n|===\na|nested _content_\n|===\n");
        assert!(
            html.contains(
                "<td class=\"tableblock halign-left valign-top\" style=\"background-color: yellow;\"><div class=\"content\">"
            ),
            "{html}"
        );
    }

    #[test]
    fn cellbgcolor_absent_leaves_cells_unstyled() {
        // With no `cellbgcolor` attribute, cells carry no `style`.
        let html = crate::convert("|===\n|a\n|===\n");
        assert!(!html.contains("style=\"background-color"), "{html}");
        assert!(
            html.contains("<td class=\"tableblock halign-left valign-top\"><p class=\"tableblock\">a</p></td>"),
            "{html}"
        );
    }

    #[test]
    fn cellbgcolor_value_is_escaped() {
        // `cellbgcolor` is document-controlled and lands inside a quoted
        // attribute, so a stray `"` must be escaped rather than break out and
        // inject markup (cf. `author_name_and_email_are_escaped`).
        let html = crate::convert(":cellbgcolor: red\" onmouseover=\"alert(1)\n\n|===\n|a\n|===\n");
        assert!(
            html.contains("style=\"background-color: red&quot; onmouseover=&quot;alert(1);\""),
            "{html}"
        );
        assert!(!html.contains("onmouseover=\"alert"), "{html}");
    }

    #[test]
    fn zero_width_column_specifier_keeps_the_default_width() {
        // `asciidoc-parser` clamps a `0` width specifier to the default width of
        // 1, so `cols="0,0"` behaves like `cols="1,1"` — an even 50/50 split.
        let html = crate::convert("[cols=\"0,0\"]\n|===\n|a |b\n|===\n");
        assert_eq!(html.matches("<col style=\"width: 50%;\">").count(), 2);
    }

    #[test]
    fn split_blank_lines_matches_ruby_split_semantics() {
        // Interior blank lines split into paragraphs; a trailing blank line is
        // dropped, mirroring Ruby's `String#split(/\n{2,}/)`.
        use super::split_blank_lines;
        assert_eq!(split_blank_lines("a\n\nb").collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(
            split_blank_lines("a\n\n\nb\n\n").collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(split_blank_lines("solo").collect::<Vec<_>>(), ["solo"]);
    }

    #[test]
    fn empty_asciidoc_cell_renders_an_empty_content_div() {
        // An AsciiDoc (`a`) cell with no content still gets its `content`
        // wrapper — and the nested render returns the empty string.
        let html = crate::convert("|===\na|\n|===\n");
        assert!(
            html.contains(
                "<td class=\"tableblock halign-left valign-top\"><div class=\"content\"></div></td>"
            ),
            "{html}"
        );
    }

    /// A minimal, document-less [`Renderer`] for exercising a single helper in
    /// isolation. Every field takes its neutral default; call the method under
    /// test and inspect [`Renderer::out`].
    fn bare_renderer() -> super::Renderer<'static> {
        super::Renderer {
            out: String::new(),
            document: None,
            custom_stylesheet: None,
            standalone: false,
            toc_mode: super::TocMode::Disabled,
            toc_html: String::new(),
            cell_toc: None,
            icons_set: false,
            icons_font: false,
            iconsdir: String::new(),
            icontype: String::new(),
            imagesdir: String::new(),
            asset_uri_scheme: String::new(),
            doc_tabsize: 0,
            source_indent: None,
            prewrap: false,
            source_highlighter: None,
            source_language: None,
            section_anchors: super::SectionAnchors::None,
            sectlinks: false,
            stem_type: super::StemType::AsciiMath,
            cellbgcolor: None,
            svg_below_secure: false,
            data_uri: false,
            svg_source: None,
        }
    }

    #[test]
    fn unsupported_block_emits_a_visible_placeholder_comment() {
        // `unsupported` is the documented forward-compat fallback for a
        // construct the baseline does not yet render (see ARCHITECTURE.md). No
        // document currently reaches it — every parser block context is either
        // handled or dropped — so its contract is pinned here directly: a single
        // well-formed HTML comment naming the offending context.
        let mut renderer = bare_renderer();
        renderer.unsupported("mystery");
        assert_eq!(
            renderer.out,
            "<!-- asciidoc-html5: unsupported block context 'mystery' -->\n"
        );
    }

    #[test]
    fn dispatching_a_bare_list_item_takes_the_unsupported_fallback() {
        // `block`'s final `other =>` arm is the forward-compat fallback for a
        // block variant with no dedicated renderer. A document never presents
        // one at top level — a `ListItem`, for instance, is only reached through
        // its parent list — so dispatch one directly to exercise the arm, the
        // same way `dlist_narrowing_helpers_handle_both_arms` drives a path a
        // real document never reaches.
        use asciidoc_parser::{blocks::FindBlocks, Parser};

        let mut parser = Parser::default();
        let doc = parser.parse("* bullet\n");
        let list = doc.child_blocks().next().unwrap();
        let item = list.child_blocks().next().unwrap();

        let mut renderer = bare_renderer();
        renderer.block(item);
        assert!(
            renderer
                .out
                .contains("asciidoc-html5: unsupported block context"),
            "{}",
            renderer.out
        );
    }

    #[test]
    fn list_item_ignores_a_non_list_item_block() {
        // `list_item` is only ever handed a `ListItem`; its narrowing guard
        // returns without output for any other block. Hand it a paragraph to
        // exercise that `else` branch directly.
        use asciidoc_parser::{blocks::FindBlocks, Parser};

        let mut parser = Parser::default();
        let doc = parser.parse("just a paragraph\n");
        let paragraph = doc.child_blocks().next().unwrap();

        let mut renderer = bare_renderer();
        renderer.list_item(paragraph, false, false);
        assert!(renderer.out.is_empty(), "{}", renderer.out);
    }
}
