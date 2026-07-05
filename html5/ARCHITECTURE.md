# HTML5 renderer architecture

This document sketches the architecture of the `asciidoc-html5` renderer: how it
consumes the parse tree produced by
[`asciidoc-parser`](https://crates.io/crates/asciidoc-parser) and turns it into
an HTML5 document compatible with Asciidoctor's default `html5` backend. It
describes the baseline that exists today and the shape the full renderer grows
into.

The companion code lives in [`src/renderer.rs`](src/renderer.rs) (the walker)
and [`src/html.rs`](src/html.rs) (attribute/escaping helpers), with the public
entry points in [`src/lib.rs`](src/lib.rs).

## Scope and the guiding principle

The single most important architectural fact is this:

> **The parser does inline. This crate does blocks.**

`asciidoc-parser` applies *inline* substitutions eagerly, at parse time, through
its default [`HtmlSubstitutionRenderer`]. By the time we hold a [`Document`],
every block's content and every title is **already an Asciidoctor-compatible
inline HTML fragment** — `<strong>`, `<em>`, `<code>`, `<a href>`, `<mark>`,
resolved cross references, escaped special characters, and so on. There is no
inline AST to walk; inline content is delivered as a finished `&str`
([`Content::rendered`], surfaced on blocks as [`IsBlock::rendered_content`]).

So this crate never parses or formats inline markup. Its whole job is to emit the
**block-level scaffolding** — the nested `<div class="…">` structure Asciidoctor
wraps around those fragments — by visiting the document's blocks in order.

Two consequences:

- If we ever want a non-HTML backend (DocBook, a diffing renderer, …), the lever
  is the parser's [`InlineSubstitutionRenderer`] trait, set *before* parsing via
  `Parser::with_inline_substitution_renderer`. It is not something this crate can
  retrofit onto an already-parsed `Document`.
- We must still HTML-escape the few strings *we* place into markup ourselves —
  attribute values like ids, roles, and image `alt`/`src`. Block content and
  titles are already escaped by the parser and are emitted verbatim.

## The pipeline

```
source ──► Parser::parse ──► Document ──► convert_document ──► HTML5 string
           (asciidoc-parser)            (this crate)
```

- [`convert`] is the convenience path: parse with a default [`Parser`], then
  render. `Parser::parse` also resolves cross references against the document's
  own catalog, so single-document output needs no extra pass.
- [`convert_document`] is the embed path for callers that already hold a
  `Document` (e.g. a future Antora-style generator that parses many files,
  merges catalogs, and calls [`Document::resolve_references`] with a combined
  index before rendering each one).

## The walker

Rendering is a recursive descent over the block tree, accumulating into a single
`String` buffer. The design is deliberately small and uniform:

```
render_document(&Document) -> String
  └── Renderer { out: String }
        ├── document()      emit <head>, header, #content, footer skeleton
        ├── header()        <div id="header"> — <h1>, authors, revision
        ├── blocks(Iter)    for each sibling block → block()
        └── block(&Block)   ── THE DISPATCH POINT ──
              ├── Simple  → paragraph() | verbatim()
              ├── Section → section()   → recurses via blocks(nested_blocks())
              ├── Preamble→ preamble()  → recurses
              ├── Break   → thematic_break()
              ├── RawDelimited → verbatim() (by resolved_context)
              └── _       → unsupported()  (visible HTML comment)
```

`block()` matches on the [`Block`] enum variant. For delimited blocks whose
variant alone is ambiguous (a `RawDelimitedBlock` is listing *or* literal *or*
passthrough), it dispatches on [`IsBlock::resolved_context`] — the parser's
resolved block "type" string (`"listing"`, `"sidebar"`, `"example"`, …).

Compound blocks (sections, the preamble, and later lists, tables, and the
delimited example/sidebar/open blocks) recurse back into `blocks()` over their
[`IsBlock::nested_blocks`]. That is the whole recursion: one dispatch function,
one `nested_blocks` iterator, and the tree walks itself. Adding a construct is
adding one match arm and one `render_*` method.

The `'src` lifetime threads through the walk because several `IsBlock` accessors
borrow `&'src self` (the self-referential-lifetime pattern the parser uses).
`document.nested_blocks()` yields `&'src Block<'src>`, and each block's own
`nested_blocks()` yields the same, so the walker methods are written
`fn block<'src>(&mut self, block: &'src Block<'src>)` and the lifetimes line up
without any cloning.

## Mapping the parse tree to HTML

The target shapes come from Asciidoctor's `html5` backend, cross-checked against
the pinned test suite in [`ref/asciidoctor/test`](../ref/asciidoctor) and the
language docs in [`ref/asciidoc-lang`](../ref/asciidoc-lang). The table below is
the working map; **✅ = wired up in the baseline**, ⬜ = next phases.

| Parse node | `resolved_context` | HTML shape (Asciidoctor `html5`) | |
|---|---|---|---|
| `Block::Simple` (Paragraph) | `paragraph` | `<div class="paragraph"><p>…</p></div>` | ✅ |
| `Block::Simple` (Listing/Source) | `listing` | `<div class="listingblock"><div class="content"><pre>…</pre></div></div>` | ✅ |
| `Block::Simple` (Literal) | `literal` | `<div class="literalblock"><div class="content"><pre>…</pre></div></div>` | ✅ |
| `Block::Section` | `section` | `<div class="sectN"><hM id>…</hM><div class="sectionbody">…</div></div>` | ✅ |
| `Block::Preamble` | `preamble` | `<div id="preamble"><div class="sectionbody">…</div></div>` | ✅ |
| `Block::Break` (Thematic) | `thematic_break` | `<hr>` | ✅ |
| `Block::RawDelimited` | `listing`/`literal` | as listing/literal above | ✅ |
| `Block::RawDelimited` | `pass` | raw passthrough (no wrapper) | ⬜ |
| `Block::List` (Unordered) | `list` | `<div class="ulist"><ul><li><p>…</p></li></ul></div>` | ⬜ |
| `Block::List` (Ordered) | `list` | `<div class="olist arabic"><ol class="arabic">…</ol></div>` | ⬜ |
| `Block::List` (Description) | `list` | `<div class="dlist"><dl><dt class="hdlist1">…</dt><dd>…</dd></dl></div>` | ⬜ |
| `Block::List` (Callout) | `list` | `<div class="colist arabic"><ol>…</ol></div>` | ⬜ |
| `Block::CompoundDelimited` | `example` | `<div class="exampleblock"><div class="content">…</div></div>` | ⬜ |
| `Block::CompoundDelimited` | `sidebar` | `<div class="sidebarblock"><div class="content">…</div></div>` | ⬜ |
| `Block::CompoundDelimited` | `open` | `<div class="openblock"><div class="content">…</div></div>` | ⬜ |
| `Block::Admonition` | `admonition` | `<div class="admonitionblock note"><table><tr><td class="icon">…</td><td class="content">…</td></tr></table></div>` | ⬜ |
| `Block::Quote` | `quote`/`verse` | `<div class="quoteblock"><blockquote>…</blockquote><div class="attribution">…</div></div>` | ⬜ |
| `Block::Media` (Image) | `image` | `<div class="imageblock"><div class="content"><img …></div></div>` | ⬜ |
| `Block::Media` (Video/Audio) | `video`/`audio` | `<div class="videoblock">…` | ⬜ |
| `Block::Table` | `table` | `<table class="tableblock frame-all grid-all">…` | ⬜ |
| `Block::Break` (Page) | `page_break` | `<div style="page-break-after: always;"></div>` | ⬜ |
| `Block::DocumentAttribute` | `attribute` | *(no output; updates attribute state)* | ⬜ |

Every wrapper additionally carries the block's `id` and roles when present (see
below), and an optional leading `<div class="title">…</div>` caption.

### Sections

The HTML heading level is the AsciiDoc section level **+ 1** (the doctitle is
level 0 = `<h1>`, so `==` is level 1 = `<h2>`). Only level-1 sections wrap their
body in `<div class="sectionbody">`; deeper levels place children directly after
the heading. Discrete headings ([`SectionType::Discrete`]) render as a bare
`<hN>` with no wrapper. Section ids come from [`SectionBlock::id`] — note that
`Block::id()` does *not* surface the auto-generated section id, only the
`SectionBlock` override does (it falls back to the synthesized `_slug`).

### The document skeleton and header

`document()` emits the standalone shell: `<!DOCTYPE html>`, `<html lang>`,
a `<head>` (charset, `X-UA-Compatible`, viewport, generator, `<title>`),
`<body class="article">`, the header, `<div id="content">`, and the footer.

`header()` emits `<div id="header">` with the `<h1>` doctitle and, when present,
a `<div class="details">` block carrying `<span id="author">` / `<span
id="email">` (numbered for co-authors) and `<span id="revnumber/revdate/
revremark">`, matching the shapes asserted in
[`ref/asciidoctor/test/document_test.rb`](../ref/asciidoctor/test/document_test.rb).

## Content models, ids, roles, titles

- **Content models.** [`ContentModel`] tells us *how* a block carries content:
  `Simple`/`Verbatim`/`Raw` blocks expose text via `rendered_content()`;
  `Compound` blocks expose children via `nested_blocks()`; `Empty` blocks
  (images, breaks) carry neither; `Table` is its own fixed structure. The
  renderer keys most leaf-vs-container decisions off the variant, but the content
  model is the fallback signal for generically handling unknown block styles.
- **Ids and roles.** [`IsBlock::id`] and [`IsBlock::roles`] are uniform across
  blocks. Roles map to extra HTML `class` tokens (Asciidoctor's convention), so
  `[.lead]` on a paragraph yields `<div class="paragraph lead">`. Assembled by
  `html::id_attribute` / `html::class_attribute`.
- **Titles and captions.** [`IsBlock::title`] returns the substituted title;
  captionable blocks also expose [`IsBlock::caption`] (a ready-made prefix like
  `"Example 1. "`) and [`IsBlock::number`]. Title placement differs by block:
  inside the wrapper before `<p>`/`<pre>` for paragraphs and verbatim blocks;
  after `<div class="content">` for images; first child for example/sidebar/open.

## Escaping model

- Block **content** (`rendered_content()`) and **titles** (`title()`) are already
  HTML with substitutions applied — emitted verbatim.
- Values **this crate** injects into attributes — ids, roles, and (later) image
  `src`/`alt`, link `href` — are escaped with `html::escape_attribute`.
- Verbatim block bodies are emitted inside `<pre>` with their literal line breaks
  preserved and no added surrounding whitespace, so the rendered text is
  byte-faithful.

## Document attributes (a known limitation)

Several skeleton decisions depend on document attributes — `lang`, `doctype`
(→ `<body class>`), `sectnums`, `icons`, `source-highlighter`, `nofooter`,
`notitle`/`noheader`, `docdatetime` (the footer's "Last updated" text). These are
readable from the **`Parser`** (`Parser::attribute_value`) but **not** from a
bare **`Document`**, which is what `convert_document` receives. Until the parser
exposes document-level attribute access on `Document` (see below), the baseline
uses Asciidoctor's defaults (`lang=en`, `article` doctype) and defers the footer
timestamp. This is the top item in "Parser API gaps".

## Cross references, footnotes, TOC (future)

- **Cross references** are resolved by `Parser::parse` for single documents; the
  rendered content already contains the resolved `<a href="#id">`. Multi-document
  pipelines use `parse_deferred` + `Document::resolve_references`.
- **Footnotes** accumulate in the [`Catalog`]; the renderer will emit the
  `<div id="footnotes">` section from `catalog().footnotes()` after the body.
- **TOC** metadata is already resolved on `Document` (`toc_mode`, `toc_levels`,
  `toc_title`, `toc_class`); rendering the `<div id="toc">` tree is a later
  phase that walks section blocks to build the list.

## Testing and parity strategy

- **Unit tests** live next to the walker in `renderer.rs`, asserting the exact
  block shapes for each supported construct (skeleton, paragraph, nested
  sections, preamble, verbatim escaping, breaks, titles/roles, and the
  unsupported-marker fallback).
- **Parity tests** (next phase) will render fixtures and compare against
  Asciidoctor's expected HTML. The pinned Ruby suite in `ref/asciidoctor/test`
  encodes the exact tags/classes/ids we target; those become the oracle.
- The three CI gates in [`CLAUDE.md`](../CLAUDE.md) (`fmt`, `clippy -D warnings`,
  `test`) all pass on the baseline.

## Roadmap

1. **Baseline (done):** skeleton, header, paragraphs, sections, preamble,
   verbatim blocks, thematic breaks, and the dispatch/recursion machinery.
2. **Block coverage:** lists (un/ordered/description/callout), the delimited
   example/sidebar/open blocks, admonitions, quotes/verses, images, page breaks.
3. **Tables** (their own content model).
4. **Document chrome:** doctype/`lang`/body classes from attributes, footer
   "Last updated", TOC, footnotes, the default stylesheet.
5. **Parity hardening:** fixture-based diff tests against Asciidoctor output.

## Parser API gaps (proposed upstream issues)

Working through the baseline surfaced several places where the renderer reaches
past what `asciidoc-parser` 0.18 comfortably exposes. These are candidates for
issues on [`asciidoc-parser`](https://github.com/asciidoc-rs/asciidoc-parser):

1. **Document-level attribute access.** `convert_document(&Document)` cannot read
   resolved document attributes (`lang`, `doctype`, `sectnums`, `icons`,
   `source-highlighter`, `nofooter`, `notitle`/`noheader`, `docdatetime`, …);
   `attribute_value`/`is_attribute_set`/`has_attribute` live only on `Parser`.
   Proposal: mirror those accessors on `Document` (the resolved attribute state
   is already retained for header/toc resolution). *This is the biggest blocker
   for a `Document`-based renderer.*
2. **Standalone title visibility.** `Document::show_doctitle()` bakes in the
   *embedded* default (hidden unless `showtitle`), so it returns `false` for an
   ordinary standalone document that should show its `<h1>`. Proposal: expose the
   raw `noheader`/`notitle` state (falls out of #1), or add a
   context-aware/standalone accessor, and document the distinction.
3. **Built-in context vocabulary.** Block dispatch is stringly-typed via
   `resolved_context()`, but the canonical set of built-in context strings is
   only in the `pub(crate)` `is_built_in_context`. Proposal: expose the
   vocabulary (public constants or a `BuiltInContext` enum) so downstreams don't
   hardcode it.
4. **Section id on `Block`.** `Block::id()` deliberately omits a section's
   auto-generated id; you must match `Block::Section(s)` and call `s.id()`. This
   is a footgun. Proposal: either surface it through `Block::id()` or document
   the asymmetry prominently.
5. **Compound-block type accessors.** `CompoundDelimitedBlock` and `Preamble`
   expose nothing beyond the trait, so example-vs-sidebar-vs-open is distinguished
   only by string matching. Proposal: a small enum accessor.
6. **Ordered-list start value.** `ListBlock` exposes `type_()` and
   `marker_style()` but not an explicit numeric `start`, needed for
   `<ol start="…">`. Proposal: expose the resolved start.
7. **Catalog enumeration.** `Catalog` offers point lookups but its `ids()` /
   `entries()` iterators are commented out, so a multi-document pipeline can't
   enumerate all anchors/sections without re-walking the tree. Proposal: expose
   read-only iterators.

[`convert`]: crate::convert
[`convert_document`]: crate::convert_document
[`Document`]: asciidoc_parser::Document
[`Document::resolve_references`]: asciidoc_parser::Document::resolve_references
[`Parser`]: asciidoc_parser::Parser
[`Block`]: asciidoc_parser::blocks::Block
[`ContentModel`]: asciidoc_parser::blocks::ContentModel
[`IsBlock::id`]: asciidoc_parser::blocks::IsBlock::id
[`IsBlock::roles`]: asciidoc_parser::blocks::IsBlock::roles
[`IsBlock::title`]: asciidoc_parser::blocks::IsBlock::title
[`IsBlock::caption`]: asciidoc_parser::blocks::IsBlock::caption
[`IsBlock::number`]: asciidoc_parser::blocks::IsBlock::number
[`IsBlock::rendered_content`]: asciidoc_parser::blocks::IsBlock::rendered_content
[`IsBlock::resolved_context`]: asciidoc_parser::blocks::IsBlock::resolved_context
[`IsBlock::nested_blocks`]: asciidoc_parser::blocks::IsBlock::nested_blocks
[`SectionBlock::id`]: asciidoc_parser::blocks::SectionBlock
[`SectionType::Discrete`]: asciidoc_parser::blocks::SectionType
[`Content::rendered`]: asciidoc_parser::content::Content::rendered
[`Catalog`]: asciidoc_parser::document::Catalog
[`HtmlSubstitutionRenderer`]: asciidoc_parser::parser::HtmlSubstitutionRenderer
[`InlineSubstitutionRenderer`]: asciidoc_parser::parser::InlineSubstitutionRenderer
