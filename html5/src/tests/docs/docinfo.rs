// This crate's own "Docinfo Files" page. It introduces docinfo -- the three
// insertion points, the `docinfo` attribute, and how the safe mode gates
// reading the files -- as descriptive documentation rather than a set of
// verifiable invocations, so the whole page is tracked as non-normative. The
// placement behavior it describes is exercised by the docinfo tests in
// `renderer.rs`.

use crate::tests::sdd::*;

track_file!("docs/modules/ROOT/pages/docinfo.adoc");

non_normative!(
    r#"
= Docinfo Files
:navtitle: Docinfo Files
:description: How asciidoc-html5 splices caller-supplied docinfo content into the head, header, and footer of a standalone HTML5 document.

_Docinfo files_ are auxiliary snippets of output-format content -- HTML5, in this
renderer's case -- that a document can pull in and have spliced verbatim into
fixed positions of the generated page. They are how you add things the AsciiDoc
body cannot express on its own, such as a `<meta>` tag, a `<link>` to an extra
stylesheet, an analytics snippet, or a custom banner.

[NOTE]
====
The prose on this page is non-normative documentation. Docinfo resolution is
performed by the `asciidoc-parser` crate; this renderer places the content it
resolves. Docinfo only appears in a xref:generate-html:index.adoc[standalone
document], since embedded output has no `<head>` or footer to splice into.
====

== The three locations

AsciiDoc defines three points where docinfo content is inserted, each fed by its
own file:

[cols="1,2"]
|===
|Location |Where it goes

|Head |Appended to the bottom of the `<head>`, below the stylesheet block.
|Header |Inserted immediately before the header `<div>`, which lets it replace the built-in header.
|Footer |Inserted immediately after the footer `<div>`, which lets it replace the built-in footer.
|===

== Enabling docinfo

Docinfo is off by default. A document opts in with the `docinfo` attribute,
whose value selects which files are read:

* `:docinfo: shared` reads the _shared_ files -- _docinfo.html_ (head),
  _docinfo-header.html_, and _docinfo-footer.html_ -- from the document's
  directory.
* `:docinfo: private` reads the _private_ files, whose names are built from the
  document's own base name: _<docname>-docinfo.html_,
  _<docname>-docinfo-header.html_, and _<docname>-docinfo-footer.html_.
* `:docinfo: shared,private` reads both; when both apply to a location, the
  shared content is placed before the private content.

Reading docinfo files touches the filesystem, so it is governed by the
xref:safe-modes.adoc[safe mode]. Under `secure` -- the API default -- docinfo is
dropped without any file being read. A document that lets `adoc` resolve docinfo
from disk therefore runs under a lower mode; the `adoc` command defaults to
`unsafe`.

== Known limitations

This renderer places docinfo content but does not itself read docinfo files;
the parser selects the applicable files, concatenates them, and applies
`docinfosubs` before handing the content over. Supplying docinfo from a document
on disk requires a base directory (an explicit one, or the one derived from the
primary input file), so a bare string conversion with no base directory resolves
no docinfo. The full authoring reference for docinfo files lives in the
https://docs.asciidoctor.org/asciidoc/latest/docinfo/[AsciiDoc language
documentation].
"#
);
