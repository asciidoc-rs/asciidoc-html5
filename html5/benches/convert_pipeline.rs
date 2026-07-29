//! End-to-end pipeline benchmarks: the full `convert` path (parse + render),
//! so total user-visible conversion cost is tracked alongside the
//! render-isolated benches in [`render_micro`] and [`render_scaling`].
//!
//! A regression here that does *not* show up in the render-isolated benches
//! points at the parser (or at parsing overhead), which helps localize the
//! cause when CodSpeed flags a slowdown.

use asciidoc_html5::{convert, convert_with, Options};
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

/// A representative document exercising a mix of the constructs the renderer
/// supports: header, preamble, sections, a source block, lists, a table, and
/// an admonition.
const REPRESENTATIVE: &str = "= A Representative Document
:author: Test Author

A short preamble that introduces the document before the first section.

== Getting Started

A paragraph with *bold*, _italic_, and a https://example.org[link].

[source,rust]
----
fn main() {
    println!(\"hello, world\");
}
----

== Details

* one
* two
* three

[cols=\"1,1\",options=\"header\"]
|===
| Key | Value

| alpha | 1
| beta | 2
|===

[NOTE]
====
Something worth noting.
====
";

pub fn pipeline(c: &mut Criterion) {
    // Full parse + render, embedded (body-only) output — the `convert` default.
    c.bench_function("pipeline/convert_representative", |b| {
        b.iter(|| convert(black_box(REPRESENTATIVE)))
    });

    // Standalone output embeds the default stylesheet and emits the full
    // document shell — a render cost unique to the standalone path.
    let standalone = Options::new().standalone(true);

    c.bench_function("pipeline/standalone_embedded_css", |b| {
        b.iter(|| convert_with(black_box(REPRESENTATIVE), black_box(&standalone)))
    });
}

criterion_group!(benches, pipeline);
criterion_main!(benches);
