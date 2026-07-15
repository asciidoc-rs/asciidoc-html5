//! A minimal read-only DOM built from the renderer's HTML output.
//!
//! [`assert_xpath`](super::assert_xpath) needs to walk the produced HTML as a
//! tree (parents, children, siblings, text). `scraper` already parses the HTML
//! with a real HTML5 tree builder, but its query surface is CSS-only. Rather
//! than teach the XPath engine to speak `scraper`/`ego_tree` node types, we
//! project the parsed tree once into this small owned [`VirtualNode`] structure
//! — the same shape `asciidoc-parser`'s test harness queries — and run the
//! XPath subset over that.
//!
//! The projection is deliberately lossy in one way that matches how the ported
//! Asciidoctor tests read the DOM: an element's `text` is the concatenation of
//! its *direct* text-node children (mirroring XPath's `text()`), while nested
//! element content lives in `children`.

use std::collections::BTreeMap;

use scraper::{ElementRef, Html, Node};

/// An element in the projected HTML tree.
#[derive(Debug, Clone)]
pub(super) struct VirtualNode {
    /// Lower-cased tag name (`div`, `p`, `h2`, …). The synthetic root carries
    /// the sentinel tag `#root` so no real selector matches it.
    pub(super) tag: String,

    /// The `id` attribute, lifted out of the generic attribute map.
    pub(super) id: Option<String>,

    /// The `class` attribute, split on whitespace.
    pub(super) classes: Vec<String>,

    /// Every other attribute, in name order.
    pub(super) attributes: BTreeMap<String, String>,

    /// Concatenated direct text-node children (what XPath `text()` sees), or
    /// `None` when the element has no direct text.
    pub(super) text: Option<String>,

    /// Child elements, in document order.
    pub(super) children: Vec<VirtualNode>,
}

impl VirtualNode {
    fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            id: None,
            classes: Vec::new(),
            attributes: BTreeMap::new(),
            text: None,
            children: Vec::new(),
        }
    }
}

/// Projects a parsed `scraper` document into a [`VirtualNode`] tree rooted at a
/// synthetic `#root`.
///
/// Anchoring queries at `#root` means a leading-`//` XPath (a descendant
/// search) sees every element in the document, matching how Asciidoctor's tests
/// query a parsed fragment.
///
/// `is_fragment` distinguishes the two parses because `scraper` wraps a parsed
/// fragment in a synthetic `<html>`, whereas Nokogiri — the oracle — models a
/// fragment with no wrapper, so its top-level elements *are* the document
/// roots. For a fragment we therefore hoist the wrapper's children directly
/// under `#root`, so a leading-`/` XPath (a child step from the root) matches
/// the fragment's top-level elements the way Nokogiri does. For a full document
/// the `<html>` element is the real root and stays in place.
pub(super) fn from_html(html: &Html, is_fragment: bool) -> VirtualNode {
    let mut root = VirtualNode::new("#root");
    let document_element = convert(html.root_element());
    if is_fragment {
        root.children = document_element.children;
    } else {
        root.children.push(document_element);
    }
    root
}

/// Recursively converts a `scraper` element into a [`VirtualNode`].
fn convert(el: ElementRef<'_>) -> VirtualNode {
    let value = el.value();
    let mut node = VirtualNode::new(value.name());

    for (name, val) in value.attrs() {
        match name {
            "id" => node.id = Some(val.to_string()),
            "class" => node.classes = val.split_whitespace().map(str::to_string).collect(),
            _ => {
                node.attributes.insert(name.to_string(), val.to_string());
            }
        }
    }

    let mut text = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => text.push_str(t),
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    node.children.push(convert(child_el));
                }
            }
            _ => {}
        }
    }
    if !text.is_empty() {
        node.text = Some(text);
    }

    node
}
