// Asciidoctor's "Docinfo Files" page in the ROOT (Asciidoctor) component is a
// relocation stub: its content was moved into the AsciiDoc language component
// (`asciidoc:docinfo:index.adoc`), which is not part of this ref repo, and the
// page that remains only redirects there. It documents no rendering rule for
// this crate to satisfy, so the whole page is tracked as non-normative -- even
// though this crate does implement docinfo (see the docinfo tests in
// `renderer.rs`). See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/ROOT/pages/docinfo.adoc");

non_normative!(
    r#"
= Docinfo Files
:page-location: asciidoc:docinfo:index.adoc

Relocated to xref:{page-location}[].
"#
);
