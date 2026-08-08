#!/usr/bin/env ruby
# frozen_string_literal: true

# Regenerates bench/corpus/: a fixed set of synthetic AsciiDoc documents used
# by both the Rust (html5/benches/bulk_conversion.rs) and Ruby
# (bench/bulk_ruby.rb) bulk-conversion benchmarks, and by bench/cli_bench.sh
# for the per-process CLI comparison.
#
# The corpus is checked into git so the benchmarks are reproducible without
# running this script. Re-run it (`ruby bench/generate_corpus.rb`) after
# changing TIERS below to resize or reshape the corpus; commit the result.
#
# Every document mixes constructs both adoc and asciidoctor render today
# (headings, paragraphs with inline formatting, lists, a source block, a
# table, an admonition, a thematic break, a cross-reference) so the two
# converters are doing comparable work. Generation is seeded, so re-running
# this script without editing it reproduces byte-identical output.

require "fileutils"
require "zlib"

CORPUS_DIR = File.join(__dir__, "corpus")

# size tier => [document count, target sections per document]
TIERS = {
  "small" => { count: 5, sections: 1 },
  "medium" => { count: 5, sections: 4 },
  "large" => { count: 5, sections: 30 },
}.freeze

WORDS = %w[
  lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod
  tempor incididunt ut labore et dolore magna aliqua enim ad minim veniam
  quis nostrud exercitation ullamco laboris nisi aliquip ex ea commodo
  consequat duis aute irure in reprehenderit voluptate velit esse cillum
  eu fugiat nulla pariatur excepteur sint occaecat cupidatat non proident
  sunt culpa qui officia deserunt mollit anim id est laborum
].freeze

def sentence(rng, words: 12)
  cap = WORDS.sample(words, random: rng)
  cap[0] = cap[0].capitalize
  "#{cap.join(' ')}."
end

def paragraph(rng, sentences: 4)
  Array.new(sentences) { sentence(rng) }.join(" ")
end

# A paragraph with a representative mix of inline formatting, matching the
# constructs exercised by html5/benches/convert_pipeline.rs's REPRESENTATIVE
# document.
def formatted_paragraph(rng)
  "#{paragraph(rng, sentences: 2)} " \
    "This word is *bold*, this one is _italic_, and this is `monospace`. " \
    "See the https://example.org/#{rng.rand(1000)}[project page] for more, " \
    "or jump to <<Overview>>."
end

def unordered_list(rng, items: 4)
  Array.new(items) { "* #{sentence(rng, words: 6)}" }.join("\n")
end

def ordered_list(rng, items: 3)
  Array.new(items) { ". #{sentence(rng, words: 6)}" }.join("\n")
end

def source_block(rng)
  <<~ADOC
    [source,rust]
    ----
    fn process(n: u32) -> u32 {
        // #{sentence(rng, words: 6)}
        n.wrapping_mul(#{rng.rand(2..97)})
    }
    ----
  ADOC
end

def table(rng, rows: 4)
  body = Array.new(rows) { |i| "| row#{i} | #{rng.rand(1..1000)} | #{sentence(rng, words: 3)}" }
  <<~ADOC
    [cols="1,1,3",options="header"]
    |===
    | Key | Count | Note

    #{body.join("\n")}
    |===
  ADOC
end

def admonition(rng)
  kind = %w[NOTE TIP IMPORTANT WARNING CAUTION].sample(random: rng)
  "#{kind}: #{sentence(rng, words: 10)}"
end

def section(rng, level, title)
  parts = ["#{'=' * level} #{title}", "", formatted_paragraph(rng), ""]

  case rng.rand(6)
  when 0
    parts << unordered_list(rng) << ""
  when 1
    parts << ordered_list(rng) << ""
  when 2
    parts << source_block(rng)
  when 3
    parts << table(rng) << ""
  when 4
    parts << admonition(rng) << ""
  when 5
    parts << "'''" << "" << paragraph(rng) << ""
  end

  parts.join("\n")
end

def document(rng, title, section_count)
  parts = [
    "= #{title}",
    ":author: Bench Author",
    ":revdate: 2026-01-01",
    "",
    "[[Overview]]",
    formatted_paragraph(rng),
    "",
  ]

  section_count.times do |i|
    parts << section(rng, 2, "Section #{i + 1}")
    # A couple of subsections on the larger documents, to exercise nesting.
    parts << section(rng, 3, "Section #{i + 1}, Detail") if section_count > 4 && i.even?
  end

  parts.join("\n")
end

FileUtils.mkdir_p(CORPUS_DIR)
Dir.glob(File.join(CORPUS_DIR, "*.adoc")).each { |f| File.delete(f) }

TIERS.each do |tier, spec|
  spec[:count].times do |i|
    # Seeded per-file so the corpus is reproducible but each file differs.
    rng = Random.new(Zlib.crc32("#{tier}-#{i}"))
    title = "#{tier.capitalize} Document #{i + 1}"
    path = File.join(CORPUS_DIR, format("%<tier>s-%<i>02d.adoc", tier: tier, i: i))
    File.write(path, "#{document(rng, title, spec[:sections])}\n")
  end
end

puts "Wrote #{TIERS.sum { |_, s| s[:count] }} documents to #{CORPUS_DIR}"
