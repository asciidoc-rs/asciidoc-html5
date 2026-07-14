//! A minimal XPath-subset evaluator over a [`VirtualNode`] tree.
//!
//! This is intentionally *not* a general XPath engine. It implements exactly
//! the location-path shapes that Asciidoctor's Ruby test suite uses to assert
//! on HTML output, enough to port those `assert_xpath` calls faithfully:
//!
//! - `//tag`, `/tag`, `//*`, `/*` — descendant / child steps and the wildcard
//! - `a/b`, `a//b` — child and descendant combinators, chained
//! - `foo/following-sibling::*`, `foo/preceding-sibling::tag` — the sibling
//!   axes
//! - predicates `[@id="x"]`, `[@class="x"]`, `[@attr="x"]`, `[@attr]`,
//!   `[text()="x"]`, and the positional `[N]` (1-indexed, per context node)
//!
//! Anything outside this subset (the general `preceding::`/`ancestor::` axes,
//! boolean `count(...)` expressions, `normalize-space()`, `contains()`, …) is
//! deliberately unsupported: a page that needs one keeps the corresponding
//! Ruby test `non_normative!` until the engine grows to cover it.

use super::dom::VirtualNode;

/// Evaluates `path` against `root`, returning the matched nodes in document
/// order with duplicates removed.
pub(super) fn query<'a>(root: &'a VirtualNode, path: &str) -> Vec<&'a VirtualNode> {
    let steps = parse_path(path);
    let mut context: Vec<&VirtualNode> = vec![root];

    for step in &steps {
        let mut next: Vec<&VirtualNode> = Vec::new();
        for &node in &context {
            let mut matched: Vec<&VirtualNode> = match step.axis {
                Axis::Child => node.children.iter().filter(|c| step.matches(c)).collect(),
                Axis::Descendant => {
                    let mut acc = Vec::new();
                    collect_descendants(node, step, &mut acc);
                    acc
                }
                Axis::FollowingSibling => following_siblings(root, node)
                    .into_iter()
                    .filter(|c| step.matches(c))
                    .collect(),
                Axis::PrecedingSibling => preceding_siblings(root, node)
                    .into_iter()
                    .filter(|c| step.matches(c))
                    .collect(),
            };

            // A positional predicate selects the Nth match within this context
            // node (1-indexed), matching XPath's per-context semantics.
            if let Some(n) = step.index {
                matched = matched
                    .into_iter()
                    .nth(n.wrapping_sub(1))
                    .into_iter()
                    .collect();
            }

            for m in matched {
                push_unique(&mut next, m);
            }
        }
        context = next;
    }

    context
}

/// Collects every descendant of `node` (excluding `node` itself) that matches
/// `step`, in document order.
fn collect_descendants<'a>(node: &'a VirtualNode, step: &Step, acc: &mut Vec<&'a VirtualNode>) {
    for child in &node.children {
        if step.matches(child) {
            acc.push(child);
        }
        collect_descendants(child, step, acc);
    }
}

/// Returns the siblings that follow `target`, in document order.
fn following_siblings<'a>(root: &'a VirtualNode, target: &VirtualNode) -> Vec<&'a VirtualNode> {
    match find_parent(root, target) {
        Some(parent) => match parent.children.iter().position(|c| std::ptr::eq(c, target)) {
            Some(i) => parent.children[i + 1..].iter().collect(),
            None => Vec::new(),
        },
        None => Vec::new(),
    }
}

/// Returns the siblings that precede `target`, in document order.
fn preceding_siblings<'a>(root: &'a VirtualNode, target: &VirtualNode) -> Vec<&'a VirtualNode> {
    match find_parent(root, target) {
        Some(parent) => match parent.children.iter().position(|c| std::ptr::eq(c, target)) {
            Some(i) => parent.children[..i].iter().collect(),
            None => Vec::new(),
        },
        None => Vec::new(),
    }
}

/// Finds the parent of `target` by walking from `root` and comparing node
/// identity (the nodes are all borrows into one owned tree).
fn find_parent<'a>(node: &'a VirtualNode, target: &VirtualNode) -> Option<&'a VirtualNode> {
    for child in &node.children {
        if std::ptr::eq(child, target) {
            return Some(node);
        }
        if let Some(parent) = find_parent(child, target) {
            return Some(parent);
        }
    }
    None
}

/// Pushes `node` onto `set` unless an identical node (by identity) is present.
fn push_unique<'a>(set: &mut Vec<&'a VirtualNode>, node: &'a VirtualNode) {
    if !set.iter().any(|existing| std::ptr::eq(*existing, node)) {
        set.push(node);
    }
}

/// The axis a single location step walks.
#[derive(Clone, Copy)]
enum Axis {
    Child,
    Descendant,
    FollowingSibling,
    PrecedingSibling,
}

/// A node test: a specific tag name, or `*` (any element).
enum NameTest {
    Any,
    Named(String),
}

/// A single predicate inside `[...]`.
enum Pred {
    Id(String),
    Class(String),
    Attr(String, String),
    AttrExists(String),
    Text(String),
}

impl Pred {
    fn matches(&self, node: &VirtualNode) -> bool {
        match self {
            Pred::Id(v) => node.id.as_deref() == Some(v.as_str()),
            Pred::Class(v) => node.classes.iter().any(|c| c == v),
            Pred::Attr(k, v) => node.attributes.get(k).map(String::as_str) == Some(v.as_str()),
            Pred::AttrExists(k) => match k.as_str() {
                "id" => node.id.is_some(),
                "class" => !node.classes.is_empty(),
                _ => node.attributes.contains_key(k),
            },
            Pred::Text(v) => node.text.as_deref() == Some(v.as_str()),
        }
    }
}

/// One location step: an axis, a node test, its (non-positional) predicates,
/// and an optional positional `[N]`.
struct Step {
    axis: Axis,
    name: NameTest,
    preds: Vec<Pred>,
    index: Option<usize>,
}

impl Step {
    fn matches(&self, node: &VirtualNode) -> bool {
        if let NameTest::Named(tag) = &self.name {
            if &node.tag != tag {
                return false;
            }
        }
        self.preds.iter().all(|p| p.matches(node))
    }
}

/// Whether a step follows a `/` (child) or `//` (descendant) separator.
#[derive(Clone, Copy)]
enum Combinator {
    Child,
    Descendant,
}

/// Parses a full location path into its steps.
fn parse_path(path: &str) -> Vec<Step> {
    split_steps(path.trim())
        .into_iter()
        .map(|(comb, token)| parse_step(comb, token))
        .collect()
}

/// Splits a path into `(combinator, token)` pairs, treating `/` inside a
/// `[...]` predicate as literal. A leading `//` (or a bare relative path)
/// starts as a descendant step; a leading `/` starts as a child step.
fn split_steps(s: &str) -> Vec<(Combinator, &str)> {
    let mut out = Vec::new();

    let (mut comb, mut start) = if let Some(rest) = s.strip_prefix("//") {
        (Combinator::Descendant, s.len() - rest.len())
    } else if let Some(rest) = s.strip_prefix('/') {
        (Combinator::Child, s.len() - rest.len())
    } else {
        (Combinator::Descendant, 0)
    };

    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => depth -= 1,
            b'/' if depth == 0 => {
                out.push((comb, s[start..i].trim()));
                if s[i..].starts_with("//") {
                    comb = Combinator::Descendant;
                    i += 2;
                } else {
                    comb = Combinator::Child;
                    i += 1;
                }
                start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push((comb, s[start..].trim()));
    out
}

/// Parses one step token, honoring an explicit sibling axis prefix; otherwise
/// the axis comes from the preceding combinator.
fn parse_step(comb: Combinator, token: &str) -> Step {
    let (axis, node_test) = if let Some(rest) = token.strip_prefix("following-sibling::") {
        (Axis::FollowingSibling, rest)
    } else if let Some(rest) = token.strip_prefix("preceding-sibling::") {
        (Axis::PrecedingSibling, rest)
    } else {
        let axis = match comb {
            Combinator::Child => Axis::Child,
            Combinator::Descendant => Axis::Descendant,
        };
        (axis, token)
    };

    let (name, preds, index) = parse_node_test(node_test);
    Step {
        axis,
        name,
        preds,
        index,
    }
}

/// Parses a node test of the form `tag`, `*`, or `tag[pred][pred]…`.
fn parse_node_test(s: &str) -> (NameTest, Vec<Pred>, Option<usize>) {
    let (base, mut rest) = match s.find('[') {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };

    let name = if base.is_empty() || base == "*" {
        NameTest::Any
    } else {
        NameTest::Named(base.to_string())
    };

    let mut preds = Vec::new();
    let mut index = None;
    while let Some(open) = rest.find('[') {
        let Some(rel_close) = rest[open..].find(']') else {
            break;
        };
        let close = open + rel_close;
        parse_predicate(rest[open + 1..close].trim(), &mut preds, &mut index);
        rest = &rest[close + 1..];
    }

    (name, preds, index)
}

/// Parses a single predicate body (the text between `[` and `]`).
fn parse_predicate(inner: &str, preds: &mut Vec<Pred>, index: &mut Option<usize>) {
    if let Ok(n) = inner.parse::<usize>() {
        *index = Some(n);
        return;
    }

    if let Some(attr) = inner.strip_prefix('@') {
        if let Some((name, value)) = attr.split_once('=') {
            let name = name.trim();
            let value = unquote(value.trim());
            match name {
                "id" => preds.push(Pred::Id(value)),
                "class" => preds.push(Pred::Class(value)),
                _ => preds.push(Pred::Attr(name.to_string(), value)),
            }
        } else {
            preds.push(Pred::AttrExists(attr.trim().to_string()));
        }
        return;
    }

    if let Some(after) = inner.strip_prefix("text()") {
        if let Some(value) = after.trim_start().strip_prefix('=') {
            preds.push(Pred::Text(unquote(value.trim())));
        }
    }
}

/// Strips one layer of matching single or double quotes from an XPath string
/// literal.
fn unquote(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'"' || bytes[0] == b'\'')
        && bytes[bytes.len() - 1] == bytes[0]
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
