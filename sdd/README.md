# Spec-driven development

This tool aims to show how code coverage and spec coverage are largely
similar. It generates [Codecov]-compatible coverage results by reading
Rust code containing special markers describing what parts of the
specification it covers.

Unlike the equivalent tool in `asciidoc-parser`, this one scans the test
modules of **both** workspace crates (`asciidoc-html5` and `adoc`), since
either may verify parts of the AsciiDoc language specification.

The spec sources it measures coverage against are the AsciiDoc language
description and the Asciidoctor reference documentation and test suite (both
under `ref/`), plus this crate's own documentation pages (under `docs/`). Those
crate pages are descriptive rather than normative, so they are tracked as
non-normative content.

Please consider this very early proof-of-concept quality code and
excuse the many shortcuts taken herein.

[Codecov]: https://about.codecov.io
