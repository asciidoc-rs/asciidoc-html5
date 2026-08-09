# Performance snapshot: `adoc`/`asciidoc-html5` vs Ruby `asciidoctor` (2026-08-08)

A point-in-time recording of the [bench/](../README.md) comparisons, and how
to read future CodSpeed results against it. CodSpeed only tracks the Rust
side (`bulk_conversion`) commit-over-commit — it has no way to re-run the
Ruby comparison — so this file is the fixed reference point for "how does
Rust's lead over Ruby look now" until someone re-runs the full comparison and
adds a new snapshot alongside this one.

## Environment

| | |
|---|---|
| Date | 2026-08-08 |
| `asciidoc-html5` / `adoc` | 0.1.6, [PR #317](https://github.com/asciidoc-rs/asciidoc-html5/pull/317) |
| `asciidoctor` gem | 2.0.26 (the version pinned in [`ref/asciidoctor`](../../ref/asciidoctor)) |
| Ruby | 3.3.6 |
| rustc / cargo | 1.94.1 |
| hyperfine | 1.18.0 |
| criterion | 0.8.2 (local `cargo bench`, walltime — *not* CodSpeed's instrumented run) |
| CPU | Intel Xeon @ 2.80GHz, 4 vCPU |
| OS | Linux 6.18 x86_64 |

Reproduce with `ruby bench/generate_corpus.rb && bash bench/cli_bench.sh &&
ruby bench/bulk_ruby.rb 100 && cargo bench -p asciidoc-html5 --bench
bulk_conversion`.

## Results

### CLI, per-process (`bench/cli_bench.sh`)

| Scenario | `adoc` | `asciidoctor` | Ratio |
|---|---:|---:|---:|
| Single small doc (~1KB) → stdout | 17.5 ms | 236.3 ms | 13.5× |
| Single medium doc (~2KB) → stdout | 17.7 ms | 233.2 ms | 13.2× |
| Single large doc (~23KB) → stdout | 21.5 ms | 239.5 ms | 11.1× |
| Whole corpus, 15 files, one invocation | 45.8 ms | 336.5 ms | 7.4× |

Dominated by Ruby's interpreter/gem-load startup (~220ms fixed cost per
process); the ratio shrinks as more conversion work is batched into one
invocation and that fixed cost amortizes.

### Library, in-process bulk (no process-startup cost)

| | Throughput | Mean/doc |
|---|---:|---:|
| `asciidoc-html5` (`cargo bench --bench bulk_conversion`) | ~972 docs/s, 8.1 MB/s | 1.03 ms |
| `Asciidoctor.convert` (`bench/bulk_ruby.rb 100`) | ~150 docs/s, 1.25 MB/s | 6.65 ms |

**~6.5× faster** — the more representative number for a bulk tool (a
static-site generator, a doc-build pipeline) that embeds either as a
library instead of shelling out per file. This is the figure CodSpeed's
`bulk/convert_corpus_once` benchmark tracks the Rust half of.

(This number moved from an earlier ~4.7× measured when PR #317 first
introduced this benchmark with a safe-mode mismatch — the Rust loop was
converting under the library's default `SafeMode::Secure` while the Ruby
loop explicitly used `:unsafe`, so the two weren't doing quite the same
work. PR #319 fixed the mismatch; both now pin unsafe mode. Fixing it
happened to *widen* the measured lead, since unsafe
mode turned out to be the cheaper path for this corpus.)

## Interpreting future CodSpeed results against this snapshot

CodSpeed (`.github/workflows/ci.yml`'s `benchmarks` job) reports
`bulk/convert_corpus_once` as a simulated CPU-instruction count for the Rust
renderer alone, compared to the previous commit/PR base — it does not, and
cannot, re-run `bench/bulk_ruby.rb`. To translate a CodSpeed delta into "what
does this do to the ~6.5× lead over Ruby":

1. Treat the reported **percentage** change as a rough stand-in for wall-clock
   change. `bulk/convert_corpus_once` is CPU-bound pure computation (parse +
   render, no I/O in the timed region), so instruction count and wall-clock
   time move together closely enough for a directional estimate — but they
   are not identical (branch prediction, cache effects, and allocator
   behavior aren't modeled the same way), so don't treat the derived ratio as
   precise.
2. Divide the ~6.5× baseline by `(1 + regression%)` for a slowdown, or
   multiply by `(1 + improvement%)` for a speedup. Example: CodSpeed reports
   `bulk/convert_corpus_once` got 10% slower → the Rust-vs-Ruby throughput
   lead is now roughly 6.5 / 1.10 ≈ **5.9×**, still comfortably ahead. A 30%
   regression would bring it to ≈5.0×; only a regression well past 500%
   would erase the lead entirely.
3. This same reasoning does **not** transfer to the CLI comparison (13.5×
   → 7.4× range) — that gap is dominated by Ruby's fixed process-startup
   cost, which a Rust-side instruction-count change doesn't touch at all.
   Don't rescale those numbers from a CodSpeed delta.

Re-run the full comparison and add a new dated file under `bench/snapshots/`
(rather than editing this one) when: the `asciidoctor` gem version pinned in
`ref/asciidoctor` changes, a change here specifically targets bulk/library
throughput, or enough small CodSpeed deltas have accumulated that the
directional estimate above is worth checking against ground truth again.
