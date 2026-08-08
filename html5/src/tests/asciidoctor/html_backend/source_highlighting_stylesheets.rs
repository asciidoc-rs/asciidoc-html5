//! Coverage of Asciidoctor's *Embed a CodeRay or Pygments Stylesheet* page.
//!
//! CodeRay and Pygments are *build-time* (server-side) syntax highlighters —
//! this crate's syntax-highlighting coverage (see
//! `html5/src/tests/asciidoctor/syntax_highlighting/`) already tracks
//! supporting neither as a settled non-goal for this crate's 1.0. Since
//! neither highlighter is implemented, the stylesheet-embedding behavior this
//! page documents has no counterpart to verify, so the whole page is tracked
//! as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/source-highlighting-stylesheets.adoc");

// Both sections describe how the CodeRay/Pygments stylesheet is embedded or
// copied when `coderay-css`/`pygments-css` is `class` — behavior that only
// applies once the corresponding server-side highlighter runs. Neither
// highlighter is implemented (see
// `html5/src/tests/asciidoctor/syntax_highlighting/coderay.rs` and
// `.../pygments.rs`), so none of this page's claims are verifiable here.
non_normative!(
    r#"
= Embed a CodeRay or Pygments Stylesheet

Asciidoctor can embed the stylesheet for the CodeRay or Pygments syntax highlighters.

== Requirements

First, make sure the appropriate library is installed on your system.
See xref:syntax-highlighting:rouge.adoc[], xref:syntax-highlighting:coderay.adoc[], or xref:syntax-highlighting:pygments.adoc[] for installation instructions.
Next, set the xref:asciidoc:verbatim:source-highlighter.adoc[source-highlighter attribute] and assign it the value that corresponds to the library you installed.

[#coderay]
== CodeRay

If the `source-highlighter` attribute is `coderay` and the `coderay-css` attribute is `class`, the CodeRay stylesheet is:

* _embedded_ by default
* _copied_ to the file [.path]_asciidoctor-coderay.css_ inside the `stylesdir` folder within the output directory if `linkcss` is set

[#pygments]
== Pygments

If the `source-highlighter` attribute is `pygments` and the `pygments-css` attribute is `class`, the Pygments stylesheet is:

* _embedded_ by default
* _copied_ to the file [.path]_asciidoctor-pygments.css_ inside the `stylesdir` folder within the output directory if `linkcss` is set
"#
);
