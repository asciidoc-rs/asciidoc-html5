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
//! - `foo/preceding::tag`, `foo/following::tag` — the general document-order
//!   axes (excluding ancestors / descendants respectively)
//! - predicates `[@id="x"]`, `[@class="x"]`, `[@attr="x"]`, `[@attr]`,
//!   `[text()="x"]`, and the positional `[N]` (1-indexed, per context node)
//!
//! Anything outside this subset (the `ancestor::`/`descendant::` named axes,
//! boolean `count(...)` expressions, `normalize-space()`, `contains()`, …) is
//! simply not built yet. An unsupported predicate or axis **panics** rather
//! than being silently ignored — a silently dropped predicate would make a step
//! match more broadly than intended (a false pass). The next test that needs
//! one should **extend this engine** (with unit tests) rather than defer the
//! test — a missing harness feature is never a reason to mark a Ruby test
//! `non_normative!`. See `crate::tests::asciidoctor_rb`'s README for the
//! rationale.
//!
//! Note: the general axes return matches in document order. XPath orders a
//! reverse axis (`preceding::`) in reverse document order, which would matter
//! for a positional predicate *on that axis* (e.g. `preceding::p[1]`); the
//! suite does not use that combination, so the harness does not model it.

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
                Axis::Following => following(root, node)
                    .into_iter()
                    .filter(|c| step.matches(c))
                    .collect(),
                Axis::Preceding => preceding(root, node)
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

/// Returns the elements on the `preceding::` axis of `target`: every element
/// that starts before `target` in document order, excluding `target`'s own
/// ancestors. Results are in document order (their relative order does not
/// affect matching or counting, the only things the harness asks of the axis).
fn preceding<'a>(root: &'a VirtualNode, target: &VirtualNode) -> Vec<&'a VirtualNode> {
    let ancestors = ancestors(root, target);
    let mut before = Vec::new();
    let mut reached = false;
    collect_preorder_before(root, target, &mut before, &mut reached);
    before
        .into_iter()
        .filter(|n| !ancestors.iter().any(|a| std::ptr::eq(*a, *n)))
        .collect()
}

/// Returns the elements on the `following::` axis of `target`: every element
/// that starts after `target`'s subtree ends in document order (which, by
/// construction, excludes `target`'s descendants and ancestors).
fn following<'a>(root: &'a VirtualNode, target: &VirtualNode) -> Vec<&'a VirtualNode> {
    let mut after = Vec::new();
    let mut found = false;
    collect_following(root, target, &mut after, &mut found);
    after
}

/// The ancestors of `target`, from its parent up to (and including) `root`.
fn ancestors<'a>(root: &'a VirtualNode, target: &VirtualNode) -> Vec<&'a VirtualNode> {
    let mut chain = Vec::new();
    let mut current: &VirtualNode = target;
    while let Some(parent) = find_parent(root, current) {
        chain.push(parent);
        current = parent;
    }
    chain
}

/// Pre-order walk collecting every element visited before `target`, stopping as
/// soon as `target` is reached. Ancestors of `target` are collected here (they
/// precede it in the walk) and filtered out by the caller.
fn collect_preorder_before<'a>(
    node: &'a VirtualNode,
    target: &VirtualNode,
    out: &mut Vec<&'a VirtualNode>,
    reached: &mut bool,
) {
    for child in &node.children {
        if *reached {
            return;
        }
        if std::ptr::eq(child, target) {
            *reached = true;
            return;
        }
        out.push(child);
        collect_preorder_before(child, target, out, reached);
    }
}

/// Pre-order walk collecting every element after `target` in document order.
/// `target`'s subtree is skipped entirely (its descendants are not
/// "following"); once `found` flips, every later element — later siblings, and
/// the later subtrees of ancestors — is collected.
fn collect_following<'a>(
    node: &'a VirtualNode,
    target: &VirtualNode,
    out: &mut Vec<&'a VirtualNode>,
    found: &mut bool,
) {
    for child in &node.children {
        if std::ptr::eq(child, target) {
            *found = true;
            continue;
        }
        if *found {
            out.push(child);
        }
        collect_following(child, target, out, found);
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
    Following,
    Preceding,
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
            // XPath `@class="v"` is exact string equality on the class
            // attribute, not CSS token containment: `[@class="paragraph"]` must
            // not match `class="paragraph lead"` (that is what CSS `.paragraph`,
            // via `assert_css`, is for). The projection split the attribute on
            // whitespace, so rejoin with a single space to compare — faithful
            // for the renderer's single-space-separated class lists.
            Pred::Class(v) => node.classes.join(" ") == *v,
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
    } else if let Some(rest) = token.strip_prefix("following::") {
        (Axis::Following, rest)
    } else if let Some(rest) = token.strip_prefix("preceding::") {
        (Axis::Preceding, rest)
    } else {
        // A `::` still in the node-test name (before any predicate) is an axis
        // the engine does not implement. Fail loudly rather than treating, say,
        // `ancestor::div` as a tag literally named "ancestor::div" — which would
        // silently match nothing and could make a "count 0" assertion pass.
        let name = token.split('[').next().unwrap_or(token);
        assert!(
            !name.contains("::"),
            "unsupported XPath axis in `{token}`: this axis is not implemented in \
             assert_html. Extend `xpath.rs` (with a unit test) — see \
             html5/src/tests/asciidoctor_rb/README.md."
        );

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
///
/// # Panics
///
/// Panics on a malformed node test — a name containing predicate punctuation
/// (e.g. a stray `]` as in `p]`), an unterminated `[`, or any leftover text
/// after the predicates. Silently accepting these would evaluate a different
/// selector than written (a false pass); see [`parse_predicate`].
fn parse_node_test(s: &str) -> (NameTest, Vec<Pred>, Option<usize>) {
    let (base, mut rest) = match s.find('[') {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };

    // The name is a bare tag or `*`; predicate punctuation here (most often an
    // unmatched `]`) means the path is malformed.
    assert!(
        !base.contains([']', '@']),
        "malformed XPath node test `{s}`: unexpected `{base}` as an element name. \
         See html5/src/tests/asciidoctor_rb/README.md."
    );

    let name = if base.is_empty() || base == "*" {
        NameTest::Any
    } else {
        NameTest::Named(base.to_string())
    };

    let mut preds = Vec::new();
    let mut index = None;
    while !rest.is_empty() {
        // Every remaining chunk must be a `[…]` predicate. Anything else —
        // trailing junk after the last predicate, or a stray `]` — is a
        // malformed path, and dropping it would silently evaluate the step
        // without the intended filter (a false pass).
        let Some(after_open) = rest.strip_prefix('[') else {
            panic!(
                "malformed XPath node test `{s}`: unexpected `{rest}` where a \
                 `[predicate]` was expected. \
                 See html5/src/tests/asciidoctor_rb/README.md."
            );
        };
        let Some(rel_close) = after_open.find(']') else {
            panic!(
                "unterminated XPath predicate in `{s}`: `[` without a closing `]`. \
                 Fix the expression — see html5/src/tests/asciidoctor_rb/README.md."
            );
        };
        parse_predicate(after_open[..rel_close].trim(), &mut preds, &mut index);
        rest = after_open[rel_close + 1..].trim_start();
    }

    (name, preds, index)
}

/// Parses a single predicate body (the text between `[` and `]`).
///
/// # Panics
///
/// Panics on any predicate the engine does not implement. A silently ignored
/// predicate would make the step match more broadly than the test intends — a
/// false pass — so an unsupported construct must fail loudly. Per the harness
/// rule (`asciidoctor_rb/README.md`), the fix is to extend this parser (with
/// unit tests), never to work around the panic.
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
            return;
        }
    }

    panic!(
        "unsupported XPath predicate `[{inner}]` in assert_html: this construct is \
         not implemented. Extend `xpath.rs` (with a unit test) rather than avoiding \
         it — see html5/src/tests/asciidoctor_rb/README.md."
    );
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
