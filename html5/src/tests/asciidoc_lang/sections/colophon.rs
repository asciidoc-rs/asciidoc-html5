//! Coverage of the AsciiDoc language description's *Colophon* page.
//!
//! The `colophon` section style requires `:doctype: book` (it is a level 0 or
//! level 1 special section of a book). This renderer pins the doctype to
//! `article` (book is out of scope for 1.0 – see issue #188), so nothing on
//! this page is verifiable here and the whole page is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/colophon.adoc");

// The `colophon` section style is book-doctype behavior (it requires
// `:doctype: book` and its level depends on whether the book has parts), which
// is out of scope for 1.0 (article is the only structural doctype modeled – see
// issue #188), so this whole page is tracked non-normatively.
non_normative!(
    r#"
= Colophon
:keywords: edition notice, impressum, masthead, imprint, disclosure, imprint, copyright

A colophon contains factual information about the book, particularly relating to its production.
It may include information such as the ISBN, publishing house, edition and copyright dates, legal notices and disclaimers, typographic style, fonts and paper used, cover art and layout credits, binding method, and any other significant production notes.

The colophon (or colophons), if present, almost always occur at the very beginning (front matter) or end (back matter) of a book.
However, they can also be placed anywhere in an AsciiDoc manuscript as a top-level section.
In a printed book, the colophon is often found on the verso side of the title page.

== Colophon section syntax

To use the `colophon` section style, the document type must be `book`.
If the book does not have parts, the colophon must be a level 1 section (`==`).

[source]
----
[colophon]
== Colophon

include::example$colophon.adoc[tag=body]
----

If the book has parts, the colophon must be a level 0 section (`=`).

[source]
----
[colophon]
= Colophon

include::example$colophon.adoc[tag=body]
----

The colophon will only be numbered if the `sectnums` attribute has the value `all`.
"#
);
