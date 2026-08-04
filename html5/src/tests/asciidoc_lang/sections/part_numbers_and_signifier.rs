//! Coverage of the AsciiDoc language description's *Part Numbers and Signifier*
//! page.
//!
//! Part numbering (`partnums`, Roman numerals) and the `part-signifier` prefix
//! only exist for book parts under `:doctype: book`. This renderer pins the
//! doctype to `article` (book is out of scope for 1.0 – see issue #188), so
//! nothing on this page is verifiable here and the whole page is tracked
//! non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/part-numbers-and-signifier.adoc");

// Part numbering (`partnums`, Roman numerals) and the `part-signifier` prefix
// are book-doctype behavior (they apply only to book parts, which require
// `:doctype: book`), which is out of scope for 1.0 (article is the only
// structural doctype modeled – see issue #188), so this whole page is tracked
// non-normatively.
non_normative!(
    r#"
= Part Numbers and Signifier
:page-aliases: part-numbers-and-labels.adoc

[#partnums]
== Number book parts

Book part numbers are controlled by the `partnums` attribute, not `sectnums`.
To autogenerate part numbers, set the `partnums` attribute in the book header.

[source]
----
= The Secret Manual
:doctype: book
:sectnums:
:partnums:

= Defensive Operations

== An Introduction to DefenseOps

= Managing Werewolves
----

When rendered, part numbers are displayed as Roman numerals.

....
Part I. Defensive Operations
1. An Introduction to DefenseOps
Part II. Managing Werewolves
....

[#part-signifier]
== Customize the part signifier

The signifier (i.e., prefix) _Part_ is automatically added to the beginning of the part titles when the `partnums` attribute is set.
The signifier is offset from the auto-generated part number by a space.
You can modify the signifier by defining the `part-signifier` attribute in the header of your book.

[source]
----
:part-signifier: Component
----

To remove the prefix, unset the `part-signifier` attribute in the document header:

----
:!part-signifier:
----
"#
);
