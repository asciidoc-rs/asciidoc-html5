//! Coverage of the AsciiDoc language description's *Preface* page.
//!
//! The `preface` section style requires `:doctype: book` (it precedes the first
//! chapter of a book or book part). This renderer pins the doctype to `article`
//! (book is out of scope for 1.0 – see issue #188), so nothing on this page is
//! verifiable here and the whole page is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/preface.adoc");

// The `preface` section style is book-doctype behavior (it can only be used
// when the doctype is `book`, and its level depends on whether the book has
// parts), which is out of scope for 1.0 (article is the only structural doctype
// modeled – see issue #188), so this whole page is tracked non-normatively.
non_normative!(
    r#"
= Preface
:keywords: front matter, forward, introduction, proem, pref

A preface is a special section that precedes the first chapter of a book or a book part.

== Preface for a book

The `preface` section style can only be used when the doctype is `book`.
A preface can contain subsections.
When a book contains parts (i.e., multipart book), the preface can be defined as either a level 0 section (`=`) or level 1 section (`==`).
When a book doesn't contain parts, the preface must be defined as a level 1 section (`==`).
In either case, any preface subsections must start at one level below the level of the preface title.

[source]
----
include::example$preface.adoc[tag=book]
----

== Preface for a book part

To create a preface for a book part, the preface must be defined as a level 1 section (`==`) and any subsections must start at level 2 (`===`).
The preface must be the first section in the part.

[source]
----
include::example$preface.adoc[tag=part]
----

////
As an alternative to a special section, the AsciiDoc processor will automatically promote the preamble to a preface.
The title of the preface is set using the `preface-title` document attribute.
When defined this way, the preface may not contain subsections.

[source]
----
= Book Title
:doctype: book
:preface-title: Documentation Preface

The basis for this documentation germinated when I awoke one morning and was confronted by the dark and stormy eyes of the chinchilla.
She had conquered the mountain of government reports that, over the course of six months, had eroded into several minor foothills and a creeping alluvial plain of loose papers.

== Chapter 1

...
----

The special section is the preferred way of defining a preface.
////
"#
);
