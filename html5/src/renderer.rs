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

use asciidoc_parser::{
    blocks::{
        AdmonitionBlock, Block, Break, BreakType, ColumnStyle, CompoundDelimitedContext,
        ContentModel, FindBlocks, Frame, Grid, HorizontalAlignment, IsBlock, ListBlock, ListItem,
        ListItemMarker, ListType, QuoteBlock, QuoteType, SectionBlock, SectionType,
        SimpleBlockStyle, Stripes, TableBlock, TableCell, TableCellContent, TableColumn, TableRow,
        VerticalAlignment,
    },
    document::{DocinfoLocation, Header, InterpretedValue},
    Document, HasSpan, SafeMode,
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
/// - a paragraph the parser reduced to empty content by stripping an isolated
///   `//` line comment — Asciidoctor emits no block for it, so the empty
///   paragraph is dropped rather than rendered as an empty `<p></p>`.
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

    // An isolated `//` line comment survives parsing as a paragraph with no
    // content; an empty paragraph is never valid Asciidoctor output either way.
    matches!(block, Block::Simple(simple) if simple.style() == SimpleBlockStyle::Paragraph)
        && block
            .rendered_content()
            .unwrap_or_default()
            .trim()
            .is_empty()
}

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

/// The numbering style of an ordered list, matching Asciidoctor's `node.style`
/// for an olist: an explicit numbering keyword from the block's declared style
/// (`[loweralpha]`, `[upperroman]`, …) wins, otherwise the style implied by the
/// first item's marker (`.` ⇒ arabic, `..` ⇒ loweralpha, …). Falls back to
/// `arabic`, which a bare `.` marker also yields.
fn olist_style<'src>(block: &'src Block<'src>, list: &'src ListBlock<'src>) -> &'src str {
    const ORDERED_LIST_STYLES: [&str; 5] = [
        "arabic",
        "loweralpha",
        "lowerroman",
        "upperalpha",
        "upperroman",
    ];

    if let Some(style) = block.declared_style() {
        if ORDERED_LIST_STYLES.contains(&style) {
            return style;
        }
    }

    list.marker_style().unwrap_or("arabic")
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
        icons_font: attribute_str(document, "icons").as_deref() == Some("font"),
        doc_tabsize: ruby_to_i(&attribute_str(document, "tabsize").unwrap_or_default()),
        source_indent: attribute_str(document, "source-indent").map(|value| ruby_to_i(&value)),
        prewrap: document.is_attribute_set("prewrap"),
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

    /// Whether the document sets `:icons: font`, which selects Font Awesome
    /// checkbox glyphs for interactive-less checklists (matching Asciidoctor).
    icons_font: bool,

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
        self.blocks(document.child_blocks());
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

        self.blocks(document.child_blocks());
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
                // A `[source]`-styled delimited listing renders like a source
                // block (the `<pre class="highlight"><code …>` shape), matching
                // Asciidoctor; a plain `----` listing is a verbatim block.
                "listing" if block.declared_style() == Some("source") => self.source(block),
                "listing" => self.verbatim(block, "listingblock"),
                "literal" => self.verbatim(block, "literalblock"),
                "pass" => self.pass_block(block),
                other => self.unsupported(other),
            },
            Block::CompoundDelimited(compound) => match compound.context_kind() {
                CompoundDelimitedContext::Open => self.open_block(block),
                CompoundDelimitedContext::Sidebar => self.sidebar(block),
                CompoundDelimitedContext::Example => self.example(block),
            },
            Block::Quote(quote) => self.quote(block, quote),
            Block::Admonition(admonition) => self.admonition(block, admonition),
            Block::List(list) => match list.type_() {
                ListType::Unordered => self.ulist(block, list),
                ListType::Ordered => self.olist(block, list),
                ListType::Description => self.dlist(block, list),

                // Callout lists are not rendered yet; see ARCHITECTURE.md for
                // the roadmap.
                ListType::Callout => self.unsupported(&block.resolved_context()),
            },
            Block::Table(table) => self.table(block, table),

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
    /// A source block resolves to the `listing` context, so its title is
    /// captioned the same way a listing block's is (see
    /// [`verbatim`](Self::verbatim)).
    fn source<'src>(&mut self, block: &'src Block<'src>) {
        self.open_block_wrapper(block, "listingblock");
        self.captioned_title(block);
        self.line("<div class=\"content\">");

        let content = self.verbatim_content(block, true);

        // A `nowrap` block (or a document with `prewrap` disabled) adds the
        // class after `highlight`, matching Asciidoctor's `pre class="highlight
        // nowrap"`.
        let highlight = if self.is_nowrap(block) {
            "highlight nowrap"
        } else {
            "highlight"
        };

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
                    "<pre class=\"{highlight}\"><code class=\"language-{language}\" \
                     data-lang=\"{language}\">{content}</code></pre>"
                ));
            }
            None => self.line(&format!(
                "<pre class=\"{highlight}\"><code>{content}</code></pre>"
            )),
        }

        self.line("</div>");
        self.line("</div>");
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

        // Asciidoctor reindents when an indent is in force, or (to expand tabs
        // only) when a positive tabsize is set with the indentation preserved.
        match indent_size {
            Some(indent) => adjust_indentation(&mut lines, indent, tab_size),
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
        let style = block.declared_style();

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
        // needs none, the others carry the matching numbering letter.
        let type_attr = match style {
            "loweralpha" => " type=\"a\"",
            "lowerroman" => " type=\"i\"",
            "upperalpha" => " type=\"A\"",
            "upperroman" => " type=\"I\"",
            _ => "",
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
            let text = blocks[0].rendered_content().unwrap_or_default();
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
        // bare `<p>`; the remainder render as ordinary nested blocks.
        let mut blocks = list_item.child_blocks();
        let principal = blocks
            .next()
            .and_then(|block| block.rendered_content())
            .unwrap_or_default();

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
                    self.icons_font,
                    self.doc_tabsize,
                    self.source_indent,
                    self.prewrap,
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

        self.line(&format!(
            "<{tag} class=\"tableblock halign-{h_align} valign-{v_align}\"{colspan}{rowspan}>{content}</{tag}>"
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
            self.blocks(block.child_blocks());
            self.line("</div>");
        } else {
            self.blocks(block.child_blocks());
        }

        self.line("</div>");
    }

    /// The preamble: content between the doctitle and the first section,
    /// wrapped as `<div id="preamble"><div
    /// class="sectionbody">…</div></div>`.
    fn preamble<'src>(&mut self, block: &'src Block<'src>) {
        self.line("<div id=\"preamble\">");
        self.line("<div class=\"sectionbody\">");
        self.blocks(block.child_blocks());
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
/// nested-document `<h1>` when the cell's title is shown.
fn render_cell_document<'s>(
    blocks: &'s [Block<'s>],
    title: Option<&str>,
    inline: bool,
    icons_font: bool,
    doc_tabsize: i64,
    source_indent: Option<i64>,
    prewrap: bool,
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

    // The cell is a nested document that inherits the parent's verbatim-layout
    // attributes (`tabsize`, `source-indent`, `prewrap`), so the sub-renderer
    // carries them forward.
    let mut renderer = Renderer {
        out: String::new(),
        custom_stylesheet: None,
        standalone: false,
        icons_font,
        doc_tabsize,
        source_indent,
        prewrap,
    };
    if let Some(title) = title {
        renderer.line(&format!("<h1>{title}</h1>"));
    }
    renderer.blocks(blocks.iter());

    // `convert` joins its lines with no trailing newline; drop the one the
    // line-oriented renderer left behind.
    match renderer.out.strip_suffix('\n') {
        Some(trimmed) => trimmed.to_string(),
        None => renderer.out,
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
        // Callout lists are not rendered yet, so they still emit the
        // placeholder marker (unordered/ordered/description lists now render).
        let html = convert("----\ncode <1>\n----\n\n<1> explanation");
        assert!(html.contains("<!-- asciidoc-html5: unsupported block context 'list' -->"));
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
    fn delimited_passthrough_emits_raw_content() {
        // A `++++` block emits its content unescaped, with no wrapping element,
        // matching Asciidoctor's `convert_pass`.
        let html = crate::convert("++++\n<b>raw</b>\n++++");
        assert!(html.contains("<b>raw</b>"));
        assert!(!html.contains("&lt;b&gt;"));
        assert!(!html.contains("unsupported"));
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
    fn source_block_honors_nowrap() {
        // A source block adds `nowrap` after `highlight` when wrapping is off
        // (here via `:prewrap!:`), matching Asciidoctor's `highlight nowrap`.
        let html = convert(":prewrap!:\n\n[source,ruby]\n    def x");
        assert!(
            html.contains("<pre class=\"highlight nowrap\"><code class=\"language-ruby\""),
            "{html}"
        );
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
}
