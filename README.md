# asciidoc-html5

[![CI](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml/badge.svg)](https://github.com/asciidoc-rs/asciidoc-html5/actions/workflows/ci.yml)
[![asciidoc-html5 on crates.io](https://img.shields.io/crates/v/asciidoc-html5.svg?label=asciidoc-html5)](https://crates.io/crates/asciidoc-html5)
[![adoc on crates.io](https://img.shields.io/crates/v/adoc.svg?label=adoc)](https://crates.io/crates/adoc)
[![Codecov](https://codecov.io/gh/asciidoc-rs/asciidoc-html5/graph/badge.svg)](https://codecov.io/gh/asciidoc-rs/asciidoc-html5)

A Rust HTML5 renderer for [AsciiDoc](https://asciidoc.org), built on the [`asciidoc-parser`](https://crates.io/crates/asciidoc-parser) crate and aiming for output compatible with [Asciidoctor](https://asciidoctor.org)'s default `html5` backend. The workspace ships a lean [`asciidoc-html5`](html5/) library — with no CLI dependencies, so other tools can embed it — and the [`adoc`](cli/) command-line front end: running `cargo install adoc` gives you the `adoc` command (much as installing ripgrep gives you `rg`), letting you convert an AsciiDoc file to HTML5 with `adoc input.adoc -o output.html`.

## Status of this project

As of July 2026, this project is in its infancy and should not be expected to be meaningfully useful. I am building this based on [`asciidoc-parser`](https://github.com/asciidoc-rs/asciidoc-parser), which is largely feature-complete. 

## Why do this?

Most of all this is a fun project that exercises different architectural and project design skills from my [day job](https://opensource.contentauthenticity.org). As part of that work, I write [technical standards for the Creator Assertions Working Group](https://cawg.io/specs/) in Asciidoc and [Antora](https://antora.org).

Once the parser is sufficiently built out, I have a few projects I’d like to build out that depend on it:

* A version of Antora that highlights differences between versions of a spec/document, as in version to version or proposed updates in a pull request.
* A version of Antora or similar that shows what portions of a spec are tested/completed/known good. (See the following section on “spec-driven development.”)
* A version of [Zola](https://getzola.org), the static site generator that I use for most of my web sites, that accepts Asciidoc formatted text as input. (See [Project proposal: Asciidoc support in Zola](https://zola.discourse.group/t/project-proposal-asciidoc-support-in-zola/2867).)

For now I’m focused on driving the rendering library to being complete enough for those projects to start.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

### License for reference materials

This dual license applies to the `asciidoc-html5` and `adoc` crates and to
everything in this repository **except** the [`ref/`](ref) folder, which holds
verbatim, pinned copies of upstream materials kept as references for
spec-driven development. Each subfolder of `ref/` carries its own upstream
license, distinct from this repository's `MIT OR Apache-2.0` dual license.

[`ref/asciidoctor/`](ref/asciidoctor) contains a verbatim copy of documentation
and test materials from the [Asciidoctor](https://asciidoctor.org) project,
taken at the `v2.0.26` release tag. Those materials are the work of the
Asciidoctor authors and remain under Asciidoctor's own license — the **MIT
License**, Copyright (C) 2012-present Dan Allen, Sarah White, Ryan Waldron, and
the individual contributors to Asciidoctor. See
[`ref/asciidoctor/README.md`](ref/asciidoctor/README.md) and
[`ref/asciidoctor/LICENSE`](ref/asciidoctor/LICENSE) for details.

[`ref/asciidoc-lang/`](ref/asciidoc-lang) contains a pinned snapshot of the
documentation site from the [AsciiDoc Language](https://gitlab.eclipse.org/eclipse/asciidoc-lang/asciidoc-lang)
project (the Eclipse Foundation's official AsciiDoc language description), kept
here as the spec to develop and measure coverage against. That documentation is
licensed **CC-BY-4.0** and the project as a whole under **EPL-2.0**, separate
from this repository's dual license. See
[`ref/asciidoc-lang/README.md`](ref/asciidoc-lang/README.md) and
[`ref/asciidoc-lang/docs/LICENSE`](ref/asciidoc-lang/docs/LICENSE) for details.
