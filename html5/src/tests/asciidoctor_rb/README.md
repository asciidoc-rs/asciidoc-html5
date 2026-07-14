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
3. For each Ruby `test '…' do … end` we port, wraps those lines in a
   `verifies!` block inside a `#[test]` and re-expresses the Ruby assertions in
   Rust against this crate's output.
4. Leaves everything else — scaffolding, and tests for behavior out of scope
   here — in `non_normative!`.

`preamble_test.rs` ports 4 of the 12 Ruby tests (the `html5` preamble cases) and
tracks the other 8 as `non_normative!`: the DocBook-backend tests (this crate
targets only `html5`), the `book`/`partintro` cases (not yet rendered), the
`toc` case ([#86] — TOC rendering not wired up yet), and one test that needs the
general `preceding::` XPath axis ([#87] — see *Limitations*).

[#86]: https://github.com/asciidoc-rs/asciidoc-html5/issues/86
[#87]: https://github.com/asciidoc-rs/asciidoc-html5/issues/87

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

### Supported XPath subset

`//tag`, `/tag`, `//*`, `/*`; chained child (`a/b`) and descendant (`a//b`)
steps; the `following-sibling::` and `preceding-sibling::` axes; predicates
`[@id="x"]`, `[@class="x"]`, `[@attr="x"]`, `[@attr]`, `[text()="x"]`, and the
positional `[N]` (1-indexed, per context node).

### Limitations (grow the engine as pages need it)

- The general `preceding::` / `following::` / `ancestor::` axes are not
  implemented (only the `*-sibling::` axes are). Tracked in [#87].
- Boolean expressions (`count(...) = N`), `normalize-space()`, `contains()`, and
  `starts-with()` predicates are not implemented.
- `text()` compares against an element's *direct* text only (matching XPath's
  `text()` node test), not its full descendant text.

A Ruby test that needs an unsupported construct stays `non_normative!` until the
engine grows to cover it — that is the honest, `sdd`-visible way to defer it.

## Dependencies

`scraper` is a **`[dev-dependencies]`** entry (test-only); it never enters the
shipped library. It is pinned to `0.23`, whose tree builds on the workspace MSRV
(Rust 1.88).
