//! Generates the HTML table of contents (the *outline*) for a document.
//!
//! This is the standalone counterpart to the block-structure walk in
//! [`renderer`](crate::renderer): where the renderer emits the whole document,
//! the outline emits only the nested `<ul class="sectlevelN">` list of section
//! links Asciidoctor's `html5` backend produces from its `convert_outline`
//! method. It is exposed through
//! [`convert_outline`](crate::convert_outline)/
//! [`convert_outline_with`](crate::convert_outline_with) so callers can
//! generate a TOC on its own — to embed in a page template, for
//! example — without rendering the full document.
//!
//! The walk mirrors Asciidoctor's `convert_outline`: it recurses over the
//! document's sections, wrapping each in a `<li><a href="#id">title</a></li>`
//! and nesting a child list under any section that has subsections within the
//! configured depth.

use asciidoc_parser::{
    blocks::{Block, IsBlock, SectionBlock, SectionType},
    document::InterpretedValue,
    Document,
};

/// Options controlling how
/// [`convert_outline_with`](crate::convert_outline_with) generates the TOC,
/// mirroring the option hash Asciidoctor's `convert_outline` accepts.
///
/// Each field is an *override*: when unset, the value falls back to the
/// matching document attribute (`toclevels`, `sectnumlevels`). Build one with
/// [`OutlineOptions::new`] and the chained setters.
///
/// # Examples
///
/// ```
/// use asciidoc_html5::OutlineOptions;
///
/// // Limit the TOC to top-level sections.
/// let options = OutlineOptions::new().toclevels(1);
/// ```
#[derive(Clone, Debug, Default)]
pub struct OutlineOptions {
    toclevels: Option<usize>,
    sectnumlevels: Option<usize>,
}

impl OutlineOptions {
    /// Creates an empty set of options; every value falls back to the
    /// document's own attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the depth of the TOC — the deepest section level included —
    /// overriding the document's `toclevels` attribute.
    pub fn toclevels(mut self, levels: usize) -> Self {
        self.toclevels = Some(levels);
        self
    }

    /// Sets the number of section levels that carry a section number in the
    /// TOC, overriding the document's `sectnumlevels` attribute. This has
    /// an effect only for a document whose sections are numbered
    /// (`sectnums`).
    pub fn sectnumlevels(mut self, levels: usize) -> Self {
        self.sectnumlevels = Some(levels);
        self
    }
}

/// Generates the HTML TOC for `document` under `options`, returning an empty
/// `String` when the document has no sections.
///
/// This is the entry point behind
/// [`convert_outline`](crate::convert_outline)/
/// [`convert_outline_with`](crate::convert_outline_with); see those for the
/// full contract.
pub(crate) fn render_outline(document: &Document<'_>, options: &OutlineOptions) -> String {
    let toclevels = options.toclevels.unwrap_or_else(|| document.toc_levels());
    let sectnumlevels = options
        .sectnumlevels
        .unwrap_or_else(|| attribute_usize(document, "sectnumlevels", 3));

    // A document with no sections has no outline; `outline_level` signals that
    // with `None`, which the public API surfaces as an empty string.
    outline_level(document.nested_blocks(), 0, toclevels, sectnumlevels).unwrap_or_default()
}

/// Emits one `<ul class="sectlevelN">` list for the sections among `blocks`,
/// recursing into each section's own subsections. `parent_level` is the level
/// of the node whose children these are (0 for the document), so the list class
/// is `sectlevel{parent_level + 1}`. Returns `None` when `blocks` holds no
/// sections, which is what lets a leaf section render without a nested list.
fn outline_level<'src>(
    blocks: std::slice::Iter<'src, Block<'src>>,
    parent_level: usize,
    toclevels: usize,
    sectnumlevels: usize,
) -> Option<String> {
    // Only real (non-discrete) sections appear in the outline, matching
    // Asciidoctor's `node.sections`, which excludes floating (discrete) titles.
    let sections: Vec<(&Block<'_>, &SectionBlock<'_>)> = blocks
        .filter_map(|block| match block {
            Block::Section(section) if section.section_type() != SectionType::Discrete => {
                Some((block, section))
            }
            _ => None,
        })
        .collect();

    if sections.is_empty() {
        return None;
    }

    let mut lines = vec![format!("<ul class=\"sectlevel{}\">", parent_level + 1)];

    for (block, section) in sections {
        let level = section.level();
        let id = block.id().unwrap_or_default();
        let title = outline_title(section, sectnumlevels);

        // A section below the configured depth contributes its own child list;
        // the recursion returns `None` when it has no subsections, which drops
        // the section to the leaf form.
        let child = if level < toclevels {
            outline_level(block.nested_blocks(), level, toclevels, sectnumlevels)
        } else {
            None
        };

        match child {
            Some(child_toc) => {
                lines.push(format!(
                    "<li><a href=\"#{id}\">{title}</a>\n{child_toc}\n</li>"
                ));
            }
            None => lines.push(format!("<li><a href=\"#{id}\">{title}</a></li>")),
        }
    }

    lines.push("</ul>".to_string());
    Some(lines.join("\n"))
}

/// The link text for a section in the TOC: the section title, prefixed with its
/// section number when the section is numbered and within `sectnumlevels`, with
/// any inline anchor tags stripped (Asciidoctor's `DropAnchorRx`) so a link in
/// a heading does not nest an `<a>` inside the TOC's own link.
fn outline_title(section: &SectionBlock<'_>, sectnumlevels: usize) -> String {
    let title = section.section_title();

    // Asciidoctor's `sectnum` appends the `.` delimiter after the number, so a
    // level-1 section reads `1.` and a level-2 section `1.1.`. The parser's
    // `SectionNumber` renders the dotted components without that trailing `.`,
    // so we add it.
    let title = match section.section_number() {
        Some(number) if section.level() <= sectnumlevels => format!("{number}. {title}"),
        _ => title.to_string(),
    };

    if title.contains("<a") {
        drop_anchor_tags(&title)
    } else {
        title
    }
}

/// Removes inline anchor tags — `<a …>` openers and `</a>` closers — from
/// `input`, keeping their text content. This mirrors Asciidoctor's
/// `DropAnchorRx` (`/<(?:a\b[^>]*|\/a)>/`): an opener is `<a` followed by a
/// word boundary (so `<abbr>` is left alone) up to the next `>`.
fn drop_anchor_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("</a>") {
            rest = after;
        } else if let Some(after_open) = anchor_open_len(rest) {
            rest = &rest[after_open..];
        } else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }

    out
}

/// If `rest` begins with an `<a …>` opening anchor tag, returns the byte length
/// of that tag (through the closing `>`); otherwise `None`. The character after
/// `a` must be a word boundary — anything but an ASCII letter, digit, or `_` —
/// so `<abbr>` is not mistaken for an anchor.
fn anchor_open_len(rest: &str) -> Option<usize> {
    let after_a = rest.strip_prefix("<a")?;
    let boundary = after_a
        .chars()
        .next()
        .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'));
    if !boundary {
        return None;
    }

    // Consume through the tag's closing `>`.
    after_a.find('>').map(|end| "<a".len() + end + 1)
}

/// Reads an unsigned-integer document attribute, falling back to `default` when
/// it is unset or does not parse.
fn attribute_usize(document: &Document<'_>, name: &str, default: usize) -> usize {
    match document.attribute_value(name) {
        InterpretedValue::Value(value) => value.parse().unwrap_or(default),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use crate::{convert_outline, convert_outline_with, load, OutlineOptions};

    // The document the reference page uses: three top-level sections, the second
    // carrying one subsection.
    const SAMPLE: &str = "\
= Document Title

== Section A

== Section B

=== Subsection

== Section C
";

    // The full outline the page shows for that document.
    const EXPECTED: &str = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_section_a\">Section A</a></li>
<li><a href=\"#_section_b\">Section B</a>
<ul class=\"sectlevel2\">
<li><a href=\"#_subsection\">Subsection</a></li>
</ul>
</li>
<li><a href=\"#_section_c\">Section C</a></li>
</ul>";

    #[test]
    fn outline_matches_asciidoctor() {
        let doc = load(SAMPLE);
        assert_eq!(convert_outline(&doc), EXPECTED);
    }

    #[test]
    fn toclevels_limits_the_depth() {
        let doc = load(SAMPLE);

        // With the depth capped at 1, the subsection under Section B is dropped
        // and Section B renders as a leaf like the others.
        let expected = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_section_a\">Section A</a></li>
<li><a href=\"#_section_b\">Section B</a></li>
<li><a href=\"#_section_c\">Section C</a></li>
</ul>";
        assert_eq!(
            convert_outline_with(&doc, &OutlineOptions::new().toclevels(1)),
            expected
        );
    }

    #[test]
    fn a_document_without_sections_yields_an_empty_outline() {
        let doc = load("= Title\n\nJust a paragraph.");
        assert_eq!(convert_outline(&doc), "");
    }

    #[test]
    fn numbered_sections_carry_their_number() {
        let doc = load("= Title\n:sectnums:\n\n== First\n\n=== Nested\n\n== Second\n");

        // The dotted numbers (with Asciidoctor's trailing `.`) prefix each title,
        // byte-identical to Asciidoctor's `convert_outline`.
        let expected = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_first\">1. First</a>
<ul class=\"sectlevel2\">
<li><a href=\"#_nested\">1.1. Nested</a></li>
</ul>
</li>
<li><a href=\"#_second\">2. Second</a></li>
</ul>";
        assert_eq!(convert_outline(&doc), expected);
    }

    #[test]
    fn a_link_in_a_heading_is_flattened() {
        let doc = load("= Title\n\n== See https://example.org[the site]\n");
        let outline = convert_outline(&doc);

        // The section's own anchor remains, but the inline link inside the title
        // is reduced to its text.
        assert!(outline.contains("the site</a></li>"));
        assert!(!outline.contains("https://example.org"));
    }

    #[test]
    fn sectnumlevels_option_caps_the_numbered_levels() {
        let doc = load("= Title\n:sectnums:\n\n== First\n\n=== Nested\n\n== Second\n");

        // Capping `sectnumlevels` at 1 numbers only the top-level sections; the
        // nested level-2 section falls back to its plain title, matching
        // Asciidoctor's `convert_outline` with the same option.
        let expected = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_first\">1. First</a>
<ul class=\"sectlevel2\">
<li><a href=\"#_nested\">Nested</a></li>
</ul>
</li>
<li><a href=\"#_second\">2. Second</a></li>
</ul>";
        assert_eq!(
            convert_outline_with(&doc, &OutlineOptions::new().sectnumlevels(1)),
            expected
        );
    }

    #[test]
    fn an_unset_sectnumlevels_attribute_uses_the_default() {
        // With `sectnumlevels` explicitly unset, there is no attribute value to
        // read, so the generator falls back to its default depth (3) — which
        // still numbers both of these levels.
        let doc = load("= Title\n:sectnums:\n:sectnumlevels!:\n\n== First\n\n=== Nested\n");
        let outline = convert_outline(&doc);
        assert!(outline.contains(">1. First</a>"));
        assert!(outline.contains(">1.1. Nested</a>"));
    }

    #[test]
    fn drop_anchor_tags_removes_only_real_anchors() {
        // A real `<a …>`/`</a>` pair is reduced to its text, while a tag whose
        // name merely starts with `a` (like `<abbr>`) is left intact — the word
        // boundary in Asciidoctor's `DropAnchorRx`.
        let input = r##"<a href="#x">link</a> and <abbr>HTML</abbr>"##;
        assert_eq!(super::drop_anchor_tags(input), "link and <abbr>HTML</abbr>");
    }
}
