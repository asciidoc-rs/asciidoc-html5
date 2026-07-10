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
    blocks::{Block, Break, BreakType, IsBlock, SectionBlock, SectionType, SimpleBlockStyle},
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
const DEFAULT_STYLESHEET: &str = include_str!("../assets/asciidoctor-default.css");

/// The `family` query string Asciidoctor uses for its Google Fonts `<link>`
/// when the `webfonts` attribute carries no explicit value: Open Sans for
/// headings, Noto Serif for body text, Droid Sans Mono for monospaced text.
const DEFAULT_WEBFONTS: &str = "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700";

/// Reads a document attribute as an explicit string value, if it has one.
/// `Set`/`Unset`/absent all yield `None` (use `is_attribute_set` for booleans).
fn attribute_str(document: &Document<'_>, name: &str) -> Option<String> {
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
fn links_stylesheet(document: &Document<'_>) -> bool {
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

/// Renders a parsed [`Document`] to a standalone HTML5 document string.
pub(crate) fn render_document(document: &Document<'_>) -> String {
    let mut renderer = Renderer { out: String::new() };
    renderer.document(document);
    renderer.out
}

/// Accumulates HTML as the document tree is walked.
struct Renderer {
    out: String,
}

impl Renderer {
    /// Appends a line of markup followed by a newline, matching Asciidoctor's
    /// convention of one element per line with no indentation.
    fn line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    /// Emits the complete standalone document: the `<head>` preamble, the
    /// `<div id="header">`, the `<div id="content">` body, and the footer.
    fn document(&mut self, document: &Document<'_>) {
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

    /// Emits the default-stylesheet portion of the `<head>`: the Google Fonts
    /// `<link>` and either an inline `<style>` (the default) or, under
    /// `linkcss`, a `<link>` to `./asciidoctor.css`.
    ///
    /// This mirrors Asciidoctor's `html5` backend, which only emits this block
    /// when the `stylesheet` attribute selects the default stylesheet — that
    /// is, when it is absent, set with no value, empty, or `DEFAULT`
    /// (Asciidoctor's `DEFAULT_STYLESHEET_KEYS`). Explicitly unsetting it
    /// (`:stylesheet!:`) drops the block entirely, and a custom `stylesheet`
    /// value is a documented limitation: this crate cannot read an external
    /// stylesheet file, so it emits nothing rather than guessing.
    fn stylesheet(&mut self, document: &Document<'_>) {
        match document.attribute_value("stylesheet") {
            // Explicitly unset (`:stylesheet!:`): no stylesheet at all.
            InterpretedValue::Unset if document.has_attribute("stylesheet") => return,
            // A custom stylesheet file — unsupported (see docs); emit nothing.
            InterpretedValue::Value(ref value) if !value.is_empty() && value != "DEFAULT" => return,
            // Absent, `Set`, empty, or `DEFAULT`: the default stylesheet.
            _ => {}
        }

        self.webfonts_link(document);

        if links_stylesheet(document) {
            // Asciidoctor writes the stylesheet out under the public name
            // `asciidoctor.css`, normalized to a relative web path.
            self.line("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">");
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
                SimpleBlockStyle::Paragraph => self.paragraph(block),
                SimpleBlockStyle::Listing | SimpleBlockStyle::Source => {
                    self.verbatim(block, "listingblock")
                }
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
    use std::collections::HashMap;

    use crate::{convert, convert_with, DocinfoFileHandler, DocumentParser, Options, SafeMode};

    /// A docinfo handler backed by a fixed file-name → content map, so the
    /// docinfo tests can supply files without touching the file system.
    #[derive(Debug)]
    struct MapHandler(HashMap<String, String>);

    impl MapHandler {
        fn new(pairs: &[(&str, &str)]) -> Self {
            Self(
                pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )
        }
    }

    impl DocinfoFileHandler for MapHandler {
        fn resolve_docinfo(
            &self,
            _docinfodir: Option<&str>,
            file_name: &str,
            _parser: &DocumentParser,
        ) -> Option<String> {
            self.0.get(file_name).cloned()
        }
    }

    /// Converts `source` with the given docinfo files available, under `Server`
    /// safe mode (docinfo is disabled at `Secure` and above) and a primary file
    /// name of `mydoc.adoc` (so private docinfo files resolve).
    fn with_docinfo(source: &str, files: &[(&str, &str)]) -> String {
        convert_with(
            source,
            &Options::new()
                .safe_mode(SafeMode::Server)
                .primary_file_name("mydoc.adoc")
                .docinfo_file_handler(MapHandler::new(files)),
        )
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
        let html = convert("= Doc\n:doctype: book\n\nBody.");
        assert!(html.contains("<body class=\"book\">"));
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

    // Docinfo splices caller-supplied content into three fixed positions of the
    // output: the bottom of the `<head>`, before the header `<div>`, and after
    // the footer `<div>`. The parser resolves which files apply (per the
    // `docinfo` attribute) and applies `docinfosubs`; the renderer only places
    // the result.

    #[test]
    fn head_docinfo_is_appended_to_the_bottom_of_the_head() {
        let html = with_docinfo(
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
            "= Doc\n:docinfo: shared\n\nBody.",
            &[("docinfo-header.html", "<div class=\"banner\">Hi</div>")],
        );
        assert!(html.contains("<div class=\"banner\">Hi</div>\n<div id=\"header\">"));
    }

    #[test]
    fn footer_docinfo_is_inserted_after_the_footer_div() {
        let html = with_docinfo(
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
            "= Doc\n:docinfo: shared\n:project: Widgets\n\nBody.",
            &[("docinfo.html", "<meta name=\"app\" content=\"{project}\">")],
        );
        assert!(html.contains("<meta name=\"app\" content=\"Widgets\">"));
    }

    #[test]
    fn no_handler_means_no_docinfo() {
        // Without a docinfo handler configured, the `docinfo` attribute has no
        // effect and the output carries no injected content.
        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert!(html.contains("</head>"));
        // Nothing spliced: head still flows stylesheet → `</head>`.
        assert!(!html.contains("<meta name=\"x\""));
    }
}
