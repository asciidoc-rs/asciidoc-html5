//! Benchmarks for the AsciiDoc → HTML5 conversion pipeline.
//!
//! The pipeline has two halves (see `ARCHITECTURE.md`): `asciidoc-parser`
//! parses the source and applies the inline substitutions, then this crate
//! walks the resulting [`Document`] and emits the block-level HTML5
//! scaffolding. Both halves are measured separately — [`load`] for the parse
//! and [`convert_document`] for the render — alongside the end-to-end entry
//! points callers actually use ([`convert`], and [`convert_with`] under
//! `standalone`), so a change can be attributed to the half it belongs to.
//!
//! Each benchmark runs against three sizes of the same synthetic document, so
//! that the cost of a change can be read both on a document small enough to be
//! dominated by fixed costs (the document header, the `<head>` shell) and on
//! one large enough to be dominated by the per-block walk.

use asciidoc_html5::{
    convert, convert_document, convert_outline, convert_with, load, Options, SafeMode,
};
use divan::{black_box, Bencher};

fn main() {
    divan::main();
}

/// The corpus sizes every benchmark is run against, named for the report.
const SIZES: [&str; 3] = ["small", "medium", "large"];

/// How many times [`SECTION`] is repeated for each of the [`SIZES`].
///
/// `small` is a handful of blocks — the fixed costs of the header and, for a
/// standalone render, of the `<head>` shell still show through. `large` is
/// roughly a long specification chapter, where the per-block walk dominates.
fn repetitions(size: &str) -> usize {
    match size {
        "small" => 1,
        "medium" => 8,
        "large" => 64,
        other => panic!("unknown corpus size: {other}"),
    }
}

/// The document header of the synthetic corpus. It carries the attributes a
/// real document sets — including `toc`, which the outline benchmark needs.
const HEADER: &str = "= Benchmark Document\nBenchmark Author <bench@example.com>\nv1.0.0, 2026-01-01\n:toc:\n:toclevels: 3\n:sectnums:\n:icons: font\n:description: A synthetic document used to measure conversion throughput.\n\nA preamble paragraph with _emphasis_, *strong* text and a `literal` span,\nplus a https://asciidoc.org[link to the language site].\n\n";

/// One repeated chunk of the synthetic corpus: a section holding the block
/// constructs the renderer supports today, with inline markup in the prose so
/// that the parser's inline substitutions are exercised too. `{n}` is replaced
/// by the repetition index to keep ids and cross references unique.
const SECTION: &str = r#"== Section {n}

A paragraph in section {n} with *strong*, _emphasis_, `code`, a
footnote:[an aside about section {n}], a passthrough +++<b>raw</b>+++ and
some special characters: 5 < 6 & 7 > 3.

.A listing with a title
[source,rust]
----
fn section_{n}() -> usize {
    let mut total = 0;
    for i in 0..{n} {
        total += i * i;
    }
    total
}
----

[NOTE]
====
An admonition block in section {n}, with a nested paragraph and a
link:https://example.com/{n}[link].
====

[quote,Someone Quotable,Source {n}]
____
A quoted paragraph, indented and attributed.
____

....
A literal block in section {n}.
  Indentation is preserved verbatim.
....

=== Subsection {n}.1

Prose under a nested heading, so the outline has more than one level.

[verse]
____
A verse block,
line broken as written.
____

'''

"#;

/// Builds the corpus for `size` by repeating [`SECTION`] under [`HEADER`].
fn source(size: &str) -> String {
    let repetitions = repetitions(size);
    let mut source = String::with_capacity(HEADER.len() + repetitions * SECTION.len());
    source.push_str(HEADER);

    for n in 1..=repetitions {
        source.push_str(&SECTION.replace("{n}", &n.to_string()));
    }

    source
}

/// Parse only: `asciidoc-parser` builds the document (and applies the inline
/// substitutions) without any HTML5 scaffolding being emitted.
#[divan::bench(args = SIZES)]
fn load_document(bencher: Bencher, size: &str) {
    let source = source(size);
    bencher.bench(|| load(black_box(&source)));
}

/// Render only: the block walk over an already-parsed document, which is the
/// half of the pipeline this crate owns.
#[divan::bench(args = SIZES)]
fn render_document(bencher: Bencher, size: &str) {
    let document = load(&source(size));
    bencher.bench(|| convert_document(black_box(&document)));
}

/// End to end, embedded: the body-only output returned by the plain entry
/// point.
#[divan::bench(args = SIZES)]
fn convert_embedded(bencher: Bencher, size: &str) {
    let source = source(size);
    bencher.bench(|| convert(black_box(&source)));
}

/// End to end, standalone with a linked stylesheet: adds the
/// `<!DOCTYPE>`/`<head>`/`<body>` shell — and, with `toc` set in the header,
/// the table of contents — on top of the body.
#[divan::bench(args = SIZES)]
fn convert_standalone(bencher: Bencher, size: &str) {
    let source = source(size);
    let options = Options::new().standalone(true).set("linkcss");
    bencher.bench(|| convert_with(black_box(&source), black_box(&options)));
}

/// End to end, standalone with the default stylesheet embedded: the same shell,
/// plus the ~30 KB of default CSS copied into a `<style>` block, which a lower
/// safe mode selects.
#[divan::bench(args = SIZES)]
fn convert_standalone_embedded_css(bencher: Bencher, size: &str) {
    let source = source(size);
    let options = Options::new()
        .standalone(true)
        .safe_mode(SafeMode::Safe)
        .unset("linkcss");
    bencher.bench(|| convert_with(black_box(&source), black_box(&options)));
}

/// The table of contents alone, walked from a parsed document.
#[divan::bench(args = SIZES)]
fn outline(bencher: Bencher, size: &str) {
    let document = load(&source(size));
    bencher.bench(|| convert_outline(black_box(&document)));
}
