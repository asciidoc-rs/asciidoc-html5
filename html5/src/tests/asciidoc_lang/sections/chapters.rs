//! Coverage of the AsciiDoc language description's *Chapters* page.
//!
//! Chapters (a level 1 section without a block style under the book doctype)
//! and the `chapter-signifier` prefix only exist under `:doctype: book`. This
//! renderer pins the doctype to `article` (book is out of scope for 1.0 – see
//! issue #188), so nothing on this page is verifiable here and the whole page
//! is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/chapters.adoc");

// Chapters and the `chapter-signifier` prefix are book-doctype behavior (a
// chapter is a level 1 section under `:doctype: book`), which is out of scope
// for 1.0 (article is the only structural doctype modeled – see issue #188),
// so this whole page is tracked non-normatively.
non_normative!(
    r#"
= Chapters

[#chapter-signifier]
== Customize the chapter signifier

If the `chapter-signifier` (i.e., prefix) is set, the value of this attribute is automatically added to the beginning of chapter titles in a book when the `sectnums` attribute is also set.
The signifier is offset from the auto-generated chapter number by a space.
For example, the chapter title may appear in the output document as follows:

 Chapter 1. Title

A chapter is defined as a level 1 section without a block style when the doctype type is book.
You can modify the signifier by defining the `chapter-signifier` attribute in the header of your book.

----
:chapter-signifier: Peatükk
----

To remove the prefix, unset the `chapter-signifier` attribute in the document header:

----
:!chapter-signifier:
----
"#
);
