#!/usr/bin/env bash
# Per-process CLI comparison: `adoc` (this crate's binary) vs the Ruby
# `asciidoctor` gem's CLI, converting the same bench/corpus/*.adoc documents.
#
# This measures wall-clock cost *per invocation*, so it includes each
# program's process-startup cost (negligible for the native `adoc` binary,
# substantial for `asciidoctor` — booting the Ruby VM and `require`-ing the
# gem). For a number that excludes process startup — sustained throughput
# when each converter is driven as a library inside one long-lived process —
# see bench/bulk_ruby.rb and `cargo bench --bench bulk_conversion` instead.
#
# Requires: a release build of `adoc` (built automatically below if missing)
# and the `asciidoctor` gem (`gem install asciidoctor`) and `hyperfine`
# (https://github.com/sharkdp/hyperfine — `apt install hyperfine`,
# `brew install hyperfine`, or `cargo install hyperfine`).
#
# Usage: bench/cli_bench.sh [hyperfine args...]
#   bench/cli_bench.sh                  # default run
#   bench/cli_bench.sh --warmup 5       # extra warmup iterations
#
# Results are written as Markdown to bench/results/cli_bench.md (not
# committed — see bench/README.md for why).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
corpus_dir="$repo_root/bench/corpus"
results_dir="$repo_root/bench/results"
adoc_bin="$repo_root/target/release/adoc"

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "error: hyperfine not found on PATH; see the comment at the top of this script" >&2
  exit 1
fi

if ! command -v asciidoctor >/dev/null 2>&1; then
  echo "error: asciidoctor not found on PATH; run 'gem install asciidoctor'" >&2
  exit 1
fi

if [[ ! -x "$adoc_bin" ]]; then
  echo "building release adoc binary..." >&2
  (cd "$repo_root" && cargo build --release --bin adoc)
fi

if [[ ! -d "$corpus_dir" || -z "$(ls -A "$corpus_dir"/*.adoc 2>/dev/null)" ]]; then
  echo "error: no corpus in $corpus_dir; run 'ruby bench/generate_corpus.rb' first" >&2
  exit 1
fi

mkdir -p "$results_dir"

# One representative document per size tier, plus the whole corpus in a
# single multi-file invocation (both CLIs accept several input files at
# once).
small_doc="$corpus_dir/small-00.adoc"
medium_doc="$corpus_dir/medium-00.adoc"
large_doc="$corpus_dir/large-00.adoc"
all_docs=("$corpus_dir"/*.adoc)

echo "=== single-document conversions (stdout, no disk write) ==="
hyperfine "$@" \
  --export-markdown "$results_dir/cli_bench_single.md" \
  -n 'adoc (small)' "'$adoc_bin' '$small_doc' -o -" \
  -n 'asciidoctor (small)' "asciidoctor '$small_doc' -o -" \
  -n 'adoc (medium)' "'$adoc_bin' '$medium_doc' -o -" \
  -n 'asciidoctor (medium)' "asciidoctor '$medium_doc' -o -" \
  -n 'adoc (large)' "'$adoc_bin' '$large_doc' -o -" \
  -n 'asciidoctor (large)' "asciidoctor '$large_doc' -o -"

echo
echo "=== whole corpus in one invocation (${#all_docs[@]} files) ==="
hyperfine "$@" \
  --export-markdown "$results_dir/cli_bench_corpus.md" \
  -n 'adoc (corpus)' "'$adoc_bin' ${all_docs[*]@Q} -D '$results_dir/adoc-out'" \
  -n 'asciidoctor (corpus)' "asciidoctor ${all_docs[*]@Q} -D '$results_dir/asciidoctor-out'"

echo
echo "results written to $results_dir/cli_bench_single.md and $results_dir/cli_bench_corpus.md"
