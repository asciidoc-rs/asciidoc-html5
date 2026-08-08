#!/usr/bin/env ruby
# frozen_string_literal: true

# Library-mode bulk-conversion benchmark for the Ruby Asciidoctor gem: the
# counterpart to html5/benches/bulk_conversion.rs. Both load the same
# bench/corpus/*.adoc documents into memory once, then convert the whole
# corpus repeatedly *within a single process*, so the number measures
# sustained conversion throughput with process-startup and gem-load cost
# excluded — the scenario a bulk tool (e.g. a static-site generator)
# actually sees.
#
# Usage:
#   ruby bench/bulk_ruby.rb [iterations]
#
# `iterations` (default 200) is how many times the whole corpus is
# converted; each pass converts every document in bench/corpus/ once.
#
# Requires the `asciidoctor` gem (`gem install asciidoctor`, pinned to the
# version in ref/asciidoctor per CLAUDE.md's parity target).

require "asciidoctor"
require "benchmark"

CORPUS_DIR = File.join(__dir__, "corpus")
ITERATIONS = (ARGV[0] || 200).to_i

paths = Dir.glob(File.join(CORPUS_DIR, "*.adoc")).sort
abort "no corpus found in #{CORPUS_DIR}; run bench/generate_corpus.rb first" if paths.empty?

docs = paths.map { |p| File.read(p) }
total_bytes = docs.sum(&:bytesize)

# One untimed warm-up pass so the benchmark isn't paying for the interpreter
# JITing/loading autoload paths that a real bulk run would also amortize.
docs.each { |src| Asciidoctor.convert(src, safe: :unsafe) }

elapsed = Benchmark.realtime do
  ITERATIONS.times do
    docs.each { |src| Asciidoctor.convert(src, safe: :unsafe) }
  end
end

total_docs = docs.length * ITERATIONS
total_mb = (total_bytes * ITERATIONS) / (1024.0 * 1024.0)

puts "Asciidoctor #{Asciidoctor::VERSION} (Ruby #{RUBY_VERSION})"
puts "corpus: #{docs.length} documents, #{total_bytes} bytes, #{ITERATIONS} iterations"
puts format("elapsed:     %.3fs", elapsed)
puts format("throughput:  %.1f docs/s, %.2f MB/s", total_docs / elapsed, total_mb / elapsed)
puts format("mean/doc:    %.3fms", (elapsed / total_docs) * 1000)
