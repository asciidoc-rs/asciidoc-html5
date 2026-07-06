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
- **Output parity is measured against Asciidoctor 2.0.26** (the version pinned
  in [`ref/asciidoctor`](ref/asciidoctor)). Its output is the definitive oracle:
  when this crate's output differs, treat Asciidoctor as correct and match it —
  unless the divergence is explicitly documented as a known limitation of this
  crate or of `asciidoc-parser`.

## Before every commit

Run these from the workspace root and make sure they pass — CI enforces all
three:

```sh
cargo +nightly fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Format with **nightly** rustfmt: `rustfmt.toml` turns on unstable options
(`wrap_comments`, `imports_granularity`, …) that stable rustfmt silently
ignores, and CI enforces format with `cargo +nightly fmt --all -- --check`. Run
`rustup toolchain add nightly` once if you don't have it.

## Running the CLI

```sh
cargo run --bin adoc -- input.adoc            # writes input.html (derived name)
cargo run --bin adoc -- input.adoc -o out.html # write to a named file
cargo run --bin adoc -- input.adoc -o -        # write HTML5 to stdout
cat input.adoc | cargo run --bin adoc          # read from stdin, write to stdout
```

> Note: the renderer is at an early **baseline** — it renders the document
> skeleton, header, paragraphs, sections, the preamble, verbatim blocks, and
> thematic and page breaks; other constructs emit a visible `<!-- unsupported … -->`
> comment for now. See [`html5/ARCHITECTURE.md`](html5/ARCHITECTURE.md) for the
> design and roadmap.

## Porting an Asciidoctor doc page ("page port")

A recurring workflow: take one Asciidoctor reference page under
`ref/asciidoctor/docs/modules/<module>/pages/<page>.adoc` and bring it into this
project. **Refer to it as a "page port" (e.g. "page port the get-started page").**
The steps, in order:

1. **Implement** whatever the page requires so this project actually delivers the
   documented behavior, matching Asciidoctor (the parity oracle — see
   *Conventions*). If the page centers on behavior we deliberately diverge from,
   confirm the direction before changing it, and update any page/test the change
   invalidates.
2. **Cover the reference page** with SDD markers. Add a test module tracking the
   page (`track_file!("ref/asciidoctor/.../<page>.adoc")`) under the crate(s) that
   can verify it — `html5/src/tests/asciidoctor/` for library/API behavior and/or
   `cli/src/tests/asciidoctor/` for CLI behavior. Mark descriptive prose
   `non_normative!` and wrap each verifiable claim in a `#[test]` with a
   `verifies!` block that drives the closest available behavior. When a page is
   tracked from *both* crates, each must reproduce the **entire page, line for
   line (blank lines included)**, differing only in which spans are `verifies!` vs
   `non_normative!` — the `sdd` tool merges the two by position, so any dropped or
   added line misaligns the merge. **Bind blank lines to the text above them:** a
   marker block starts on its first line of content (no leading blank line) and
   carries the blank line that follows its content as a trailing blank, so each
   boundary blank belongs to the block it *follows*, not the one it precedes.
3. **Write the docs page** under `docs/modules/ROOT/pages/<page>.adoc`, adapted to
   this project (`adoc` / the `asciidoc_html5` API), using only constructs the
   renderer supports so its shown output is accurate, and calling out known
   limitations. Add it to `docs/modules/ROOT/nav.adoc`.
4. **Cover the new docs page** the same way (both crates, `non_normative!` prose +
   `verifies!` invocations, full-page reproduction).
5. **Verify:** `cargo fmt --all`, `cargo clippy --workspace --all-targets
   --all-features -- -D warnings`, `cargo test --workspace --all-features`, and
   `(cd sdd && cargo run)` — confirm the new pages show the intended `verifies!`
   lines and no unintended `0` (uncovered) lines.
6. **Open a draft PR** with a Conventional-Commit title (unless asked otherwise).
