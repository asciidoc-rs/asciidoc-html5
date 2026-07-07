# Asciidoctor reference materials

This folder contains a **verbatim copy** of selected materials from the
[Asciidoctor](https://asciidoctor.org) project, kept here as a reference for
spec-driven development of `asciidoc-html5`. Because this renderer aims for
output compatible with Asciidoctor's default `html5` backend, having its
documentation and test suite alongside the source makes it easy to compare
behavior and track down the expected rendering of a given construct.

## What was copied, and from where

The contents were copied from the Asciidoctor repository at the **`v2.0.26`**
release tag:

- Source: <https://github.com/asciidoctor/asciidoctor/tree/v2.0.26>
- Commit: `0b99b39c9df884d4aec13bba45f03cdbab505769`

The following were copied, unmodified:

| Path in this repo | Origin in `asciidoctor/asciidoctor` |
| ----------------- | ----------------------------------- |
| [`docs/`](docs)   | [`docs/`](https://github.com/asciidoctor/asciidoctor/tree/v2.0.26/docs) — the Antora documentation site for Asciidoctor |
| [`test/`](test)   | [`test/`](https://github.com/asciidoctor/asciidoctor/tree/v2.0.26/test) — the Ruby test suite (Minitest) and its fixtures |
| [`data/stylesheets/asciidoctor-default.css`](data/stylesheets/asciidoctor-default.css) | [`data/stylesheets/asciidoctor-default.css`](https://github.com/asciidoctor/asciidoctor/blob/v2.0.26/data/stylesheets/asciidoctor-default.css) — the compiled default stylesheet Asciidoctor embeds in standalone HTML output |

Only the single `asciidoctor-default.css` file was copied from `data/`, not the
rest of that folder. It is the exact stylesheet Asciidoctor's `html5` backend
embeds in a standalone document (Asciidoctor reads it via
`Stylesheets#primary_stylesheet_data` and writes it out publicly as
`asciidoctor.css`), so it is the oracle for this renderer's own embedded copy in
[`html5/assets/`](../../html5/assets/asciidoctor-default.css).

These files are **not** built, run, or otherwise used by the `asciidoc-html5`
crates. They are included purely as a snapshot for reference. This project does
not attempt to track upstream changes with any git subtree/submodule machinery;
the copy is pinned to the tag above. To refresh it, re-copy from a newer
Asciidoctor tag and update this file accordingly.

## License and copyright

Everything in this folder is the work of the **Asciidoctor** project and its
contributors, and is redistributed here under Asciidoctor's own license — the
**MIT License**. It is **not** covered by the `MIT OR Apache-2.0` dual license
that applies to the rest of this repository.

> MIT License
>
> Copyright (C) 2012-present Dan Allen, Sarah White, Ryan Waldron, and the
> individual contributors to Asciidoctor.

The full text of the license as distributed with Asciidoctor v2.0.26 is
reproduced verbatim in [`LICENSE`](LICENSE) in this folder. All copyright and
credit for the material in this folder belongs to the Asciidoctor authors; it
is included here in unmodified form to comply with that license and to serve as
a faithful reference.
