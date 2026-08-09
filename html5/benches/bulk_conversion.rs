//! Library-mode bulk-conversion benchmark: the counterpart to
//! `bench/bulk_ruby.rb` at the workspace root, which runs the same scenario
//! against the Ruby `Asciidoctor` gem. Both load the shared
//! `bench/corpus/*.adoc` documents once, then convert the whole corpus
//! repeatedly *within a single process*, isolating sustained library
//! throughput from process-startup cost — the scenario a bulk tool (e.g. a
//! static-site generator embedding this crate) actually sees.
//!
//! Regenerate the shared corpus with `ruby bench/generate_corpus.rb`; this
//! bench reads whatever `.adoc` files are present at run time, so it needs no
//! changes when the corpus is resized.

use std::{fs, path::PathBuf};

use asciidoc_html5::{convert_with, Options, SafeMode};
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

/// The shared corpus directory, resolved relative to this crate regardless of
/// the working directory the bench is invoked from.
fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bench/corpus")
}

/// Reads every `*.adoc` file in the shared corpus, sorted by name so the
/// benchmark iterates in a stable order.
fn load_corpus() -> Vec<String> {
    let dir = corpus_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("reading corpus dir {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "adoc"))
        .collect();
    paths.sort();

    assert!(
        !paths.is_empty(),
        "no corpus found in {}; run `ruby bench/generate_corpus.rb`",
        dir.display()
    );

    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
        })
        .collect()
}

pub fn bulk(c: &mut Criterion) {
    let corpus = load_corpus();

    // Unsafe mode, matching bench/bulk_ruby.rb's `safe: :unsafe` — `convert`'s
    // plain default (`SafeMode::Secure`) would do less work than the Ruby
    // side's explicit unsafe mode, so pin it here to keep the two comparable.
    let options = Options::new().safe_mode(SafeMode::Unsafe);

    // One iteration converts every document in the corpus once, mirroring
    // bench/bulk_ruby.rb's inner loop body.
    c.bench_function("bulk/convert_corpus_once", |b| {
        b.iter(|| {
            for source in &corpus {
                black_box(convert_with(black_box(source), black_box(&options)));
            }
        })
    });
}

criterion_group!(benches, bulk);
criterion_main!(benches);
