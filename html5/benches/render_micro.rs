//! Fixed-cost micro-benchmarks: one small document per construct family.
//!
//! Each bench parses its source once with [`load`] and times only
//! [`convert_document`], so a regression here points at the renderer rather
//! than at the `asciidoc-parser` dependency. Keep the inputs small and
//! single-purpose — these guard the per-construct render path, not scaling.

use asciidoc_html5::{convert_document, load};
use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

/// Parses `source` once, then benchmarks the render step in isolation.
fn bench_render(c: &mut Criterion, name: &str, source: &str) {
    let doc = load(source);

    c.bench_function(name, |b| b.iter(|| convert_document(black_box(&doc))));
}

const HELLO: &str = "= Hello\n\nWorld.";

const INLINE_FORMATTING: &str = "= Inline Formatting

A paragraph with *bold*, _italic_, `monospace`, a https://example.org[link],
and a cross-reference to <<Hello>> plus a footnote:[a note] for good measure.
";

const NESTED_SECTIONS: &str = "= Document Title

== Level 1

Intro text.

=== Level 2

More text.

==== Level 3

Even more text.

===== Level 4

Deepest text.
";

const VERBATIM_SOURCE: &str = "= Source Block

[source,rust]
----
fn main() {
    let items = vec![1, 2, 3, 4, 5];
    let total: i32 = items.iter().sum();
    println!(\"total = {total}\");
    for (i, item) in items.iter().enumerate() {
        println!(\"items[{i}] = {item} (<escaped> & \\\"quoted\\\")\");
    }
}
----
";

const LISTS: &str = "= Lists

* apples
* oranges
** navel
** blood
* pears

. first
. second
. third

CPU:: the brain
Memory:: the short-term storage
Disk:: the long-term storage
";

const TABLE: &str = "= Table

[cols=\"1,1,1\",options=\"header\"]
|===
| Name | Role | Location

| Alice | Engineer | Seattle
| Bob | Designer | Portland
| Carol | Manager | Denver
|===
";

const ADMONITION_QUOTE: &str = "= Blocks

[NOTE]
====
This is an admonition with a nested paragraph.
====

[quote,Ada Lovelace,Notes]
____
That brain of mine is something more than merely mortal, as time will show.
____

[verse,Carl Sandburg,Fog]
____
The fog comes
on little cat feet.
____
";

pub fn micro(c: &mut Criterion) {
    bench_render(c, "micro/hello", HELLO);
    bench_render(c, "micro/inline_formatting", INLINE_FORMATTING);
    bench_render(c, "micro/nested_sections", NESTED_SECTIONS);
    bench_render(c, "micro/verbatim_source", VERBATIM_SOURCE);
    bench_render(c, "micro/lists", LISTS);
    bench_render(c, "micro/table", TABLE);
    bench_render(c, "micro/admonition_quote", ADMONITION_QUOTE);
}

criterion_group!(benches, micro);
criterion_main!(benches);
