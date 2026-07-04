# asciidoc-html5

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

### License for Asciidoctor reference materials

This dual license applies to the `asciidoc-html5` and `adoc` crates and to
everything in this repository **except** the [`ref/`](ref) folder.

The `ref/` folder contains a verbatim copy of documentation and test materials
from the [Asciidoctor](https://asciidoctor.org) project, taken at the `v2.0.26`
release tag and kept as a reference for spec-driven development. Those materials
are the work of the Asciidoctor authors and remain under Asciidoctor's own
license — the **MIT License**, Copyright (C) 2012-present Dan Allen, Sarah
White, Ryan Waldron, and the individual contributors to Asciidoctor. They are
**not** covered by the `MIT OR Apache-2.0` dual license above. See
[`ref/README.md`](ref/README.md) and [`ref/LICENSE`](ref/LICENSE) for details.
