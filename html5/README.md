# asciidoc-html5

[![CI](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml/badge.svg)](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml)
[![Latest Version](https://img.shields.io/crates/v/asciidoc-html5.svg)](https://crates.io/crates/asciidoc-html5)
[![docs.rs](https://img.shields.io/docsrs/asciidoc-html5)](https://docs.rs/asciidoc-html5/)
[![Codecov](https://codecov.io/gh/asciidoc-rs/asciidoc-html5/graph/badge.svg)](https://codecov.io/gh/asciidoc-rs/asciidoc-html5)

A Rust HTML5 renderer for [AsciiDoc](https://asciidoc.org), built on the
[`asciidoc-parser`](https://crates.io/crates/asciidoc-parser) crate and aiming
for output compatible with [Asciidoctor](https://asciidoctor.org)'s default
`html5` backend.

This is the **library** crate of the
[`asciidoc-html5` workspace](https://github.com/asciidoc-rs/asciidoc-html5). It
is kept deliberately lean — it depends only on `asciidoc-parser` and the
standard library, with no CLI, argument-parsing, or I/O-framework dependencies —
so that other tools can embed it as one step of a larger pipeline. For the
command-line front end, see the [`adoc`](../cli/) crate.

## 🚧 Status: placeholder, not ready for use 🚧

**As of July 2026 this crate is a placeholder and does nothing useful yet.** The
public API is sketched out, but the renderer itself is unimplemented: calling
[`convert`] or [`convert_document`] will panic with a `todo!()`. The surface
described below documents *intended* behavior, not what ships today. Do not
depend on this crate for anything real — the API, and everything it produces, is
expected to change without notice.

## Intended API

```rust
// Once implemented — today this panics with `todo!()`.
let html = asciidoc_html5::convert("= Hello\n\nWorld.");
println!("{html}");
```

- `convert(source: &str) -> String` — parse AsciiDoc source with a default
  parser and render it to a complete HTML5 document.
- `convert_file(path) -> io::Result<String>` — read an AsciiDoc file from disk
  and render it, the file-based counterpart to `convert`.
- `convert_document(document: &Document) -> String` — render an already-parsed
  `Document`, for callers that want to inspect or transform it first.

The plan is byte-for-byte parity with Asciidoctor's `html5` backend for the
constructs that are supported, filled in incrementally. See the crate-level
documentation for the running list of what the renderer aims to emit.

## Why this exists

See the [workspace README](../README.md) for the motivation and the projects
this is meant to enable.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or
[MIT license](../LICENSE-MIT) at your option.
