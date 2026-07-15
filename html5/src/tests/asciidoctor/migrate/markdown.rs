// The Asciidoctor "Migrate from Markdown to Asciidoctor" page tells authors
// how to move Markdown content to AsciiDoc: it notes which Markdown syntax
// Asciidoctor recognizes so a migration can proceed gradually, points to the
// syntax comparison, and recommends the Kramdown AsciiDoc tool to automate the
// conversion. It is authoring-migration guidance about external tooling and
// states no rule for this HTML5 renderer to satisfy, so the whole page is
// tracked as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/migrate/pages/markdown.adoc");

non_normative!(
    r#"
= Migrate from Markdown to Asciidoctor
:navtitle: Migrate from Markdown

Asciidoctor recognizes some Markdown syntax, thus allowing you to migrate from Markdown to AsciiDoc gradually.
See xref:asciidoc::syntax-quick-reference.adoc#markdown-compatibility[Markdown compatibility] to learn what syntax is shared.
The syntax you must change is listed in the table under the xref:asciidoc::asciidoc-vs-markdown.adoc#comparison-by-example[Comparison by example section].

You can use https://github.com/asciidoctor/kramdown-asciidoc[Kramdown AsciiDoc^] to automate the migration from Markdown to AsciiDoc.
"#
);
