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
- [`load`]/[`load_file`] are the *load* half — parse only, returning the owned
  `Document<'static>` without rendering — so the two steps above the arrow can be
  run separately: `load` then `convert_document`. They exist because the load
  step has to honor [`Options`] (attributes, safe mode, the `include::`/docinfo
  jail), which the bare `asciidoc-parser` [`Parser`] exposes only as scattered
  builder methods; `load`/`load_with` apply the whole `Options` bundle in one
  call, the same seeding [`convert`]/[`convert_with`] perform. By construction
  `convert_document(&load(source))` equals `convert(source)`.

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
              ├── Break   → break_block()
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
| `Block::Break` (Page) | `page_break` | `<div style="page-break-after: always;"></div>` | ✅ |
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

## Document attributes

Several skeleton decisions depend on document attributes — `lang`, `doctype`
(→ `<body class>`), `sectnums`, `icons`, `source-highlighter`, `nofooter`,
`notitle`/`noheader`, `docdatetime` (the footer's "Last updated" text). As of
`asciidoc-parser` 0.19 these are readable directly from a `Document` via
[`Document::attribute_value`] / [`Document::has_attribute`] /
[`Document::is_attribute_set`], so `convert_document(&Document)` is fully
self-contained. The baseline reads `lang` and `doctype` from those accessors
(defaulting to Asciidoctor's `en` / `article`) and gates the header, the doctitle
`<h1>`, and the footer on `noheader` / `notitle` / `nofooter`. Two skeleton
details remain deliberately deferred: the footer's "Last updated" text needs a
caller-supplied `docdatetime`, and `<body class>` currently carries just the bare
doctype (Asciidoctor also appends TOC classes such as `toc2 toc-left`).

## Safe mode and the default stylesheet

The [safe mode](https://docs.asciidoctor.org/asciidoc/latest/safe-modes/) is a
conversion setting rather than a block construct, but it reaches the renderer
through document attributes, so it fits the same self-contained model. `Options`
carries the `SafeMode` (defaulting to `Secure`, Asciidoctor's API default) and,
in `Options::apply`, seeds it onto the parser with `Parser::with_safe_mode` —
which also populates the `safe-mode-level` / `safe-mode-name` / `safe-mode-<name>`
intrinsic attributes a document can reference. The `adoc` CLI overrides the
default to `Unsafe` (Asciidoctor's CLI default), exposed as `-S`/`--safe-mode`.

The safe mode's one rendered effect today is whether the default stylesheet is
*linked* (`<link href="./asciidoctor.css">`) or *embedded* (`<style>…`). Matching
Asciidoctor's `document.rb`, `Options::apply` force-sets `linkcss` (locked, via
`ModificationContext::ApiOnly`) under `Secure` unless the caller supplied
`linkcss` themselves — so a document `:linkcss!:` cannot re-enable embedding under
`Secure`, but an API-level `linkcss` unset can. The renderer's `links_stylesheet`
then decides from the resolved state: an explicit `linkcss` wins either way, and
otherwise a `safe-mode-level` of `Secure` (20) or greater links. Keying the
render-time default off `safe-mode-level` keeps `convert_document(&Document)`
consistent with `convert` even for a document parsed by a bare `Parser` (whose
built-in attributes already default to `Secure`).

A *linked* stylesheet has to exist next to the HTML, which Asciidoctor handles
with the `copycss` attribute — set by default in every safe mode but `secure`.
`Options::apply` seeds `copycss` the same way (document-overridable, so a header
`:!copycss:` can turn it off), and the `copycss` module turns the resolved state
into a *plan*: the destination path relative to the output directory (the
default stylesheet as `asciidoctor.css`, a custom one at its `stylesdir` web
path, honoring a `copycss=<path>` read-from override) plus the bytes to write
(the embedded default CSS, or a custom stylesheet read through the same
safe-mode jail as `include::`). Because the library converts text to text and
does not own the output directory, it never writes the file itself: the
`_with_writer` entry points hand the plan to a caller-supplied `AssetWriter`
(the file-writing seam, with `DirAssetWriter` as the on-disk implementation the
CLI roots at the output directory). `copycss` is a pure file side effect — the
returned HTML is byte-identical to the writer-less path.

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
   verbatim blocks, thematic and page breaks, the dispatch/recursion machinery,
   and the attribute-driven skeleton (`lang`, `doctype`,
   `notitle`/`noheader`/`nofooter`).
2. **Block coverage:** lists (un/ordered/description/callout), the delimited
   example/sidebar/open blocks, admonitions, quotes/verses, images.
3. **Tables** (their own content model).
4. **Document chrome:** footer "Last updated" (`docdatetime`), the full
   `<body class>` (TOC classes), TOC, footnotes, the default stylesheet.
5. **Parity hardening:** fixture-based diff tests against Asciidoctor output.

## Parser API history (resolved in 0.19)

Working through the baseline surfaced several places where the renderer reached
past what `asciidoc-parser` 0.18 exposed. These were filed as
[asciidoc-parser#620](https://github.com/asciidoc-rs/asciidoc-parser/issues/620)
(attribute access) and
[asciidoc-parser#621](https://github.com/asciidoc-rs/asciidoc-parser/issues/621)
(ergonomics), and **all landed in `asciidoc-parser` 0.19**, which this crate now
depends on:

1. **Document-level attribute access** — [`Document::attribute_value`] /
   [`Document::has_attribute`] / [`Document::is_attribute_set`] make
   `convert_document(&Document)` self-contained (`lang`, `doctype`,
   `notitle`/`noheader`/`nofooter`, `sectnums`, …). `Document::show_doctitle()`,
   which baked in the embedded default, was removed in favor of reading the raw
   `notitle`/`noheader` state.
2. **Built-in context vocabulary** — the [`BuiltInContext`] enum (with `ALL` /
   `from_str` / `as_str`) replaces string-matching against the private
   `is_built_in_context`.
3. **Section id on `Block`** — [`IsBlock::id`] now delegates to the `SectionBlock`
   (and `MediaBlock`) override, so `block.id()` returns a section's
   auto-generated id.
4. **Compound-block type accessor** — `CompoundDelimitedBlock::context_kind()`
   returns a typed `CompoundDelimitedContext` (Example / Open / Sidebar).
5. **Ordered-list start value** — `ListBlock::start() -> Option<i64>` for
   `<ol start="…">`.
6. **Catalog enumeration** — `Catalog::ids()` / `Catalog::entries()` expose
   read-only iterators for building a multi-document cross-reference index.

[`convert`]: crate::convert
[`convert_with`]: crate::convert_with
[`convert_document`]: crate::convert_document
[`load`]: crate::load
[`load_file`]: crate::load_file
[`Options`]: crate::Options
[`Document`]: asciidoc_parser::Document
[`Document::attribute_value`]: asciidoc_parser::Document::attribute_value
[`Document::has_attribute`]: asciidoc_parser::Document::has_attribute
[`Document::is_attribute_set`]: asciidoc_parser::Document::is_attribute_set
[`Document::resolve_references`]: asciidoc_parser::Document::resolve_references
[`Parser`]: asciidoc_parser::Parser
[`Block`]: asciidoc_parser::blocks::Block
[`BuiltInContext`]: asciidoc_parser::blocks::BuiltInContext
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
