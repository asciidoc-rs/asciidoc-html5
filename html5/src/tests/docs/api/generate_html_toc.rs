use crate::{convert_outline, convert_outline_with, load, tests::sdd::*, Document, OutlineOptions};

track_file!("docs/modules/api/pages/generate-html-toc.adoc");

// This crate's own "Generate an HTML Table of Contents Using the API" page. The
// prose is descriptive documentation, tracked as non-normative; the Rust
// snippets and the HTML outputs it shows are verified by the tests below,
// driving `convert_outline`/`convert_outline_with` against the page's sample
// document. Like the other API pages, it is tracked only from this crate.

// The sample document the page shows: three top-level sections, the second
// holding a subsection, with auto-generated ids `_section_a`, `_section_b`,
// `_subsection`, and `_section_c`.
const SAMPLE: &str = "\
= Document Title

== Section A

== Section B

=== Subsection

== Section C
";

// The full-depth TOC the page shows for that document.
const EXPECTED_OUTLINE: &str = r##"<ul class="sectlevel1">
<li><a href="#_section_a">Section A</a></li>
<li><a href="#_section_b">Section B</a>
<ul class="sectlevel2">
<li><a href="#_subsection">Subsection</a></li>
</ul>
</li>
<li><a href="#_section_c">Section C</a></li>
</ul>"##;

fn sample_doc() -> Document<'static> {
    load(SAMPLE)
}

non_normative!(
    r#"
= Generate an HTML Table of Contents Using the API
:navtitle: Generate an HTML TOC
:description: How to generate a standalone HTML table of contents from a document with the asciidoc_html5 Rust API.

`asciidoc-html5` can generate the HTML table of contents -- a document's section
_outline_ -- on its own, apart from rendering the whole document. Give it a
loaded document and it returns the nested `<ul>` list of section links, the same
markup that appears in a rendered document's TOC, ready to embed in a page
template.

[NOTE]
====
The prose on this page is non-normative documentation. The API calls it shows are
normative: they are verified against the implementation, so the documented
behavior is guaranteed.
====

"#
);

// The core example: `convert_outline` on a loaded document returns the nested
// section list.
#[test]
fn convert_outline_returns_the_section_list() {
    verifies!(
        r##"
== Generate the outline

Suppose you have loaded a document with a few sections, the second of which holds
a subsection:

[,asciidoc]
----
= Document Title

== Section A

== Section B

=== Subsection

== Section C
----

Pass the loaded document to `convert_outline`:

[,rust]
----
let toc = asciidoc_html5::convert_outline(&doc);
----

`convert_outline` returns a `String` holding the nested section list, or an empty
`String` when the document has no sections, since there is then no TOC to build.
For the document above it returns:

[,html]
----
<ul class="sectlevel1">
<li><a href="#_section_a">Section A</a></li>
<li><a href="#_section_b">Section B</a>
<ul class="sectlevel2">
<li><a href="#_subsection">Subsection</a></li>
</ul>
</li>
<li><a href="#_section_c">Section C</a></li>
</ul>
----

"##
    );

    let doc = sample_doc();
    assert_eq!(convert_outline(&doc), EXPECTED_OUTLINE);
}

non_normative!(
    r#"
Each section becomes a list item linking to its auto-generated id, and a section
with subsections nests a child `<ul class="sectlevelN">` inside its item. The
markup matches Asciidoctor's default `html5` backend exactly, so it drops into a
page template that expects Asciidoctor's TOC.

"#
);

// Limiting the depth with `convert_outline_with` and
// `OutlineOptions::toclevels`: capping at 1 drops the subsection so every
// top-level section is a leaf.
#[test]
fn toclevels_limits_the_depth() {
    verifies!(
        r##"
== Limit the depth

By default the outline reaches as deep as the document's `toclevels` attribute
allows (two levels). To override that depth, pass `OutlineOptions` to
`convert_outline_with`:

[,rust]
----
use asciidoc_html5::OutlineOptions;

let toc = asciidoc_html5::convert_outline_with(&doc, &OutlineOptions::new().toclevels(1));
----

With the depth limited to 1, the subsection is dropped and every top-level
section renders as a leaf item:

[,html]
----
<ul class="sectlevel1">
<li><a href="#_section_a">Section A</a></li>
<li><a href="#_section_b">Section B</a></li>
<li><a href="#_section_c">Section C</a></li>
</ul>
----

"##
    );

    let doc = sample_doc();
    let expected = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_section_a\">Section A</a></li>
<li><a href=\"#_section_b\">Section B</a></li>
<li><a href=\"#_section_c\">Section C</a></li>
</ul>";
    assert_eq!(
        convert_outline_with(&doc, &OutlineOptions::new().toclevels(1)),
        expected
    );
}

non_normative!(
    r#"
`OutlineOptions` also carries `sectnumlevels`, the depth to which numbered
sections (those under the `sectnums` attribute) show their section number in the
TOC. Any option left unset falls back to the document's own attribute value.

That covers generating an HTML table of contents using the API.
"#
);
