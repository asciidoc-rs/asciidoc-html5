# Performance benchmarks: `adoc`/`asciidoc-html5` vs Ruby `asciidoctor`

Infrastructure for comparing this project's performance against the Ruby
`asciidoctor` gem/CLI it targets for output parity (see the root
`CLAUDE.md`). Two distinct comparisons live here, because they measure very
different things:

1. **CLI, per-process** (`cli_bench.sh`): `adoc document.adoc` vs
   `asciidoctor document.adoc`, timed end to end including process startup.
   This is what a user invoking either tool from a shell actually feels.
2. **Library, in-process bulk** (`bulk_ruby.rb` +
   `html5/benches/bulk_conversion.rs`): many documents converted back to
   back inside one long-lived process — via the `asciidoc-html5` crate
   directly, and via `Asciidoctor.convert` from Ruby — with process-startup
   cost excluded. This is the scenario a bulk tool (a static-site generator,
   a doc-build pipeline) sees when it embeds either library instead of
   shelling out per file.

The two tell different stories: Ruby's interpreter/gem-load startup cost is
large and fixed per process, so it dominates comparison (1) far more than
comparison (2), where it's paid once regardless of corpus size. Report both;
neither alone characterizes "performance" completely.

## Shared corpus

Both comparisons convert the same documents, under `corpus/`: 15 synthetic
AsciiDoc files in three size tiers (`small-*`, `medium-*`, `large-*`,
roughly 1KB/2KB/23KB). Every document has headings, a cross-reference, and
formatted paragraphs; across its sections it also draws from lists, a source
block, a table, an admonition, and a thematic break, so the corpus as a
whole — though not every individual document — exercises the constructs
both converters render today. The corpus is checked into git so results are
reproducible without regenerating it.

Regenerate (or resize, via the `TIERS` constant) with:

```sh
ruby bench/generate_corpus.rb
```

Generation is seeded, so re-running it unchanged reproduces the same files
byte for byte; commit the result when you do change `TIERS`.

## Running the benchmarks

**CLI comparison** — requires a release build of `adoc`, the `asciidoctor`
gem (`gem install asciidoctor`, pinned to the version in
[`ref/asciidoctor`](../ref/asciidoctor) for parity), and
[hyperfine](https://github.com/sharkdp/hyperfine):

```sh
bash bench/cli_bench.sh              # extra args are passed through to hyperfine
```

Prints a hyperfine comparison for one document per size tier (converted to
stdout, no disk write) and for the whole corpus in a single multi-file
invocation. Markdown reports land in `bench/results/` (git-ignored — machine-
and environment-dependent, so not committed).

**Library bulk comparison** — Ruby side:

```sh
ruby bench/bulk_ruby.rb [iterations]   # default 200 passes over the corpus
```

Rust side, via the existing criterion/CodSpeed bench harness (see
`html5/benches/`):

```sh
cargo bench -p asciidoc-html5 --bench bulk_conversion
```

`bulk_conversion` also runs as part of `cargo codspeed run` in CI (see
`.github/workflows/ci.yml`), so a regression in sustained library throughput
is tracked the same way as the other renderer benchmarks — it just isn't
compared against Ruby there, since CI has no Ruby/asciidoctor toolchain.

## Interpreting results

- The CLI comparison's gap is dominated by Ruby's interpreter/gem-load
  startup — expect it to shrink as documents get larger (fixed startup cost
  amortized over more conversion work) and to shrink further when several
  files are converted in one `asciidoctor`/`adoc` invocation (startup paid
  once for the whole batch).
- The library bulk comparison isolates sustained conversion throughput from
  that startup cost, so it's the more representative number for embedding
  either as a library in a bulk pipeline.
- Both scripts pin `safe: :unsafe` / the CLI default safe mode so neither
  side pays extra for a stricter mode the other isn't also paying.
- Numbers vary by machine; re-run locally rather than trusting numbers from
  an old PR description or issue comment.

## Snapshots

`snapshots/` holds dated point-in-time recordings of both comparisons above,
each with the tool versions and environment they were measured on. Since
CodSpeed only tracks the Rust side of `bulk_conversion` commit-over-commit
(it can't run Ruby), the most recent snapshot is the reference point for
translating a CodSpeed delta into "what does this do to the lead over Ruby" —
see [`snapshots/2026-08-08-ruby-baseline.md`](snapshots/2026-08-08-ruby-baseline.md)
for the current one and the worked-through method. Add a new dated file
(don't edit an old one) when re-measuring.
