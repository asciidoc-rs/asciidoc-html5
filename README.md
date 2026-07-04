# asciidoc-html5

A Rust HTML5 renderer for [AsciiDoc](https://asciidoc.org), built on the [`asciidoc-parser`](https://crates.io/crates/asciidoc-parser) crate and aiming for output compatible with [Asciidoctor](https://asciidoctor.org)'s default `html5` backend. The workspace ships a lean [`asciidoc-html5`](html5/) library — with no CLI dependencies, so other tools can embed it — and the [`adoc`](cli/) command-line front end: running `cargo install adoc` gives you the `adoc` command (much as installing ripgrep gives you `rg`), letting you convert an AsciiDoc file to HTML5 with `adoc input.adoc -o output.html`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
