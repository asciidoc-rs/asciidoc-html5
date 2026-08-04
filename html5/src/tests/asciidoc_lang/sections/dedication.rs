//! Coverage of the AsciiDoc language description's *Dedication* page.
//!
//! The `dedication` section style requires `:doctype: book` (it is a level 0 or
//! level 1 special section of a book). This renderer pins the doctype to
//! `article` (book is out of scope for 1.0 – see issue #188), so nothing on
//! this page is verifiable here and the whole page is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/dedication.adoc");

// The `dedication` section style is book-doctype behavior (it requires
// `:doctype: book` and its level depends on whether the book has parts), which
// is out of scope for 1.0 (article is the only structural doctype modeled – see
// issue #188), so this whole page is tracked non-normatively.
non_normative!(
    r#"
= Dedication
:keywords: respect, homage, front matter

A dedication page is used to express gratitude.

== Dedication section syntax

To use the `dedication` section style, the document type must be `book`.
The dedication section must be a level 1 section (`==`), unless the book has parts.

[source]
----
[dedication]
== Dedication

include::example$dedication.adoc[tag=body]
----

If the book has parts, the dedication section must be a level 0 section (`=`).

[source]
----
[dedication]
= Dedication

include::example$dedication.adoc[tag=body]
----
"#
);
