# Porting Asciidoctor's Ruby test suite

This directory ports the [Ruby Asciidoctor test suite](../../../../ref/asciidoctor/test)
(vendored verbatim under `ref/asciidoctor/test/`) into this crate, one
`*_test.rb` file at a time, and lets the workspace's `sdd` tool measure how much
of each file we've reproduced.

Asciidoctor's `html5` backend is this renderer's compatibility oracle, so its
own test suite is the most direct statement of the behavior we must match. This
is a long, incremental effort; the point of wiring it through `sdd` is to make
partial progress **visible** and **non-regressing**.

## How coverage is observed (it already works for `.rb`)

The `sdd` tool's spec sources already include the Ruby suite:

```rust
// sdd/src/main.rs
const SPEC_SOURCES: &[(&str, &str, Option<&str>)] = &[
    // …
    ("../ref/asciidoctor/test", ".rb", None),
    // …
];
```

So a `*_test.rb` file is a first-class tracked spec, measured line for line
exactly like a `.adoc` page. Running `(cd sdd && cargo run)` prints Codecov JSON
in which each reproduced line is `1` (sits inside a `verifies!` block — a real
`#[test]` exercises it) or `0` (the reproduction ran out before that line —
uncovered). Lines wrapped in `non_normative!` are tracked but emit nothing.

## How a file is ported

One module per Ruby file (e.g. [`preamble_test.rs`](preamble_test.rs)), which:

1. Declares `track_file!("ref/asciidoctor/test/<name>_test.rb")`.
2. **Reproduces the entire `.rb` file, line for line, blank lines included**,
   partitioned into `non_normative!` and `verifies!` blocks. Because `sdd`
   aligns the reproduction against the reference *by position*, a single dropped
   or added line (even a blank) misaligns everything after it. Bind each
   boundary blank line to the block it *follows* (a block starts on its first
   line of content and carries the trailing blank).
3. **Mirrors the Ruby file's `context` structure with Rust modules.** Each
   nested `context '…' do` becomes a nested `mod` (named after the context), so
   the Rust module tree matches the suite's own partitioning. The file's
   top-level `context` corresponds to the module itself, so it needs no extra
   `mod`. (`preamble_test.rb` has a single top-level `context`, so no nested
   module is introduced.)
4. For each Ruby `test '…' do … end` we port, wraps those lines in a
   `verifies!` block inside a `#[test]` and re-expresses the Ruby assertions in
   Rust against this crate's output.
5. Leaves in `non_normative!` only what this crate genuinely does not produce —
   scaffolding, and tests for behavior that is out of scope (other backends) or
   not yet rendered. A gap in the *XPath assertion harness* is never a reason to
   defer a test; see the harness rule below.

### Driving the renderer

The Ruby helpers map to this crate as:

| Ruby (`test_helper.rb`)         | This crate                                            |
| ------------------------------- | ----------------------------------------------------- |
| `convert_string(input)`         | `convert_with(input, &Options::new().standalone(true))` |
| `convert_string_to_embedded(input)` | `convert(input)`                                  |

## The assertion harness: [`crate::tests::assert_html`](../assert_html/mod.rs)

`assert_css` and `assert_xpath` mirror Asciidoctor's Nokogiri-backed helpers but
query the **rendered HTML string** (this crate's output), not a parsed
`Document`. This is the counterpart to `asciidoc-parser`'s `assert_dom` harness,
which queries the parse tree instead.

### Decisions

- **Parse with `scraper` (html5ever).** The DOM the assertions see is built with
  the same HTML5 tree-construction rules a browser — and Nokogiri — applies, so
  our view of the output matches the oracle's. `assert_html::parse` picks a
  full-document vs. fragment parse by sniffing the string, mirroring Nokogiri's
  `xmldoc_from_string`: standalone output (leading doctype/`<html>`) parses as a
  document, embedded output as a fragment.
- **`assert_css` uses `scraper`'s native selector engine** (Servo's
  `selectors`). It already supports every selector idiom the Ruby suite uses
  (`>`, descendant, `:nth-child`, `:last-of-type`, `[attr*="…"]`, `:not`,
  `:empty`, …), so there is no reason to reimplement CSS.
- **`assert_xpath` uses a small hand-rolled XPath subset** ([`xpath.rs`](../assert_html/xpath.rs))
  over a lightweight [`VirtualNode`](../assert_html/dom.rs) projection of the
  parsed tree. `scraper` has no XPath support. A faithful XPath engine over real
  HTML would mean either a C dependency (libxml2, via the `libxml` crate — the
  exact engine Nokogiri uses, but against this workspace's pure-Rust, lean
  ethos) or a large amount of code. The Ruby suite only exercises a narrow,
  well-understood slice of XPath, so we implement exactly that slice. `sxd-xpath`
  was rejected outright: it requires well-formed XML, and `html5` output emits
  unclosed void elements (`<br>`, `<hr>`, `<img>`, `<col>`) that break a strict
  XML parser.
- **Explicit match counts.** Both helpers take an exact expected count, the most
  common form in the suite (`assert_xpath expr, output, N`). Asciidoctor also
  allows omitting the count ("at least one") and passing a boolean (for
  `count(...)` expressions); those are added as sibling helpers when a ported
  page first needs them.
- **XPath-harness gaps are fixed, not deferred.** When a Ruby `assert_xpath`
  uses a construct the engine does not yet support, **extend the engine** — add
  the axis/predicate to [`xpath.rs`](../assert_html/xpath.rs) with unit tests —
  and port the test. Do **not** mark a test `non_normative!` to sidestep a
  missing harness feature. `non_normative!` is reserved for behavior this crate
  does not produce (other backends, features not yet rendered), never for a
  limitation of the test harness itself. (This is why the general
  `preceding::`/`following::` axes were added rather than deferring the "no title
  with preamble and section" test.)

### Supported XPath subset

`//tag`, `/tag`, `//*`, `/*`; chained child (`a/b`) and descendant (`a//b`)
steps; the `following-sibling::` / `preceding-sibling::` sibling axes and the
general `following::` / `preceding::` document-order axes; predicates `[@id="x"]`,
`[@class="x"]`, `[@attr="x"]`, `[@attr]`, `[text()="x"]`, and the positional
`[N]` (1-indexed, per context node).

### Not yet implemented (add on first use)

The engine covers what the ported pages have needed so far; the following are
simply not built yet. Per the harness rule above, the next test that needs one
**adds it** (with unit tests) — it is not a reason to defer the test:

- The `ancestor::` / `descendant::` named axes.
- A positional predicate *on* a reverse axis (e.g. `preceding::p[1]`): the
  general axes return matches in document order, whereas XPath orders a reverse
  axis in reverse. The suite does not use that combination.
- Boolean expressions (`count(...) = N`), `normalize-space()`, `contains()`, and
  `starts-with()` predicates.
- `text()` compares against an element's *direct* text only (matching XPath's
  `text()` node test), not its full descendant text.

## Dependencies

`scraper` is a **`[dev-dependencies]`** entry (test-only); it never enters the
shipped library. It is pinned to `0.23`, whose tree builds on the workspace MSRV
(Rust 1.88).
