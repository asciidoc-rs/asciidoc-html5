# adoc

[![CI](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml/badge.svg)](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml)
[![Latest Version](https://img.shields.io/crates/v/adoc.svg)](https://crates.io/crates/adoc)
[![Codecov](https://codecov.io/gh/asciidoc-rs/asciidoc-html5/graph/badge.svg)](https://codecov.io/gh/asciidoc-rs/asciidoc-html5)

A command-line [AsciiDoc](https://asciidoc.org) to HTML5 converter — the `adoc`
command. Reads AsciiDoc from a file (or standard input) and writes HTML5 that
aims to be compatible with [Asciidoctor](https://asciidoctor.org)'s default
`html5` backend.

`adoc` produces HTML5 only. Asciidoctor's other backends are out of scope; in
particular, **DocBook and man page output are not planned**, and `adoc` emits
HTML5 syntax only — **Asciidoctor's XHTML syntax (the `xhtml`/`xhtml5` backends)
is not supported.**

This is the **binary** crate of the
[`asciidoc-html5` workspace](https://github.com/asciidoc-rs/asciidoc-html5). It
is a thin front end over the [`asciidoc-html5`](../html5/) library: read
AsciiDoc, call `asciidoc_html5::convert`, write HTML5.

## 🚧 Status: placeholder, not ready for use 🚧

**As of July 2026 this tool does not work yet.** The command-line plumbing is in
place, but the underlying renderer in the
[`asciidoc-html5`](../html5/) library is unimplemented — so running `adoc`
against real input will **panic** rather than produce HTML. Nothing here is
ready for prime-time; the interface and behavior are expected to change without
notice. Don't install it expecting a working converter yet.

## Intended usage

Once the renderer lands, installing the crate will give you the `adoc` command
(much as installing ripgrep gives you `rg`):

```sh
cargo install adoc            # not useful yet — see status above
```

```sh
adoc input.adoc               # writes input.html (name derived from input)
adoc input.adoc -o out.html   # write to a named file
adoc input.adoc -o -          # write HTML5 to stdout
cat input.adoc | adoc         # read from stdin, write to stdout
```

| Argument            | Description                                                  |
| ------------------- | ----------------------------------------------------------- |
| `input`             | AsciiDoc input file. Omit (or pass `-`) to read stdin.      |
| `-o`, `--output`    | Output file (`-` for stdout). Default: derived from input.  |

Run `adoc -h` for a short summary or `adoc --help` for the full description,
including per-argument details and usage examples.

To run from a checkout of the workspace:

```sh
cargo run --bin adoc -- input.adoc -o out.html
```

## Why this exists

See the [workspace README](../README.md) for the motivation and the projects
this is meant to enable.

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or
[MIT license](../LICENSE-MIT) at your option.
