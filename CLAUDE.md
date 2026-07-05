# CLAUDE.md

Guidance for working in this repository.

## What this is

A Cargo workspace with two member crates, both at the repo root:

- `html5/` — the `asciidoc-html5` **library**. HTML5 renderer built on
  `asciidoc-parser`. Keep it lean: it depends **only** on `asciidoc-parser`
  (plus std). Do not add CLI, argument-parsing, or I/O-framework dependencies
  here — other tools (e.g. a future Antora-style generator) embed this library.
- `cli/` — the `adoc` **binary**. Thin front end over the library: read
  AsciiDoc, call `asciidoc_html5::convert`, write HTML5. The default binary name
  matches the package (`adoc`), so no `[[bin]]` stanza is needed.

Shared metadata (version, edition, license, repository) lives in
`[workspace.package]`; shared dependency versions live in
`[workspace.dependencies]`. Member crates inherit these with `field.workspace =
true`. Bump versions and dependency versions in the root `Cargo.toml`.

## Conventions

- **Commits:** use [Conventional Commits](https://www.conventionalcommits.org)
  (`feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`). Keep the
  subject imperative and scoped, start the description with a capital letter,
  and omit the trailing period — CI enforces this on PR titles. For example,
  `feat(html5): Render section headings`.
- **Comments:** put a blank line before a code comment (unless it is the first
  line of its block), so the comment visually attaches to the code it precedes.
- **Edition:** Rust 2021. **License:** `MIT OR Apache-2.0` (dual, matching
  `asciidoc-parser`).
- **Compatibility target:** the renderer aims for output compatible with
  Asciidoctor's default `html5` backend.

## Before every commit

Run these from the workspace root and make sure they pass — CI enforces all
three:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Running the CLI

```sh
cargo run --bin adoc -- input.adoc            # HTML5 to stdout
cargo run --bin adoc -- input.adoc -o out.html
cat input.adoc | cargo run --bin adoc         # read from stdin
```

> Note: the renderer is at an early **baseline** — it renders the document
> skeleton, header, paragraphs, sections, the preamble, verbatim blocks, and
> thematic breaks; other constructs emit a visible `<!-- unsupported … -->`
> comment for now. See [`html5/ARCHITECTURE.md`](html5/ARCHITECTURE.md) for the
> design and roadmap.
