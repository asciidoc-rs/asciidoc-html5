//! Scaling benchmarks: large inputs that guard against super-linear render
//! cost. These are the highest-value benches — a per-construct path that
//! quietly becomes O(n²) shows up here long before it hurts a small doc.
//!
//! As in [`render_micro`], each source is parsed once with [`load`] and only
//! [`convert_document`] is timed, isolating the renderer from the parser.

use asciidoc_html5::{convert_document, load};
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

/// Number of repetitions for each scaling shape. Tuned so a healthy render
/// stays in the low-milliseconds range; bump if the benches get too fast to
/// measure reliably.
const PARAGRAPH_COUNT: usize = 1000;
const SECTION_COUNT: usize = 200;
const TABLE_ROWS: usize = 100;

/// Parses `source` once, then benchmarks the render step in isolation.
fn bench_render(c: &mut Criterion, name: &str, source: &str) {
    let doc = load(source);

    c.bench_function(name, |b| b.iter(|| convert_document(black_box(&doc))));
}

/// A document of `PARAGRAPH_COUNT` short paragraphs. Guards linear block
/// dispatch and inline substitution across many sibling blocks.
fn many_paragraphs() -> String {
    let mut src = String::from("= Many Paragraphs\n\n");

    for i in 0..PARAGRAPH_COUNT {
        src.push_str(&format!(
            "Paragraph number {i} with some *bold* and _italic_ inline content.\n\n"
        ));
    }

    src
}

/// A document of `SECTION_COUNT` sections with the table of contents enabled.
/// Guards outline construction and TOC rendering at scale.
fn sections_with_toc() -> String {
    let mut src = String::from("= Sections With TOC\n:toc:\n\n");

    for i in 0..SECTION_COUNT {
        src.push_str(&format!("== Section {i}\n\nBody text for section {i}.\n\n"));
    }

    src
}

/// A single header + `TABLE_ROWS`-row, six-column table. Guards per-cell
/// rendering across a large grid.
fn large_table() -> String {
    let mut src =
        String::from("= Large Table\n\n[cols=\"1,1,1,1,1,1\",options=\"header\"]\n|===\n");
    src.push_str("| A | B | C | D | E | F\n\n");

    for i in 0..TABLE_ROWS {
        src.push_str(&format!("| a{i} | b{i} | c{i} | d{i} | e{i} | f{i}\n"));
    }

    src.push_str("|===\n");
    src
}

pub fn scaling(c: &mut Criterion) {
    bench_render(c, "scaling/many_paragraphs", &many_paragraphs());
    bench_render(c, "scaling/sections_with_toc", &sections_with_toc());
    bench_render(c, "scaling/large_table", &large_table());
}

criterion_group!(benches, scaling);
criterion_main!(benches);
