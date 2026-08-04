//! Coverage of the AsciiDoc language description's *Section Styles for Articles
//! and Books* page.
//!
//! This page is a catalog: it enumerates which special-section styles are
//! permitted in the `article` and `book` document types. It introduces no new
//! rendering rule of its own – every listed style is documented and verified on
//! its own dedicated page – so the whole page is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/styles.adoc");

// This page is purely a catalog of the special-section styles available per
// document type. It states no rendering behavior to verify: each article style
// (abstract, appendix, glossary, bibliography, index) is covered on its own
// dedicated page, and the book-only styles and structure (colophon, dedication,
// acknowledgments, preface, partintro, part, chapter) are out of scope for 1.0
// (article is the only structural doctype modeled – see issue #188). The whole
// page is therefore non-normative.
non_normative!(
    r#"
= Section Styles for Articles and Books

AsciiDoc provides built-in styles for the specialized front matter and back matter sections found in journal articles, academic papers, and books.
These styled sections are referred to as [.term]*special sections*.
The document type, `article` or `book`, determines which section styles are available for use.

== Book section styles

The following section styles are permitted in the `book` document type:

// front
* abstract (becomes a chapter)
* colophon
* dedication
* acknowledgments
* preface
* partintro (must be first child of part)
// back
* appendix
* glossary
* bibliography
* index

The following styles are implied by the location of the section in the document and are thus not special sections.

* part
* chapter

== Article section styles

The following section styles are permitted in the `article` document type:

// front
* abstract
// back
* appendix
* glossary
* bibliography
* index

////
Only these section styles can have subsections:

// front matter
* abstract
//(translated into a chapter)
* preface
// back matter
* appendix
////
"#
);
